package dev.homeverb.homeverb_mobile

import android.app.Application
import org.homeverb.NativeCredentials

class HomeVerbApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        try {
            System.loadLibrary("homeverb_ffi")
            // This installs the backend only; Core checks storage on each operation.
            NativeCredentials.install(applicationContext)
        } catch (_: LinkageError) {
            // Dart reports the missing Core; credentials never fall back to plaintext.
        }
    }
}
