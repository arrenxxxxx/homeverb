//! Resolve one selected device's cloud credentials without contacting a LAN peer.

use super::{DeviceBinding, MeijuAccount, ProductProfile, family_code, identity_conflict};
use crate::{
    CloudId, CloudTransport, ReadEndpoint, ReadTransport,
    lan::{LanUdpIdMethod, V3Handshake},
    parse_appliances, parse_homes,
};
use homeverb_core::Operation;
use homeverb_domain::{Error, Result};

impl MeijuAccount {
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
