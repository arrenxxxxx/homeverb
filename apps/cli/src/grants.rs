//! Explicit owner-controlled startup policy, never ordinary command or credential input.

use homeverb_domain::{Error, Result};
use homeverb_host::{
    MeijuControlGrants, MeijuFanGrant, MeijuModeGrant, MeijuPowerGrant, MeijuTemperatureGrant,
};
#[cfg(target_os = "linux")]
use homeverb_host::{
    meiju_fan_policy, meiju_mode_policy, meiju_power_policy, meiju_temperature_policy,
};
use serde::Deserialize;
use std::path::Path;

#[cfg(target_os = "linux")]
const MAX_BYTES: u64 = 32 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Grants {
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    schema_version: String,
    #[serde(default)]
    pub meiju_power_grants: Vec<MeijuPowerGrant>,
    #[serde(default)]
    pub meiju_fan_grants: Vec<MeijuFanGrant>,
    #[serde(default)]
    pub meiju_mode_grants: Vec<MeijuModeGrant>,
    #[serde(default)]
    pub meiju_temperature_grants: Vec<MeijuTemperatureGrant>,
}

impl Default for Grants {
    fn default() -> Self {
        Self {
            schema_version: "1.0".into(),
            meiju_power_grants: vec![],
            meiju_fan_grants: vec![],
            meiju_mode_grants: vec![],
            meiju_temperature_grants: vec![],
        }
    }
}

impl Grants {
    pub fn controls(&self) -> MeijuControlGrants<'_> {
        MeijuControlGrants {
            ac_features: &[],
            power: &self.meiju_power_grants,
            fan: &self.meiju_fan_grants,
            mode: &self.meiju_mode_grants,
            temperature: &self.meiju_temperature_grants,
            dishwasher: &[],
            dishwasher_cycle: &[],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.meiju_power_grants.is_empty()
            && self.meiju_fan_grants.is_empty()
            && self.meiju_mode_grants.is_empty()
            && self.meiju_temperature_grants.is_empty()
    }

    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path else {
            return Ok(Self::default());
        };
        #[cfg(target_os = "linux")]
        {
            use std::{
                fs::OpenOptions,
                io::Read,
                os::unix::fs::{MetadataExt, OpenOptionsExt},
            };
            if !path.is_absolute() {
                return Err(Error::invalid());
            }
            let file = OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                .open(path)
                .map_err(|_| refused())?;
            let metadata = file.metadata().map_err(|_| refused())?;
            // Inspect the opened descriptor, not a path that could be replaced before the read.
            if !metadata.is_file()
                || metadata.uid() != unsafe { libc::geteuid() }
                || metadata.mode() & 0o022 != 0
                || metadata.len() > MAX_BYTES
            {
                return Err(refused());
            }
            let mut bytes = Vec::new();
            file.take(MAX_BYTES + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| refused())?;
            if bytes.len() as u64 > MAX_BYTES {
                return Err(refused());
            }
            let grants: Self = serde_json::from_slice(&bytes).map_err(|_| Error::invalid())?;
            if grants.schema_version != "1.0" {
                return Err(Error::invalid());
            }
            meiju_power_policy(&grants.meiju_power_grants)?;
            meiju_fan_policy(&grants.meiju_fan_grants)?;
            meiju_mode_policy(&grants.meiju_mode_grants)?;
            meiju_temperature_policy(&grants.meiju_temperature_grants)?;
            Ok(grants)
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = path;
            Err(refused())
        }
    }
}

