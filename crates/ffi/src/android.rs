use jni::{
    JNIEnv,
    objects::{JClass, JObject},
    sys::{JNI_FALSE, JNI_TRUE, jboolean},
};

/// Java startup supplies the native Android backend before any account operation.
/// # Safety
/// Called by the JVM with valid local references for this native invocation.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_org_homeverb_NativeCredentials_nativeInstall(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    backend: JObject<'_>,
) -> jboolean {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if homeverb_secrets::android::install(&mut env, backend) {
            JNI_TRUE
        } else {
            JNI_FALSE
        }
    }))
    .unwrap_or(JNI_FALSE)
}
