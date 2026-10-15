package org.homeverb;

import android.app.Activity;
import android.app.Instrumentation;
import android.app.KeyguardManager;
import android.content.Context;
import android.content.SharedPreferences;
import android.database.Cursor;
import android.database.sqlite.SQLiteDatabase;
import android.os.Bundle;
import android.os.Process;
import java.lang.reflect.Constructor;
import java.lang.reflect.Method;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyStore;
import java.util.Arrays;
import java.util.UUID;
import org.json.JSONObject;
import org.json.JSONArray;

/** Isolated Android acceptance with fictional bytes and no Internet permission. */
public final class AcceptanceRunner extends Instrumentation {
    private static final byte[] FIRST = "homeverb-fictional-session-v1".getBytes(StandardCharsets.US_ASCII);
    private static final byte[] SECOND = "homeverb-fictional-session-v2".getBytes(StandardCharsets.US_ASCII);
    private Bundle arguments;
    private int assertions;
    private Object backend;
    private Method operate;
    private String nativeError = "unavailable";

    private static native String request(int action, long handle, String json);
    private static native void foreground(long handle, boolean active);
    private static native void close(long handle);
    private static native int holdLock(String path);
    private static native void releaseLock(int fd);

    @Override public void onCreate(Bundle input) {
        super.onCreate(input);
        arguments = input == null ? new Bundle() : input;
        start();
    }

    @Override public void onStart() {
        Bundle result = new Bundle();
        String phase = arguments.getString("phase", "unsecured");
        try {
            Context context = getTargetContext();
            System.loadLibrary("homeverb_ffi");
            System.loadLibrary("homeverb_acceptance");
            check(NativeCredentials.install(context));
            check(NativeCredentials.install(context));
            Constructor<NativeCredentials> constructor = NativeCredentials.class.getDeclaredConstructor(Context.class);
            constructor.setAccessible(true);
            backend = constructor.newInstance(context);
            operate = NativeCredentials.class.getDeclaredMethod("operate", int.class, String.class, String.class, byte[].class);
            operate.setAccessible(true);
            KeyguardManager keyguard = (KeyguardManager) context.getSystemService(Context.KEYGUARD_SERVICE);
            if ("unsecured".equals(phase)) {
                check(!keyguard.isDeviceSecure());
                check(store(1, UUID.randomUUID().toString(), UUID.randomUUID().toString(), FIRST).status == CredentialResult.UNAVAILABLE);
                nativeFlow(context, "SECRET_STORE_UNAVAILABLE");
            } else if ("write".equals(phase)) {
                check(keyguard.isDeviceSecure() && !keyguard.isDeviceLocked());
                write(context);
            } else if ("locked".equals(phase)) {
                check(keyguard.isDeviceSecure() && keyguard.isDeviceLocked());
                SharedPreferences state = state(context);
                check(store(0, state.getString("profile", ""), state.getString("reference", ""), null).status == CredentialResult.LOCKED);
                nativeFlow(context, "SECRET_STORE_LOCKED");
            } else if ("read".equals(phase)) {
                check(keyguard.isDeviceSecure() && !keyguard.isDeviceLocked());
                readAndClean(context);
            } else {
                throw new AssertionError();
            }
            result.putString("homeverb_result", "PASS");
            result.putInt("assertions", assertions);
            result.putString("phase", phase);
            finish(Activity.RESULT_OK, result);
        } catch (Throwable failure) {
            // Fixed output only: never expose exception messages, files or credential bytes.
            result.putString("homeverb_result", "FAIL");
            result.putInt("completed_assertions", assertions);
            result.putString("native_error", nativeError);
            finish(Activity.RESULT_CANCELED, result);
        }
    }

    private void check(boolean value) {
        if (!value) throw new AssertionError();
        assertions++;
    }

    private CredentialResult store(int action, String profile, String reference, byte[] value) throws Exception {
        byte[] input = value == null ? null : value.clone();
        CredentialResult result = (CredentialResult) operate.invoke(backend, action, profile, reference, input);
        if (input != null) check(Arrays.equals(input, new byte[input.length]));
        return result;
    }

    private SharedPreferences state(Context context) {
        return context.getSharedPreferences("acceptance", Context.MODE_PRIVATE);
    }

    private Path entry(Context context, String profile, String reference) {
        return context.getNoBackupFilesDir().toPath().resolve("homeverb-credentials")
            .resolve(profile).resolve(reference + ".bin");
    }

