use super::*;
use jni::{InitArgsBuilder, JNIVersion};
use tokio_util::sync::CancellationToken;

fn operation() -> Operation {
    Operation::new(1000, CancellationToken::new()).unwrap()
}

#[test]
fn invalid_backend_results_never_become_confirmed_mutations_or_missing_reads() {
    assert_eq!(
        decode(0, None, false).unwrap_err().code,
        "SECRET_STORE_UNAVAILABLE"
    );
    assert_eq!(
        decode(1, None, true).unwrap_err().code,
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    assert_eq!(
        decode(0, Some(SecretBytes::new(vec![1])), true)
            .unwrap_err()
            .code,
        "SECRET_STORE_WRITE_UNKNOWN"
    );
    for status in [-1, 6, 99] {
        assert_eq!(
            decode(status, None, false).unwrap_err().code,
            "SECRET_STORE_UNAVAILABLE"
        );
        assert_eq!(
            decode(status, None, true).unwrap_err().code,
            "SECRET_STORE_WRITE_UNKNOWN"
        );
    }
}

#[tokio::test]
#[ignore = "Run scripts/verify_android_store.py with a JDK and official Android API stubs."]
async fn jvm_bridge() {
    assert_eq!(
        perform(
            Uuid::new_v4(),
            &CredentialRef::new(),
            Request::Get,
            &operation()
        )
        .await
        .unwrap_err()
        .code,
        "SECRET_STORE_UNAVAILABLE"
    );
    let classes = std::env::var("HOMEVERB_ANDROID_TEST_CLASSES")
        .expect("Compiled synthetic JVM fixtures are required.");
    let args = InitArgsBuilder::new()
        .version(JNIVersion::V8)
        .option(format!("-Djava.class.path={classes}"))
        .build()
        .unwrap();
    let vm = JavaVM::new(args).unwrap();
    let mut env = vm.attach_current_thread().unwrap();
    let directory = std::env::temp_dir().join(format!("homeverb-jni-{}", Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let root = JObject::from(
        env.new_string(directory.join("entries").to_str().unwrap())
            .unwrap(),
    );
    let object = env
        .new_object(
            "org/homeverb/TestCredentials",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&root)],
        )
        .unwrap();
    let fixture = env.new_global_ref(&object).unwrap();
    assert!(install(&mut env, object));
    let repeated = env.new_local_ref(fixture.as_obj()).unwrap();
    assert!(!install(&mut env, repeated));
    let profile = Uuid::new_v4();
    let reference = CredentialRef::new();
    let secret = SecretBytes::new(b"FICTIONAL-JNI-SECRET".to_vec());
    assert!(
        perform(profile, &reference, Request::Get, &operation())
            .await
            .unwrap()
            .is_none()
    );
    perform(profile, &reference, Request::Put(&secret), &operation())
        .await
        .unwrap();
    let input = JByteArray::from(
        env.get_field(fixture.as_obj(), "lastInput", "[B")
            .unwrap()
            .l()
            .unwrap(),
    );
    assert!(
        env.convert_byte_array(input)
            .unwrap()
            .iter()
            .all(|b| *b == 0)
    );
    let value = perform(profile, &reference, Request::Get, &operation())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(value.expose(), secret.expose());
    for name in ["lastInput", "lastOutput"] {
        let value = JByteArray::from(
            env.get_field(fixture.as_obj(), name, "[B")
                .unwrap()
                .l()
                .unwrap(),
        );
        if !value.is_null() {
            assert!(
                env.convert_byte_array(&value)
                    .unwrap()
                    .iter()
                    .all(|b| *b == 0)
            );
        }
    }
    assert!(
        perform(Uuid::new_v4(), &reference, Request::Get, &operation())
            .await
            .unwrap()
            .is_none()
    );
    for (mode, expected) in [
        (2, "SECRET_STORE_UNAVAILABLE"),
        (3, "SECRET_STORE_LOCKED"),
        (4, "RESOURCE_BUSY"),
        (5, "SECRET_STORE_WRITE_UNKNOWN"),
        (6, "SECRET_STORE_WRITE_UNKNOWN"),
        (7, "SECRET_STORE_WRITE_UNKNOWN"),
        (8, "SECRET_STORE_WRITE_UNKNOWN"),
    ] {
        env.set_field(fixture.as_obj(), "mode", "I", JValue::Int(mode))
            .unwrap();
        let error = perform(profile, &reference, Request::Put(&secret), &operation())
            .await
            .unwrap_err();
        assert_eq!(error.code, expected);
        assert!(!format!("{error:?}").contains("FICTIONAL"));
        assert!(!env.exception_check().unwrap());
        let input = JByteArray::from(
            env.get_field(fixture.as_obj(), "lastInput", "[B")
                .unwrap()
                .l()
                .unwrap(),
        );
        assert!(
            env.convert_byte_array(input)
                .unwrap()
                .iter()
                .all(|b| *b == 0)
        );
    }
    env.set_field(fixture.as_obj(), "mode", "I", JValue::Int(0))
        .unwrap();
    let calls = env
        .get_field(fixture.as_obj(), "calls", "I")
        .unwrap()
        .i()
        .unwrap();
    assert_eq!(
        perform(
            profile,
            &reference,
            Request::Put(&SecretBytes::new(vec![0; MAX_BYTES + 1])),
            &operation()
        )
        .await
        .unwrap_err()
        .code,
        "INVALID_ARGUMENT"
    );
    let cancelled = operation();
    cancelled.cancellation.cancel();
    assert_eq!(
        perform(profile, &reference, Request::Delete, &cancelled)
            .await
            .unwrap_err()
            .code,
        "CANCELLED"
    );
    assert_eq!(
        env.get_field(fixture.as_obj(), "calls", "I")
            .unwrap()
            .i()
            .unwrap(),
        calls
    );
    env.set_field(fixture.as_obj(), "delay", "I", JValue::Int(50))
        .unwrap();
    for deadline in [false, true] {
        let operation = if deadline {
            Operation::new(1, CancellationToken::new()).unwrap()
        } else {
            operation()
        };
        let cancellation = operation.cancellation.clone();
        let cancel = (!deadline).then(|| {
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(5));
                cancellation.cancel();
            })
        });
        assert_eq!(
            perform(profile, &reference, Request::Put(&secret), &operation)
                .await
                .unwrap_err()
                .code,
            "SECRET_STORE_WRITE_UNKNOWN"
        );
        if let Some(cancel) = cancel {
            cancel.join().unwrap();
        }
    }
    env.set_field(fixture.as_obj(), "delay", "I", JValue::Int(0))
        .unwrap();
    assert_eq!(
        perform(profile, &reference, Request::Get, &operation())
            .await
            .unwrap()
            .unwrap()
            .expose(),
        secret.expose()
    );
    perform(profile, &reference, Request::Delete, &operation())
        .await
        .unwrap();
    assert!(
        perform(profile, &reference, Request::Get, &operation())
            .await
            .unwrap()
            .is_none()
    );
    std::fs::remove_dir_all(directory).unwrap();
}
