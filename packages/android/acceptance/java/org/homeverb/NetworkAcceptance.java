package org.homeverb;

import android.Manifest;
import android.content.Context;
import android.content.pm.PackageManager;
import java.util.List;

/** Read assigned emulator networks only; this APK cannot open Internet/LAN sockets. */
final class NetworkAcceptance {
    private final AcceptanceRunner runner;
    private final Context context;
    NetworkAcceptance(AcceptanceRunner runner, Context context) {
        this.runner = runner;
        this.context = context;
    }
    private interface Work { void run() throws Exception; }
    private void error(String expected, Work work) throws Exception {
        try { work.run(); throw new AssertionError(); }
        catch (NativeLanNetworks.Failure failure) { runner.check(expected.equals(failure.getCode())); }
    }
    void run() throws Exception {
        runner.check(context.checkSelfPermission(Manifest.permission.INTERNET) == PackageManager.PERMISSION_DENIED);
        runner.check(context.checkSelfPermission(Manifest.permission.ACCESS_NETWORK_STATE) == PackageManager.PERMISSION_GRANTED);
        NativeLanNetworks networks = new NativeLanNetworks(context);
        NativeLanNetworks other = new NativeLanNetworks(context);
        try {
            error("HOST_BACKGROUND", networks::snapshot);
            networks.setForeground(true);
            other.setForeground(true);
            List<NativeLanNetworks.Selection> choices = networks.snapshot();
            runner.check(!choices.isEmpty());
            for (NativeLanNetworks.Selection selection : choices) {
                runner.check(selection.getLocalAddress() > 0 && selection.getLocalAddress() <= 0xffffffffL);
                runner.check(selection.getPrefixLength() >= 8 && selection.getPrefixLength() <= 30);
                runner.check(!selection.getInterfaceName().isEmpty());
                runner.check("wifi".equals(selection.getTransport()) || "ethernet".equals(selection.getTransport()));
                runner.check(selection.getAddressText().split("\\.").length == 4);
                runner.check("Selection([REDACTED])".equals(selection.toString()));
                networks.requireCurrent(selection);
                runner.check(true);
            }
            NativeLanNetworks.Selection selected = choices.get(0);
            error("LAN_NETWORK_CHANGED", () -> other.requireCurrent(selected));
            networks.setForeground(false);
            error("HOST_BACKGROUND", networks::snapshot);
            networks.setForeground(true);
            error("LAN_NETWORK_CHANGED", () -> networks.requireCurrent(selected));
            NativeLanNetworks.Selection fresh = networks.snapshot().get(0);
            networks.requireCurrent(fresh);
            runner.check(true);
            networks.close();
            networks.setForeground(true);
            error("HOST_CLOSED", () -> networks.requireCurrent(fresh));
            error("HOST_CLOSED", networks::snapshot);
        } finally {
            networks.close();
            other.close();
        }
    }
}
