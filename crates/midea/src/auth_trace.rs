//! Compile-time opt-in Android authentication stage markers, without private values.

use std::ffi::CStr;

#[derive(Clone, Copy)]
pub(crate) enum Event {
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