    private void write(Context context) throws Exception {
        SharedPreferences state = state(context);
        check(!state.contains("profile"));
        String profile = UUID.randomUUID().toString();
        String reference = UUID.randomUUID().toString();
        // Persist only local test references, so another process can verify the same entry.
        check(state.edit().putString("profile", profile).putString("reference", reference)
            .putInt("writer_pid", Process.myPid()).commit());
        check(store(1, profile, reference, FIRST).status == CredentialResult.OK);
        check(Arrays.equals(store(0, profile, reference, null).value, FIRST));
        check(store(0, UUID.randomUUID().toString(), reference, null).status == CredentialResult.MISSING);
        KeyStore keys = KeyStore.getInstance("AndroidKeyStore");
        keys.load(null);
        check(keys.getKey("homeverb." + profile, null).getEncoded() == null);
        byte[] encrypted = Files.readAllBytes(entry(context, profile, reference));
        check(encrypted.length > FIRST.length);
        check(!new String(encrypted, StandardCharsets.ISO_8859_1).contains(new String(FIRST, StandardCharsets.US_ASCII)));
        seedNativeAccount(context);
    }

    private void readAndClean(Context context) throws Exception {
        SharedPreferences state = state(context);
        String profile = state.getString("profile", "");
        String reference = state.getString("reference", "");
        check(state.getInt("writer_pid", Process.myPid()) != Process.myPid());
        check(Arrays.equals(store(0, profile, reference, null).value, FIRST));
        Path entry = entry(context, profile, reference);
        byte[] previous = Files.readAllBytes(entry);
        check(store(1, profile, reference, SECOND).status == CredentialResult.OK);
        byte[] replacement = Files.readAllBytes(entry);
        check(!Arrays.equals(Arrays.copyOfRange(previous, 4, 16), Arrays.copyOfRange(replacement, 4, 16)));
        check(Arrays.equals(store(0, profile, reference, null).value, SECOND));
        byte[] corrupt = replacement.clone();
        corrupt[corrupt.length - 1] ^= 1;
        Files.write(entry, corrupt);
        check(store(0, profile, reference, null).status == CredentialResult.UNAVAILABLE);
        check(Arrays.equals(Files.readAllBytes(entry), corrupt));
        Files.write(entry, replacement);
        KeyStore keys = KeyStore.getInstance("AndroidKeyStore");
        keys.load(null);
        keys.deleteEntry("homeverb." + profile);
        check(store(0, profile, reference, null).status == CredentialResult.UNAVAILABLE);
        check(store(1, profile, reference, FIRST).status == CredentialResult.UNAVAILABLE);
        check(!keys.containsAlias("homeverb." + profile));
        check(Arrays.equals(Files.readAllBytes(entry), replacement));
        check(store(2, profile, reference, null).status == CredentialResult.OK);
        check(store(0, profile, reference, null).status == CredentialResult.MISSING);
        check(!Files.exists(entry));
        readNativeAccount(context, state);
        check(state.edit().clear().commit());
    }

    private JSONObject accountConfig(Context context) throws Exception {
        return new JSONObject().put("schema_version", "1.0").put("adapter", "midea.meiju")
            .put("allow_mock_writes", false).put("data_dir", context.getFilesDir().toPath().resolve("meiju-fixture").toString());
    }

    private JSONArray bytes(String text) {
        JSONArray result = new JSONArray();
        for (byte value : text.getBytes(StandardCharsets.US_ASCII)) result.put(value & 255);
        return result;
    }

    private void seedNativeAccount(Context context) throws Exception {
        close(open(accountConfig(context)));
        String account = UUID.randomUUID().toString();
        String reference = UUID.randomUUID().toString();
        String profile;
        // Test-only fixture insertion into the schema created by native Core; no production import API.
        try (SQLiteDatabase db = SQLiteDatabase.openDatabase(
                context.getFilesDir().toPath().resolve("meiju-fixture/homeverb.sqlite3").toString(), null, SQLiteDatabase.OPEN_READWRITE)) {
            check(db.getVersion() == 4);
            try (Cursor rows = db.rawQuery("SELECT namespace FROM profile_namespace WHERE singleton=1", null)) {
                check(rows.moveToFirst());
                profile = rows.getString(0);
            }
            db.execSQL("INSERT INTO accounts (id,provider,region,label,credential_ref,credential_state) VALUES (?, 'midea.meiju', 'CN', 'Fictional', ?, 'stored')",
                new Object[]{account, reference});
        }
        String subject = "meiju.login-id.v1:fictional";
        JSONObject session = new JSONObject().put("version", 1).put("subject", subject)
            .put("token", "fictional-token").put("key", "synthetic-key-01");
        JSONObject payload = new JSONObject().put("version", 3).put("account_id", account).put("credential_ref", reference)
            .put("provider", "midea.meiju").put("region", "CN").put("subject", bytes(subject))
            .put("session", bytes(session.toString())).put("bindings", new JSONArray());
        check(store(1, profile, reference, payload.toString().getBytes(StandardCharsets.US_ASCII)).status == CredentialResult.OK);
        check(state(context).edit().putString("native_account", account).putString("native_profile", profile)
            .putString("native_reference", reference).commit());
    }

