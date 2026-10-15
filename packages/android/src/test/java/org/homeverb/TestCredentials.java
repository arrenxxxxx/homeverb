package org.homeverb;

import java.nio.file.Paths;
import java.security.GeneralSecurityException;
import java.util.HashMap;
import java.util.Map;
import javax.crypto.KeyGenerator;
import javax.crypto.SecretKey;

/** Synthetic JNI provider; never used in the Android application package. */
public final class TestCredentials {
    public int mode;
    public int delay;
    public int calls;
    public byte[] lastInput;
    public byte[] lastOutput;
    private final EncryptedEntries entries;

    public TestCredentials(String root) {
        entries = new EncryptedEntries(Paths.get(root), new EncryptedEntries.Keys() {
            private final Map<String, SecretKey> keys = new HashMap<>();
            public SecretKey find(String profile) { return keys.get(profile); }
            public SecretKey create(String profile) throws GeneralSecurityException {
                KeyGenerator generator = KeyGenerator.getInstance("AES");
                generator.init(256);
                SecretKey key = generator.generateKey();
                keys.put(profile, key);
                return key;
            }
        });
    }

    private CredentialResult operate(int action, String profile, String reference, byte[] input) {
        calls++;
        lastInput = input;
        try {
            if (delay > 0) Thread.sleep(delay);
            if (mode == 8) throw new IllegalStateException("FICTIONAL-PRIVATE-ERROR");
            CredentialResult result;
            if (mode == 7) result = new CredentialResult(0, new byte[EncryptedEntries.MAX_BYTES + 1]);
            else if (mode == 6) result = new CredentialResult(99, null);
            else if (mode != 0) result = new CredentialResult(mode, null);
            else result = entries.run(action, profile, reference, input);
            lastOutput = result.value;
            return result;
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            return new CredentialResult(CredentialResult.UNAVAILABLE, null);
        }
    }
}
