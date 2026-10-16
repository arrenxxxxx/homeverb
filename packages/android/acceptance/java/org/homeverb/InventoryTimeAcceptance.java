package org.homeverb;

import android.content.Context;
import android.database.sqlite.SQLiteDatabase;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.UUID;
import org.json.JSONObject;

/** Isolated metadata and migration checks with no credential or provider access. */
final class InventoryTimeAcceptance {
    private InventoryTimeAcceptance() {}

    static void run(AcceptanceRunner runner, Context context) throws Exception {
        Path directory = context.getFilesDir().toPath().resolve("inventory-time-" + UUID.randomUUID());
        Path database = directory.resolve("homeverb.sqlite3");
        String account = UUID.randomUUID().toString();
        JSONObject config = new JSONObject().put("schema_version", "1.0")
            .put("adapter", "midea.meiju").put("allow_mock_writes", false)
            .put("data_dir", directory.toString());
        AcceptanceRunner.close(runner.open(config));
        // Reconstruct schema 4 only inside this disposable, account-free test profile.
        try (SQLiteDatabase db = SQLiteDatabase.openDatabase(database.toString(), null, SQLiteDatabase.OPEN_READWRITE)) {
            runner.check(db.getVersion() == 5);
            db.execSQL("INSERT INTO accounts (id,provider,region,label,credential_ref,credential_state) VALUES (?, 'midea.meiju', 'CN', 'Fictional', ?, 'pending')",
                new Object[]{account, UUID.randomUUID().toString()});
            db.execSQL("ALTER TABLE accounts DROP COLUMN last_complete_inventory_at");
            db.setVersion(4);
        }
        JSONObject details = new JSONObject().put("method", "get_account").put("account_id", account);
        long handle = runner.open(config);
        try {
            JSONObject summary = runner.call(handle, details).getJSONObject("data").getJSONObject("account");
            runner.check(summary.has("last_complete_inventory_at") && summary.isNull("last_complete_inventory_at"));
            runner.check("not_checked".equals(summary.getString("authorization")));
            runner.check(account.equals(summary.getString("id")));
        } finally { AcceptanceRunner.close(handle); }
        try (java.util.stream.Stream<Path> backups = Files.list(directory.resolve("backups"))) {
            runner.check(backups.filter(path -> path.getFileName().toString().startsWith("schema-4-")).count() == 1);
        }
        String timestamp = "2026-10-16T06:00:00+00:00";
        try (SQLiteDatabase db = SQLiteDatabase.openDatabase(database.toString(), null, SQLiteDatabase.OPEN_READWRITE)) {
            runner.check(db.getVersion() == 5);
            db.execSQL("UPDATE accounts SET last_complete_inventory_at=? WHERE id=?", new Object[]{timestamp, account});
        }
        handle = runner.open(config);
        try {
            runner.check(timestamp.equals(runner.call(handle, new JSONObject().put("method", "list_accounts"))
                .getJSONArray("data").getJSONObject(0).getString("last_complete_inventory_at")));
            runner.check(timestamp.equals(runner.call(handle, new JSONObject().put("method", "rename_account")
                .put("account_id", account).put("label", "Renamed")).getJSONObject("data").getString("last_complete_inventory_at")));
            runner.check(runner.call(handle, new JSONObject().put("method", "clear_cached_state")).isNull("error"));
            runner.check(runner.call(handle, new JSONObject().put("method", "clear_execution_history")).isNull("error"));
            runner.check(timestamp.equals(runner.call(handle, details).getJSONObject("data")
                .getJSONObject("account").getString("last_complete_inventory_at")));
        } finally { AcceptanceRunner.close(handle); }
    }
}
