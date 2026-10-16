package org.homeverb;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.atomic.AtomicLong;

public final class NativeLanNetworksTest {
    private static int checks;
    private static void check(boolean condition) {
        checks++;
        if (!condition) throw new AssertionError("Network selection assertion " + checks);
    }
    private interface Work { void run() throws Exception; }
    private static void error(String code, Work work) throws Exception {
        try { work.run(); throw new AssertionError("Expected " + code); }
        catch (NativeLanNetworks.Failure failure) {
            check(failure.getCode().equals(code));
            check(failure.getMessage().equals(code));
            check(failure.getCause() == null);
        }
    }
    private static final class Source implements NativeLanNetworks.Source {
        List<NativeLanNetworks.Link> links = new ArrayList<>();
        int calls;
        RuntimeException failure;
        CountDownLatch started;
        CountDownLatch resume;
        public List<NativeLanNetworks.Link> read() {
            calls++;
            if (started != null) {
                started.countDown();
                try { resume.await(); }
                catch (InterruptedException interrupted) { throw new IllegalStateException("PRIVATE-INTERRUPTION"); }
            }
            if (failure != null) throw failure;
            return links;
        }
    }
    private static NativeLanNetworks.Link link(long handle, String name, long address, int prefix, String transport) {
        return new NativeLanNetworks.Link(handle, name, (int) address, prefix, transport);
    }
    private static NativeLanNetworks.Link wifi() {
        return link(1, "wlan-fictional", 0xc0a82a02L, 24, "wifi");
    }

    public static void main(String[] arguments) throws Exception {
        Source source = new Source();
        AtomicLong time = new AtomicLong(1000);
        source.links.add(wifi());
        NativeLanNetworks networks = new NativeLanNetworks(source, time::get);
        error("HOST_BACKGROUND", networks::snapshot);
        check(source.calls == 0);
        networks.setForeground(true);
        NativeLanNetworks.Selection selection = networks.snapshot().get(0);
        check(selection.getLocalAddress() == 0xc0a82a02L);
        check(selection.getAddressText().equals("192.168.42.2"));
        check(selection.getPrefixLength() == 24);
        check(selection.getInterfaceName().equals("wlan-fictional"));
        check(selection.getTransport().equals("wifi"));
        check(selection.toString().equals("Selection([REDACTED])"));
        check(networks.toString().equals("NativeLanNetworks([REDACTED])"));
        networks.requireCurrent(selection);
        check(source.calls == 2);
        try { networks.snapshot().clear(); throw new AssertionError(); }
        catch (UnsupportedOperationException immutable) { check(true); }

        NativeLanNetworks other = new NativeLanNetworks(source, time::get);
        other.setForeground(true);
        int calls = source.calls;
        error("LAN_NETWORK_CHANGED", () -> other.requireCurrent(selection));
        check(source.calls == calls);
        error("LAN_NETWORK_CHANGED", () -> networks.requireCurrent(null));
        for (NativeLanNetworks.Link changed : Arrays.asList(
                link(2, "wlan-fictional", 0xc0a82a02L, 24, "wifi"),
                link(1, "another-interface", 0xc0a82a02L, 24, "wifi"),
                link(1, "wlan-fictional", 0xc0a82a03L, 24, "wifi"),
                link(1, "wlan-fictional", 0xc0a82a02L, 25, "wifi"),
                link(1, "wlan-fictional", 0xc0a82a02L, 24, "ethernet"))) {
            source.links = Arrays.asList(changed);
            error("LAN_NETWORK_CHANGED", () -> networks.requireCurrent(selection));
        }
        source.links = new ArrayList<>();
        error("LAN_NETWORK_CHANGED", () -> networks.requireCurrent(selection));
        source.links.add(wifi());
        time.set(61000);
        error("LAN_NETWORK_CHANGED", () -> networks.requireCurrent(selection));
        time.set(999);
        error("LAN_NETWORK_CHANGED", () -> networks.requireCurrent(selection));
        time.set(1000);

        source.links = Arrays.asList(wifi(), wifi(),
                link(2, "ethernet", 0x0a000002L, 8, "ethernet"),
                link(3, "link-local", 0xa9fe0203L, 16, "wifi"),
                link(4, "cell", 0x0a000003L, 24, "cellular"),
                link(5, "vpn", 0x0a000004L, 24, "vpn"),
                link(6, "public", 0x08080808L, 24, "wifi"),
                link(7, "network", 0xc0a82b00L, 24, "wifi"),
                link(8, "broadcast", 0xc0a82bffL, 24, "wifi"),
                link(9, "wide", 0xc0a82b02L, 8, "wifi"),
                link(10, "point", 0xc0a82c02L, 31, "wifi"),
                link(11, "loopback", 0x7f000001L, 24, "wifi"),
                link(12, "private-edge", 0xac100002L, 12, "ethernet"));
        List<NativeLanNetworks.Selection> choices = networks.snapshot();
        check(choices.size() == 4);
        for (NativeLanNetworks.Selection choice : choices) networks.requireCurrent(choice);
        source.links = Arrays.asList(wifi(), link(2, "ethernet", 0xc0a82a02L, 24, "ethernet"));
        check(networks.snapshot().isEmpty());
        error("LAN_NETWORK_CHANGED", () -> networks.requireCurrent(selection));
        source.links = Arrays.asList(wifi(), link(1, "wlan-fictional", 0xc0a82a02L, 25, "wifi"));
        check(networks.snapshot().isEmpty());

        source.failure = new SecurityException("PRIVATE-PERMISSION-DETAIL");
        error("LAN_NETWORK_PERMISSION_REQUIRED", networks::snapshot);
        source.failure = new IllegalStateException("PRIVATE-PLATFORM-DETAIL");
        error("LAN_NETWORK_UNAVAILABLE", networks::snapshot);
        source.failure = null;
        source.links = Arrays.asList(wifi());
        networks.setForeground(false);
        error("HOST_BACKGROUND", () -> networks.requireCurrent(selection));
        networks.setForeground(true);
        error("LAN_NETWORK_CHANGED", () -> networks.requireCurrent(selection));
        NativeLanNetworks.Selection fresh = networks.snapshot().get(0);
        networks.requireCurrent(fresh);

        // Lifecycle cancellation must not wait behind a synchronous platform call.
        source.started = new CountDownLatch(1);
        source.resume = new CountDownLatch(1);
        final String[] result = new String[1];
        Thread reader = new Thread(() -> {
            try { networks.snapshot(); result[0] = "UNEXPECTED_SUCCESS"; }
            catch (NativeLanNetworks.Failure failure) { result[0] = failure.getCode(); }
        });
        reader.start();
        source.started.await();
        networks.setForeground(false);
        networks.setForeground(true);
        source.resume.countDown();
        reader.join(5000);
        check(!reader.isAlive());
        check("LAN_NETWORK_CHANGED".equals(result[0]));
        source.started = null;
        networks.close();
        networks.setForeground(true);
        error("HOST_CLOSED", networks::snapshot);
        error("HOST_CLOSED", () -> networks.requireCurrent(fresh));
        other.close();
        System.out.println("PASS: " + checks + " Android network-selection assertions; fictional metadata, no sockets.");
    }
}
