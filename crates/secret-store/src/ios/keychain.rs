//! Minimal Security/Core Foundation boundary. Owned results never escape a call.

use super::*;
use core_foundation_sys::{
    array::{CFArrayGetCount, CFArrayGetTypeID, CFArrayGetValueAtIndex},
    base::{CFAllocatorRef, CFEqual, CFGetTypeID, CFOptionFlags, CFRelease, CFTypeID, CFTypeRef},
    data::{CFDataCreate, CFDataGetBytePtr, CFDataGetLength, CFDataGetTypeID},
    dictionary::{
        CFDictionaryCreateMutable, CFDictionaryGetTypeID, CFDictionaryGetValue, CFDictionaryRef,
        CFDictionarySetValue, kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks,
    },
    error::CFErrorRef,
    number::{
        CFBooleanGetTypeID, CFNumberCreate, CFNumberGetTypeID, CFNumberGetValue,
        CFNumberIsFloatType, kCFBooleanFalse, kCFBooleanTrue, kCFNumberSInt32Type,
        kCFNumberSInt64Type,
    },
    string::{CFStringCreateWithBytes, CFStringRef, kCFStringEncodingUTF8},
};
use std::ptr;

#[link(name = "Security", kind = "framework")]
unsafe extern "C" {
    fn SecAccessControlGetTypeID() -> CFTypeID;
    fn SecAccessControlCreateWithFlags(
        allocator: CFAllocatorRef,
        protection: CFTypeRef,
        flags: CFOptionFlags,
        error: *mut CFErrorRef,
    ) -> CFTypeRef;
    fn SecItemCopyMatching(query: CFDictionaryRef, result: *mut CFTypeRef) -> i32;
    fn SecItemAdd(attributes: CFDictionaryRef, result: *mut CFTypeRef) -> i32;
    fn SecItemUpdate(query: CFDictionaryRef, attributes: CFDictionaryRef) -> i32;
    fn SecItemDelete(query: CFDictionaryRef) -> i32;
    static kSecClass: CFStringRef;
    static kSecClassGenericPassword: CFStringRef;
    static kSecAttrService: CFStringRef;
    static kSecAttrAccount: CFStringRef;
    static kSecAttrAccessGroup: CFStringRef;
    static kSecAttrAccessible: CFStringRef;
    static kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly: CFStringRef;
    #[cfg(test)]
    static kSecAttrAccessibleWhenUnlocked: CFStringRef;
    #[cfg(test)]
    static kSecAttrAccessibleAfterFirstUnlock: CFStringRef;
    static kSecAttrSynchronizable: CFStringRef;
    static kSecAttrSynchronizableAny: CFStringRef;
    static kSecAttrAccessControl: CFStringRef;
    static kSecMatchLimit: CFStringRef;
    static kSecMatchLimitOne: CFStringRef;
    static kSecReturnAttributes: CFStringRef;
    static kSecReturnPersistentRef: CFStringRef;
    static kSecReturnData: CFStringRef;
    static kSecValuePersistentRef: CFStringRef;
    static kSecValueData: CFStringRef;
    static kSecUseAuthenticationUI: CFStringRef;
    static kSecUseAuthenticationUIFail: CFStringRef;
}

const NOT_FOUND: i32 = -25300;

struct Owned(CFTypeRef);

impl Owned {
    // Only +1 references from Create/Copy functions may enter this wrapper.
    fn new(value: CFTypeRef) -> Result<Self> {
        if value.is_null() {
            Err(unavailable())
        } else {
            Ok(Self(value))
        }
    }
}

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: this wrapper owns one nonnull Create/Copy reference.
        unsafe { CFRelease(self.0) }
    }
}

fn string(value: &str) -> Result<Owned> {
    // SAFETY: Rust bytes remain valid for this copying constructor.
    unsafe {
        Owned::new(
            CFStringCreateWithBytes(
                ptr::null(),
                value.as_ptr(),
                value.len() as isize,
                kCFStringEncodingUTF8,
                0,
            )
            .cast(),
        )
    }
}

