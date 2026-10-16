package org.homeverb;

import android.content.Context;
import android.database.Cursor;
import android.database.sqlite.SQLiteDatabase;
import android.os.SystemClock;
import java.nio.file.Path;
import java.util.UUID;
import java.util.concurrent.atomic.AtomicReference;
import org.json.JSONArray;
import org.json.JSONObject;

/** Exercises the production ABI with fictional controls, never a provider account. */
final class ControlAcceptance {
    private final AcceptanceRunner test;
    private final Path root;

    ControlAcceptance(AcceptanceRunner test, Context context) {
        this.test = test;
        root = context.getFilesDir().toPath().resolve("controls-" + UUID.randomUUID());
    }

    void run() throws Exception {
        permissions();
        approval();
        interruption();
    }

    private JSONObject config(String name, String scenario, boolean allow) throws Exception {
        return new JSONObject().put("schema_version", "1.0").put("allow_mock_writes", allow)
            .put("scenario", scenario).put("data_dir", root.resolve(name).toString());
    }

    private JSONObject query(long handle, String method) throws Exception {
        return test.call(handle, new JSONObject().put("method", method));
    }

    private JSONObject command(String capability, String action, JSONObject arguments) throws Exception {
        return new JSONObject().put("schema_version", "1.0").put("request_id", UUID.randomUUID().toString())
            .put("device_id", "dev-mock-ac").put("component_id", "main").put("capability", capability)
            .put("action", action).put("arguments", arguments).put("connection", "cloud")
            .put("timeout_ms", 10000).put("wait_for", "verified");
    }

    private void error(JSONObject response, String code) throws Exception {
        test.check(code.equals(response.getJSONObject("error").getString("code")));
    }

    private JSONObject descriptor(long handle) throws Exception {
        return test.call(handle, new JSONObject().put("method", "describe_device").put("device", "dev-mock-ac"))
            .getJSONObject("data");
    }

    private JSONObject inspection() throws Exception {
        return new JSONObject().put("method", "inspect_session_grants").put("device_id", "dev-mock-ac")
            .put("connection_id", "conn-example-cloud");
    }

    private void permissions() throws Exception {
        JSONObject config = config("permissions", "normal", true);
        long handle = test.open(config);
        JSONObject saved;
        try {
            test.check(query(handle, "get_api_info").getJSONObject("data").getJSONObject("methods")
                .has("inspect_session_grants"));
            test.check(query(handle, "discover").getJSONArray("data").length() == 1);
            saved = descriptor(handle);
            JSONObject data = test.call(handle, inspection()).getJSONObject("data");
            test.check("session_policy".equals(data.getString("source")));
            test.check("not_checked".equals(data.getString("provider_authorization")));
            JSONArray grants = data.getJSONArray("grants");
            test.check(grants.length() == 2);
            test.check("power".equals(grants.getJSONObject(0).getString("capability")));
            test.check("climate".equals(grants.getJSONObject(1).getString("capability")));
            error(test.call(handle, inspection().put("grants", grants)), "INVALID_ARGUMENT");
        } finally { AcceptanceRunner.close(handle); }
        handle = test.open(config.put("allow_mock_writes", false));
        try {
            test.check(test.call(handle, inspection()).getJSONObject("data").getJSONArray("grants").length() == 0);
            test.check(saved.toString().equals(descriptor(handle).toString()));
            JSONObject power = command("power", "set", new JSONObject().put("state", true));
            error(test.call(handle, new JSONObject().put("method", "execute").put("command", power)), "POLICY_DENIED");
            test.check(query(handle, "recent_executions").getJSONArray("data").length() == 0);
            AcceptanceRunner.foreground(handle, false);
            error(test.call(handle, inspection()), "HOST_BACKGROUND");
        } finally { AcceptanceRunner.close(handle); }
        error(test.call(handle, inspection()), "HOST_CLOSED");
    }

    private JSONObject prepare(long handle) throws Exception {
        JSONObject cycle = command("vendor.example.cycle", "start", new JSONObject().put("program", "normal"));
        JSONObject input = new JSONObject().put("schema_version", "1.0").put("operation_id", UUID.randomUUID().toString())
            .put("command", cycle);
        JSONObject preview = new JSONObject(AcceptanceRunner.request(3, handle, input.toString())).getJSONObject("data");
        JSONObject returned = preview.getJSONObject("command");
        test.check(returned.length() == cycle.length());
        java.util.Iterator<String> keys = cycle.keys();
        while (keys.hasNext()) {
            String key = keys.next();
            test.check(cycle.get(key).toString().equals(returned.get(key).toString()));
        }
        test.check("elevated".equals(preview.getJSONObject("action").getString("risk")));
        test.check("cloud".equals(preview.getJSONObject("connection").getString("transport")));
        return preview;
    }

    private JSONObject confirm(long handle, long identifier) throws Exception {
        JSONObject input = new JSONObject().put("schema_version", "1.0")
            .put("operation_id", UUID.randomUUID().toString()).put("approval_id", identifier);
        return new JSONObject(AcceptanceRunner.request(4, handle, input.toString()));
    }

