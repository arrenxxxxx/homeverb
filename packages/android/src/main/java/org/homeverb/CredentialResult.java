package org.homeverb;

/** Fixed JNI results; never carry exception text or account identifiers. */
final class CredentialResult {
    static final int OK = 0;
    static final int MISSING = 1;
    static final int UNAVAILABLE = 2;
    static final int LOCKED = 3;
    static final int BUSY = 4;
    static final int WRITE_UNKNOWN = 5;

    final int status;
    final byte[] value;

    CredentialResult(int status, byte[] value) {
        this.status = status;
        this.value = value;
    }
}
