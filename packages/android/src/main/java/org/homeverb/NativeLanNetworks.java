package org.homeverb;

import android.Manifest;
import android.content.Context;
import android.content.pm.PackageManager;
import android.net.ConnectivityManager;
import android.net.LinkAddress;
import android.net.LinkProperties;
import android.net.Network;
import android.net.NetworkCapabilities;
import android.os.SystemClock;
import java.net.Inet4Address;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.function.LongSupplier;

/** Foreground Android metadata only. This does not grant socket access or scan consent. */
public final class NativeLanNetworks implements AutoCloseable {
    private final Source source;
    private final LongSupplier clock;
    private boolean foreground;
    private boolean closed;
    private long generation;

    public NativeLanNetworks(Context context) {
        this(new AndroidSource(context), SystemClock::elapsedRealtime);
    }

    NativeLanNetworks(Source source, LongSupplier clock) {
        this.source = source;
        this.clock = clock;
    }

    public synchronized void setForeground(boolean active) {
        foreground = active && !closed;
        if (!active) generation++;
    }

    @Override public synchronized void close() {
        closed = true;
        foreground = false;
        generation++;
    }

    /** Call on a worker after explicit foreground interaction, never on a timer. */
    public List<Selection> snapshot() throws Failure {
        long expected;
        synchronized (this) { checkForeground(); expected = generation; }
        List<Link> links = read();
        synchronized (this) {
            checkForeground();
            if (expected != generation) throw new Failure("LAN_NETWORK_CHANGED");
            long created = clock.getAsLong();
            List<Selection> choices = new ArrayList<>();
            for (Link link : links) choices.add(new Selection(this, link, expected, created));
            return Collections.unmodifiableList(choices);
        }
    }

    /** Recheck before native preparation and confirmation. Socket permission remains separate. */
    public void requireCurrent(Selection selection) throws Failure {
        synchronized (this) { checkSelection(selection); }
        List<Link> links = read();
        synchronized (this) {
            checkSelection(selection);
            for (Link link : links) if (link.same(selection.link)) return;
            throw new Failure("LAN_NETWORK_CHANGED");
        }
    }

    private void checkForeground() throws Failure {
        if (closed) throw new Failure("HOST_CLOSED");
        if (!foreground) throw new Failure("HOST_BACKGROUND");
    }

    private void checkSelection(Selection selection) throws Failure {
        checkForeground();
        long now = clock.getAsLong();
        if (selection == null || selection.owner != this || selection.generation != generation
                || now < selection.created || now - selection.created >= 60000) {
            throw new Failure("LAN_NETWORK_CHANGED");
        }
    }

    private List<Link> read() throws Failure {
        try {
            Map<Integer, Link> unique = new LinkedHashMap<>();
            Set<Integer> ambiguous = new HashSet<>();
            for (Link link : source.read()) {
                Link previous = unique.put(link.address, link);
                if (previous != null && !previous.same(link)) ambiguous.add(link.address);
            }
            List<Link> result = new ArrayList<>();
            for (Link link : unique.values()) {
                if (!ambiguous.contains(link.address) && link.eligible()) result.add(link);
            }
            return result;
        } catch (SecurityException denied) {
            throw new Failure("LAN_NETWORK_PERMISSION_REQUIRED");
        } catch (RuntimeException unavailable) {
            throw new Failure("LAN_NETWORK_UNAVAILABLE");
        }
    }

    @Override public String toString() { return "NativeLanNetworks([REDACTED])"; }

    /** Opaque instance-owned selection. Never serialize, persist or log its display fields. */
    public static final class Selection {
        private final NativeLanNetworks owner;
        private final Link link;
        private final long generation;
        private final long created;
        private Selection(NativeLanNetworks owner, Link link, long generation, long created) {
            this.owner = owner; this.link = link; this.generation = generation; this.created = created;
        }
        public long getLocalAddress() { return Integer.toUnsignedLong(link.address); }
        public int getPrefixLength() { return link.prefix; }
        public String getInterfaceName() { return link.name; }
        public String getTransport() { return link.transport; }
        public String getAddressText() {
            return (link.address >>> 24) + "." + ((link.address >>> 16) & 255) + "."
                    + ((link.address >>> 8) & 255) + "." + (link.address & 255);
        }
        @Override public String toString() { return "Selection([REDACTED])"; }
    }

