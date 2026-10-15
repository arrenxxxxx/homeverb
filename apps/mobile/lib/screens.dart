import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'blocks.dart';
import 'skin.dart';

typedef Json = Map<String, dynamic>;

class ScreenCatalog {
  ScreenCatalog(Json data) {
    for (final board in data['boards']) {
      for (final s in board['screens']) {
        screens[s['id']] = s as Json;
      }
    }
    branches = (data['interactions'] as List).cast<Json>();
  }
  final Map<String, Json> screens = {};
  late final List<Json> branches;
  static Future<ScreenCatalog> load() async => ScreenCatalog(
    jsonDecode(await rootBundle.loadString('assets/prototype.json')) as Json,
  );
  Json ui(String id) => screens[id]!['ui'] as Json;
}

class ScreenView extends StatelessWidget {
  const ScreenView({
    required this.id,
    required this.catalog,
    required this.onAction,
    required this.fields,
    required this.choices,
    required this.values,
    this.onChanged,
    this.screenOverride,
    this.home,
    this.resolveScreen,
    this.resolveHome,
    this.testChrome = false,
    this.preview = false,
    this.busy = false,
    this.error,
    super.key,
  });
  final String id;
  final ScreenCatalog catalog;
  final ActionHandler onAction;
  final Map<String, TextEditingController> fields;
  final Map<String, int> choices;
  final Map<String, double> values;
  final VoidCallback? onChanged;
  final Json? screenOverride;
  final Widget? home;
  final Json Function(String id)? resolveScreen;
  final Widget? Function(String id)? resolveHome;
  final bool testChrome, preview, busy;
  final Object? error;
  Widget blocks(List b) => Blocks(
    b,
    onAction: onAction,
    fields: fields,
    choices: choices,
    values: values,
    onChanged: onChanged,
    skinPreview: id == 'S11'
        ? ScreenView(
            id: 'D01',
            catalog: catalog,
            onAction: (_) {},
            fields: {},
            choices: {},
            values: {},
            testChrome: true,
          )
        : null,
  );
  @override
  Widget build(BuildContext context) {
    final ui = screenOverride ?? catalog.ui(id),
        root = ui['root'] as String?,
        base = ui['base'] as String?;
    return CallbackShortcuts(
      bindings: {
        if (base != null || root == null)
          const SingleActivator(LogicalKeyboardKey.escape): () =>
              onAction('Back'),
      },
      child: FocusScope(
        key: ValueKey('screen-focus:$id'),
        autofocus: true,
        child: Semantics(
          scopesRoute: true,
          explicitChildNodes: true,
          child: content(context),
        ),
      ),
    );
  }

