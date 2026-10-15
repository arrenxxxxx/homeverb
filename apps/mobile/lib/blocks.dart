import 'package:flutter/material.dart';
import 'skin.dart';

typedef ActionHandler = void Function(String label);

class Blocks extends StatelessWidget {
  const Blocks(
    this.blocks, {
    required this.onAction,
    required this.fields,
    required this.choices,
    required this.values,
    this.onChanged,
    this.skinPreview,
    super.key,
  });
  final List<dynamic> blocks;
  final ActionHandler onAction;
  final Map<String, TextEditingController> fields;
  final Map<String, int> choices;
  final Map<String, double> values;
  final VoidCallback? onChanged;
  final Widget? skinPreview;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: blocks.map((b) => block(context, b as List)).toList(),
  );
  Widget block(BuildContext context, List b) {
    final type = b[0];
    switch (type) {
      case 'note':
        return Padding(
          padding: const EdgeInsets.fromLTRB(2, 2, 2, 10),
          child: Copy('${b[1]}', color: muted),
        );
      case 'section':
        return Padding(
          padding: const EdgeInsets.fromLTRB(3, 4, 3, 11),
          child: Copy(b[1], size: 10, color: muted, bold: true),
        );
      case 'link':
        return Align(
          alignment: Alignment.centerLeft,
          child: TextButton(
            style: TextButton.styleFrom(
              padding: const EdgeInsets.symmetric(horizontal: 2),
              minimumSize: const Size(48, 48),
            ),
            onPressed: () => onAction(b[1]),
            child: Copy(b[1], color: accent, bold: true),
          ),
        );
      case 'rows':
        return Rows(b[1] as List, onAction: onAction);
      case 'card':
        return Rows(b[2] as List, title: b[1], onAction: onAction);
      case 'button':
        return ActionButton(
          b[1],
          style: b.length > 2 ? b[2] : '',
          onPressed: b.length > 2 && b[2] == 'disabled'
              ? null
              : () => onAction(b[1]),
        );
      case 'field':
        final label = b[1] as String;
        final field = fields.putIfAbsent(
          label,
          () => TextEditingController(text: b[2]),
        );
        return Padding(
          padding: const EdgeInsets.only(bottom: 16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Copy(label, size: 12, color: muted),
              const SizedBox(height: 9),
              Surface(
                radius: 14,
                child: TextField(
                  controller: field,
                  key: ValueKey('field:$label'),
                  onChanged: (_) => onChanged?.call(),
                  keyboardType: label == 'Target temperature'
                      ? const TextInputType.numberWithOptions(
                          decimal: true,
                          signed: true,
                        )
                      : TextInputType.text,
                  style: const TextStyle(fontSize: 15),
                  decoration: InputDecoration(
                    contentPadding: const EdgeInsets.symmetric(
                      horizontal: 15,
                      vertical: 15,
                    ),
                    border: InputBorder.none,
                    hintText: 'Search by name',
                    suffixIcon: label.startsWith('Search')
                        ? IconButton(
                            tooltip: 'Clear search',
                            onPressed: () {
                              field.clear();
                              onChanged?.call();
                            },
                            icon: const Icon(Icons.close, size: 18),
                          )
                        : null,
                  ),
                ),
              ),
              if (b.length > 3)
                Padding(
                  padding: const EdgeInsets.only(top: 8),
                  child: Copy(b[3], size: 12, color: const Color(0xff9e3421)),
                ),
            ],
          ),
        );
      case 'choices':
        final items = (b[1] as List).cast<String>();
        final key = items.join('|');
        final initial = b.length > 2 ? b[2] : 0;
        final selected = choices.putIfAbsent(
          key,
          () => initial is int ? initial : initial[0] as int,
        );
        return Padding(
          padding: const EdgeInsets.only(bottom: 4),
          child: Column(
            children: List.generate(
              items.length,
              (i) => Padding(
                padding: const EdgeInsets.only(bottom: 9),
                child: Semantics(
                  selected: i == selected,
                  child: Surface(
                    radius: 16,
                    border: i == selected ? accent : null,
                    child: InkWell(
                      borderRadius: BorderRadius.circular(16),
                      onTap: () {
                        choices[key] = i;
                        onAction('choice:${items[i]}');
                        onChanged?.call();
                      },
                      child: Padding(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 15,
                          vertical: 17,
                        ),
                        child: Row(
                          children: [
                            Expanded(
                              child: Copy(items[i], bold: i == selected),
                            ),
                            const SizedBox(width: 12),
                            Container(
                              width: 18,
                              height: 18,
                              decoration: BoxDecoration(
                                shape: BoxShape.circle,
                                color: i == selected ? accent : null,
                                border: Border.all(
                                  color: i == selected
                                      ? accent
                                      : const Color(0xffb3b3ac),
                                ),
                              ),
                              child: i == selected
                                  ? const Glyph(
                                      'check',
                                      size: 14,
                                      color: Colors.white,
                                    )
                                  : null,
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          ),
        );
      case 'chips':
      case 'tabs':
        return Padding(
          padding: const EdgeInsets.only(bottom: 15),
          child: Wrap(
            spacing: 7,
            runSpacing: 8,
            children: List.generate((b[1] as List).length, (i) {
              final label = b[1][i] as String;
              final selected =
                  choices['tabs'] ?? (type == 'tabs' ? b[2] as int : 2);
              return Semantics(
                selected: i == selected,
                child: ActionChip(
                  backgroundColor: i == selected
                      ? ink
                      : const Color(0xfff8f5ee),
                  side: BorderSide.none,
                  label: Copy(
                    label,
                    size: 12,
                    color: i == selected ? Colors.white : muted,
                  ),
                  onPressed: () {
                    choices['tabs'] = i;
                    onAction(label);
                    onChanged?.call();
                  },
                ),
              );
            }),
          ),
        );
      case 'hero':
        if (MediaQuery.textScalerOf(context).scale(13) <= 13) {
          return Padding(
            padding: const EdgeInsets.only(bottom: 14),
            child: Surface(
              child: SizedBox(
                height: 200,
                child: Stack(
                  children: [
                    Positioned(
                      top: 13,
                      left: 75,
                      right: 75,
                      height: 87,
                      child: product(b[1]),
                    ),
                    Positioned(
                      top: 119,
                      left: 10,
                      right: 10,
                      child: Copy(b[2], size: 23, bold: true, center: true),
                    ),
                    Positioned(
                      top: 158,
                      left: 20,
                      right: 20,
                      child: Copy(b[3], size: 12, color: muted),
                    ),
                  ],
                ),
              ),
            ),
          );
        }
        return Padding(
          padding: const EdgeInsets.only(bottom: 14),
          child: Surface(
            child: Padding(
              padding: const EdgeInsets.fromLTRB(20, 13, 20, 20),
              child: Column(
                children: [
                  SizedBox(height: 87, child: product(b[1], size: 87)),
                  const SizedBox(height: 19),
                  Copy(b[2], size: 23, bold: true, center: true),
                  const SizedBox(height: 13),
                  Align(
                    alignment: Alignment.centerLeft,
                    child: Copy(b[3], size: 12, color: muted),
                  ),
                ],
              ),
            ),
          ),
        );
      case 'climate':
        return Climate(
          onAction: onAction,
          freshness: b.length > 1 ? b[1] : 'Updated just now',
          enabled: b.length <= 2 || b[2] == true,
          target: values['observed_target'] ?? 24,
          mode: fields['observed_mode']?.text ?? 'Cooling · Room 26°C',
        );
      case 'status':
        const palette = {
          'warning': [Color(0xfff8ead9), Color(0xff855428)],
          'success': [Color(0xffe8eee2), Color(0xff476344)],
          'error': [Color(0xfff7e4de), Color(0xff9e3421)],
          'neutral': [Color(0xffe7ebea), Color(0xff4c6067)],
        };
        final colors = palette[b[3]]!;
        return Padding(
          padding: const EdgeInsets.only(bottom: 13),
          child: Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: colors[0],
              borderRadius: BorderRadius.circular(17),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Copy(b[1], size: 14, bold: true, color: colors[1]),
                const SizedBox(height: 10),
                Copy(b[2], size: 12, color: colors[1]),
              ],
            ),
          ),
        );
      case 'progress':
        return Padding(
          padding: const EdgeInsets.only(bottom: 16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const SizedBox(height: 7),
              LinearProgressIndicator(
                value: (b[1] as num).toDouble(),
                minHeight: 6,
                borderRadius: BorderRadius.circular(3),
                color: accent,
                backgroundColor: const Color(0xffdeded6),
              ),
              const SizedBox(height: 12),
              Copy(b[2], size: 12, color: muted),
            ],
          ),
        );
      case 'slider':
        final label = b[1] as String,
            value = values[b[1]] ?? (b[3] as num).toDouble();
        return Padding(
          padding: const EdgeInsets.only(bottom: 13),
          child: Surface(
            radius: 18,
            child: Column(
              children: [
                Padding(
                  padding: const EdgeInsets.fromLTRB(16, 16, 16, 0),
                  child: Row(
                    children: [
                      Expanded(child: Copy(label)),
                      Copy(
                        values.containsKey(label)
                            ? '${(value * 100).round()}'
                            : b[2],
                        bold: true,
                      ),
                    ],
                  ),
                ),
                Slider(
                  value: value.clamp(0, 1),
                  onChanged: (v) {
                    values[label] = v;
                    onChanged?.call();
                  },
                  onChangeEnd: (_) => onAction('slider:$label'),
                ),
              ],
            ),
          ),
        );
      case 'stepper':
        return Rows([
          [b[1], '−       ${b[2]}       +'],
        ], onAction: onAction);
      case 'code':
        return Padding(
          padding: const EdgeInsets.only(bottom: 15),
          child: Surface(
            radius: 20,
            padding: const EdgeInsets.symmetric(vertical: 30),
            child: Copy(b[1], size: 32, bold: true, center: true),
          ),
        );
      case 'devices':
        return DeviceGrid(
          keys: (b[1] as List).cast<String>(),
          onAction: onAction,
        );
      case 'skin':
        return Padding(
          padding: const EdgeInsets.only(bottom: 16),
          child: Surface(
            border: accent,
            padding: const EdgeInsets.all(12),
            child: Column(
              children: [
                SizedBox(
                  height: 135,
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(14),
                    child: LightBackground(
                      child: Padding(
                        padding: const EdgeInsets.all(12),
                        child: Row(
                          children: [
                            Expanded(
                              child: Surface(dark: true, child: product('ac')),
                            ),
                            const SizedBox(width: 9),
                            Expanded(child: Surface(child: product('lamp'))),
                          ],
                        ),
                      ),
                    ),
                  ),
                ),
                const SizedBox(height: 12),
                const Padding(
                  padding: EdgeInsets.fromLTRB(6, 0, 6, 12),
                  child: Row(
                    children: [
                      Expanded(
                        child: Copy('Window Light', size: 17, bold: true),
                      ),
                      Copy('Current', size: 12, color: accent, bold: true),
                    ],
                  ),
                ),
              ],
            ),
          ),
        );
      case 'skin_preview':
        return Padding(
          padding: const EdgeInsets.only(bottom: 15),
          child: SizedBox(
            height: 393,
            child: IgnorePointer(
              child: FittedBox(
                child: SizedBox(
                  width: 390,
                  height: 844,
                  child: skinPreview ?? const SizedBox(),
                ),
              ),
            ),
          ),
        );
      default:
        return const SizedBox.shrink();
    }
  }
}

Widget product(String key, {double size = 80}) =>
    ['ac', 'lamp', 'tv', 'sensor'].contains(key)
    ? Image.asset(
        'assets/$key.png',
        fit: BoxFit.contain,
        filterQuality: FilterQuality.high,
      )
    : Center(child: Glyph(key, size: 52, color: accent));

class Rows extends StatelessWidget {
  const Rows(this.items, {required this.onAction, this.title, super.key});
  final List items;
  final String? title;
  final ActionHandler onAction;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: 12),
    child: Surface(
      radius: 18,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (title != null)
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 13, 16, 7),
              child: Copy(title!, size: 14, bold: true),
            ),
          for (var i = 0; i < items.length; i++) ...[
            if (i > 0)
              const Padding(
                padding: EdgeInsets.symmetric(horizontal: 16),
                child: Divider(
                  height: .6,
                  thickness: .6,
                  color: Color(0xffdeded7),
                ),
              ),
            Builder(
              builder: (context) {
                final label = '${items[i][0]}', value = '${items[i][1]}';
                final toggle =
                    ['Power', 'Oscillation', 'Muted'].contains(label) &&
                    ['On', 'Off'].contains(value);
                return InkWell(
                  onTap: () => onAction(label),
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(minHeight: 44),
                    child: Padding(
                      padding: EdgeInsets.symmetric(
                        horizontal: 16,
                        vertical: toggle ? 0 : 12,
                      ),
                      child: LayoutBuilder(
                        builder: (context, c) {
                          final scale =
                              MediaQuery.textScalerOf(context).scale(13) / 13;
                          final estimate =
                              (label.length * 6.8 + value.length * 6.2) * scale;
                          final leading = Copy(label);
                          final trailing = toggle
                              ? Row(
                                  mainAxisSize: MainAxisSize.min,
                                  children: [
                                    Copy(value, size: 12, color: muted),
                                    const SizedBox(width: 8),
                                    SizedBox(
                                      height: 40,
                                      width: 44,
                                      child: FittedBox(
                                        child: Switch(
                                          value: value == 'On',
                                          onChanged: (_) => onAction(label),
                                        ),
                                      ),
                                    ),
                                  ],
                                )
                              : Copy(value, size: 12, color: muted);
                          if (!toggle && estimate > c.maxWidth - 18)
                            return Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                leading,
                                const SizedBox(height: 9),
                                trailing,
                              ],
                            );
                          return Row(
                            children: [
                              Expanded(child: leading),
                              const SizedBox(width: 10),
                              trailing,
                            ],
                          );
                        },
                      ),
                    ),
                  ),
                );
              },
            ),
          ],
        ],
      ),
    ),
  );
}

