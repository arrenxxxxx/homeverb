package org.homeverb;

import android.app.KeyguardManager;
import android.content.Context;
import android.os.Build;
import android.security.keystore.KeyGenParameterSpec;
import android.security.keystore.KeyProperties;
import java.nio.file.Path;
import java.security.GeneralSecurityException;
import java.security.KeyStore;
import java.util.Arrays;
import javax.crypto.KeyGenerator;
import javax.crypto.SecretKey;

/** Install from trusted Android startup after loading libhomeverb_ffi.so. */
public final class NativeCredentials {
    private static boolean installed;
    private final Path directory;
    private final KeyguardManager keyguard;

    private NativeCredentials(Context context) {
        directory = context.getNoBackupFilesDir().toPath().resolve("homeverb-credentials");
        keyguard = (KeyguardManager) context.getSystemService(Context.KEYGUARD_SERVICE);
    }

    public static synchronized boolean install(Context context) {
        if (installed) return true;
        if (Build.VERSION.SDK_INT < 28 || context == null || context.getApplicationContext() == null) return false;
        try {
            installed = nativeInstall(new NativeCredentials(context.getApplicationContext()));
        } catch (RuntimeException unavailable) {
            return false;
        }
        return installed;
    }

    private static native boolean nativeInstall(NativeCredentials backend);

    // Invoked only by Rust JNI on the calling native worker, never through a JSON channel.
    private CredentialResult operate(int action, String profile, String reference, byte[] input) {
        try {
            if (keyguard == null) return new CredentialResult(CredentialResult.UNAVAILABLE, null);
            if (!keyguard.isDeviceSecure()) return new CredentialResult(CredentialResult.UNAVAILABLE, null);
            if (keyguard.isDeviceLocked()) {
                return new CredentialResult(CredentialResult.LOCKED, null);
            }
            final KeyStore store = KeyStore.getInstance("AndroidKeyStore");
            store.load(null);
            EncryptedEntries.Keys keys = new EncryptedEntries.Keys() {
                public SecretKey find(String namespace) throws GeneralSecurityException {
                    return (SecretKey) store.getKey("homeverb." + namespace, null);
                }
                public SecretKey create(String namespace) throws GeneralSecurityException {
                    KeyGenerator generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore");
                    generator.init(new KeyGenParameterSpec.Builder("homeverb." + namespace,
                            KeyProperties.PURPOSE_ENCRYPT | KeyProperties.PURPOSE_DECRYPT)
                            .setKeySize(256)
                            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                            .setRandomizedEncryptionRequired(true)
                            .setUserAuthenticationRequired(false)
                            .setUnlockedDeviceRequired(true)
                            .build());
                    return generator.generateKey();
                }
            };
            return new EncryptedEntries(directory, keys).run(action, profile, reference, input);
        } catch (Exception failure) {
            return new CredentialResult(CredentialResult.UNAVAILABLE, null);
        } finally {
            if (input != null) Arrays.fill(input, (byte) 0);
        }
    }
}
