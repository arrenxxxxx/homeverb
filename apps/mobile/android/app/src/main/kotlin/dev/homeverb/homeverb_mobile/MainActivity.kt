package dev.homeverb.homeverb_mobile

import android.app.Activity
import android.content.Intent
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {
    private val exportRequest = 5821
    private var documents: MethodChannel? = null
    private var pendingResult: MethodChannel.Result? = null
    private var pendingText: String? = null

    override fun configureFlutterEngine(flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        documents = MethodChannel(flutterEngine.dartExecutor.binaryMessenger, "dev.homeverb/document_export")
        documents?.setMethodCallHandler { call, result ->
            if (call.method != "saveDiagnostics") {
                result.notImplemented()
                return@setMethodCallHandler
            }
            if (pendingResult != null) {
                result.error("EXPORT_BUSY", "A document export is already open.", null)
                return@setMethodCallHandler
            }
            val text = call.argument<String>("text")
            if (text == null) {
                result.error("INVALID_ARGUMENT", "The previewed report is required.", null)
                return@setMethodCallHandler
            }
            pendingResult = result
            pendingText = text
            try {
                val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
                    addCategory(Intent.CATEGORY_OPENABLE)
                    type = "application/json"
                    putExtra(Intent.EXTRA_TITLE, "homeverb-diagnostics.json")
                    putExtra(Intent.EXTRA_LOCAL_ONLY, true)
                }
                startActivityForResult(intent, exportRequest)
            } catch (_: Exception) {
                pendingResult = null
                pendingText = null
                result.error("EXPORT_FAILED", "Could not open the document picker.", null)
            }
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode != exportRequest) return
        val result = pendingResult ?: return
        val text = pendingText
        pendingText = null
        if (resultCode == Activity.RESULT_CANCELED) {
            pendingResult = null
            result.success(false)
            return
        }
        val uri = data?.data
        if (resultCode != Activity.RESULT_OK || uri == null || text == null) {
            pendingResult = null
            result.error("EXPORT_FAILED", "No writable document was returned.", null)
            return
        }
        // The picker grants access to this document only; no storage permission is needed.
        Thread {
            val saved = try {
                val stream = contentResolver.openOutputStream(uri, "wt")
                    ?: throw IllegalStateException("No document stream.")
                stream.use { it.write(text.toByteArray(Charsets.UTF_8)) }
                true
            } catch (_: Exception) {
                false
            }
            runOnUiThread {
                if (pendingResult === result) {
                    pendingResult = null
                    if (saved) result.success(true)
                    else result.error("EXPORT_FAILED", "Could not finish saving the report.", null)
                }
            }
        }.start()
    }

    override fun onDestroy() {
        documents?.setMethodCallHandler(null)
        pendingResult?.error("EXPORT_INTERRUPTED", "The document export was interrupted.", null)
        pendingResult = null
        pendingText = null
        super.onDestroy()
    }
}