fn data(value: &[u8]) -> Result<Owned> {
    // SAFETY: bounded input is copied during this call, not retained by pointer.
    unsafe { Owned::new(CFDataCreate(ptr::null(), value.as_ptr(), value.len() as isize).cast()) }
}

fn dictionary() -> Result<Owned> {
    // SAFETY: the standard CF callbacks retain every inserted key and value.
    unsafe {
        Owned::new(
            CFDictionaryCreateMutable(
                ptr::null(),
                0,
                &kCFTypeDictionaryKeyCallBacks,
                &kCFTypeDictionaryValueCallBacks,
            )
            .cast(),
        )
    }
}

// SAFETY requirements for the following helpers: pointers are borrowed from a
// live CF result or a declared framework constant; dictionary types are checked
// before reading attributes. Created dictionaries are mutable until dispatch.
unsafe fn set(dict: &Owned, key: CFStringRef, value: CFTypeRef) {
    unsafe { CFDictionarySetValue(dict.0.cast_mut().cast(), key.cast(), value) }
}

unsafe fn attribute(dict: CFTypeRef, key: CFStringRef) -> CFTypeRef {
    unsafe { CFDictionaryGetValue(dict.cast(), key.cast()) }
}

unsafe fn equal(left: CFTypeRef, right: CFTypeRef) -> bool {
    !left.is_null() && !right.is_null() && unsafe { CFEqual(left, right) != 0 }
}

unsafe fn decode_synchronizable(value: CFTypeRef) -> Result<bool> {
    unsafe {
        if value.is_null() {
            return Err(unavailable());
        }
        if CFGetTypeID(value) == CFBooleanGetTypeID() {
            return Ok(equal(value, kCFBooleanTrue.cast()));
        }
        // Security may return this flag as an integer CFNumber rather than CFBoolean.
        if CFGetTypeID(value) != CFNumberGetTypeID() || CFNumberIsFloatType(value.cast()) != 0 {
            return Err(unavailable());
        }
        let mut flag: i64 = 0;
        if !CFNumberGetValue(
            value.cast(),
            kCFNumberSInt64Type,
            (&mut flag as *mut i64).cast(),
        ) {
            return Err(unavailable());
        }
        match flag {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(unavailable()),
        }
    }
}

fn access_control() -> Result<Owned> {
    // SAFETY: public constants and no extra flags produce the required policy.
    // No Keychain lookup or authentication is performed by this constructor.
    unsafe {
        Owned::new(SecAccessControlCreateWithFlags(
            ptr::null(),
            kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly.cast(),
            0,
            ptr::null_mut(),
        ))
    }
}

unsafe fn unsupported_access_control(value: CFTypeRef) -> Result<bool> {
    unsafe {
        if value.is_null() {
            return Ok(false);
        }
        if CFGetTypeID(value) != SecAccessControlGetTypeID() {
            return Ok(true);
        }
        // Compare the complete policy, not its description or accessibility alone.
        Ok(!equal(value, access_control()?.0))
    }
}

unsafe fn copy_data(value: CFTypeRef, limit: usize) -> Result<SecretBytes> {
    unsafe {
        if value.is_null() || CFGetTypeID(value) != CFDataGetTypeID() {
            return Err(unavailable());
        }
        let length = CFDataGetLength(value.cast());
        if length < 0 || length as usize > limit {
            return Err(unavailable());
        }
        if length == 0 {
            return Ok(SecretBytes::new(Vec::new()));
        }
        let bytes = CFDataGetBytePtr(value.cast());
        if bytes.is_null() {
            return Err(unavailable());
        }
        Ok(SecretBytes::new(
            std::slice::from_raw_parts(bytes, length as usize).to_vec(),
        ))
    }
}