    private void readNativeAccount(Context context, SharedPreferences state) throws Exception {
        String account = state.getString("native_account", "");
        String profile = state.getString("native_profile", "");
        String reference = state.getString("native_reference", "");
        long handle = open(accountConfig(context));
        try {
            JSONObject inspect = new JSONObject().put("method", "inspect_account_inventory").put("account_id", account);
            check(call(handle, inspect).getJSONObject("data").getJSONArray("devices").length() == 0);
            int locked = holdLock(context.getFilesDir().toPath().resolve("meiju-fixture/locks/account-" + account).toString());
            try {
                check(locked >= 0);
                check("RESOURCE_BUSY".equals(call(handle, inspect).getJSONObject("error").getString("code")));
            } finally { releaseLock(locked); }
            check(call(handle, inspect).getJSONObject("data").getJSONArray("devices").length() == 0);
            JSONObject removal = new JSONObject().put("method", "remove_account").put("account_id", account)
                .put("expected_device_ids", new JSONArray());
            check("removed".equals(call(handle, removal).getJSONObject("data").getString("status")));
            check(!Files.exists(entry(context, profile, reference)));
            check(call(handle, new JSONObject().put("method", "list_accounts")).getJSONArray("data").length() == 0);
        } finally { close(handle); }
        KeyStore keys = KeyStore.getInstance("AndroidKeyStore");
        keys.load(null);
        keys.deleteEntry("homeverb." + profile);
    }

    private JSONObject call(long handle, JSONObject operation) throws Exception {
        JSONObject envelope = new JSONObject().put("schema_version", "1.0")
            .put("operation_id", UUID.randomUUID().toString()).put("timeout_ms", 10000).put("request", operation);
        return new JSONObject(request(1, handle, envelope.toString()));
    }

    private long open(JSONObject config) throws Exception {
        JSONObject result = new JSONObject(request(0, 0, config.toString()));
        if (!result.isNull("error")) {
            String code = result.getJSONObject("error").getString("code");
            if ("STORAGE_ERROR".equals(code) || "RESOURCE_BUSY".equals(code)) nativeError = code;
        }
        return result.getJSONObject("data").getLong("handle");
    }

    private void nativeFlow(Context context, String storageError) throws Exception {
        JSONObject config = new JSONObject().put("schema_version", "1.0")
            .put("data_dir", context.getFilesDir().toPath().resolve("mock").toString()).put("allow_mock_writes", true);
        long handle = open(config);
        try {
            int migration = holdLock(context.getFilesDir().toPath().resolve("mock/migration.lock").toString());
            try { check(migration == -1); } finally { releaseLock(migration); }
            long second = open(config);
            close(second);
            check(call(handle, new JSONObject().put("method", "get_api_info")).getJSONObject("data").has("api_version"));
            JSONArray devices = call(handle, new JSONObject().put("method", "discover")).getJSONArray("data");
            check(devices.length() == 1);
            JSONObject command = new JSONObject().put("schema_version", "1.0").put("request_id", UUID.randomUUID().toString())
                .put("device_id", devices.getJSONObject(0).getString("id")).put("component_id", "main").put("capability", "power").put("action", "set")
                .put("arguments", new JSONObject().put("state", false)).put("connection", "auto")
                .put("timeout_ms", 10000).put("wait_for", "verified");
            check("verified".equals(call(handle, new JSONObject().put("method", "execute").put("command", command)).getString("status")));
            StringBuilder name = new StringBuilder();
            for (byte value : command.getString("device_id").getBytes(StandardCharsets.UTF_8)) {
                name.append(String.format(java.util.Locale.ROOT, "%02x", value));
            }
            int locked = holdLock(context.getFilesDir().toPath().resolve("mock/locks").resolve(name.toString()).toString());
            try {
                check(locked >= 0);
                command.put("request_id", UUID.randomUUID().toString());
                check("RESOURCE_BUSY".equals(call(handle, new JSONObject().put("method", "execute").put("command", command))
                    .getJSONObject("error").getString("code")));
            } finally { releaseLock(locked); }
            check("verified".equals(call(handle, new JSONObject().put("method", "execute").put("command", command)).getString("status")));
            foreground(handle, false);
            check("HOST_BACKGROUND".equals(call(handle, new JSONObject().put("method", "discover")).getJSONObject("error").getString("code")));
            foreground(handle, true);
            check(call(handle, new JSONObject().put("method", "list_devices")).getJSONArray("data").length() == 1);
        } finally {
            close(handle);
        }
        check("HOST_CLOSED".equals(call(handle, new JSONObject().put("method", "list_devices")).getJSONObject("error").getString("code")));
        config.put("adapter", "midea.meiju").put("allow_mock_writes", false)
            .put("data_dir", context.getFilesDir().toPath().resolve("meiju").toString());
        handle = open(config);
        try {
            JSONObject metadata = new JSONObject().put("schema_version", "1.0").put("operation_id", "synthetic-login")
                .put("timeout_ms", 10000).put("label", "Fictional");
            check(storageError.equals(new JSONObject(request(2, handle, metadata.toString())).getJSONObject("error").getString("code")));
            check(call(handle, new JSONObject().put("method", "list_accounts")).getJSONArray("data").length() == 0);
        } finally {
            close(handle);
        }
    }
}