    private void approval() throws Exception {
        long handle = test.open(config("approval", "approval-required", true));
        long other = test.open(config("other", "approval-required", true));
        long invalidated;
        try {
            query(handle, "discover");
            query(other, "discover");
            JSONObject preview = prepare(handle);
            JSONObject cycle = preview.getJSONObject("command");
            for (String method : new String[]{"plan", "execute"}) {
                error(test.call(handle, new JSONObject().put("method", method).put("command", cycle)), "POLICY_DENIED");
            }
            long id = preview.getLong("approval_id");
            error(confirm(other, id), "APPROVAL_INVALID");
            test.check(!AcceptanceRunner.discard(other, id));
            JSONObject result = confirm(handle, id);
            test.check("accepted".equals(result.getString("status")));
            error(result, "VERIFICATION_PENDING");
            test.check(result.getJSONArray("attempts").length() == 1);
            JSONObject receipt = test.call(handle, new JSONObject().put("method", "get_execution")
                .put("request_id", cycle.getString("request_id"))).getJSONObject("data");
            test.check(receipt.toString().equals(result.toString()));
            error(confirm(handle, id), "APPROVAL_INVALID");
            id = prepare(handle).getLong("approval_id");
            test.check(AcceptanceRunner.discard(handle, id));
            test.check(!AcceptanceRunner.discard(handle, id));
            error(confirm(handle, id), "APPROVAL_INVALID");
            id = prepare(handle).getLong("approval_id");
            AcceptanceRunner.foreground(handle, false);
            error(confirm(handle, id), "HOST_BACKGROUND");
            AcceptanceRunner.foreground(handle, true);
            error(confirm(handle, id), "APPROVAL_INVALID");
            error(test.call(handle, new JSONObject().put("method", "execute_approved").put("approval_id", id)), "INVALID_ARGUMENT");
            invalidated = prepare(handle).getLong("approval_id");
        } finally {
            AcceptanceRunner.close(handle);
            AcceptanceRunner.close(other);
        }
        error(confirm(handle, invalidated), "HOST_CLOSED");
        test.check(!AcceptanceRunner.discard(handle, invalidated));
    }

    private boolean appliancePower(Path path) throws Exception {
        try (SQLiteDatabase db = SQLiteDatabase.openDatabase(path.toString(), null, SQLiteDatabase.OPEN_READONLY);
             Cursor rows = db.rawQuery("SELECT state FROM appliance WHERE id=1", null)) {
            if (!rows.moveToFirst()) throw new AssertionError();
            return new JSONObject(rows.getString(0)).getBoolean("power.state");
        }
    }

    private void interruption() throws Exception {
        JSONObject config = config("interruption", "slow-after-send", true);
        long handle = test.open(config);
        Path appliance = root.resolve("interruption/mock-appliance.sqlite3");
        JSONObject operation;
        JSONObject uncertain;
        boolean original;
        try {
            query(handle, "discover");
            original = appliancePower(appliance);
            operation = command("power", "set", new JSONObject().put("state", !original));
            AtomicReference<JSONObject> result = new AtomicReference<>();
            AtomicReference<Throwable> failure = new AtomicReference<>();
            final long active = handle;
            final JSONObject submitted = operation;
            Thread worker = new Thread(() -> {
                try { result.set(test.call(active, new JSONObject().put("method", "execute").put("command", submitted))); }
                catch (Throwable error) { failure.set(error); }
            });
            worker.start();
            try {
                // Test-only barrier: observe the simulator commit before cancelling its acknowledgement wait.
                long deadline = SystemClock.elapsedRealtime() + 5000;
                while (appliancePower(appliance) == original && SystemClock.elapsedRealtime() < deadline && worker.isAlive()) {
                    Thread.sleep(10);
                }
                test.check(appliancePower(appliance) != original);
            } finally {
                AcceptanceRunner.foreground(handle, false);
                worker.join(5000);
            }
            test.check(!worker.isAlive() && failure.get() == null);
            uncertain = result.get();
            test.check("unknown".equals(uncertain.getString("status")));
            test.check("possibly_sent".equals(uncertain.getJSONArray("attempts").getJSONObject(0).getString("phase")));
        } finally { AcceptanceRunner.close(handle); }
        handle = test.open(config.put("scenario", "normal"));
        try {
            JSONObject receipt = test.call(handle, new JSONObject().put("method", "get_execution")
                .put("request_id", operation.getString("request_id"))).getJSONObject("data");
            test.check(receipt.toString().equals(uncertain.toString()));
            JSONObject reset = command("power", "set", new JSONObject().put("state", original));
            test.check("verified".equals(test.call(handle, new JSONObject().put("method", "execute").put("command", reset)).getString("status")));
            JSONObject replay = test.call(handle, new JSONObject().put("method", "execute").put("command", operation));
            test.check(replay.toString().equals(uncertain.toString()));
            test.check(appliancePower(appliance) == original);
        } finally { AcceptanceRunner.close(handle); }
    }
}
