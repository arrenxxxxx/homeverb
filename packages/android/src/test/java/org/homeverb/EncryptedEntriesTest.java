package org.homeverb;

import java.nio.channels.FileChannel;
import java.nio.channels.FileLock;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.nio.file.StandardOpenOption;
import java.security.GeneralSecurityException;
import java.util.Arrays;
import java.util.Comparator;
import java.util.HashMap;
import java.util.Map;
import java.util.UUID;
import javax.crypto.KeyGenerator;
import javax.crypto.SecretKey;

public final class EncryptedEntriesTest {
    private static int checks;
    private static void check(boolean value) {
        checks++;
        if (!value) throw new AssertionError("Encrypted entry assertion " + checks);
    }
    private static final class Keys implements EncryptedEntries.Keys {
        final Map<String, SecretKey> values = new HashMap<>();
        int created;
        boolean unavailable;
        public SecretKey find(String profile) throws GeneralSecurityException {
            if (unavailable) throw new GeneralSecurityException("FICTIONAL-PRIVATE-ERROR");
            return values.get(profile);
        }
        public SecretKey create(String profile) throws GeneralSecurityException {
            created++;
            KeyGenerator generator = KeyGenerator.getInstance("AES");
            generator.init(256);
            SecretKey key = generator.generateKey();
            values.put(profile, key);
            return key;
        }
    }

    public static void main(String[] arguments) throws Exception {
        Path parent = Files.createTempDirectory("homeverb-android-cipher-");
        try {
            Path root = parent.resolve("entries");
            Keys keys = new Keys();
            EncryptedEntries store = new EncryptedEntries(root, keys);
            String profile = UUID.randomUUID().toString();
            String reference = UUID.randomUUID().toString();
            byte[] secret = "FICTIONAL-PRIVATE-SESSION-AND-BINDINGS".getBytes(StandardCharsets.UTF_8);
            check(store.run(0, profile, reference, null).status == CredentialResult.MISSING);
            check(!Files.exists(root));
            check(store.run(1, profile, reference, secret).status == CredentialResult.OK);
            Path file = root.resolve(profile).resolve(reference + ".bin");
            byte[] first = Files.readAllBytes(file);
            check(!new String(first, StandardCharsets.ISO_8859_1).contains("FICTIONAL-PRIVATE"));
            check(Arrays.equals(store.run(0, profile, reference, null).value, secret));
            check(Arrays.equals(new EncryptedEntries(root, keys).run(0, profile, reference, null).value, secret));
            check(store.run(1, profile, reference, secret).status == CredentialResult.OK);
            byte[] second = Files.readAllBytes(file);
            check(!Arrays.equals(first, second));
            check(keys.created == 1);

            String another = UUID.randomUUID().toString();
            check(store.run(1, profile, another, secret).status == CredentialResult.OK);
            Path otherFile = root.resolve(profile).resolve(another + ".bin");
            Files.copy(file, otherFile, StandardCopyOption.REPLACE_EXISTING);
            check(store.run(0, profile, another, null).status == CredentialResult.UNAVAILABLE);
            String otherProfile = UUID.randomUUID().toString();
            keys.values.put(otherProfile, keys.values.get(profile));
            check(store.run(1, otherProfile, reference, secret).status == CredentialResult.OK);
            Path otherProfileFile = root.resolve(otherProfile).resolve(reference + ".bin");
            Files.copy(file, otherProfileFile, StandardCopyOption.REPLACE_EXISTING);
            check(store.run(0, otherProfile, reference, null).status == CredentialResult.UNAVAILABLE);

            for (int offset : new int[] {0, 4, second.length - 1}) {
                byte[] broken = second.clone();
                broken[offset] ^= 1;
                Files.write(file, broken);
                check(store.run(0, profile, reference, null).status == CredentialResult.UNAVAILABLE);
            }
            Files.write(file, second);
            keys.unavailable = true;
            check(store.run(1, profile, reference, secret).status == CredentialResult.UNAVAILABLE);
            check(Arrays.equals(Files.readAllBytes(file), second));
            keys.unavailable = false;
            keys.values.clear();
            check(store.run(0, profile, reference, null).status == CredentialResult.UNAVAILABLE);
            check(store.run(1, profile, reference, secret).status == CredentialResult.UNAVAILABLE);
            check(keys.created == 1);
            check(Arrays.equals(Files.readAllBytes(file), second));

            for (String invalid : new String[] {"../outside", "1-1-4-8-1", profile.toUpperCase(), "00000000-0000-0000-0000-000000000000"}) {
                check(store.run(1, invalid, reference, secret).status == CredentialResult.UNAVAILABLE);
                check(store.run(1, profile, invalid, secret).status == CredentialResult.UNAVAILABLE);
            }
            check(store.run(3, profile, reference, null).status == CredentialResult.UNAVAILABLE);
            check(store.run(0, profile, reference, secret).status == CredentialResult.UNAVAILABLE);
            check(store.run(1, profile, reference, null).status == CredentialResult.UNAVAILABLE);
            try (FileChannel channel = FileChannel.open(root.resolve(profile).resolve("lock"), StandardOpenOption.WRITE);
                    FileLock lock = channel.lock()) {
                check(lock.isValid());
                check(store.run(2, profile, reference, null).status == CredentialResult.BUSY);
            }
            Path outside = parent.resolve("outside");
            Files.write(outside, secret);
            Files.delete(otherFile);
            Files.createSymbolicLink(otherFile, outside);
            check(store.run(0, profile, another, null).status == CredentialResult.UNAVAILABLE);
            check(store.run(1, profile, another, secret).status == CredentialResult.UNAVAILABLE);
            check(store.run(2, profile, another, null).status == CredentialResult.UNAVAILABLE);
            check(Arrays.equals(Files.readAllBytes(outside), secret));
            Path linkedRoot = parent.resolve("linked");
            Files.createSymbolicLink(linkedRoot, root);
            check(new EncryptedEntries(linkedRoot, keys).run(0, profile, reference, null).status == CredentialResult.UNAVAILABLE);

            check(store.run(2, profile, reference, null).status == CredentialResult.OK);
            check(store.run(2, profile, reference, null).status == CredentialResult.OK);
            check(store.run(0, profile, reference, null).status == CredentialResult.MISSING);
            String boundedProfile = UUID.randomUUID().toString();
            check(store.run(1, boundedProfile, reference, new byte[EncryptedEntries.MAX_BYTES]).status == CredentialResult.OK);
            check(store.run(0, boundedProfile, reference, null).value.length == EncryptedEntries.MAX_BYTES);
            check(store.run(1, boundedProfile, reference, new byte[EncryptedEntries.MAX_BYTES + 1]).status == CredentialResult.UNAVAILABLE);
            Path boundedFile = root.resolve(boundedProfile).resolve(reference + ".bin");
            Files.write(boundedFile, new byte[EncryptedEntries.MAX_BYTES + 33]);
            check(store.run(0, boundedProfile, reference, null).status == CredentialResult.UNAVAILABLE);
            check(store.run(2, boundedProfile, reference, null).status == CredentialResult.OK);
            check(store.run(1, boundedProfile, reference, new byte[0]).status == CredentialResult.OK);
            check(store.run(0, boundedProfile, reference, null).value.length == 0);
            System.out.println("PASS: " + checks + " encrypted-entry assertions; synthetic JVM keys, not Android Keystore acceptance.");
        } finally {
            try (java.util.stream.Stream<Path> paths = Files.walk(parent)) {
                for (Path path : (Iterable<Path>) paths.sorted(Comparator.reverseOrder())::iterator) Files.deleteIfExists(path);
            }
        }
    }
}