fn query(scope: &Scope) -> Result<Owned> {
    let dict = dictionary()?;
    let service = string(&scope.service)?;
    let account = string(&scope.account)?;
    let group = string(&scope.access_group)?;
    // SAFETY: all values live through insertion and are retained by the dictionary.
    unsafe {
        set(&dict, kSecClass, kSecClassGenericPassword.cast());
        set(&dict, kSecAttrService, service.0);
        set(&dict, kSecAttrAccount, account.0);
        set(&dict, kSecAttrAccessGroup, group.0);
        // Deliberate deprecated fail-without-prompt policy; see ADR-0062.
        set(
            &dict,
            kSecUseAuthenticationUI,
            kSecUseAuthenticationUIFail.cast(),
        );
    }
    Ok(dict)
}

fn protected_query(scope: &Scope, entry: Option<&Entry>) -> Result<Owned> {
    let dict = query(scope)?;
    // SAFETY: the dictionary retains these declared constants and copied data.
    unsafe {
        set(&dict, kSecAttrSynchronizable, kCFBooleanFalse.cast());
        set(
            &dict,
            kSecAttrAccessible,
            kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly.cast(),
        );
        if let Some(entry) = entry {
            entry.validate()?;
            let reference = data(entry.persistent_ref.expose())?;
            set(&dict, kSecValuePersistentRef, reference.0);
        }
    }
    Ok(dict)
}

fn add_attributes(scope: &Scope) -> Result<Owned> {
    let dict = query(scope)?;
    let protection = access_control()?;
    // SAFETY: the dictionary retains this explicit policy and synchronization flag.
    // Do not also supply kSecAttrAccessible: Security refuses conflicting forms.
    unsafe {
        set(&dict, kSecAttrAccessControl, protection.0);
        set(&dict, kSecAttrSynchronizable, kCFBooleanFalse.cast());
    }
    Ok(dict)
}

fn search(query: &Owned) -> Result<Option<Owned>> {
    let mut output = ptr::null();
    // SAFETY: query is a valid CF dictionary; output receives a +1 reference.
    let status = unsafe { SecItemCopyMatching(query.0.cast(), &mut output) };
    let output = if output.is_null() {
        None
    } else {
        Some(Owned(output))
    };
    match status {
        0 => output.map(Some).ok_or_else(unavailable),
        NOT_FOUND => Ok(None),
        _ => Err(status_error(status)),
    }
}

unsafe fn decode_entry(value: CFTypeRef, scope: &Scope) -> Result<Entry> {
    unsafe {
        if value.is_null() || CFGetTypeID(value) != CFDictionaryGetTypeID() {
            return Err(unavailable());
        }
        let service = string(&scope.service)?;
        let account = string(&scope.account)?;
        let group = string(&scope.access_group)?;
        let synchronizable = decode_synchronizable(attribute(value, kSecAttrSynchronizable))?;
        Ok(Entry {
            matches_scope: equal(attribute(value, kSecAttrService), service.0)
                && equal(attribute(value, kSecAttrAccount), account.0)
                && equal(attribute(value, kSecAttrAccessGroup), group.0),
            passcode_only: equal(
                attribute(value, kSecAttrAccessible),
                kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly.cast(),
            ),
            synchronizable,
            unsupported_access_control: unsupported_access_control(attribute(
                value,
                kSecAttrAccessControl,
            ))?,
            persistent_ref: copy_data(attribute(value, kSecValuePersistentRef), MAX_REFERENCE)?,
        })
    }
}

unsafe fn decode_secret(value: CFTypeRef, scope: &Scope, expected: &Entry) -> Result<SecretBytes> {
    unsafe {
        let observed = decode_entry(value, scope)?;
        observed.validate()?;
        if observed.persistent_ref.expose() != expected.persistent_ref.expose() {
            return Err(unavailable());
        }
        copy_data(attribute(value, kSecValueData), MAX_BYTES)
    }
}

pub(super) struct Keychain;

