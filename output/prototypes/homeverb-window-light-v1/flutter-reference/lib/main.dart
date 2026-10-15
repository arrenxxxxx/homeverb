import 'package:flutter/material.dart';

void main() => runApp(const HomeVerbReference());

@immutable
class HomeVerbSkin extends ThemeExtension<HomeVerbSkin> {
  const HomeVerbSkin({
    required this.backgroundAsset,
    required this.surface,
    required this.activeSurface,
    required this.accent,
    required this.muted,
  });

  final String backgroundAsset;
  final LinearGradient surface;
  final LinearGradient activeSurface;
  final Color accent;
  final Color muted;

  @override
  HomeVerbSkin copyWith({
    String? backgroundAsset,
    LinearGradient? surface,
    LinearGradient? activeSurface,
    Color? accent,
    Color? muted,
  }) => HomeVerbSkin(
    backgroundAsset: backgroundAsset ?? this.backgroundAsset,
    surface: surface ?? this.surface,
    activeSurface: activeSurface ?? this.activeSurface,
    accent: accent ?? this.accent,
    muted: muted ?? this.muted,
  );

  @override
  HomeVerbSkin lerp(covariant HomeVerbSkin? other, double t) {
    if (other == null) return this;
    return HomeVerbSkin(
      backgroundAsset: t < .5 ? backgroundAsset : other.backgroundAsset,
      surface: LinearGradient.lerp(surface, other.surface, t)!,
      activeSurface: LinearGradient.lerp(activeSurface, other.activeSurface, t)!,
      accent: Color.lerp(accent, other.accent, t)!,
      muted: Color.lerp(muted, other.muted, t)!,
    );
  }
}

const defaultSkinId = 'window_light';
const skinRegistry = <String, HomeVerbSkin>{
  defaultSkinId: HomeVerbSkin(
    backgroundAsset: 'assets/window-light-background.png',
    surface: LinearGradient(
      begin: Alignment.topLeft,
      end: Alignment.bottomRight,
      colors: [Color(0xfffdfaf3), Color(0xffeeefeb)],
    ),
    activeSurface: LinearGradient(
      begin: Alignment.topLeft,
      end: Alignment.bottomRight,
      colors: [Color(0xff42413e), Color(0xff2b2e2f)],
    ),
    accent: Color(0xffb94e29),
    muted: Color(0xff626566),
  ),
};

HomeVerbSkin resolveSkin(String? id) =>
    skinRegistry[id] ?? skinRegistry[defaultSkinId]!;

HomeVerbSkin skinOf(BuildContext context) =>
    Theme.of(context).extension<HomeVerbSkin>()!;

class HomeVerbReference extends StatelessWidget {
  const HomeVerbReference({super.key});

  @override
  Widget build(BuildContext context) {
    final skin = resolveSkin(defaultSkinId);
    final scheme = ColorScheme.fromSeed(
      seedColor: skin.accent,
      brightness: Brightness.light,
    ).copyWith(
      primary: skin.accent,
      onPrimary: const Color(0xfffffaf5),
      surface: const Color(0xfff6f2ea),
      onSurface: const Color(0xff292b2d),
    );
    return MaterialApp(
      title: 'HomeVerb visual reference',
      debugShowCheckedModeBanner: false,
      themeMode: ThemeMode.light,
      theme: ThemeData(
        useMaterial3: true,
        colorScheme: scheme,
        scaffoldBackgroundColor: Colors.transparent,
        extensions: [skin],
        appBarTheme: const AppBarTheme(
          backgroundColor: Colors.transparent,
          surfaceTintColor: Colors.transparent,
          elevation: 0,
        ),
        filledButtonTheme: FilledButtonThemeData(
          style: FilledButton.styleFrom(
            minimumSize: const Size(48, 48),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(15),
            ),
          ),
        ),
      ),
      home: const ReferenceShell(),
    );
  }
}

class WindowLightBackground extends StatelessWidget {
  const WindowLightBackground({required this.child, super.key});
  final Widget child;

  @override
  Widget build(BuildContext context) => Stack(
    fit: StackFit.expand,
    children: [
      ExcludeSemantics(
        child: IgnorePointer(
          child: Image.asset(
            skinOf(context).backgroundAsset,
            fit: BoxFit.cover,
            alignment: Alignment.topCenter,
          ),
        ),
      ),
      child,
    ],
  );
}

class Porcelain extends StatelessWidget {
  const Porcelain({required this.child, this.active = false, super.key});
  final Widget child;
  final bool active;

  @override
  Widget build(BuildContext context) {
    final skin = skinOf(context);
    return DecoratedBox(
      decoration: BoxDecoration(
        gradient: active ? skin.activeSurface : skin.surface,
        borderRadius: BorderRadius.circular(22),
        border: Border.all(color: Colors.white.withOpacity(active ? .12 : .7)),
        boxShadow: const [
          BoxShadow(color: Color(0x182b2c2e), blurRadius: 16, offset: Offset(0, 6)),
        ],
      ),
      child: child,
    );
  }
}