  Widget content(BuildContext context) {
    final ui = screenOverride ?? catalog.ui(id),
        root = ui['root'] as String?,
        base = ui['base'] as String?;
    return LightBackground(
      child: LayoutBuilder(
        builder: (context, c) {
          final wide = c.maxWidth >= 800;
          if (wide &&
              id.startsWith('W') &&
              MediaQuery.textScalerOf(context).scale(13) <= 17)
            return wideReference(c.maxWidth);
          final header = ui['header'] as String;
          final content =
              home ??
              ((ui['blocks'] as List).firstOrNull?[0] == 'home'
                  ? DeviceHome(mode: ui['blocks'][0][1], onAction: onAction)
                  : blocks(ui['blocks'] as List));
          if (base != null) {
            final dialog = ui['overlay'] == 'dialog';
            return Stack(
              children: [
                ExcludeFocus(
                  child: ExcludeSemantics(
                    child: IgnorePointer(
                      child: ScreenView(
                        id: base,
                        catalog: catalog,
                        screenOverride: resolveScreen?.call(base),
                        home: resolveHome?.call(base),
                        resolveScreen: resolveScreen,
                        resolveHome: resolveHome,
                        onAction: (_) {},
                        fields: resolveScreen == null ? {} : fields,
                        choices: {},
                        values: resolveScreen == null ? {} : values,
                        testChrome: testChrome,
                      ),
                    ),
                  ),
                ),
                ModalBarrier(
                  color: const Color(0x5f14181b),
                  dismissible: true,
                  onDismiss: () => onAction('Back'),
                  semanticsLabel: 'Dismiss $header',
                ),
                Align(
                  alignment: dialog ? Alignment.center : Alignment.bottomCenter,
                  child: Padding(
                    padding: EdgeInsets.fromLTRB(
                      dialog ? 26 : 0,
                      dialog ? 0 : 80,
                      dialog ? 26 : 0,
                      0,
                    ),
                    child: ConstrainedBox(
                      constraints: BoxConstraints(
                        maxWidth: wide ? 520 : double.infinity,
                        maxHeight: c.maxHeight - 100,
                      ),
                      child: Surface(
                        radius: dialog ? 25 : 28,
                        bottomBleed: dialog ? 0 : 24,
                        padding: EdgeInsets.fromLTRB(
                          20,
                          dialog ? 25 : 11,
                          20,
                          dialog ? 14 : 20,
                        ),
                        child: SingleChildScrollView(
                          child: Column(
                            mainAxisSize: MainAxisSize.min,
                            crossAxisAlignment: CrossAxisAlignment.stretch,
                            children: [
                              if (!dialog) ...[
                                Center(
                                  child: Container(
                                    width: 36,
                                    height: 4,
                                    decoration: BoxDecoration(
                                      color: const Color(0xffbbbcb5),
                                      borderRadius: BorderRadius.circular(2),
                                    ),
                                  ),
                                ),
                                const SizedBox(height: 24),
                              ],
                              Semantics(
                                header: true,
                                namesRoute: true,
                                child: Copy(
                                  header,
                                  size: dialog ? 22 : 23,
                                  bold: true,
                                ),
                              ),
                              SizedBox(height: dialog ? 17 : 21),
                              content,
                              if (error != null)
                                Semantics(
                                  liveRegion: true,
                                  child: Copy(
                                    '$error',
                                    color: const Color(0xff9e3421),
                                  ),
                                ),
                            ],
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
                if (busy)
                  const Positioned(
                    top: 0,
                    left: 0,
                    right: 0,
                    child: OperationProgress(),
                  ),
              ],
            );
          }
          return Column(
            children: [
              if (testChrome && !wide)
                SizedBox(
                  height: 44,
                  child: Stack(
                    children: [
                      const Positioned(
                        left: 24,
                        top: 16,
                        child: Copy('9:41', size: 12, bold: true),
                      ),
                      Positioned.fill(
                        child: CustomPaint(painter: _StatusChrome()),
                      ),
                    ],
                  ),
                )
              else
                SizedBox(height: MediaQuery.paddingOf(context).top),
              if (preview && !testChrome)
                InkWell(
                  onTap: () => onAction('Preview catalog'),
                  child: Container(
                    width: double.infinity,
                    color: const Color(0xfff8ead9),
                    padding: const EdgeInsets.all(4),
                    child: const Copy(
                      'UI preview · Tap to browse states · No native operations',
                      size: 11,
                      center: true,
                    ),
                  ),
                ),
              Expanded(
                child: Row(
                  children: [
                    if (wide)
                      _rail(
                        root ??
                            (id.startsWith('P')
                                ? 'Platforms'
                                : id.startsWith('S') || id.startsWith('H')
                                ? 'Settings'
                                : 'Devices'),
                      ),
                    Expanded(
                      child: Column(
                        children: [
                          if (root == null)
                            Row(
                              children: [
                                Transform.translate(
                                  offset: const Offset(4, 2),
                                  child: IconAction(
                                    'back',
                                    'Back',
                                    () => onAction('Back'),
                                  ),
                                ),
                                const Spacer(),
                                SizedBox(
                                  width:
                                      88 *
                                      MediaQuery.textScalerOf(
                                        context,
                                      ).scale(11) /
                                      11,
                                  child: const Copy(
                                    'HomeVerb',
                                    size: 11,
                                    color: muted,
                                  ),
                                ),
                              ],
                            ),
                          Expanded(
                            child: SingleChildScrollView(
                              key: ValueKey('scroll:$id'),
                              padding: EdgeInsets.fromLTRB(
                                wide ? 30 : 20,
                                root != null ? (id == 'P01' ? 13 : 16) : 8,
                                wide ? 30 : 20,
                                20,
                              ),
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.stretch,
                                children: [
                                  Row(
                                    crossAxisAlignment:
                                        CrossAxisAlignment.start,
                                    children: [
                                      Expanded(
                                        child: Semantics(
                                          header: true,
                                          namesRoute: true,
                                          child: Copy(
                                            header,
                                            size: root != null
                                                ? (id == 'P01' ? 28 : 30)
                                                : 25,
                                            bold: true,
                                          ),
                                        ),
                                      ),
                                      if (root == 'Devices') ...[
                                        SizedBox(
                                          height: 36,
                                          width: 45,
                                          child: Transform.translate(
                                            offset: const Offset(8.5, -2),
                                            child: IconAction(
                                              'search',
                                              'Search',
                                              () => onAction('Search'),
                                              size: 22,
                                            ),
                                          ),
                                        ),
                                        SizedBox(
                                          height: 36,
                                          width: 45,
                                          child: Transform.translate(
                                            offset: const Offset(9, -2.5),
                                            child: IconAction(
                                              'plus',
                                              'Add device',
                                              () => onAction('Add device'),
                                              size: 23,
                                            ),
                                          ),
                                        ),
                                      ],
                                    ],
                                  ),
                                  SizedBox(
                                    height: root == 'Devices'
                                        ? ((ui['blocks'] as List)
                                                          .firstOrNull?[0] ==
                                                      'home' ||
                                                  home is DeviceHome
                                              ? 8
                                              : 16)
                                        : root == 'Settings'
                                        ? 16
                                        : root == 'Platforms'
                                        ? (MediaQuery.textScalerOf(
                                                    context,
                                                  ).scale(13) <=
                                                  13
                                              ? 22.4
                                              : 23)
                                        : 22,
                                  ),
                                  if (ui['subtitle'] != '' &&
                                      ui['subtitle'] != null) ...[
                                    Transform.translate(
                                      offset: const Offset(0, -9),
                                      child: Copy(ui['subtitle'], color: muted),
                                    ),
                                    const SizedBox(height: 9),
                                  ],
                                  if (busy)
                                    const Padding(
                                      padding: EdgeInsets.only(bottom: 12),
                                      child: OperationProgress(),
                                    ),
                                  if (error != null) ...[
                                    blocks([
                                      [
                                        'status',
                                        'Could not complete this operation',
                                        '$error',
                                        'error',
                                      ],
                                    ]),
                                    ActionButton(
                                      'Dismiss error',
                                      style: 'secondary',
                                      onPressed: () =>
                                          onAction('Dismiss error'),
                                    ),
                                  ],
                                  if (wide && id == 'W02')
                                    Row(
                                      crossAxisAlignment:
                                          CrossAxisAlignment.start,
                                      children: [
                                        SizedBox(
                                          width: 276,
                                          child: Rows(const [
                                            ['Bedroom AC', 'Selected'],
                                            ['Floor lamp', 'Living Room'],
                                            ['Living room TV', 'Living Room'],
                                            ['Air sensor', 'Living Room'],
                                          ], onAction: onAction),
                                        ),
                                        const SizedBox(width: 36),
                                        Expanded(
                                          child: Column(
                                            children: [
                                              Climate(onAction: onAction),
                                              Rows(const [
                                                ['Power', 'On'],
                                                ['Mode', 'Cool  ›'],
                                                ['Fan', 'Auto  ›'],
                                              ], onAction: onAction),
                                              ActionButton(
                                                'Refresh State',
                                                style: 'secondary',
                                                onPressed: () =>
                                                    onAction('Refresh State'),
                                              ),
                                            ],
                                          ),
                                        ),
                                      ],
                                    )
                                  else if (id == 'W01')
                                    DeviceHome(mode: 'all', onAction: onAction)
                                  else
                                    content,
                                ],
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
              if (root != null && !wide) _nav(root, context),
              if (testChrome && !wide)
                Container(
                  color: root != null ? const Color(0xfff6f2ea) : null,
                  height: 20,
                  alignment: Alignment.center,
                  child: Container(
                    width: 92,
                    height: 4,
                    decoration: BoxDecoration(
                      color: ink,
                      borderRadius: BorderRadius.circular(2),
                    ),
                  ),
                )
              else
                SizedBox(height: MediaQuery.paddingOf(context).bottom),
            ],
          );
        },
      ),
    );
  }

  Widget wideReference(double width) {
    final detail = id == 'W02';
    final panelWidth = width - 474;
    return Stack(
      children: [
        Positioned(left: 0, top: 0, bottom: 0, child: _rail('Devices')),
        Positioned(
          left: 118,
          top: 37,
          child: Semantics(
            header: true,
            namesRoute: true,
            child: Copy('HomeVerb', size: 28, bold: true),
          ),
        ),
        const Positioned(left: 118, top: 83, child: Copy('My home', size: 16)),
        if (!detail) ...[
          Positioned(
            right: 63,
            top: 28,
            child: IconAction('search', 'Search', () => onAction('Search')),
          ),
          Positioned(
            right: 16,
            top: 27,
            child: IconAction(
              'plus',
              'Add device',
              () => onAction('Add device'),
            ),
          ),
          Positioned(
            right: 16,
            top: 70,
            child: IconAction(
              'settings',
              'Manage rooms',
              () => onAction('Manage rooms'),
              color: muted,
              size: 20,
            ),
          ),
          Positioned(
            left: 118,
            top: 127,
            right: 78,
            child: blocks([
              [
                'tabs',
                ['All', 'Living Room', 'Primary Bedroom', 'Bathroom'],
                0,
              ],
            ]),
          ),
          const Positioned(
            left: 118,
            top: 183,
            child: Copy('4 devices · Updated just now', size: 13, color: muted),
          ),
          Positioned(
            left: 118,
            right: 78,
            top: 215,
            bottom: 0,
            child: SingleChildScrollView(
              child: LayoutBuilder(
                builder: (context, c) => Wrap(
                  spacing: 18,
                  runSpacing: 18,
                  children: [
                    for (final key in ['ac', 'lamp', 'tv', 'sensor'])
                      SizedBox(
                        width: (c.maxWidth - 36) / 3,
                        child: DeviceTile(
                          key,
                          onAction: onAction,
                          compactHeight: 235,
                        ),
                      ),
                  ],
                ),
              ),
            ),
          ),
        ] else ...[
          Positioned(
            left: 118,
            top: 138,
            width: 276,
            child: Rows(const [
              ['Bedroom AC', 'Selected'],
              ['Floor lamp', 'Living Room'],
              ['Living room TV', 'Living Room'],
              ['Air sensor', 'Living Room'],
            ], onAction: onAction),
          ),
          const Positioned(
            left: 430,
            top: 137,
            child: Copy('Bedroom AC', size: 27, bold: true),
          ),
          const Positioned(
            left: 430,
            top: 178,
            child: Copy('Primary Bedroom', size: 14, color: muted),
          ),
          Positioned(
            left: 430,
            top: 216,
            width: panelWidth,
            child: Climate(onAction: onAction),
          ),
          Positioned(
            left: 430,
            top: 459,
            width: panelWidth,
            child: Rows(const [
              ['Power', 'On'],
              ['Mode', 'Cool  ›'],
              ['Fan', 'Auto  ›'],
            ], onAction: onAction),
          ),
          Positioned(
            left: 430,
            top: 620,
            width: panelWidth,
            child: ActionButton(
              'Refresh State',
              style: 'secondary',
              onPressed: () => onAction('Refresh State'),
            ),
          ),
        ],
      ],
    );
  }

  Widget _nav(String root, BuildContext context) => Container(
    color: const Color(0xfff6f2ea),
    constraints: const BoxConstraints(minHeight: 58),
    height: MediaQuery.textScalerOf(context).scale(11) > 11 ? null : 58,
    child: Row(
      children: List.generate(3, (i) {
        const labels = ['Devices', 'Platforms', 'Settings'],
            icons = ['home', 'platform', 'settings'];
        final color = root == labels[i] ? accent : muted;
        return Expanded(
          child: Semantics(
            selected: root == labels[i],
            button: true,
            child: InkWell(
              onTap: () => onAction(labels[i]),
              child: MediaQuery.textScalerOf(context).scale(11) <= 11
                  ? Stack(
                      children: [
                        Positioned(
                          top: 12,
                          left: 0,
                          right: 0,
                          child: Center(
                            child: Glyph(icons[i], size: 22, color: color),
                          ),
                        ),
                        Positioned(
                          top: 40,
                          left: 0,
                          right: 0,
                          child: Copy(
                            labels[i],
                            size: 11,
                            color: color,
                            bold: root == labels[i],
                            center: true,
                          ),
                        ),
                      ],
                    )
                  : Column(
                      mainAxisSize: MainAxisSize.min,
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        const SizedBox(height: 10),
                        Glyph(icons[i], size: 22, color: color),
                        const SizedBox(height: 6),
                        Copy(
                          labels[i],
                          size: 11,
                          color: color,
                          bold: root == labels[i],
                          center: true,
                        ),
                        const SizedBox(height: 10),
                      ],
                    ),
            ),
          ),
        );
      }),
    ),
  );
  Widget _rail(String root) => Container(
    width: 88,
    color: const Color(0xfff4f0e8),
    child: SingleChildScrollView(
      child: Column(
        children: [
          const SizedBox(height: 26),
          const Glyph('home', size: 28, color: accent),
          const SizedBox(height: 64),
          for (final entry in const {
            'Devices': 'home',
            'Platforms': 'platform',
            'Settings': 'settings',
          }.entries)
            Padding(
              padding: const EdgeInsets.only(bottom: 20),
              child: Semantics(
                selected: root == entry.key,
                button: true,
                child: InkWell(
                  onTap: () => onAction(entry.key),
                  child: Container(
                    width: 64,
                    constraints: const BoxConstraints(minHeight: 72),
                    padding: const EdgeInsets.only(top: 16, bottom: 10),
                    decoration: BoxDecoration(
                      color: root == entry.key ? const Color(0xffe7d3c4) : null,
                      borderRadius: BorderRadius.circular(20),
                    ),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Glyph(
                          entry.value,
                          size: 22,
                          color: root == entry.key ? accent : muted,
                        ),
                        const SizedBox(height: 10),
                        Copy(
                          entry.key,
                          size: 10,
                          color: root == entry.key ? accent : muted,
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ),
        ],
      ),
    ),
  );
}

class DeviceHome extends StatelessWidget {
  const DeviceHome({
    required this.mode,
    required this.onAction,
    this.cards,
    this.roomNames,
    this.filter = 'All',
    super.key,
  });
  final String mode, filter;
  final ActionHandler onAction;
  final Widget? cards;
  final List<String>? roomNames;
  @override
  Widget build(BuildContext context) {
    if (MediaQuery.textScalerOf(context).scale(13) <= 13) {
      final labels =
          roomNames ?? const ['All', 'Living Room', 'Bedroom', 'Bathroom'];
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          SizedBox(
            height: 46,
            child: Stack(
              clipBehavior: Clip.none,
              children: [
                const Positioned(
                  left: 0,
                  top: 0,
                  child: Copy('My home', size: 16, bold: true),
                ),
                const Positioned(left: 81, top: 0, child: Copy('⌄', size: 16)),
                Positioned(
                  right: -12,
                  top: -15,
                  width: 48,
                  height: 48,
                  child: IconButton(
                    tooltip: 'Manage rooms',
                    onPressed: () => onAction('Manage rooms'),
                    icon: const Glyph('settings', size: 20, color: muted),
                  ),
                ),
              ],
            ),
          ),
          SizedBox(
            height: 49,
            child: ListView(
              scrollDirection: Axis.horizontal,
              children: [
                for (var i = 0; i < labels.length; i++)
                  SizedBox(
                    width: roomNames == null
                        ? [45.0, 104.0, 90.0, 90.0][i]
                        : null,
                    child: Padding(
                      padding: EdgeInsets.only(
                        right: roomNames == null ? 0 : 18,
                      ),
                      child: InkWell(
                        onTap: () => onAction('room:${labels[i]}'),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Copy(
                              labels[i],
                              bold:
                                  labels[i] ==
                                  (mode == 'living' ? 'Living Room' : filter),
                              color:
                                  labels[i] ==
                                      (mode == 'living'
                                          ? 'Living Room'
                                          : filter)
                                  ? ink
                                  : muted,
                            ),
                            const SizedBox(height: 6.8),
                            if (labels[i] ==
                                (mode == 'living' ? 'Living Room' : filter))
                              Container(
                                width: labels[i] == 'All' ? 17 : 58,
                                height: 3,
                                decoration: BoxDecoration(
                                  color: accent,
                                  borderRadius: BorderRadius.circular(2),
                                ),
                              ),
                          ],
                        ),
                      ),
                    ),
                  ),
              ],
            ),
          ),
          SizedBox(
            height: 26,
            child: Stack(
              clipBehavior: Clip.none,
              children: [
                Positioned(
                  left: 0,
                  top: 0,
                  child: Copy(
                    cards != null
                        ? 'Devices'
                        : mode == 'living'
                        ? '2 devices'
                        : '4 devices',
                    size: 11,
                    color: muted,
                  ),
                ),
                Positioned(
                  right: 24,
                  top: 0,
                  child: InkWell(
                    onTap: () => onAction('Inspect cached state'),
                    child: Copy(
                      cards != null
                          ? 'Saved locally'
                          : mode == 'cached'
                          ? 'Cached · 2h ago'
                          : 'Updated just now',
                      size: 11,
                      color: muted,
                    ),
                  ),
                ),
                Positioned(
                  right: -15,
                  top: -17,
                  width: 48,
                  height: 48,
                  child: IconButton(
                    tooltip: 'Refresh State',
                    onPressed: () => onAction('Refresh State'),
                    icon: const Glyph('refresh', size: 16, color: muted),
                  ),
                ),
              ],
            ),
          ),
          cards ??
              DeviceGrid(
                keys: mode == 'living'
                    ? const ['lamp', 'tv']
                    : mode == 'large'
                    ? const ['ac', 'lamp']
                    : const ['ac', 'lamp', 'tv', 'sensor'],
                onAction: onAction,
                cached: mode == 'cached',
                large: mode == 'large',
              ),
          if (cards == null && mode == 'large') ...[
            const SizedBox(height: 20),
            const Copy('2 more devices below', size: 14, color: muted),
          ],
        ],
      );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            const Copy('My home', size: 16, bold: true),
            const SizedBox(width: 8),
            const Icon(Icons.keyboard_arrow_down, size: 16),
            const Spacer(),
            SizedBox(
              height: 32,
              child: IconAction(
                'settings',
                'Manage rooms',
                () => onAction('Manage rooms'),
                color: muted,
              ),
            ),
          ],
        ),
        const SizedBox(height: 14),
        SizedBox(
          height: 28 + MediaQuery.textScalerOf(context).scale(13),
          child: ListView(
            scrollDirection: Axis.horizontal,
            children: [
              for (final label
                  in roomNames ??
                      const ['All', 'Living Room', 'Bedroom', 'Bathroom'])
                Padding(
                  padding: const EdgeInsets.only(right: 18),
                  child: InkWell(
                    onTap: () => onAction('room:$label'),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Copy(
                          label,
                          color:
                              label ==
                                  (mode == 'living' ? 'Living Room' : filter)
                              ? ink
                              : muted,
                          bold:
                              label ==
                              (mode == 'living' ? 'Living Room' : filter),
                        ),
                        const SizedBox(height: 12),
                        if (label ==
                            (mode == 'living' ? 'Living Room' : filter))
                          Container(
                            height: 3,
                            width: label.length < 4 ? 18 : 58,
                            decoration: BoxDecoration(
                              color: accent,
                              borderRadius: BorderRadius.circular(2),
                            ),
                          ),
                      ],
                    ),
                  ),
                ),
            ],
          ),
        ),
        Row(
          children: [
            Copy(
              cards != null
                  ? 'Devices'
                  : mode == 'living'
                  ? '2 devices'
                  : '4 devices',
              size: 11,
              color: muted,
            ),
            const Spacer(),
            Flexible(
              child: InkWell(
                onTap: () => onAction('Inspect cached state'),
                child: Copy(
                  cards != null
                      ? 'Saved locally'
                      : mode == 'cached'
                      ? 'Cached · 2h ago'
                      : 'Updated just now',
                  size: 11,
                  color: muted,
                ),
              ),
            ),
            SizedBox(
              width: 26,
              height: 26,
              child: IconButton(
                padding: EdgeInsets.zero,
                tooltip: 'Refresh State',
                onPressed: () => onAction('Refresh State'),
                icon: const Glyph('refresh', size: 16, color: muted),
              ),
            ),
          ],
        ),
        const SizedBox(height: 10),
        cards ??
            DeviceGrid(
              keys: mode == 'living'
                  ? const ['lamp', 'tv']
                  : mode == 'large'
                  ? const ['ac', 'lamp']
                  : const ['ac', 'lamp', 'tv', 'sensor'],
              onAction: onAction,
              cached: mode == 'cached',
              large: mode == 'large',
            ),
        if (cards == null && mode == 'large') ...[
          const SizedBox(height: 20),
          const Copy('2 more devices below', size: 14, color: muted),
        ],
      ],
    );
  }
}

class _StatusChrome extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()..color = ink;
    for (var i = 0; i < 4; i++) {
      canvas.drawRRect(
        RRect.fromLTRBR(
          size.width - 69 + i * 4,
          25 - i * 2,
          size.width - 67 + i * 4,
          28,
          const Radius.circular(1),
        ),
        paint,
      );
    }
    canvas.drawRRect(
      RRect.fromLTRBR(
        size.width - 44,
        18,
        size.width - 24,
        28,
        const Radius.circular(3),
      ),
      paint
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1,
    );
    paint.style = PaintingStyle.fill;
    canvas.drawRRect(
      RRect.fromLTRBR(
        size.width - 42,
        20,
        size.width - 27,
        26,
        const Radius.circular(1),
      ),
      paint,
    );
    canvas.drawRRect(
      RRect.fromLTRBR(
        size.width - 23,
        21,
        size.width - 21,
        25,
        const Radius.circular(1),
      ),
      paint,
    );
  }

  @override
  bool shouldRepaint(covariant _StatusChrome oldDelegate) => false;
}

/// Isolated, explicitly labelled visual fixtures. Never constructed with a native client.
class PreviewApp extends StatefulWidget {
  const PreviewApp({
    required this.catalog,
    this.initial = 'D01',
    this.testChrome = false,
    super.key,
  });
  final ScreenCatalog catalog;
  final String initial;
  final bool testChrome;
  @override
  State<PreviewApp> createState() => PreviewAppState();
}

class PreviewAppState extends State<PreviewApp> {
  late String id = widget.initial;
  final List<String> history = [];
  final Map<String, TextEditingController> fields = {};
  final Map<String, int> choices = {};
  final Map<String, double> values = {};
  String? message;
  bool showCatalog = false;
  void go(String target) {
    setState(() {
      showCatalog = false;
      history.add(id);
      id = target;
      for (final field in fields.values) {
        field.dispose();
      }
      fields.clear();
      choices.clear();
      values.clear();
    });
  }

  void action(String label) {
    if (label == 'Preview catalog') {
      setState(() => showCatalog = !showCatalog);
      return;
    }
    if (label == 'Back' || label == 'Cancel' || label == 'Close') {
      setState(() {
        id = history.isEmpty
            ? (widget.catalog.ui(id)['base'] ?? 'D01')
            : history.removeLast();
      });
      return;
    }
    const direct = {
      'Devices': 'D01',
      'Platforms': 'P01',
      'Settings': 'S01',
      'Search': 'D03',
      'Add device': 'A01',
      'Add Device': 'A01',
      'Manage rooms': 'R01',
      'Appearance': 'S02',
      'Rooms': 'R01',
      'This Device': 'S10',
      'Connections': 'S03',
      'Recent Operations': 'S05',
      'Local Data': 'S04',
      'Diagnostics': 'S07',
      'About': 'S08',
      'open:ac': 'C01',
      'open:lamp': 'C05',
      'open:tv': 'C07',
      'open:sensor': 'C13',
      'room:Living Room': 'D02',
      'room:All': 'D01',
      'room:Bathroom': 'D09',
      'room:Bedroom': 'D02',
      'Rename': 'D07',
      'Name': 'D07',
      'Move to Room': 'R03',
      'Room': 'R03',
      'Reorder Devices': 'D10',
      'Device Information': 'D06',
      'Connection': 'D08',
      'Mode': 'C02',
      'Target temperature': 'C14',
      'Increase target': 'C03',
      'Decrease target': 'C03',
      'Save Mode': 'C03',
      'Save': 'C03',
      'View Details': 'S05',
      'View Evidence': 'S06',
      'Preview Skin': 'S11',
      'Text size and accessibility': 'D12',
      'Account': 'P01',
      'Discovery': 'A02',
      'Continue': 'A03',
      'Add 2 Devices': 'A06',
      'Open Living Room': 'D02',
      'Save Room': 'D09',
      'Move 2 Devices': 'D02',
      'Save Preference': 'D06',
      'Clear Cached State': 'S04',
      'Clear Recent Operations': 'S04',
      'Remove Local Data': 'S09',
      'Device information · Recent operations': 'D06',
      'Home account': 'P09',
      'Studio account': 'P10',
      'Living room': 'P09',
      'Example Climate': 'P13',
      'Regions and compatibility': 'S12',
      'Compatibility': 'S12',
      'Remove From This Device': 'P11',
      'Revoke Access · if supported': 'P12',
      'Check Connection': 'P10',
      'View Devices': 'D01',
      'Back to Accounts': 'H01',
      'Alex phone': 'H03',
      'Morgan tablet': 'H03',
      'Accept Authorization': 'H08',
      'Cancel Transfer': 'H15',
      'Remove Imported Authorization': 'P11',
    };
    if (label.startsWith('preview-step:')) {
      setState(() => message = 'Preview only: no device setting was changed.');
      return;
    }
    if (label.startsWith('menu:')) {
      go('D05');
      return;
    }
    if (label.startsWith('power:')) {
      setState(() => message = 'Preview only: no command was sent.');
      return;
    }
    if (label == 'Refresh State') {
      if (id == 'C04' || id == 'C03') {
        go('C01');
      } else if (id == 'D11') {
        go('D01');
      } else {
        setState(() => message = 'Preview only: no device was contacted.');
      }
      return;
    }
    if (label == 'Save Name') {
      go(id == 'S10' ? 'S01' : 'D06');
      return;
    }
    if (label == 'Start Cycle') {
      go(id == 'C10' ? 'C11' : 'C10');
      return;
    }
    if (label == 'Remove Local Data' && id == 'S09') {
      go('D04');
      return;
    }
    final matching = widget.catalog.branches
        .where(
          (b) =>
              b['source'] == id &&
              ('${b['action']}').toLowerCase() == label.toLowerCase(),
        )
        .firstOrNull;
    if (matching != null) {
      go(matching['target']);
      return;
    }
    if (direct[label] != null) {
      go(direct[label]!);
      return;
    }
    if (label.startsWith('choice:')) return;
    setState(
      () =>
          message = 'Preview only: $label. No native operation was performed.',
    );
  }

  @override
  void dispose() {
    for (final f in fields.values) {
      f.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => MaterialApp(
    debugShowCheckedModeBanner: false,
    theme: windowTheme(),
    highContrastTheme: windowTheme(highContrast: true),
    home: Scaffold(
      body: Column(
        children: [
          if (!widget.testChrome)
            Material(
              color: const Color(0xfff8ead9),
              child: InkWell(
                onTap: () => action('Preview catalog'),
                child: const Padding(
                  padding: EdgeInsets.all(8),
                  child: Copy(
                    'UI preview · Tap to browse states · No native operations',
                    size: 11,
                    center: true,
                  ),
                ),
              ),
            ),
          Expanded(
            child: ScreenView(
              id: id,
              catalog: widget.catalog,
              onAction: action,
              fields: fields,
              choices: choices,
              values: values,
              onChanged: () => setState(() {}),
              testChrome: widget.testChrome,
              preview: false,
            ),
          ),
        ],
      ),
      bottomSheet: showCatalog
          ? Material(
              child: SizedBox(
                height: 420,
                child: ListView(
                  children: [
                    ListTile(
                      title: const Text('Isolated preview scenarios'),
                      subtitle: const Text('Simulated events never call Core.'),
                      trailing: IconButton(
                        tooltip: 'Close catalog',
                        icon: const Icon(Icons.close),
                        onPressed: () => setState(() => showCatalog = false),
                      ),
                    ),
                    for (final branch in widget.catalog.branches.where(
                      (b) => b['source'] == id,
                    ))
                      ListTile(
                        key: ValueKey(
                          'branch:${branch['source']}:${branch['action']}',
                        ),
                        title: Text('${branch['action']}'),
                        subtitle: Text('Simulate event → ${branch['target']}'),
                        onTap: () => go(branch['target']),
                      ),
                    const Divider(),
                    for (final screen in widget.catalog.screens.values)
                      ListTile(
                        title: Text('${screen['id']} · ${screen['title']}'),
                        onTap: () => go(screen['id']),
                      ),
                  ],
                ),
              ),
            )
          : message == null
          ? null
          : Material(
              child: ListTile(
                title: Text(message!),
                trailing: IconButton(
                  tooltip: 'Dismiss',
                  onPressed: () => setState(() => message = null),
                  icon: const Icon(Icons.close),
                ),
              ),
            ),
    ),
  );
}