    public static final class Failure extends Exception {
        private static final long serialVersionUID = 1L;
        private final String code;
        Failure(String code) { super(code); this.code = code; }
        public String getCode() { return code; }
    }

    // Package-private injection supports fictional JVM tests; production uses AndroidSource only.
    interface Source { List<Link> read() throws Failure; }

    static final class Link {
        final long network;
        final String name;
        final int address;
        final int prefix;
        final String transport;
        Link(long network, String name, int address, int prefix, String transport) {
            this.network = network; this.name = name; this.address = address;
            this.prefix = prefix; this.transport = transport;
        }
        boolean same(Link other) {
            return network == other.network && address == other.address && prefix == other.prefix
                    && name.equals(other.name) && transport.equals(other.transport);
        }
        boolean eligible() {
            if (network == 0 || name == null || name.isEmpty() || prefix < 8 || prefix > 30
                    || !("wifi".equals(transport) || "ethernet".equals(transport))) return false;
            int mask = -1 << (32 - prefix);
            int first = address & mask;
            int last = first | ~mask;
            return local(first) && local(last) && address != first && address != last;
        }
        private static boolean local(int address) {
            return (address >>> 24) == 10 || (address >>> 20) == 0xac1
                    || (address >>> 16) == 0xc0a8 || (address >>> 16) == 0xa9fe;
        }
    }

    private static final class AndroidSource implements Source {
        private final Context context;
        AndroidSource(Context context) {
            if (context == null || context.getApplicationContext() == null) throw new IllegalArgumentException();
            this.context = context.getApplicationContext();
        }
        // Snapshot enumeration is explicit and foreground-bound; callbacks would retain listeners.
        @SuppressWarnings("deprecation")
        public List<Link> read() throws Failure {
            if (context.checkSelfPermission(Manifest.permission.ACCESS_NETWORK_STATE) != PackageManager.PERMISSION_GRANTED) {
                throw new Failure("LAN_NETWORK_PERMISSION_REQUIRED");
            }
            ConnectivityManager manager = (ConnectivityManager) context.getSystemService(Context.CONNECTIVITY_SERVICE);
            if (manager == null) throw new Failure("LAN_NETWORK_UNAVAILABLE");
            Network active = manager.getActiveNetwork();
            if (active != null) {
                NetworkCapabilities capabilities = manager.getNetworkCapabilities(active);
                if (capabilities == null) throw new Failure("LAN_NETWORK_UNAVAILABLE");
                if (capabilities.hasTransport(NetworkCapabilities.TRANSPORT_VPN)
                        || !capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN)) {
                    throw new Failure("LAN_NETWORK_UNSUPPORTED");
                }
            }
            List<Link> links = new ArrayList<>();
            for (Network network : manager.getAllNetworks()) {
                NetworkCapabilities capabilities = manager.getNetworkCapabilities(network);
                LinkProperties properties = manager.getLinkProperties(network);
                if (capabilities == null || properties == null || properties.getInterfaceName() == null) continue;
                boolean wifi = capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI);
                boolean ethernet = capabilities.hasTransport(NetworkCapabilities.TRANSPORT_ETHERNET);
                boolean direct = capabilities.hasCapability(NetworkCapabilities.NET_CAPABILITY_NOT_VPN)
                        && !capabilities.hasTransport(NetworkCapabilities.TRANSPORT_VPN)
                        && !capabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR);
                String transport = direct && wifi != ethernet ? (wifi ? "wifi" : "ethernet") : "unsupported";
                for (LinkAddress address : properties.getLinkAddresses()) {
                    if (!(address.getAddress() instanceof Inet4Address)) continue;
                    byte[] bytes = address.getAddress().getAddress();
                    int numeric = 0;
                    for (byte value : bytes) numeric = (numeric << 8) | (value & 255);
                    links.add(new Link(network.getNetworkHandle(), properties.getInterfaceName(),
                            numeric, address.getPrefixLength(), transport));
                }
            }
            return links;
        }
    }
}
