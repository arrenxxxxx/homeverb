//! Reconcile protected identities and resolve credentials without contacting a LAN peer.

use super::{
    DeviceBinding, MeijuAccount, PROVIDER, Payload, ProductProfile, StoredBinding, family_code,
    identity_conflict,
};
use crate::{
    ApplianceFamily, CloudId, CloudTransport, ReadEndpoint, ReadTransport,
    lan::{DiscoveryCandidate, DiscoveryNetwork, DiscoveryReport, LanUdpIdMethod, V3Handshake},
    parse_appliances, parse_homes,
};
use homeverb_core::Operation;
use homeverb_domain::{Connection, Device, Error, Result, Transport};

/// Saved identity reconciliation only, never permission or endpoint authentication.
/// ```compile_fail
/// fn export(result: &homeverb_midea::LanCandidateMatch<'_>) {
///     let _ = serde_json::to_string(result);
/// }
/// ```
#[derive(Debug)]
pub enum LanCandidateMatch<'a> {
    NotObserved,
    Incomplete,
    Ambiguous,
    UniqueCandidate(&'a DiscoveryCandidate),
}

impl MeijuAccount {
    fn require_lan_region(&self) -> Result<()> {
        if self.record().region != "CN" {
            return Err(Error::new(
                "AUTH_FLOW_UNSUPPORTED",
                "Only mainland Meiju is admitted.",
            ));
        }
        Ok(())
    }

