//! Compile-time opt-in Android authentication stage markers, without private values.

use std::ffi::CStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    RecoveryRequested,
    RecoveryBindingValidated,
    RecoveryRequiresLegacy,
    RecoveryEmptyBindings,
    RecoveryMembershipUnconfirmed,
    RecoveryLoginStarted,
    RecoveryMissingCanonicalIdentity,
    RecoveryLoginInputMismatch,
    RecoveryInventoryStarted,
    RecoveryInventoryIncomplete,
    RecoveryInventoryCountMismatch,
    RecoveryInventoryIdentityMismatch,
    RecoveryInventoryModelMismatch,
    RecoveryProfileConflict,
    RecoveryReady,
    RecoveryConfirmRequested,
    RecoveryExpired,
    RecoveryDuplicateSubject,
    RecoverySavedMetadataChanged,
    RecoverySavedBytesChanged,
    RecoverySessionSaved,
    RecoveryComplete,
    LoginRootIotUidPresent,
    LoginRootIotUidAbsent,
    LoginRootUidPresent,
    LoginRootUidAbsent,
    LoginUserInfoUidPresent,
    LoginUserInfoUidAbsent,
    LoginUserInfoIotUidPresent,
    LoginUserInfoIotUidAbsent,
    LoginIdentityShapeUnavailable,
    SelectedAccount,
    ProtectedBindingValidated,
    LoginSucceeded,
    SubjectMatches,
    SubjectDiffers,
    SubjectDiffersOnlyInAsciiCase,
    ClientIdentifierMatches,
    ClientIdentifierDiffers,
    LegacyClientIdentifierAbsent,
    SessionSaved,
    InventorySucceeded,
    InventoryFailed,
}

impl Event {
    fn marker(self) -> &'static CStr {
        match self {
            Self::RecoveryRequested => c"recovery_requested",
            Self::RecoveryBindingValidated => c"recovery_binding_validated",
            Self::RecoveryRequiresLegacy => c"recovery_requires_legacy",
            Self::RecoveryEmptyBindings => c"recovery_empty_bindings",
            Self::RecoveryMembershipUnconfirmed => c"recovery_membership_unconfirmed",
            Self::RecoveryLoginStarted => c"recovery_login_started",
            Self::RecoveryMissingCanonicalIdentity => c"recovery_missing_canonical_identity",
            Self::RecoveryLoginInputMismatch => c"recovery_login_input_mismatch",
            Self::RecoveryInventoryStarted => c"recovery_inventory_started",
            Self::RecoveryInventoryIncomplete => c"recovery_inventory_incomplete",
            Self::RecoveryInventoryCountMismatch => c"recovery_inventory_count_mismatch",
            Self::RecoveryInventoryIdentityMismatch => c"recovery_inventory_identity_mismatch",
            Self::RecoveryInventoryModelMismatch => c"recovery_inventory_model_mismatch",
            Self::RecoveryProfileConflict => c"recovery_profile_conflict",
            Self::RecoveryReady => c"recovery_ready",
            Self::RecoveryConfirmRequested => c"recovery_confirm_requested",
            Self::RecoveryExpired => c"recovery_expired",
            Self::RecoveryDuplicateSubject => c"recovery_duplicate_subject",
            Self::RecoverySavedMetadataChanged => c"recovery_saved_metadata_changed",
            Self::RecoverySavedBytesChanged => c"recovery_saved_bytes_changed",
            Self::RecoverySessionSaved => c"recovery_session_saved",
            Self::RecoveryComplete => c"recovery_complete",
            Self::LoginRootIotUidPresent => c"login_root_iot_uid_present",
            Self::LoginRootIotUidAbsent => c"login_root_iot_uid_absent",
            Self::LoginRootUidPresent => c"login_root_uid_present",
            Self::LoginRootUidAbsent => c"login_root_uid_absent",
            Self::LoginUserInfoUidPresent => c"login_user_info_uid_present",
            Self::LoginUserInfoUidAbsent => c"login_user_info_uid_absent",
            Self::LoginUserInfoIotUidPresent => c"login_user_info_iot_uid_present",
            Self::LoginUserInfoIotUidAbsent => c"login_user_info_iot_uid_absent",
            Self::LoginIdentityShapeUnavailable => c"login_identity_shape_unavailable",

            Self::SelectedAccount => c"selected_account",
            Self::ProtectedBindingValidated => c"protected_binding_validated",
            Self::LoginSucceeded => c"login_succeeded",
            Self::SubjectMatches => c"subject_matches",
            Self::SubjectDiffers => c"subject_differs",
            Self::SubjectDiffersOnlyInAsciiCase => c"subject_differs_only_in_ascii_case",
            Self::ClientIdentifierMatches => c"client_identifier_matches",
            Self::ClientIdentifierDiffers => c"client_identifier_differs",
            Self::LegacyClientIdentifierAbsent => c"legacy_client_identifier_absent",
            Self::SessionSaved => c"session_saved",
            Self::InventorySucceeded => c"inventory_succeeded",
            Self::InventoryFailed => c"inventory_failed",
        }
    }
}