fn refused() -> Error {
    Error::new(
        "POLICY_DENIED",
        "CLI grants require a readable owner-controlled regular Linux file of at most 32 KiB, without a final symlink or group/other write access.",
    )
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};

    fn write(path: &Path, text: &str) {
        std::fs::write(path, text).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }

    #[test]
    fn explicit_file_preserves_independent_exact_grants() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("grants.json");
        write(
            &path,
            include_str!("../../../examples/cli-meiju-grants.json"),
        );
        let grants = Grants::load(Some(&path)).unwrap();
        assert!(!grants.is_empty());
        assert_eq!(grants.meiju_power_grants.len(), 1);
        assert!(!grants.meiju_power_grants[0].state);
        assert_eq!(grants.meiju_fan_grants[0].wind_speed, 40);
        assert!(Grants::load(None).unwrap().is_empty());
        write(&path, r#"{"schema_version":"1.0"}"#);
        assert!(Grants::load(Some(&path)).unwrap().is_empty());
        write(
            &path,
            &format!(
                "{:<width$}",
                r#"{"schema_version":"1.0"}"#,
                width = MAX_BYTES as usize
            ),
        );
        assert!(Grants::load(Some(&path)).unwrap().is_empty());
    }

    #[test]
    fn malformed_or_broadened_policy_is_rejected_as_a_whole() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("grants.json");
        let original: serde_json::Value =
            serde_json::from_str(include_str!("../../../examples/cli-meiju-grants.json")).unwrap();
        for (pointer, value) in [
            ("/schema_version", serde_json::json!("2.0")),
            ("/meiju_power_grants/0/device_id", serde_json::json!("*")),
            ("/meiju_power_grants/0/state", serde_json::json!("off")),
            ("/meiju_fan_grants/0/wind_speed", serde_json::json!(101)),
            ("/meiju_fan_grants/0/wind_speed", serde_json::json!(40.5)),
            (
                "/meiju_fan_grants",
                serde_json::json!([
                    original["meiju_fan_grants"][0],
                    original["meiju_fan_grants"][0]
                ]),
            ),
            (
                "/meiju_power_grants",
                serde_json::json!(vec![original["meiju_power_grants"][0].clone(); 65]),
            ),
        ] {
            let mut value_copy = original.clone();
            *value_copy.pointer_mut(pointer).unwrap() = value;
            write(&path, &value_copy.to_string());
            assert_eq!(
                Grants::load(Some(&path)).err().unwrap().code,
                "INVALID_ARGUMENT"
            );
        }
        for text in [
            "{",
            r#"{"schema_version":"1.0","confirmed":true}"#,
            r#"{"schema_version":"1.0","schema_version":"1.0"}"#,
        ] {
            write(&path, text);
            assert_eq!(
                Grants::load(Some(&path)).err().unwrap().code,
                "INVALID_ARGUMENT"
            );
        }
    }

    #[test]
    fn mode_only_files_admit_exact_activation_without_other_authority() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("grants.json");
        let mut value = serde_json::json!({"schema_version":"1.0","meiju_mode_grants":[
            {"device_id":"30000000-0000-4000-8000-000000000001","mode":"cool"}]});
        write(&path, &value.to_string());
        let grants = Grants::load(Some(&path)).unwrap();
        assert!(!grants.is_empty());
        assert!(grants.meiju_power_grants.is_empty());
        assert!(grants.meiju_fan_grants.is_empty());
        assert_eq!(grants.meiju_mode_grants.len(), 1);
        value["meiju_mode_grants"][0]["mode"] = serde_json::json!("off");
        write(&path, &value.to_string());
        assert_eq!(
            Grants::load(Some(&path)).err().unwrap().code,
            "INVALID_ARGUMENT"
        );
    }

    #[test]
    fn temperature_only_files_capture_exact_targets_without_other_authority() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("grants.json");
        let value = serde_json::json!({"schema_version":"1.0", "meiju_temperature_grants":[
            {"device_id":"30000000-0000-4000-8000-000000000001", "target_temperature":26.5}]});
        write(&path, &value.to_string());
        let grants = Grants::load(Some(&path)).unwrap();
        assert!(!grants.is_empty());
        let controls = grants.controls();
        assert_eq!(controls.temperature[0].target_temperature, 26.5);
        assert!(controls.power.is_empty() && controls.fan.is_empty() && controls.mode.is_empty());
        let mut invalid = value;
        invalid["meiju_temperature_grants"][0]["target_temperature"] = serde_json::json!(26.25);
        write(&path, &invalid.to_string());
        assert_eq!(
            Grants::load(Some(&path)).err().unwrap().code,
            "INVALID_ARGUMENT"
        );
    }

    #[test]
    fn unsafe_paths_permissions_and_size_do_not_admit_policy() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("grants.json");
        write(&path, r#"{"schema_version":"1.0"}"#);
        let link = directory.path().join("link.json");
        symlink(&path, &link).unwrap();
        let fifo = directory.path().join("fifo");
        let name = std::ffi::CString::new(fifo.as_os_str().as_encoded_bytes()).unwrap();
        // A FIFO must be refused immediately, without waiting for another process to open it.
        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        for rejected in [
            &link,
            &fifo,
            directory.path(),
            &directory.path().join("absent"),
        ] {
            assert_eq!(
                Grants::load(Some(rejected)).err().unwrap().code,
                "POLICY_DENIED"
            );
        }
        assert_eq!(
            Grants::load(Some(Path::new("relative.json")))
                .err()
                .unwrap()
                .code,
            "INVALID_ARGUMENT"
        );
        for mode in [0o620, 0o602, 0o666] {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
            assert_eq!(
                Grants::load(Some(&path)).err().unwrap().code,
                "POLICY_DENIED"
            );
        }
        write(&path, &" ".repeat(MAX_BYTES as usize + 1));
        assert_eq!(
            Grants::load(Some(&path)).err().unwrap().code,
            "POLICY_DENIED"
        );
    }
}