impl Backend for Keychain {
    fn find(&mut self, scope: &Scope) -> Result<Vec<Entry>> {
        let query = query(scope)?;
        // SAFETY: the result of this metadata-only, limit-two search is an array.
        unsafe {
            let limit: i32 = 2;
            let limit = Owned::new(
                CFNumberCreate(
                    ptr::null(),
                    kCFNumberSInt32Type,
                    (&limit as *const i32).cast(),
                )
                .cast(),
            )?;
            set(&query, kSecMatchLimit, limit.0);
            set(
                &query,
                kSecAttrSynchronizable,
                kSecAttrSynchronizableAny.cast(),
            );
            set(&query, kSecReturnAttributes, kCFBooleanTrue.cast());
            set(&query, kSecReturnPersistentRef, kCFBooleanTrue.cast());
            let Some(output) = search(&query)? else {
                return Ok(Vec::new());
            };
            if CFGetTypeID(output.0) != CFArrayGetTypeID() {
                return Err(unavailable());
            }
            match CFArrayGetCount(output.0.cast()) {
                1 => Ok(vec![decode_entry(
                    CFArrayGetValueAtIndex(output.0.cast(), 0),
                    scope,
                )?]),
                2.. => Err(conflict()),
                _ => Err(unavailable()),
            }
        }
    }

    fn read(&mut self, scope: &Scope, entry: &Entry) -> Result<Option<SecretBytes>> {
        let query = protected_query(scope, Some(entry))?;
        // SAFETY: this exact one-item result requests attributes and data together.
        unsafe {
            set(&query, kSecMatchLimit, kSecMatchLimitOne.cast());
            set(&query, kSecReturnAttributes, kCFBooleanTrue.cast());
            set(&query, kSecReturnPersistentRef, kCFBooleanTrue.cast());
            set(&query, kSecReturnData, kCFBooleanTrue.cast());
            let Some(output) = search(&query)? else {
                return Ok(None);
            };
            Ok(Some(decode_secret(output.0, scope, entry)?))
        }
    }