class Climate extends StatelessWidget {
  const Climate({
    required this.onAction,
    this.target = 24,
    this.mode = 'Cooling · Room 26°C',
    this.freshness = 'Updated just now',
    this.enabled = true,
    super.key,
  });
  final ActionHandler onAction;
  final double target;
  final String mode, freshness;
  final bool enabled;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(bottom: 12),
    child: Surface(
      dark: true,
      padding: const EdgeInsets.fromLTRB(31, 22, 31, 13),
      child: Column(
        children: [
          const Copy(
            'OBSERVED TARGET',
            size: 10,
            color: Color(0xffd3d1ca),
            bold: true,
          ),
          const SizedBox(height: 15),
          InkWell(
            onTap: () => onAction('Target temperature'),
            child: Copy(
              target.isNaN
                  ? '—'
                  : '${target == target.roundToDouble() ? target.toInt() : target}°',
              size: 68,
              color: const Color(0xfffffaf2),
              center: true,
            ),
          ),
          const SizedBox(height: 10),
          Copy(mode, size: 14, color: const Color(0xffe0ddd5), center: true),
          const SizedBox(height: 17),
          Row(
            children: [
              _step('−', 'Decrease target', const Color(0xff535452)),
              Expanded(
                child: Copy(
                  freshness,
                  size: 10,
                  color: const Color(0xffd3d1ca),
                  center: true,
                ),
              ),
              _step('+', 'Increase target', accent),
            ],
          ),
        ],
      ),
    ),
  );
  Widget _step(String text, String label, Color color) => Semantics(
    label: label,
    button: true,
    child: SizedBox(
      width: 48,
      height: 40,
      child: FilledButton(
        style: FilledButton.styleFrom(
          padding: EdgeInsets.zero,
          backgroundColor: color,
        ),
        onPressed: enabled ? () => onAction(label) : null,
        child: Text(text, style: const TextStyle(fontSize: 24)),
      ),
    ),
  );
}