void explain(BuildContext context, String message) {
  showDialog<void>(
    context: context,
    builder: (context) => AlertDialog(
      title: const Text('Visual reference'),
      content: Text(message),
      actions: [TextButton(onPressed: () => Navigator.pop(context), child: const Text('Close'))],
    ),
  );
}

class ReferenceShell extends StatefulWidget {
  const ReferenceShell({super.key});
  @override
  State<ReferenceShell> createState() => _ReferenceShellState();
}

class _ReferenceShellState extends State<ReferenceShell> {
  int selected = 0;
  static const labels = ['Devices', 'Platforms', 'Settings'];
  static const icons = [Icons.home_outlined, Icons.layers_outlined, Icons.settings_outlined];

  @override
  Widget build(BuildContext context) => WindowLightBackground(
    child: LayoutBuilder(builder: (context, constraints) {
      final wide = constraints.maxWidth >= 800;
      final page = [const DevicesPage(), const PlatformsPage(), const SettingsPage()][selected];
      return Scaffold(
        body: SafeArea(
          child: Row(children: [
            if (wide) NavigationRail(
              selectedIndex: selected,
              onDestinationSelected: (value) => setState(() => selected = value),
              labelType: NavigationRailLabelType.all,
              destinations: List.generate(3, (i) => NavigationRailDestination(
                icon: Icon(icons[i]), label: Text(labels[i]),
              )),
            ),
            Expanded(child: page),
          ]),
        ),
        bottomNavigationBar: wide ? null : NavigationBar(
          selectedIndex: selected,
          onDestinationSelected: (value) => setState(() => selected = value),
          destinations: List.generate(3, (i) => NavigationDestination(
            icon: Icon(icons[i]), label: labels[i],
          )),
        ),
      );
    }),
  );
}

class SampleDevice {
  const SampleDevice(this.asset, this.name, this.room, this.value, this.detail, {this.active = false, this.writable = true});
  final String asset, name, room, value, detail;
  final bool active, writable;
}

const sampleDevices = [
  SampleDevice('ac', 'Bedroom AC', 'Primary Bedroom', '24°C', 'Cooling', active: true),
  SampleDevice('lamp', 'Floor lamp', 'Living Room', '65%', 'Brightness'),
  SampleDevice('tv', 'Living room TV', 'Living Room', 'Off', 'Power'),
  SampleDevice('sensor', 'Air sensor', 'Living Room', '22°C', '46% humidity', writable: false),
];

class DevicesPage extends StatefulWidget {
  const DevicesPage({super.key});
  @override
  State<DevicesPage> createState() => _DevicesPageState();
}

class _DevicesPageState extends State<DevicesPage> {
  String room = 'All';

  @override
  Widget build(BuildContext context) => LayoutBuilder(builder: (context, constraints) {
    final textScale = MediaQuery.textScalerOf(context).scale(16) / 16;
    final columns = textScale > 1.3 || constraints.maxWidth < 350 ? 1 : constraints.maxWidth >= 850 ? 3 : 2;
    final devices = sampleDevices.where((d) => room == 'All' || d.room == room).toList();
    return CustomScrollView(slivers: [
      SliverPadding(
        padding: const EdgeInsets.fromLTRB(20, 20, 20, 12),
        sliver: SliverToBoxAdapter(child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          Row(children: [
            Expanded(child: Text('HomeVerb', style: Theme.of(context).textTheme.headlineLarge)),
            IconButton(tooltip: 'Search devices', onPressed: () => explain(context, 'Search and the remaining states are specified in the complete atlas.'), icon: const Icon(Icons.search)),
            IconButton(tooltip: 'Add device', onPressed: () => explain(context, 'This isolated reference makes no discovery or appliance calls.'), icon: const Icon(Icons.add)),
          ]),
          const Text('My home'),
          const SizedBox(height: 24),
          Wrap(spacing: 8, runSpacing: 8, children: ['All', 'Living Room', 'Primary Bedroom', 'Bathroom'].map((value) => ChoiceChip(
            label: Text(value), selected: room == value,
            onSelected: (_) => setState(() => room = value),
          )).toList()),
          const SizedBox(height: 20),
          Text('${devices.length} devices · Fictional preview', style: TextStyle(color: skinOf(context).muted)),
        ])),
      ),
      if (devices.isEmpty) const SliverPadding(
        padding: EdgeInsets.all(20),
        sliver: SliverToBoxAdapter(child: Text('No devices in this room.')),
      ),
      SliverPadding(
        padding: const EdgeInsets.fromLTRB(20, 0, 20, 24),
        sliver: SliverGrid(
          gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
            crossAxisCount: columns, mainAxisSpacing: 12, crossAxisSpacing: 12,
            mainAxisExtent: 150 + 140 * textScale,
          ),
          delegate: SliverChildBuilderDelegate(
            (context, index) => DeviceCard(device: devices[index]),
            childCount: devices.length,
          ),
        ),
      ),
    ]);
  });
}

class DeviceCard extends StatelessWidget {
  const DeviceCard({required this.device, super.key});
  final SampleDevice device;

