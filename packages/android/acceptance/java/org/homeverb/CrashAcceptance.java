package org.homeverb;

import android.content.Context;
import android.content.SharedPreferences;
import android.database.Cursor;
import android.database.sqlite.SQLiteDatabase;
import android.os.Process;
import android.os.SystemClock;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicReference;
import org.json.JSONArray;
import org.json.JSONObject;

/** Test-only cross-process write recovery; the host stops this isolated APK. */
final class CrashAcceptance {
    private final AcceptanceRunner test;
    private final String runId;
    private final Path root;
    private final Path ready;
    private final SharedPreferences state;

    CrashAcceptance(AcceptanceRunner test, Context context, String runId) {
        if (!UUID.fromString(runId).toString().equals(runId)) throw new IllegalArgumentException();
        this.test = test;
        this.runId = runId;
        root = context.getFilesDir().toPath().resolve("crash-" + runId);
        ready = context.getFilesDir().toPath().resolve("interrupted-control-ready.json");
        state = context.getSharedPreferences("interrupted-control", Context.MODE_PRIVATE);
    }

    private JSONObject config(String scenario, boolean allow) throws Exception {
        return new JSONObject().put("schema_version", "1.0").put("data_dir", root.toString())
            .put("scenario", scenario).put("allow_mock_writes", allow);
    }

    private JSONObject command(boolean power) throws Exception {
        return new JSONObject().put("schema_version", "1.0").put("request_id", UUID.randomUUID().toString())
            .put("device_id", "dev-mock-ac").put("component_id", "main").put("capability", "power")
            .put("action", "set").put("arguments", new JSONObject().put("state", power))
            .put("connection", "cloud").put("timeout_ms", 60000).put("wait_for", "verified");
    }

    private JSONObject execute(long handle, JSONObject command) throws Exception {
        JSONObject request = new JSONObject().put("schema_version", "1.0")
            .put("operation_id", UUID.randomUUID().toString()).put("timeout_ms", 60000)
            .put("request", new JSONObject().put("method", "execute").put("command", command));
        return new JSONObject(AcceptanceRunner.request(1, handle, request.toString()));
    }

    private boolean power() throws Exception {
        try (SQLiteDatabase db = SQLiteDatabase.openDatabase(root.resolve("mock-appliance.sqlite3").toString(), null, SQLiteDatabase.OPEN_READONLY);
             Cursor rows = db.rawQuery("SELECT state FROM appliance WHERE id=1", null)) {
            if (!rows.moveToFirst()) throw new AssertionError();
            return new JSONObject(rows.getString(0)).getBoolean("power.state");
        }
    }

    private JSONObject journal(String requestId, int finalized) throws Exception {
        try (SQLiteDatabase db = SQLiteDatabase.openDatabase(root.resolve("homeverb.sqlite3").toString(), null, SQLiteDatabase.OPEN_READONLY);
             Cursor rows = db.rawQuery("SELECT result,finalized FROM executions WHERE request_id=?", new String[]{requestId})) {
            test.check(rows.moveToFirst());
            test.check(rows.getInt(1) == finalized);
            JSONObject result = new JSONObject(rows.getString(0));
            test.check(!rows.moveToNext());
            return result;
        }
    }

    private boolean sameJson(Object left, Object right) throws Exception {
        if (left instanceof JSONObject && right instanceof JSONObject) {
            JSONObject a = (JSONObject) left;
            JSONObject b = (JSONObject) right;
            if (a.length() != b.length()) return false;
            java.util.Iterator<String> keys = a.keys();
            while (keys.hasNext()) {
                String key = keys.next();
                if (!b.has(key) || !sameJson(a.get(key), b.get(key))) return false;
            }
            return true;
        }
        if (left instanceof JSONArray && right instanceof JSONArray) {
            JSONArray a = (JSONArray) left;
            JSONArray b = (JSONArray) right;
            if (a.length() != b.length()) return false;
            for (int i = 0; i < a.length(); i++) {
                if (!sameJson(a.get(i), b.get(i))) return false;
            }
            return true;
        }
        return left.equals(right);
    }