const sampleNames = {
  'ac': ['Bedroom AC', 'Primary Bedroom', '24°C', 'Cooling'],
  'lamp': ['Floor lamp', 'Living Room', '65%', 'Brightness'],
  'tv': ['Living room TV', 'Living Room', 'Off', 'Power'],
  'sensor': ['Air sensor', 'Living Room', '22°C', '46% humidity'],
};

class DeviceGrid extends StatelessWidget {
  const DeviceGrid({
    required this.keys,
    required this.onAction,
    this.cached = false,
    this.large = false,
    this.data,
    super.key,
  });
  final List<String> keys;
  final List<List<String>>? data;
  final ActionHandler onAction;
  final bool cached, large;
  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, c) {
      final scale = MediaQuery.textScalerOf(context).scale(13) / 13;
      final columns = large || scale > 1.3 || c.maxWidth < 310
          ? 1
          : c.maxWidth > 780
          ? 3
          : 2;
      final width = (c.maxWidth - (columns - 1) * 12) / columns;
      return Wrap(
        spacing: 12,
        runSpacing: 14,
        children: List.generate(
          keys.length,
          (i) => SizedBox(
            width: width,
            child: DeviceTile(
              keys[i],
              data: data?[i],
              cached: cached,
              onAction: onAction,
              large: columns == 1,
            ),
          ),
        ),
      );
    },
  );
}