    fn checked_lan_binding<'a>(
        &self,
        payload: &'a Payload,
        selected: DeviceBinding,
    ) -> Result<&'a StoredBinding> {
        let binding = payload
            .bindings
            .iter()
            .find(|binding| binding.local == selected)
            .ok_or_else(identity_conflict)?;
        binding.require_listed()?;
        if !matches!(binding.family, 0xac | 0xe1 | 0xca)
            || !binding
                .appliance_id
                .parse::<u64>()
                .is_ok_and(|id| (1..=0xffff_ffff_ffff).contains(&id))
        {
            return Err(Error::new(
                "CAPABILITY_UNSUPPORTED",
                "The appliance is outside the reviewed LAN reconciliation scope.",
            ));
        }
        Ok(binding)
    }

    /// Validate a saved public selection against protected evidence before any LAN scan.
    pub(crate) async fn validate_lan_scan(
        &self,
        device: &Device,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<DeviceBinding> {
        operation
            .run(async {
                self.require_lan_region()?;
                let selected = DeviceBinding {
                    device_id: device.id.parse().map_err(|_| identity_conflict())?,
                    connection_id: connection.id.parse().map_err(|_| identity_conflict())?,
                    identity_ref: connection
                        .identity_ref
                        .parse()
                        .map_err(|_| identity_conflict())?,
                };
                let payload = self.load(operation).await?;
                let binding = self.checked_lan_binding(&payload, selected)?;
                let expected = Connection {
                    id: binding.local.connection_id.to_string(),
                    adapter: PROVIDER.into(),
                    transport: Transport::Cloud,
                    account_id: Some(self.record().id.to_string()),
                    identity_ref: binding.local.identity_ref.to_string(),
                };
                if *connection != expected
                    || device.id != binding.local.device_id.to_string()
                    || !device.connections.contains(connection)
                    || binding
                        .model
                        .as_ref()
                        .is_some_and(|model| *model != device.model)
                {
                    return Err(identity_conflict());
                }
                Ok(selected)
            })
            .await
    }
    /// Compare one protected binding with bounded, unauthenticated LAN hints.
    /// Performs no network or persistence work and does not admit any route/model.
    pub async fn reconcile_lan_candidate<'a>(
        &self,
        selected: DeviceBinding,
        network: &DiscoveryNetwork,
        report: &'a DiscoveryReport,
        operation: &Operation,
    ) -> Result<LanCandidateMatch<'a>> {
        operation
            .run(async {
                self.require_lan_region()?;
                let payload = self.load(operation).await?;
                let binding = self.checked_lan_binding(&payload, selected)?;
                report.validate(network)?;
                if report.limit_reached {
                    return Ok(LanCandidateMatch::Incomplete);
                }
                let Some(candidate) = report
                    .candidates
                    .iter()
                    .find(|hint| hint.appliance_id.expose() == binding.appliance_id)
                else {
                    return Ok(LanCandidateMatch::NotObserved);
                };
                if !matches!(
                    (candidate.family, binding.family),
                    (ApplianceFamily::AirConditioner, 0xac)
                        | (ApplianceFamily::Dishwasher, 0xe1)
                        | (ApplianceFamily::Refrigerator, 0xca)
                ) || report.candidates.iter().any(|hint| {
                    if hint.appliance_id == candidate.appliance_id {
                        hint.endpoint != candidate.endpoint
                            || hint.protocol != candidate.protocol
                            || hint.family != candidate.family
                    } else {
                        hint.endpoint == candidate.endpoint
                    }
                }) {
                    return Ok(LanCandidateMatch::Ambiguous);
                }
                Ok(LanCandidateMatch::UniqueCandidate(candidate))
            })
            .await
    }

    /// Prepare one V3 handshake from a protected binding and fresh unique home membership.
    /// This performs explicit cloud reads only; it neither authenticates a LAN peer nor
    /// admits a route. The trusted caller must separately authorize any appliance access.
    pub async fn lan_v3_handshake(
        &self,
        transport: &dyn CloudTransport,
        selected: DeviceBinding,
        method: LanUdpIdMethod,
        operation: &Operation,
    ) -> Result<V3Handshake> {
        operation
            .run(async {
                if self.record().region != "CN" {
                    return Err(Error::new(
                        "AUTH_FLOW_UNSUPPORTED",
                        "Only mainland Meiju is admitted.",
                    ));
                }
                let payload = self.load(operation).await?;
                let binding = payload
                    .bindings
                    .iter()
                    .find(|binding| binding.local == selected)
                    .ok_or_else(identity_conflict)?;
                binding.require_listed()?;
                if !matches!(binding.family, 0xac | 0xe1 | 0xca) {
                    return Err(Error::new(
                        "CAPABILITY_UNSUPPORTED",
                        "The appliance family is outside the reviewed LAN credential scope.",
                    ));
                }
                let udp_id = method.udp_id(&binding.appliance_id)?;
                let appliance =
                    CloudId::parse(&serde_json::Value::String(binding.appliance_id.clone()))?;
                let session = payload.cloud_session()?;
                session.lan_client_id()?;
                let reads = session.reads(transport, self.record().credential_ref);
                let body = reads
                    .request(
                        &self.record().credential_ref,
                        ReadEndpoint::Homes,
                        operation,
                    )
                    .await?;
                let homes = parse_homes(&body)?;
                if homes.len() > 32 {
                    return Err(crate::invalid());
                }
                let mut matched_home = None;
                for home in homes {
                    let body = reads
                        .request(
                            &self.record().credential_ref,
                            ReadEndpoint::HomeAppliances(&home),
                            operation,
                        )
                        .await?;
                    for current in parse_appliances(&body, &home)? {
                        if current.id != appliance {
                            continue;
                        }
                        if matched_home.is_some()
                            || family_code(current.family) != binding.family
                            || current.model != binding.model
                        {
                            return Err(identity_conflict());
                        }
                        ProductProfile::from_stored(&binding.product_profile)
                            .merge(&current.product_profile)?;
                        matched_home = Some(home.clone());
                    }
                }
                let home = matched_home.ok_or_else(|| {
                    Error::new(
                        "MIDEA_DEVICE_NOT_LISTED",
                        "The selected appliance was absent from the fresh account inventory.",
                    )
                })?;
                session
                    .lan_handshake(transport, &appliance, &home, &udp_id, operation)
                    .await
            })
            .await
    }
}