    void prepare() throws Exception {
        test.check(!state.contains("run_id"));
        long handle = test.open(config("slow-after-send", true));
        Thread worker = null;
        try {
            test.check(test.call(handle, new JSONObject().put("method", "discover")).getJSONArray("data").length() == 1);
            boolean original = power();
            JSONObject command = command(!original);
            String requestId = command.getString("request_id");
            test.check(state.edit().putString("run_id", runId).putInt("pid", Process.myPid())
                .putBoolean("original", original).putString("command", command.toString()).commit());
            AtomicReference<JSONObject> result = new AtomicReference<>();
            AtomicReference<Throwable> failure = new AtomicReference<>();
            worker = new Thread(() -> {
                try { result.set(execute(handle, command)); }
                catch (Throwable error) { failure.set(error); }
            });
            worker.start();
            long deadline = SystemClock.elapsedRealtime() + 5000;
            while (power() == original && SystemClock.elapsedRealtime() < deadline && worker.isAlive()) {
                Thread.sleep(10);
            }
            test.check(power() != original);
            JSONObject pending = journal(requestId, 0);
            test.check("unknown".equals(pending.getString("status")));
            test.check("possibly_sent".equals(pending.getJSONArray("attempts").getJSONObject(0).getString("phase")));
            test.check(worker.isAlive() && result.get() == null && failure.get() == null);
            JSONObject marker = new JSONObject().put("run_id", runId).put("pid", Process.myPid());
            Path temporary = ready.resolveSibling("interrupted-control-ready.tmp");
            Files.write(temporary, marker.toString().getBytes(StandardCharsets.UTF_8));
            Files.move(temporary, ready, StandardCopyOption.ATOMIC_MOVE, StandardCopyOption.REPLACE_EXISTING);
            // Only the host's force-stop ends a successful preparation. Never emit a PASS callback.
            worker.join(20000);
            throw new AssertionError();
        } finally {
            // This executes only if the host failed to stop us within the bounded window.
            AcceptanceRunner.foreground(handle, false);
            if (worker != null) worker.join(5000);
            AcceptanceRunner.close(handle);
        }
    }

    void recover() throws Exception {
        test.check(runId.equals(state.getString("run_id", "")));
        test.check(state.getInt("pid", Process.myPid()) != Process.myPid());
        JSONObject marker = new JSONObject(new String(Files.readAllBytes(ready), StandardCharsets.UTF_8));
        test.check(runId.equals(marker.getString("run_id")));
        test.check(marker.getInt("pid") == state.getInt("pid", 0));
        JSONObject command = new JSONObject(state.getString("command", ""));
        String requestId = command.getString("request_id");
        boolean original = state.getBoolean("original", false);
        test.check(power() != original);
        JSONObject pending = journal(requestId, 0);
        test.check("unknown".equals(pending.getString("status")));
        test.check("WRITE_OUTCOME_UNKNOWN".equals(pending.getJSONObject("error").getString("code")));
        long handle = test.open(config("normal", false));
        try {
            JSONObject receipt = test.call(handle, new JSONObject().put("method", "get_execution")
                .put("request_id", requestId)).getJSONObject("data");
            test.check(sameJson(receipt, pending));
            test.check(sameJson(journal(requestId, 1), pending));
            test.check(power() != original);
            JSONObject grants = test.call(handle, new JSONObject().put("method", "inspect_session_grants")
                .put("device_id", "dev-mock-ac").put("connection_id", "conn-example-cloud")).getJSONObject("data");
            test.check(grants.getJSONArray("grants").length() == 0);
        } finally { AcceptanceRunner.close(handle); }
        handle = test.open(config("normal", true));
        try {
            test.check("verified".equals(execute(handle, command(original)).getString("status")));
            test.check(sameJson(execute(handle, command), pending));
            test.check(power() == original);
            JSONObject cleared = test.call(handle, new JSONObject().put("method", "clear_execution_history"));
            test.check(cleared.getJSONObject("data").getInt("removed_receipts") == 2);
            test.check("REQUEST_RETIRED".equals(execute(handle, command).getJSONObject("error").getString("code")));
            test.check(power() == original);
        } finally { AcceptanceRunner.close(handle); }
        test.check(state.edit().clear().commit());
        test.check(Files.deleteIfExists(ready));
    }
}