class DeviceTile extends StatelessWidget {
  const DeviceTile(
    this.deviceKey, {
    required this.onAction,
    this.cached = false,
    this.large = false,
    this.data,
    super.key,
  });
  final String deviceKey;
  final List<String>? data;
  final ActionHandler onAction;
  final bool cached, large;
  @override
  Widget build(BuildContext context) {
    final kind = sampleNames.containsKey(deviceKey) ? deviceKey : 'ac',
        d = data ?? sampleNames[kind]!;
    final dark = kind == 'ac',
        color = dark ? const Color(0xfffffaf1) : ink,
        secondary = dark ? const Color(0xffd3d1ca) : muted;
    final scale = MediaQuery.textScalerOf(context).scale(13) / 13;
    final largeText = scale > 1.3;
    final active = kind != 'tv' && !cached;
    Widget power() => Semantics(
      label: 'Power ${d[0]}',
      button: true,
      child: Tooltip(
        message: 'Power ${d[0]}',
        child: InkWell(
          onTap: () => onAction('power:$deviceKey'),
          borderRadius: BorderRadius.circular(24),
          child: SizedBox.square(
            dimension: 48,
            child: Center(
              child: Container(
                width: 42,
                height: 42,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: active ? accent : const Color(0xffe0ddd5),
                ),
                child: Center(
                  child: Glyph(
                    'power',
                    size: 22,
                    color: active ? Colors.white : ink,
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
    Widget textAt(double top, Widget child, {double right = 14}) =>
        Positioned(left: 14, right: right, top: top, child: child);
    return Surface(
      dark: dark,
      child: Stack(
        children: [
          Semantics(
            label: 'Open ${d[0]}',
            button: true,
            child: InkWell(
              borderRadius: BorderRadius.circular(22),
              onTap: () => onAction('open:$deviceKey'),
              onLongPress: () => onAction('menu:$deviceKey'),
              child: largeText
                  ? Padding(
                      padding: const EdgeInsets.all(14),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          SizedBox(
                            height: 98,
                            child: Center(child: product(kind)),
                          ),
                          const SizedBox(height: 12),
                          Copy(d[0], size: 19, bold: true, color: color),
                          const SizedBox(height: 8),
                          Copy(d[1], size: 14, color: secondary),
                          const SizedBox(height: 12),
                          Copy(d[2], size: 26, bold: true, color: color),
                          const SizedBox(height: 8),
                          Copy(d[3], size: 14, color: secondary),
                          const SizedBox(height: 48),
                        ],
                      ),
                    )
                  : SizedBox(
                      height: large ? 197 : 233,
                      child: Stack(
                        children: [
                          Positioned(
                            left: large ? 18 : 15,
                            top: large ? 22 : 12,
                            width: large ? 100 : null,
                            right: large ? null : 15,
                            height: large ? 95 : 98,
                            child: product(kind),
                          ),
                          if (large) ...[
                            Positioned(
                              left: 138,
                              right: 14,
                              top: 23,
                              child: Copy(
                                d[0],
                                size: 19,
                                bold: true,
                                color: color,
                              ),
                            ),
                            Positioned(
                              left: 138,
                              right: 14,
                              top: 77,
                              child: Copy(d[1], size: 14, color: secondary),
                            ),
                            Positioned(
                              left: 138,
                              right: 14,
                              top: 116,
                              child: Copy(
                                d[2],
                                size: 26,
                                bold: true,
                                color: color,
                              ),
                            ),
                            Positioned(
                              left: 138,
                              right: 14,
                              top: 154,
                              child: Copy(d[3], size: 14, color: secondary),
                            ),
                          ] else ...[
                            textAt(
                              117,
                              Copy(d[0], size: 15, bold: true, color: color),
                            ),
                            textAt(143, Copy(d[1], size: 11, color: secondary)),
                            textAt(
                              174,
                              Copy(d[2], size: 24, bold: true, color: color),
                              right: 50,
                            ),
                            textAt(
                              207,
                              Copy(
                                cached ? 'Cached · 2h ago' : d[3],
                                size: 10,
                                color: secondary,
                              ),
                            ),
                          ],
                        ],
                      ),
                    ),
            ),
          ),
          if (kind != 'sensor')
            Positioned(right: 8, bottom: 19, child: power()),
          Positioned(
            right: 0,
            top: 0,
            child: Semantics(
              label: 'Menu ${d[0]}',
              button: true,
              child: Tooltip(
                message: 'Menu ${d[0]}',
                child: InkWell(
                  onTap: () => onAction('menu:$deviceKey'),
                  child: const SizedBox(width: 48, height: 48),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
