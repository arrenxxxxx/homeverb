package org.homeverb;

import java.io.ByteArrayOutputStream;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.nio.channels.FileChannel;
import java.nio.channels.FileLock;
import java.nio.channels.OverlappingFileLockException;
import java.nio.charset.StandardCharsets;
import java.nio.file.DirectoryStream;
import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.nio.file.StandardOpenOption;
import java.security.GeneralSecurityException;
import java.util.Arrays;
import javax.crypto.Cipher;
import javax.crypto.SecretKey;
import javax.crypto.spec.GCMParameterSpec;

/** Portable ciphertext engine. Android supplies nonexportable Keystore keys. */
final class EncryptedEntries {
    static final int MAX_BYTES = 1_048_576;
    private static final int OVERHEAD = 32;
    private static final byte[] MAGIC = {'H', 'V', 'S', '1'};
    private static final String UUID = "[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}";

    interface Keys {
        SecretKey find(String profile) throws GeneralSecurityException;
        SecretKey create(String profile) throws GeneralSecurityException;
    }

    private final Path root;
    private final Keys keys;

    EncryptedEntries(Path root, Keys keys) {
        this.root = root;
        this.keys = keys;
    }

    CredentialResult run(int action, String profile, String reference, byte[] input) {
        boolean mutationStarted = false;
        Path temporary = null;
        try {
            if (action < 0 || action > 2 || !profile.matches(UUID) || !reference.matches(UUID)
                    || (action == 1 && (input == null || input.length > MAX_BYTES))
                    || (action != 1 && input != null)) {
                return result(CredentialResult.UNAVAILABLE);
            }
            if (!Files.exists(root, LinkOption.NOFOLLOW_LINKS)) {
                if (action != 1) return result(action == 0 ? CredentialResult.MISSING : CredentialResult.OK);
                Files.createDirectory(root);
            }
            requireDirectory(root);
            Path directory = root.resolve(profile);
            if (!Files.exists(directory, LinkOption.NOFOLLOW_LINKS)) {
                if (action != 1) return result(action == 0 ? CredentialResult.MISSING : CredentialResult.OK);
                Files.createDirectory(directory);
            }
            requireDirectory(directory);
            Path lockPath = directory.resolve("lock");
            try (FileChannel channel = FileChannel.open(lockPath, StandardOpenOption.CREATE,
                    StandardOpenOption.WRITE, LinkOption.NOFOLLOW_LINKS);
                    FileLock lock = channel.tryLock()) {
                if (lock == null) return result(CredentialResult.BUSY);
                Path entry = directory.resolve(reference + ".bin");
                boolean exists = Files.exists(entry, LinkOption.NOFOLLOW_LINKS);
                if (exists && !Files.isRegularFile(entry, LinkOption.NOFOLLOW_LINKS)) {
                    return result(CredentialResult.UNAVAILABLE);
                }
                if (action == 2) {
                    if (exists) {
                        mutationStarted = true;
                        Files.delete(entry);
                        syncDirectory(directory);
                    }
                    return result(CredentialResult.OK);
                }
                if (action == 0 && !exists) return result(CredentialResult.MISSING);
                SecretKey key = keys.find(profile);
                if (key == null) {
                    if (action == 0 || containsEntries(directory)) return result(CredentialResult.UNAVAILABLE);
                    key = keys.create(profile);
                }
                if (key == null) return result(CredentialResult.UNAVAILABLE);
                byte[] aad = ("homeverb:1:" + profile + ":" + reference).getBytes(StandardCharsets.US_ASCII);
                Cipher cipher = Cipher.getInstance("AES/GCM/NoPadding");
                if (action == 0) {
                    byte[] stored = readBounded(entry);
                    if (stored.length < OVERHEAD || !Arrays.equals(MAGIC, Arrays.copyOf(stored, 4))) {
                        return result(CredentialResult.UNAVAILABLE);
                    }
                    cipher.init(Cipher.DECRYPT_MODE, key, new GCMParameterSpec(128, stored, 4, 12));
                    cipher.updateAAD(aad);
                    return new CredentialResult(CredentialResult.OK,
                            cipher.doFinal(stored, 16, stored.length - 16));
                }
                cipher.init(Cipher.ENCRYPT_MODE, key);
                cipher.updateAAD(aad);
                byte[] nonce = cipher.getIV();
                if (nonce.length != 12) return result(CredentialResult.UNAVAILABLE);
                byte[] encrypted = cipher.doFinal(input);
                temporary = Files.createTempFile(directory, reference + "-", ".tmp");
                try (FileOutputStream output = new FileOutputStream(temporary.toFile())) {
                    output.write(MAGIC);
                    output.write(nonce);
                    output.write(encrypted);
                    output.getFD().sync();
                }
                mutationStarted = true;
                Files.move(temporary, entry, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING);
                syncDirectory(directory);
                return result(CredentialResult.OK);
            }
        } catch (OverlappingFileLockException busy) {
            return result(CredentialResult.BUSY);
        } catch (Exception failure) {
            return result(mutationStarted ? CredentialResult.WRITE_UNKNOWN : CredentialResult.UNAVAILABLE);
        } finally {
            if (temporary != null) {
                try { Files.deleteIfExists(temporary); } catch (Exception ignored) { /* Ciphertext only. */ }
            }
        }
    }

    private static CredentialResult result(int status) {
        return new CredentialResult(status, null);
    }

    private static void requireDirectory(Path path) throws GeneralSecurityException {
        if (!Files.isDirectory(path, LinkOption.NOFOLLOW_LINKS)) throw new GeneralSecurityException();
    }

    private static boolean containsEntries(Path path) throws Exception {
        try (DirectoryStream<Path> entries = Files.newDirectoryStream(path)) {
            for (Path entry : entries) {
                if (!entry.getFileName().toString().equals("lock")) return true;
            }
        }
        return false;
    }

    private static byte[] readBounded(Path path) throws Exception {
        if (Files.size(path) > MAX_BYTES + OVERHEAD) throw new GeneralSecurityException();
        try (InputStream input = Files.newInputStream(path, LinkOption.NOFOLLOW_LINKS);
                ByteArrayOutputStream output = new ByteArrayOutputStream()) {
            byte[] buffer = new byte[8192];
            int count;
            while ((count = input.read(buffer)) != -1) {
                if (output.size() + count > MAX_BYTES + OVERHEAD) throw new GeneralSecurityException();
                output.write(buffer, 0, count);
            }
            return output.toByteArray();
        }
    }

    private static void syncDirectory(Path path) throws Exception {
        try (FileChannel directory = FileChannel.open(path, StandardOpenOption.READ)) {
            directory.force(true);
        }
    }
}
