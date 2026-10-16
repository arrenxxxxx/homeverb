use super::*;
use crate::{
    LanCandidateMatch,
    lan::{DiscoveryCandidate, DiscoveryNetwork, LanDiscovery},
};

/// Unauthenticated result for the selected saved identity only, never route authority.
/// ```compile_fail
/// fn export(value: &homeverb_midea::LanScanOutcome) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
#[derive(Debug)]
pub enum LanScanOutcome {
    NotObserved,
    Incomplete,
    Ambiguous,
    UniqueCandidate(DiscoveryCandidate),
}

impl MeijuAdapter {
    /// Trusted compiled transport injection; construction performs no scan.
    pub fn with_lan_discovery(mut self, discovery: Arc<dyn LanDiscovery>) -> Self {
        self.lan_discovery = discovery;
        self
    }

    async fn lan_account(
        &self,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<MeijuAccount> {
        let id = connection
            .account_id
            .as_deref()
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(crate::account::identity_conflict)?;
        MeijuAccount::open(self.vault.lease(id, operation).await?, operation).await
    }

    /// Protected local preflight only. This is neither network consent nor discovery.
    pub async fn validate_lan_scan(
        &self,
        device: &Device,
        connection: &Connection,
        operation: &Operation,
    ) -> Result<()> {
        let account = self.lan_account(connection, operation).await?;
        account
            .validate_lan_scan(device, connection, operation)
            .await?;
        Ok(())
    }

    /// Call only after explicit trusted foreground network consent. Collect once, then
    /// reconcile under the same account lease without credentials, cloud requests or TCP.
    pub async fn scan_lan_candidate(
        &self,
        device: &Device,
        connection: &Connection,
        network: &DiscoveryNetwork,
        operation: &Operation,
    ) -> Result<LanScanOutcome> {
        let account = self.lan_account(connection, operation).await?;
        let selected = account
            .validate_lan_scan(device, connection, operation)
            .await?;
        let report = operation
            .run(self.lan_discovery.discover(network, operation))
            .await?;
        // Recheck public selection and protected identity after collection as well.
        account
            .validate_lan_scan(device, connection, operation)
            .await?;
        let matched = account
            .reconcile_lan_candidate(selected, network, &report, operation)
            .await?;
        operation
            .run(async {
                Ok(match matched {
                    LanCandidateMatch::NotObserved => LanScanOutcome::NotObserved,
                    LanCandidateMatch::Incomplete => LanScanOutcome::Incomplete,
                    LanCandidateMatch::Ambiguous => LanScanOutcome::Ambiguous,
                    LanCandidateMatch::UniqueCandidate(candidate) => {
                        LanScanOutcome::UniqueCandidate(DiscoveryCandidate {
                            appliance_id: candidate.appliance_id.clone(),
                            endpoint: candidate.endpoint,
                            protocol: candidate.protocol,
                            family: candidate.family,
                        })
                    }
                })
            })
            .await
    }
}