  @override
  Widget build(BuildContext context) {
    final ink = device.active ? const Color(0xfffffaf1) : const Color(0xff292b2d);
    final muted = device.active ? const Color(0xffd3d1ca) : skinOf(context).muted;
    return Porcelain(active: device.active, child: Padding(
      padding: const EdgeInsets.all(14),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Expanded(child: Semantics(
          button: true,
          label: 'Open ${device.name}, ${device.room}, ${device.value}, ${device.detail}',
          child: InkWell(
            onTap: () => explain(context, 'Device controls use native widgets and capability metadata. See the atlas for the full control states.'),
            child: ExcludeSemantics(child: SizedBox(width: double.infinity, child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(child: Center(child: Image.asset('assets/${device.asset}.png', height: 100, fit: BoxFit.contain))),
                const SizedBox(height: 12),
                Text(device.name, style: TextStyle(color: ink, fontSize: 16, fontWeight: FontWeight.w600)),
                const SizedBox(height: 4),
                Text(device.room, style: TextStyle(color: muted, fontSize: 12)),
              ],
            ))),
          ),
        )),
        const SizedBox(height: 18),
        Row(children: [
          Expanded(child: Text(device.value, style: TextStyle(color: ink, fontSize: 27, fontWeight: FontWeight.w600))),
          if (device.writable) IconButton.filled(
            tooltip: 'Preview power action for ${device.name}',
            constraints: const BoxConstraints(minWidth: 48, minHeight: 48),
            onPressed: () => explain(context, 'Fictional preview only. No command was sent and no observed state was changed.'),
            icon: const Icon(Icons.power_settings_new),
          ),
        ]),
        Text(device.detail, style: TextStyle(color: muted, fontSize: 12)),
      ]),
    ));
  }
}

class PlatformsPage extends StatelessWidget {
  const PlatformsPage({super.key});
  @override
  Widget build(BuildContext context) => ListView(padding: const EdgeInsets.all(20), children: [
    Text('Smart Appliance Platforms', style: Theme.of(context).textTheme.headlineLarge),
    const SizedBox(height: 28),
    const Porcelain(child: ListTile(title: Text('Example Home'), subtitle: Text('Home account · Fictional preview'), trailing: Icon(Icons.chevron_right))),
    const SizedBox(height: 16),
    const Text('Provider authorization is specified in the atlas. This visual reference has no accounts or credentials.'),
  ]);
}

class SettingsPage extends StatelessWidget {
  const SettingsPage({super.key});
  @override
  Widget build(BuildContext context) => ListView(padding: const EdgeInsets.all(20), children: [
    Text('Settings', style: Theme.of(context).textTheme.headlineLarge),
    const SizedBox(height: 28),
    Porcelain(child: Column(children: [
      ListTile(title: const Text('Rooms'), trailing: const Icon(Icons.chevron_right), onTap: () => explain(context, 'Room management is specified in the complete atlas.')),
      ListTile(title: const Text('Send Authorization'), trailing: const Icon(Icons.chevron_right), onTap: () => explain(context, 'Sharing is a separate planned foreground flow. This reference sends nothing.')),
      ListTile(title: const Text('Receive Authorization'), trailing: const Icon(Icons.chevron_right), onTap: () => explain(context, 'This reference never advertises or opens a receiver.')),
    ])),
    const SizedBox(height: 20),
    Porcelain(child: ListTile(
      title: const Text('Appearance'), subtitle: const Text('Window Light'), trailing: const Icon(Icons.chevron_right),
      onTap: () => Navigator.of(context).push(MaterialPageRoute<void>(builder: (_) => const AppearancePage())),
    )),
    const SizedBox(height: 20),
    const Text('Visual reference · Local fictional data only'),
  ]);
}

class AppearancePage extends StatelessWidget {
  const AppearancePage({super.key});
  @override
  Widget build(BuildContext context) => WindowLightBackground(child: Scaffold(
    appBar: AppBar(title: const Text('Appearance')),
    body: ListView(padding: const EdgeInsets.all(20), children: [
      const Text('Skin'),
      const SizedBox(height: 16),
      Semantics(selected: true, label: 'Window Light, current skin', child: Porcelain(child: Padding(
        padding: const EdgeInsets.all(18),
        child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
          ClipRRect(borderRadius: BorderRadius.circular(16), child: SizedBox(
            height: 150, width: double.infinity,
            child: Image.asset(skinOf(context).backgroundAsset, fit: BoxFit.cover),
          )),
          const SizedBox(height: 16),
          const Wrap(spacing: 24, runSpacing: 8, children: [Text('Window Light'), Text('Current')]),
        ]),
      ))),
      const SizedBox(height: 16),
      const Text('Window Light is the only included skin and is already selected.'),
      const SizedBox(height: 24),
      FilledButton.tonal(onPressed: () => Navigator.of(context).push(MaterialPageRoute<void>(builder: (_) => WindowLightBackground(child: Scaffold(
        appBar: AppBar(title: const Text('Window Light preview')),
        body: const SafeArea(child: DevicesPage()),
      )))), child: const Text('Preview Skin')),
      const SizedBox(height: 20),
      const Text('Text follows system accessibility settings. Skin preferences are local; persistence is outside this isolated reference.'),
    ]),
  ));
}