pub(crate) fn enabled() -> bool {
    option_env!("HOMEVERB_AUTH_TRACE") == Some("1")
}

/// Inspect only known field presence. IgnoredAny never retains private values.
/// These observations do not participate in authentication or identity decisions.
fn identity_shape(body: &[u8]) -> Option<[Event; 4]> {
    use serde::{Deserialize, de::IgnoredAny};
    #[derive(Default, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct User {
        uid: Option<IgnoredAny>,
        iot_user_id: Option<IgnoredAny>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Data {
        iot_user_id: Option<IgnoredAny>,
        uid: Option<IgnoredAny>,
        user_info: Option<User>,
    }
    #[derive(Deserialize)]
    struct Envelope {
        data: Data,
    }
    let data = serde_json::from_slice::<Envelope>(body).ok()?.data;
    let user = data.user_info.unwrap_or_default();
    Some([
        if data.iot_user_id.is_some() {
            Event::LoginRootIotUidPresent
        } else {
            Event::LoginRootIotUidAbsent
        },
        if data.uid.is_some() {
            Event::LoginRootUidPresent
        } else {
            Event::LoginRootUidAbsent
        },
        if user.uid.is_some() {
            Event::LoginUserInfoUidPresent
        } else {
            Event::LoginUserInfoUidAbsent
        },
        if user.iot_user_id.is_some() {
            Event::LoginUserInfoIotUidPresent
        } else {
            Event::LoginUserInfoIotUidAbsent
        },
    ])
}

pub(crate) fn login_identity_shape(body: &[u8]) {
    if !enabled() {
        return;
    }
    if let Some(events) = identity_shape(body) {
        for event in events {
            emit(event);
        }
    } else {
        emit(Event::LoginIdentityShapeUnavailable);
    }
}

pub(crate) fn emit(event: Event) {
    if !enabled() {
        return;
    }
    let marker = event.marker();
    #[cfg(target_os = "android")]
    {
        #[link(name = "log")]
        unsafe extern "C" {
            fn __android_log_write(
                priority: std::ffi::c_int,
                tag: *const std::ffi::c_char,
                text: *const std::ffi::c_char,
            ) -> std::ffi::c_int;
        }
        // Only closed enum literals reach Android logging. No formatting, account
        // references, identifiers, response bodies or caller strings are accepted.
        unsafe { __android_log_write(4, c"HomeVerbAuthTrace".as_ptr(), marker.as_ptr()) };
    }
    #[cfg(not(target_os = "android"))]
    let _ = marker;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_shape_ignores_values_and_unknown_fields() {
        let events = identity_shape(br#"{"data":{"iotUserId":"PRIVATE-A","uid":123,"userInfo":{"uid":"PRIVATE-B","iotUserId":{"unexpected":"PRIVATE-C"}},"mdata":{"accessToken":"PRIVATE-TOKEN"}}}"#).unwrap();
        assert_eq!(
            events,
            [
                Event::LoginRootIotUidPresent,
                Event::LoginRootUidPresent,
                Event::LoginUserInfoUidPresent,
                Event::LoginUserInfoIotUidPresent
            ]
        );
        for event in events {
            assert!(!event.marker().to_str().unwrap().contains("PRIVATE"));
        }
        assert_eq!(
            identity_shape(br#"{"data":{"iotUserId":null,"userInfo":null}}"#),
            Some([
                Event::LoginRootIotUidAbsent,
                Event::LoginRootUidAbsent,
                Event::LoginUserInfoUidAbsent,
                Event::LoginUserInfoIotUidAbsent
            ])
        );
        assert_eq!(identity_shape(br#"{"data":{"userInfo":false}}"#), None);
        assert_eq!(identity_shape(b"invalid"), None);
    }
}