    fn change(
        &mut self,
        scope: &Scope,
        change: Change<'_>,
        operation: &Operation,
        dispatched: &mut bool,
    ) -> Result<()> {
        let entry = match &change {
            Change::Add(_) => None,
            Change::Update(entry, _) | Change::Delete(entry) => Some(*entry),
        };
        let query = if entry.is_none() {
            add_attributes(scope)?
        } else {
            protected_query(scope, entry)?
        };
        let update = dictionary()?;
        // SAFETY: prepare all objects before marking dispatch; OS calls retain no
        // borrowed Rust data. Update/delete queries contain no return-result keys.
        unsafe {
            if let Change::Add(secret) | Change::Update(_, secret) = &change {
                let bytes = data(secret.expose())?;
                let target = if matches!(&change, Change::Add(_)) {
                    &query
                } else {
                    &update
                };
                set(target, kSecValueData, bytes.0);
            }
            before_dispatch(operation)?;
            *dispatched = true;
            let status = match change {
                Change::Add(_) => SecItemAdd(query.0.cast(), ptr::null_mut()),
                Change::Update(_, _) => SecItemUpdate(query.0.cast(), update.0.cast()),
                Change::Delete(_) => SecItemDelete(query.0.cast()),
            };
            if status == 0 || (status == NOT_FOUND && matches!(change, Change::Delete(_))) {
                Ok(())
            } else {
                Err(status_error(status))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope() -> Scope {
        Scope::new(
            Uuid::new_v4(),
            &CredentialRef::new(),
            Some("SYNTHETIC.org.homeverb.test"),
        )
        .unwrap()
    }

    fn entry() -> Entry {
        Entry {
            matches_scope: true,
            passcode_only: true,
            synchronizable: false,
            unsupported_access_control: false,
            persistent_ref: SecretBytes::new(vec![3, 1, 4]),
        }
    }

    #[test]
    fn real_cf_queries_preserve_explicit_scope_protection_and_no_prompt_policy() {
        // Construct the real backend vtable without invoking Security services.
        let _: &mut dyn Backend = &mut Keychain;
        let scope = scope();
        let expected = entry();
        let query = protected_query(&scope, Some(&expected)).unwrap();
        // SAFETY: these are live dictionaries/constants made by production constructors.
        unsafe {
            for (key, text) in [
                (kSecAttrService, &scope.service),
                (kSecAttrAccount, &scope.account),
                (kSecAttrAccessGroup, &scope.access_group),
            ] {
                let text = string(text).unwrap();
                assert!(equal(attribute(query.0, key), text.0));
            }
            assert!(equal(
                attribute(query.0, kSecClass),
                kSecClassGenericPassword.cast()
            ));
            assert!(equal(
                attribute(query.0, kSecAttrAccessible),
                kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly.cast()
            ));
            assert!(equal(
                attribute(query.0, kSecAttrSynchronizable),
                kCFBooleanFalse.cast()
            ));
            assert!(equal(
                attribute(query.0, kSecUseAuthenticationUI),
                kSecUseAuthenticationUIFail.cast()
            ));
            assert_eq!(
                copy_data(attribute(query.0, kSecValuePersistentRef), MAX_REFERENCE)
                    .unwrap()
                    .expose(),
                expected.persistent_ref.expose()
            );
            for key in [
                kSecReturnData,
                kSecReturnAttributes,
                kSecReturnPersistentRef,
                kSecValueData,
            ] {
                assert!(attribute(query.0, key).is_null());
            }
        }
    }

    #[test]
    fn real_cf_results_reject_wrong_types_foreign_scope_and_changed_references() {
        let scope = scope();
        let expected = entry();
        let output = protected_query(&scope, Some(&expected)).unwrap();
        let secret = data(&[0, 128, 255]).unwrap();
        // SAFETY: synthetic CF results exercise production type checks, not the OS store.
        unsafe {
            set(&output, kSecValueData, secret.0);
            assert_eq!(
                decode_secret(output.0, &scope, &expected).unwrap().expose(),
                &[0, 128, 255]
            );
            assert!(decode_secret(kCFBooleanFalse.cast(), &scope, &expected).is_err());
            assert!(decode_secret(ptr::null(), &scope, &expected).is_err());
            for key in [
                kSecAttrService,
                kSecAttrAccount,
                kSecAttrAccessGroup,
                kSecAttrAccessible,
                kSecAttrSynchronizable,
                kSecValuePersistentRef,
                kSecValueData,
            ] {
                let malformed = protected_query(&scope, Some(&expected)).unwrap();
                set(&malformed, kSecValueData, secret.0);
                set(&malformed, key, kCFBooleanTrue.cast());
                assert!(decode_secret(malformed.0, &scope, &expected).is_err());
            }
            let other_ref = data(&[2, 7, 1]).unwrap();
            set(&output, kSecValuePersistentRef, other_ref.0);
            assert!(decode_secret(output.0, &scope, &expected).is_err());
        }
    }

    #[test]
    fn real_cf_synchronization_flags_accept_only_boolean_or_integer_zero() {
        use core_foundation_sys::{
            dictionary::CFDictionaryRemoveValue, number::kCFNumberFloat64Type,
        };

        let scope = scope();
        let expected = entry();
        let output = protected_query(&scope, Some(&expected)).unwrap();
        let secret = data(&[3, 1, 4]).unwrap();
        // SAFETY: all values are owned CF objects retained by the synthetic result.
        unsafe {
            set(&output, kSecValueData, secret.0);
            for (value, flag) in [(kCFBooleanFalse, false), (kCFBooleanTrue, true)] {
                set(&output, kSecAttrSynchronizable, value.cast());
                assert_eq!(decode_entry(output.0, &scope).unwrap().synchronizable, flag);
                assert_eq!(decode_secret(output.0, &scope, &expected).is_ok(), !flag);
            }
            for flag in [i64::MIN, -1, 0, 1, 2, i64::MAX] {
                let value = Owned::new(
                    CFNumberCreate(
                        ptr::null(),
                        kCFNumberSInt64Type,
                        (&flag as *const i64).cast(),
                    )
                    .cast(),
                )
                .unwrap();
                set(&output, kSecAttrSynchronizable, value.0);
                let decoded = decode_entry(output.0, &scope);
                assert_eq!(decoded.is_ok(), matches!(flag, 0 | 1));
                if let Ok(decoded) = decoded {
                    assert_eq!(decoded.synchronizable, flag == 1);
                }
                assert_eq!(
                    decode_secret(output.0, &scope, &expected).is_ok(),
                    flag == 0
                );
            }
            for flag in [0.0, 1.0, 0.5, f64::NAN, f64::INFINITY] {
                let value = Owned::new(
                    CFNumberCreate(
                        ptr::null(),
                        kCFNumberFloat64Type,
                        (&flag as *const f64).cast(),
                    )
                    .cast(),
                )
                .unwrap();
                set(&output, kSecAttrSynchronizable, value.0);
                assert!(decode_entry(output.0, &scope).is_err());
            }
            for value in [
                string("0").unwrap(),
                string("false").unwrap(),
                data(&[]).unwrap(),
            ] {
                set(&output, kSecAttrSynchronizable, value.0);
                assert!(decode_entry(output.0, &scope).is_err());
            }
            CFDictionaryRemoveValue(output.0.cast_mut().cast(), kSecAttrSynchronizable.cast());
            assert!(decode_entry(output.0, &scope).is_err());
        }
    }

    #[test]
    fn real_cf_data_obeys_copy_limits_and_refuses_unsupported_access_control_objects() {
        let scope = scope();
        let expected = entry();
        let output = protected_query(&scope, Some(&expected)).unwrap();
        // SAFETY: no borrowed pointer outlives its retained synthetic CF object.
        unsafe {
            let empty = data(&[]).unwrap();
            assert!(copy_data(empty.0, MAX_BYTES).unwrap().expose().is_empty());
            let oversized = data(&vec![7; MAX_BYTES + 1]).unwrap();
            assert!(copy_data(oversized.0, MAX_BYTES).is_err());
            set(&output, kSecValueData, empty.0);
            set(&output, kSecAttrAccessControl, kCFBooleanTrue.cast());
            assert!(decode_secret(output.0, &scope, &expected).is_err());
        }
    }

    #[test]
    fn real_cf_access_control_admits_only_the_explicit_passcode_policy() {
        let scope = scope();
        let expected = entry();
        let output = protected_query(&scope, Some(&expected)).unwrap();
        let attributes = add_attributes(&scope).unwrap();
        let secret = data(&[3, 1, 4]).unwrap();
        // SAFETY: these are real policy objects and synthetic dictionaries only;
        // no Keychain access, prompt or device authentication is requested.
        unsafe {
            let policy = attribute(attributes.0, kSecAttrAccessControl);
            assert_eq!(CFGetTypeID(policy), SecAccessControlGetTypeID());
            assert!(!unsupported_access_control(policy).unwrap());
            assert!(attribute(attributes.0, kSecAttrAccessible).is_null());
            assert!(equal(
                attribute(attributes.0, kSecAttrSynchronizable),
                kCFBooleanFalse.cast()
            ));
            for key in [
                kSecAttrService,
                kSecAttrAccount,
                kSecAttrAccessGroup,
                kSecUseAuthenticationUI,
            ] {
                assert!(equal(
                    attribute(attributes.0, key),
                    attribute(output.0, key)
                ));
            }
            set(&output, kSecValueData, secret.0);
            set(&output, kSecAttrAccessControl, policy);
            assert_eq!(
                decode_secret(output.0, &scope, &expected).unwrap().expose(),
                &[3, 1, 4]
            );

            for (protection, flags) in [
                (kSecAttrAccessibleWhenUnlocked.cast(), 0),
                (kSecAttrAccessibleAfterFirstUnlock.cast(), 0),
                // Public kSecAccessControlUserPresence; the protection still matches.
                (kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly.cast(), 1),
            ] {
                let incompatible = Owned::new(SecAccessControlCreateWithFlags(
                    ptr::null(),
                    protection,
                    flags,
                    ptr::null_mut(),
                ))
                .unwrap();
                set(&output, kSecAttrAccessControl, incompatible.0);
                assert!(decode_secret(output.0, &scope, &expected).is_err());
            }
        }
    }
}
