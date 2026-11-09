import 'package:flutter/material.dart';
import 'skin.dart';
import 'localization.dart';
import 'device_presentation.dart';

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
    children: blocks.map((b) {
      final child = block(context, b as List);
      return b[0] == 'status'
          ? Semantics(liveRegion: true, child: child)
          : child;
    }).toList(),
  );
  Widget block(BuildContext context, List b) {
    final type = b[0];
    switch (type) {
      case 'note':
        final note = Padding(
          padding: const EdgeInsets.fromLTRB(2, 2, 2, 10),
          child: Copy(b[1], color: muted, paragraph: true),
        );
        return b.length > 2
            ? Semantics(
                button: true,
                child: InkWell(onTap: () => onAction(b[2]), child: note),
              )
            : note;
      case 'section':
        return Padding(
          padding: const EdgeInsets.fromLTRB(3, 4, 3, 7),
          child: Copy(b[1], size: 10, color: muted, bold: true),
        );
      case 'link':
        return Align(
          alignment: Alignment.centerLeft,
          child: TextButton(
            style: TextButton.styleFrom(
              padding: const EdgeInsets.symmetric(horizontal: 2),
              minimumSize: const Size(48, 40.2),
              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
            ),
            onPressed: () => onAction(b[1]),
            child: Copy(
              b[1],
              color: skinAccent(context),
              bold: true,
              control: true,
            ),
          ),
        );
      case 'summary':
        return Padding(
          padding: const EdgeInsets.only(bottom: 16),
          child: Surface(
            padding: const EdgeInsets.all(20),
            child: Row(
              children: [
                Icon(
                  Icons.person_outline_rounded,
                  size: 30,
                  color: MediaQuery.highContrastOf(context)
                      ? Colors.black
                      : skinAccent(context),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Copy(b[1], size: 20, bold: true),
                      const SizedBox(height: 4),
                      Copy(b[2], size: 12, color: muted),
                    ],
                  ),
                ),
              ],
            ),
          ),
        );
      case 'disclosure':
        return Padding(
          padding: const EdgeInsets.only(bottom: 12),
          child: Surface(
            padding: EdgeInsets.zero,
            child: ExpansionTile(
              key: PageStorageKey('${b[1]}:${b.length > 3 ? b[3] : ''}'),
              shape: const Border(),
              collapsedShape: const Border(),
              title: Copy(b[1], bold: true),
              childrenPadding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
              children: [
                Blocks(
                  b[2] as List,
                  onAction: onAction,
                  fields: fields,
                  choices: choices,
                  values: values,
                  onChanged: onChanged,
                ),
              ],
            ),
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
          loading: b.length > 3 && b[3] == true,
          onPressed: b.length > 2 && b[2] == 'disabled'
              ? null
              : () => onAction(b[1]),
        );
      case 'field':
        final label = b[1] as String;
        final hint = label == 'Account name' ? label : 'Search by name';
        final field = fields.putIfAbsent(
          label,
          () => TextEditingController(text: b[2]),
        );
        return Padding(
          padding: EdgeInsets.only(bottom: b.length > 3 ? 2 : 16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 2),
                child: Copy(label, size: 12, color: muted, paragraph: true),
              ),
              const SizedBox(height: 5.2),
              Surface(
                radius: 14,
                border: b.length > 3 ? const Color(0xffae452d) : null,
                borderWidth: b.length > 3 ? 1 : .6,
                child: Semantics(
                  label: localize(context, label),
                  child: ValueListenableBuilder<TextEditingValue>(
                    valueListenable: field,
                    builder: (context, value, _) => TextField(
                      controller: field,
                      textAlignVertical: TextAlignVertical.center,
                      key: ValueKey('field:$label'),
                      onChanged: (_) => onChanged?.call(),
                      keyboardType: label == 'Target temperature'
                          ? const TextInputType.numberWithOptions(
                              decimal: true,
                              signed: true,
                            )
                          : TextInputType.text,
                      style: windowFieldTextStyle,
                      decoration: InputDecoration(
                        contentPadding: fieldTextPadding(
                          context,
                          value.text.isEmpty ? hint : value.text,
                        ),
                        border: InputBorder.none,
                        hintText: localize(context, hint),
                        suffixIcon: label.startsWith('Search')
                            ? IconButton(
                                tooltip: localize(context, 'Clear search'),
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
                ),
              ),
              if (b.length > 3)
                Padding(
                  padding: const EdgeInsets.fromLTRB(2, 9, 2, 0),
                  child: Copy(
                    b[3],
                    size: 12,
                    color: const Color(0xff9e3421),
                    paragraph: true,
                  ),
                ),
            ],
          ),
        );
      case 'choices':
        final items = (b[1] as List).cast<Object>();
        final key = items.join('|');
        final initial = b.length > 2 ? b[2] : 0;
        final multiple = initial is List;
        final selected = choices.putIfAbsent(
          key,
          () => multiple
              ? initial.fold<int>(
                  0,
                  (mask, index) => mask | (1 << (index as int)),
                )
              : initial as int,
        );
        bool isSelected(int index) =>
            multiple ? (selected & (1 << index)) != 0 : selected == index;
        return Padding(
          padding: const EdgeInsets.only(bottom: 4),
          child: Column(
            children: List.generate(
              items.length,
              (i) => ChoiceTile(
                label: items[i],
                selected: isSelected(i),
                multiple: multiple,
                onTap: () {
                  choices[key] = multiple ? selected ^ (1 << i) : i;
                  onAction('choice:${items[i]}');
                  onChanged?.call();
                },
              ),
            ),
          ),
        );
      case 'chips':
      case 'tabs':
        if (MediaQuery.textScalerOf(context).scale(12) <= 12) {
          return Wrap(
            spacing: 7,
            children: List.generate((b[1] as List).length, (i) {
              final label = b[1][i] as String;
              final selected = type == 'tabs'
                  ? i == (b.length > 3 ? b[2] : choices['tabs'] ?? b[2])
                  : label == '3';
              final measure = TextPainter(
                text: TextSpan(
                  text: localize(context, label),
                  style: const TextStyle(fontFamily: 'Inter', fontSize: 12),
                ),
                textDirection: Directionality.of(context),
              )..layout();
              final width = measure.width + 24;
              measure.dispose();
              return Semantics(
                selected: selected,
                button: true,
                child: SizedBox(
                  width: width,
                  height: 48,
                  child: InkWell(
                    onTap: () {
                      if (b.length <= 3) choices['tabs'] = i;
                      onAction(
                        type == 'tabs' && b.length > 3 ? b[3][i] : label,
                      );
                      onChanged?.call();
                    },
                    child: Stack(
                      children: [
                        Positioned(
                          top: 0,
                          left: 0,
                          right: 0,
                          height: 32,
                          child: DecoratedBox(
                            decoration: BoxDecoration(
                              color: selected ? ink : const Color(0xfff8f5ee),
                              borderRadius: BorderRadius.circular(16),
                            ),
                          ),
                        ),
                        Positioned(
                          top: 0,
                          height: 32,
                          left: 12,
                          right: 12,
                          child: Center(
                            child: Copy(
                              label,
                              size: 12,
                              color: selected ? Colors.white : muted,
                              control: true,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              );
            }),
          );
        }
        return Padding(
          padding: const EdgeInsets.only(bottom: 15),
          child: Wrap(
            spacing: 7,
            runSpacing: 8,
            children: List.generate((b[1] as List).length, (i) {
              final label = b[1][i] as String;
              final selected = type == 'tabs' && b.length > 3
                  ? b[2] as int
                  : choices['tabs'] ?? (type == 'tabs' ? b[2] as int : 2);
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
                    control: true,
                  ),
                  onPressed: () {
                    if (b.length <= 3) choices['tabs'] = i;
                    onAction(type == 'tabs' && b.length > 3 ? b[3][i] : label);
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
                      top: ['ac', 'lamp', 'tv', 'sensor'].contains(b[1])
                          ? 13
                          : 27,
                      left: ['ac', 'lamp', 'tv', 'sensor'].contains(b[1])
                          ? 75
                          : 0,
                      right: ['ac', 'lamp', 'tv', 'sensor'].contains(b[1])
                          ? 75
                          : 0,
                      height: ['ac', 'lamp', 'tv', 'sensor'].contains(b[1])
                          ? 87
                          : 52,
                      child: product(b[1]),
                    ),
                    Positioned(
                      top: 119,
                      left: 10,
                      right: 10,
                      child: Copy(
                        b[2],
                        size: 23,
                        bold: true,
                        center: true,
                        maxLines: 1,
                      ),
                    ),
                    Positioned(
                      top: 158,
                      left: 20,
                      right: 20,
                      child: Copy(b[3], size: 12, color: muted, maxLines: 1),
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
          loading: b.length > 3 && b[3] == true,
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
        if (MediaQuery.textScalerOf(context).scale(13) <= 13) {
          return LayoutBuilder(
            builder: (context, constraints) {
              final text = TextPainter(
                text: TextSpan(
                  text: localize(context, b[2]),
                  style: const TextStyle(fontFamily: 'Inter', fontSize: 12),
                ),
                textDirection: Directionality.of(context),
              )..layout();
              final lines = (text.width / (constraints.maxWidth - 32))
                  .ceil()
                  .clamp(1, 100);
              text.dispose();
              return Padding(
                padding: const EdgeInsets.only(bottom: 13),
                child: Container(
                  height: 52 + lines * 17.0,
                  decoration: BoxDecoration(
                    color: colors[0],
                    borderRadius: BorderRadius.circular(17),
                  ),
                  child: Stack(
                    children: [
                      Positioned(
                        top: 15,
                        left: 16,
                        right: 16,
                        child: Copy(
                          b[1],
                          size: 14,
                          bold: true,
                          color: colors[1],
                        ),
                      ),
                      Positioned(
                        top: 39,
                        left: 16,
                        right: 16,
                        child: Copy(b[2], size: 12, color: colors[1]),
                      ),
                    ],
                  ),
                ),
              );
            },
          );
        }
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
        // Match the prototype's total of 38 pixels plus the caption height.
        return Padding(
          padding: const EdgeInsets.only(bottom: 13),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const SizedBox(height: 7),
              LinearProgressIndicator(
                value: (b[1] as num).toDouble(),
                minHeight: 6,
                borderRadius: BorderRadius.circular(3),
                color: skinAccent(context),
                backgroundColor: const Color(0xffdeded6),
              ),
              const SizedBox(height: 12),
              Copy(b[2], size: 12, color: muted, paragraph: true),
            ],
          ),
        );
      case 'slider':
        final label = b[1] as String,
            value = values[b[1]] ?? (b[3] as num).toDouble();
        return ValueSlider(
          label: label,
          displayValue: values.containsKey(label)
              ? '${(value * 100).round()}'
              : b[2],
          value: value,
          formatValue: (v) => '$label ${(v * 100).round()}',
          onChanged: (v) {
            values[label] = v;
            onChanged?.call();
          },
          onChangeEnd: (_) => onAction('slider:$label'),
        );
      case 'stepper':
        return Rows([
          [b[1], '−       ${b[2]}       +'],
        ], onAction: onAction);
      case 'code':
        final displayCode = (b[1] as String)
            .split(' ')
            .where((part) => part.isNotEmpty)
            .join(' ');
        if (MediaQuery.textScalerOf(context).scale(13) <= 13) {
          return Padding(
            padding: const EdgeInsets.only(bottom: 15),
            child: Surface(
              radius: 20,
              child: SizedBox(
                height: 94,
                child: Center(
                  child: Copy(
                    displayCode,
                    size: 32,
                    bold: true,
                    center: true,
                    control: true,
                  ),
                ),
              ),
            ),
          );
        }
        return Padding(
          padding: const EdgeInsets.only(bottom: 15),
          child: Surface(
            radius: 20,
            padding: const EdgeInsets.symmetric(vertical: 30),
            child: Copy(
              displayCode,
              size: 32,
              bold: true,
              center: true,
              control: true,
            ),
          ),
        );
      case 'devices':
        return Padding(
          padding: const EdgeInsets.only(bottom: 14),
          child: DeviceGrid(
            keys: (b[1] as List).cast<String>(),
            onAction: onAction,
          ),
        );
      case 'skin':
        if (MediaQuery.textScalerOf(context).scale(13) <= 13) {
          return Padding(
            padding: const EdgeInsets.only(bottom: 16),
            child: Surface(
              border: skinAccent(context),
              borderWidth: 1.2,
              child: SizedBox(
                height: 205,
                child: Stack(
                  children: [
                    Positioned(
                      left: 12,
                      right: 12,
                      top: 12,
                      height: 135,
                      child: FittedBox(
                        child: SizedBox(
                          width: 166,
                          height: 100,
                          child: LightBackground(
                            child: Stack(
                              children: [
                                const Positioned(
                                  left: 10,
                                  top: 12,
                                  width: 65,
                                  height: 76,
                                  child: Surface(
                                    dark: true,
                                    radius: 12,
                                    child: SizedBox(),
                                  ),
                                ),
                                const Positioned(
                                  left: 84,
                                  top: 12,
                                  width: 71,
                                  height: 76,
                                  child: Surface(radius: 12, child: SizedBox()),
                                ),
                                Positioned(
                                  left: 17,
                                  top: 23,
                                  width: 52,
                                  height: 30,
                                  child: product('ac'),
                                ),
                                Positioned(
                                  left: 104,
                                  top: 20,
                                  width: 31,
                                  height: 47,
                                  child: product('lamp'),
                                ),
                                Positioned(
                                  left: 22,
                                  top: 65,
                                  width: 40,
                                  height: 7,
                                  child: DecoratedBox(
                                    decoration: BoxDecoration(
                                      color: skinAccent(context),
                                      borderRadius: BorderRadius.circular(3),
                                    ),
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ),
                      ),
                    ),
                    const Positioned(
                      left: 18,
                      top: 147,
                      height: 58,
                      child: Center(
                        child: Copy(
                          'Window Light',
                          size: 17,
                          bold: true,
                          control: true,
                        ),
                      ),
                    ),
                    Positioned(
                      right: 18,
                      top: 147,
                      height: 58,
                      child: Center(
                        child: Copy(
                          'Current',
                          control: true,
                          size: 12,
                          color: skinAccent(context),
                          bold: true,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          );
        }
        return Padding(
          padding: const EdgeInsets.only(bottom: 16),
          child: Surface(
            border: skinAccent(context),
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
                Padding(
                  padding: const EdgeInsets.fromLTRB(6, 0, 6, 12),
                  child: Row(
                    children: [
                      Expanded(
                        child: Copy(
                          'Window Light',
                          size: 17,
                          bold: true,
                          control: true,
                        ),
                      ),
                      Copy(
                        'Current',
                        size: 12,
                        color: skinAccent(context),
                        bold: true,
                        control: true,
                      ),
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

Widget product(String key, {double size = 80}) {
  const dimensions = {
    'ac': Size(216, 77),
    'lamp': Size(87, 137),
    'tv': Size(172, 111),
    'sensor': Size(84, 98),
    'refrigerator': Size(1254, 1254),
    'dishwasher': Size(1254, 1254),
  };
  final source = dimensions[key];
  if (source == null)
    return Center(
      child: key == 'appliance'
          ? Builder(
              builder: (context) => Icon(
                Icons.devices_other_outlined,
                size: 52,
                color: skinAccent(context),
              ),
            )
          : Builder(
              builder: (context) =>
                  Glyph(key, size: 52, color: skinAccent(context)),
            ),
    );
  final asset = {'refrigerator', 'dishwasher'}.contains(key)
      ? 'assets/$key-v1.png'
      : 'assets/$key.png';
  return LayoutBuilder(
    builder: (context, constraints) {
      if (!constraints.hasBoundedWidth || !constraints.hasBoundedHeight) {
        return Image.asset(
          asset,
          fit: BoxFit.contain,
          filterQuality: FilterQuality.high,
        );
      }
      final ratio = MediaQuery.devicePixelRatioOf(context);
      final fitted = applyBoxFit(BoxFit.contain, source, constraints.biggest);
      double pixel(double value) {
        final scaled = value * ratio;
        final lower = scaled.floor();
        final fraction = scaled - lower;
        final rounded = fraction > .5 || (fraction == .5 && lower.isOdd)
            ? lower + 1
            : lower;
        return rounded / ratio;
      }

      final width = pixel(fitted.destination.width);
      final height = pixel(fitted.destination.height);
      // Keep the image bounds on physical pixels, as in the source artwork.
      return Stack(
        children: [
          Positioned(
            left: pixel((constraints.maxWidth - width) / 2),
            top: pixel((constraints.maxHeight - height) / 2),
            width: width,
            height: height,
            child: Image.asset(
              asset,
              cacheWidth: source.width > 1000
                  ? (width * ratio).ceil().clamp(1, source.width.toInt())
                  : null,
              fit: BoxFit.fill,
              filterQuality: FilterQuality.high,
            ),
          ),
        ],
      );
    },
  );
}

class Rows extends StatelessWidget {
  const Rows(
    this.items, {
    required this.onAction,
    this.title,
    this.trailing = const {},
    super.key,
  });
  final List items;
  final Object? title;
  final Map<int, Widget> trailing;
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
              padding: EdgeInsets.symmetric(
                horizontal: 16,
                vertical: MediaQuery.highContrastOf(context) ? 8 : 7,
              ),
              child: Copy(title!, size: 14, bold: true, control: true),
            ),
          for (var i = 0; i < items.length; i++) ...[
            if (i > 0)
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                child: CustomPaint(
                  painter: _RowDivider(MediaQuery.devicePixelRatioOf(context)),
                  child: const SizedBox(height: 0, width: double.infinity),
                ),
              ),
            Builder(
              builder: (context) {
                final label = '${items[i][0]}', value = '${items[i][1]}';
                final action = items[i].length > 2
                    ? items[i][2] as String?
                    : label;
                final toggle =
                    ['Power', 'Oscillation', 'Muted'].contains(label) &&
                    ['On', 'Off'].contains(value);
                final stepper =
                    ['Target', 'Speed'].contains(label) && value.contains('+');
                final rowTrailing = items[i].length > 3 && items[i][3] is Widget
                    ? items[i][3] as Widget
                    : null;
                final inline =
                    stepper || trailing.containsKey(i) || rowTrailing != null;
                return InkWell(
                  canRequestFocus: !toggle,
                  excludeFromSemantics: toggle,
                  key: action == null ? null : ValueKey(action),
                  onTap: action == null ? null : () => onAction(action),
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(minHeight: 44),
                    child: Padding(
                      padding: EdgeInsets.fromLTRB(
                        16,
                        toggle || inline ? 0 : 12,
                        inline ? 8 : 16,
                        toggle || inline ? 0 : 12,
                      ),
                      child: LayoutBuilder(
                        builder: (context, c) {
                          final scale =
                              MediaQuery.textScalerOf(context).scale(13) / 13;
                          final measurement = TextPainter(
                            text: TextSpan(
                              text: localize(context, items[i][1]),
                              style: const TextStyle(
                                fontFamily: 'Inter',
                                fontSize: 12,
                                fontWeight: FontWeight.w400,
                                letterSpacing: 0,
                              ),
                            ),
                            textDirection: Directionality.of(context),
                            textScaler: MediaQuery.textScalerOf(context),
                          )..layout();
                          final valueWidth = measurement.width;
                          measurement.text = TextSpan(
                            text: localize(context, items[i][0]),
                            style: const TextStyle(
                              fontFamily: 'Inter',
                              fontSize: 13,
                              fontWeight: FontWeight.w400,
                              letterSpacing: 0,
                            ),
                          );
                          measurement.layout();
                          final combinedWidth = measurement.width + valueWidth;
                          measurement.dispose();
                          final leading = Copy(items[i][0], control: true);
                          final trailing =
                              this.trailing[i] ??
                              rowTrailing ??
                              (stepper
                                  ? InlineStepper(
                                      label: label,
                                      value: value
                                          .replaceFirst('−', '')
                                          .replaceFirst('+', '')
                                          .trim(),
                                      onIncrease: action == null
                                          ? null
                                          : () => onAction(
                                              'preview-step:increase:$label',
                                            ),
                                      onDecrease: action == null
                                          ? null
                                          : () => onAction(
                                              'preview-step:decrease:$label',
                                            ),
                                    )
                                  : toggle
                                  ? Row(
                                      mainAxisSize: MainAxisSize.min,
                                      children: [
                                        Copy(
                                          items[i][1],
                                          size: 12,
                                          color: muted,
                                          control: true,
                                        ),
                                        const SizedBox(width: 8),
                                        SizedBox(
                                          height: 40,
                                          width: 44,
                                          child: FittedBox(
                                            child: Semantics(
                                              label: localize(context, label),
                                              child: Switch(
                                                value: value == 'On',
                                                onChanged: action == null
                                                    ? null
                                                    : (_) => onAction(action),
                                              ),
                                            ),
                                          ),
                                        ),
                                      ],
                                    )
                                  : value.contains('≡')
                                  ? Row(
                                      mainAxisSize: MainAxisSize.min,
                                      children: [
                                        Copy(
                                          value.replaceAll('≡', '').trim(),
                                          control: true,
                                          size: 12,
                                          color: muted,
                                        ),
                                        const SizedBox(width: 13),
                                        const Glyph(
                                          'drag',
                                          size: 11,
                                          color: muted,
                                        ),
                                      ],
                                    )
                                  : Copy(
                                      items[i][1],
                                      size: 12,
                                      color: muted,
                                      control: true,
                                    ));
                          if ((!toggle &&
                                  !inline &&
                                  combinedWidth > c.maxWidth - 18) ||
                              (inline && scale > 1))
                            return Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              mainAxisAlignment: MainAxisAlignment.center,
                              children: [
                                leading,
                                SizedBox(height: scale <= 1 ? 3 : 9),
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

class _RowDivider extends CustomPainter {
  const _RowDivider(this.pixelRatio);
  final double pixelRatio;

  @override
  void paint(Canvas canvas, Size size) {
    final thickness =
        (.6 * pixelRatio).round().clamp(1, double.infinity) / pixelRatio;
    // The prototype line includes its endpoint and starts at the row boundary.
    canvas.drawRect(
      Rect.fromLTWH(0, 0, size.width + 1 / pixelRatio, thickness),
      Paint()
        ..color = const Color(0xffdeded7)
        ..isAntiAlias = false,
    );
  }

  @override
  bool shouldRepaint(_RowDivider oldDelegate) =>
      oldDelegate.pixelRatio != pixelRatio;
}

class Climate extends StatelessWidget {
  const Climate({
    required this.onAction,
    this.target = 24,
    this.mode = 'Cooling · Room 26°C',
    this.freshness = 'Updated just now',
    this.enabled = true,
    this.loading = false,
    super.key,
  });
  final ActionHandler onAction;
  final double target;
  final String mode, freshness;
  final bool enabled, loading;
  @override
  Widget build(BuildContext context) {
    if (MediaQuery.textScalerOf(context).scale(13) <= 13) {
      return Padding(
        padding: const EdgeInsets.only(bottom: 12),
        child: Surface(
          dark: true,
          child: SizedBox(
            height: 218,
            child: Stack(
              children: [
                const Positioned(
                  top: 22,
                  left: 0,
                  right: 0,
                  child: Copy(
                    'OBSERVED TARGET',
                    size: 10,
                    color: Color(0xffd3d1ca),
                    bold: true,
                    center: true,
                  ),
                ),
                Positioned(
                  top: 52,
                  left: 0,
                  right: 0,
                  child: InkWell(
                    onTap: enabled
                        ? () => onAction('Target temperature')
                        : null,
                    child: loading
                        ? const Center(
                            child: SizedBox.square(
                              dimension: 24,
                              child: CircularProgressIndicator(
                                color: Colors.white,
                                strokeWidth: 2,
                              ),
                            ),
                          )
                        : Copy(
                            !target.isFinite
                                ? '—'
                                : '${target == target.roundToDouble() ? target.toInt() : target}°',
                            size: 68,
                            color: const Color(0xfffffaf2),
                            center: true,
                          ),
                  ),
                ),
                Positioned(
                  top: 134,
                  left: 20,
                  right: 20,
                  child: Copy(
                    mode,
                    size: 14,
                    color: const Color(0xffe0ddd5),
                    center: true,
                  ),
                ),
                Positioned(
                  top: 161,
                  left: 30.5,
                  child: _step('−', 'Decrease target', const Color(0xff535452)),
                ),
                Positioned(
                  top: 161,
                  right: 30.5,
                  child: _step(
                    '+',
                    'Increase target',
                    enabled ? skinAccent(context) : const Color(0xff535452),
                  ),
                ),
                Positioned(
                  top: 165,
                  height: 40,
                  left: 80,
                  right: 80,
                  child: Center(
                    child: Copy(
                      freshness,
                      size: 10,
                      color: const Color(0xffd3d1ca),
                      center: true,
                      control: true,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      );
    }
    return Padding(
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
              onTap: enabled ? () => onAction('Target temperature') : null,
              child: loading
                  ? const SizedBox.square(
                      dimension: 24,
                      child: CircularProgressIndicator(
                        color: Colors.white,
                        strokeWidth: 2,
                      ),
                    )
                  : Copy(
                      !target.isFinite
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
                _step('+', 'Increase target', skinAccent(context)),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _step(String text, String label, Color color) => Builder(
    builder: (context) => Semantics(
      label: localize(context, label),
      button: true,
      enabled: enabled,
      child: SizedBox(
        width: 48,
        height: 48,
        child: InkWell(
          onTap: enabled ? () => onAction(label) : null,
          borderRadius: BorderRadius.circular(24),
          child: Center(
            child: Container(
              width: 47,
              height: 40,
              decoration: BoxDecoration(
                color: color,
                borderRadius: BorderRadius.circular(20),
              ),
              child: Center(
                child: Glyph(
                  text == '+' ? 'step-plus' : 'step-minus',
                  size: 24,
                  color: Colors.white,
                ),
              ),
            ),
          ),
        ),
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

/// Decorative outlined appliance symbol. Device capabilities, never this
/// symbol, determine which controls are offered.
class CardDeviceIcon extends StatelessWidget {
  const CardDeviceIcon(this.kind, {super.key});
  final String kind;

  @override
  Widget build(BuildContext context) => ExcludeSemantics(
    child: Center(
      child: SizedBox.square(
        dimension: 42,
        child: CustomPaint(
          painter: _CardDeviceIconPainter(
            kind,
            MediaQuery.highContrastOf(context) ? Colors.black : muted,
          ),
        ),
      ),
    ),
  );
}

class _CardDeviceIconPainter extends CustomPainter {
  const _CardDeviceIconPainter(this.kind, this.color);
  final String kind;
  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    canvas.save();
    canvas.scale(size.width / 24, size.height / 24);
    final pen = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;
    void line(double x1, double y1, double x2, double y2) =>
        canvas.drawLine(Offset(x1, y1), Offset(x2, y2), pen);
    void rounded(double x, double y, double w, double h, double r) =>
        canvas.drawRRect(
          RRect.fromRectAndRadius(
            Rect.fromLTWH(x, y, w, h),
            Radius.circular(r),
          ),
          pen,
        );
    void circle(double x, double y, double r) =>
        canvas.drawCircle(Offset(x, y), r, pen);
    void path(void Function(Path) draw) {
      final shape = Path();
      draw(shape);
      canvas.drawPath(shape, pen);
    }

    switch (kind) {
      case 'ac':
        rounded(2, 4, 20, 11, 2);
        line(6, 8, 18, 8);
        line(6, 12, 18, 12);
        path(
          (p) => p
            ..moveTo(8, 15)
            ..cubicTo(8, 17, 6, 17, 6, 19),
        );
        path(
          (p) => p
            ..moveTo(12, 15)
            ..lineTo(12, 21),
        );
        path(
          (p) => p
            ..moveTo(16, 15)
            ..cubicTo(16, 17, 18, 17, 18, 19),
        );
      case 'refrigerator':
        rounded(5, 2, 14, 20, 2);
        line(5, 10, 19, 10);
        line(9, 5, 9, 7);
        line(9, 13, 9, 17);
      case 'dishwasher':
        rounded(3, 2, 18, 20, 2);
        line(3, 7, 21, 7);
        circle(7, 4.5, .5);
        circle(10, 4.5, .5);
        circle(12, 14, 4);
      case 'fan':
        circle(12, 12, 9);
        circle(12, 12, 1.4);
        path(
          (p) => p
            ..moveTo(12, 10.6)
            ..cubicTo(8, 5, 14, 4, 14, 8)
            ..lineTo(13, 11),
        );
        path(
          (p) => p
            ..moveTo(13.2, 12.7)
            ..cubicTo(20, 11, 19, 17, 15, 16)
            ..lineTo(12.8, 13.3),
        );
        path(
          (p) => p
            ..moveTo(11, 13)
            ..cubicTo(10, 20, 5, 17, 8, 14)
            ..lineTo(10.6, 12),
        );
      case 'lamp':
        path(
          (p) => p
            ..moveTo(8, 15)
            ..cubicTo(8, 13, 5, 12, 5, 9)
            ..cubicTo(5, 5, 8, 2, 12, 2)
            ..cubicTo(16, 2, 19, 5, 19, 9)
            ..cubicTo(19, 12, 16, 13, 16, 15)
            ..close(),
        );
        line(9, 18, 15, 18);
        line(10, 21, 14, 21);
      case 'tv':
        rounded(2, 4, 20, 14, 2);
        line(8, 22, 16, 22);
        line(12, 18, 12, 22);
      case 'sensor':
        rounded(4, 3, 16, 18, 3);
        circle(12, 10, 3);
        line(9, 17, 15, 17);
      default:
        rounded(4, 3, 16, 18, 2);
        line(8, 7, 16, 7);
        circle(12, 15, 3);
    }
    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant _CardDeviceIconPainter oldDelegate) =>
      oldDelegate.kind != kind || oldDelegate.color != color;
}

class DeviceGrid extends StatelessWidget {
  const DeviceGrid({
    required this.keys,
    required this.onAction,
    this.cached = false,
    this.large = false,
    this.data,
    this.presentations = const {},
    super.key,
  });
  final List<String> keys;
  final List<List<Object>>? data;
  final Map<String, DevicePresentation> presentations;
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
      final gap = skinOf(context).compactDeviceCards ? 10.0 : 12.0;
      final width = (c.maxWidth - (columns - 1) * gap) / columns;
      return Wrap(
        spacing: gap,
        runSpacing: skinOf(context).compactDeviceCards ? 10 : (large ? 16 : 14),
        children: List.generate(
          keys.length,
          (i) => SizedBox(
            width: width,
            child: RepaintBoundary(
              child: DeviceTile(
                keys[i],
                data: data?[i],
                presentation: presentations[keys[i]],
                cached: cached,
                onAction: onAction,
                large: columns == 1,
              ),
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
    this.compactHeight = 233,
    this.data,
    this.presentation,
    super.key,
  });
  final String deviceKey;
  final double compactHeight;
  final List<Object>? data;
  final DevicePresentation? presentation;
  final ActionHandler onAction;
  final bool cached, large;
  @override
  Widget build(BuildContext context) {
    final kind =
            presentation?.artwork ??
            (sampleNames.containsKey(deviceKey) ? deviceKey : 'ac'),
        d = data ?? sampleNames[kind] ?? sampleNames['ac']!;
    final hasCaption = d[3].toString().isNotEmpty;
    final skin = skinOf(context);
    final compact = skin.compactDeviceCards;
    final highContrast = MediaQuery.highContrastOf(context);
    const color = ink, secondary = muted;
    final scale = MediaQuery.textScalerOf(context).scale(13) / 13;
    final largeText = scale > 1.3;
    final active = kind != 'tv' && !cached;
    final state = presentation?.power ?? false;
    final updating = presentation?.updating ?? false;
    final powerEnabled = (presentation?.powerEnabled ?? true) && !updating;
    final powerAction = state ? 'Turn Off' : 'Turn On';
    final powerColor = compact && !highContrast
        ? ((presentation == null ? active : state == true)
              ? skin.cardPowerOn
              : skin.cardPowerOff)
        : presentation == null
        ? (active ? skinAccent(context) : const Color(0xffe0ddd5))
        : powerBackground(context, state);
    final powerInk = compact && !highContrast
        ? ((presentation == null ? active : state == true)
              ? skin.cardPowerInk
              : const Color(0xff42596c))
        : presentation == null
        ? (active ? Colors.white : ink)
        : powerForeground(context, state);
    Widget power() => Semantics(
      container: compact,
      label: localize(context, UiMessage('Power {name}', {'name': d[0]})),
      button: true,
      enabled: powerEnabled,
      toggled: presentation == null ? null : state,
      liveRegion: updating,
      value: presentation == null
          ? null
          : localize(
              context,
              updating
                  ? 'Updating device…'
                  : state
                  ? 'Powered on'
                  : 'Powered off',
            ),
      child: Tooltip(
        message: localize(
          context,
          presentation == null
              ? UiMessage('Power {name}', {'name': d[0]})
              : powerAction,
        ),
        child: InkWell(
          key: ValueKey('card-power:$deviceKey'),
          onTap: powerEnabled ? () => onAction('power:$deviceKey') : null,
          borderRadius: BorderRadius.circular(24),
          child: SizedBox.square(
            dimension: 48,
            child: Center(
              child: Container(
                width: large ? 48 : 42,
                height: large ? 48 : 42,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  color: powerColor,
                  border:
                      presentation != null && MediaQuery.highContrastOf(context)
                      ? Border.all(color: Colors.black)
                      : null,
                ),
                child: Center(
                  child: updating
                      ? OperationPulse(color: powerInk, size: large ? 24 : 22)
                      : presentation != null && state != true
                      ? Icon(
                          Icons.power_off_outlined,
                          size: large ? 24 : 22,
                          color: powerInk,
                        )
                      : Glyph('power', size: large ? 24 : 22, color: powerInk),
                ),
              ),
            ),
          ),
        ),
      ),
    );
    if (compact) {
      final showPower = presentation?.showPower ?? (kind != 'sensor');
      final title = data == null ? d[0] : LiteralText(d[0]);
      return Surface(
        radius: 22,
        backgroundColor: (presentation == null ? active : state == true)
            ? Colors.white
            : skin.cardColor,
        border: Colors.transparent,
        child: Semantics(
          label: localize(context, UiMessage('Open {name}', {'name': d[0]})),
          button: true,
          child: InkWell(
            key: ValueKey('card-open:$deviceKey'),
            borderRadius: BorderRadius.circular(22),
            onTap: () => onAction('open:$deviceKey'),
            child: ConstrainedBox(
              constraints: const BoxConstraints(minHeight: 154),
              child: Padding(
                padding: const EdgeInsets.fromLTRB(14, 10, 10, 10),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        SizedBox(
                          width: 56,
                          height: 48,
                          child: CardDeviceIcon(kind),
                        ),
                        const Spacer(),
                        if (showPower)
                          GestureDetector(
                            // A disabled shortcut must not fall through to the
                            // surrounding card's navigation gesture.
                            behavior: HitTestBehavior.opaque,
                            excludeFromSemantics: true,
                            onTap: powerEnabled ? null : () {},
                            child: power(),
                          ),
                      ],
                    ),
                    const SizedBox(height: 8),
                    Copy(
                      title,
                      size: 15,
                      bold: true,
                      maxLines: largeText ? null : 1,
                    ),
                    Row(
                      crossAxisAlignment: CrossAxisAlignment.end,
                      children: [
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Copy(
                                d[1],
                                size: 11,
                                color: secondary,
                                maxLines: largeText ? null : 1,
                              ),
                              Copy(
                                d[2],
                                size: 17,
                                bold: true,
                                maxLines: largeText ? null : 1,
                              ),
                              if (hasCaption)
                                Copy(
                                  cached ? 'Cached · 2h ago' : d[3],
                                  size: 10,
                                  color: secondary,
                                  maxLines: largeText ? null : 3,
                                ),
                            ],
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      );
    }
    Widget textAt(double top, Widget child, {double right = 14}) =>
        Positioned(left: 14, right: right, top: top, child: child);
    return Surface(
      backgroundColor: (presentation == null ? active : state == true)
          ? Colors.white
          : const Color(0xfff0f1ee),
      border: const Color(0xffe1e3de),
      child: Stack(
        children: [
          Semantics(
            label: localize(context, UiMessage('Open {name}', {'name': d[0]})),
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
                          SizedBox(height: 98, child: CardDeviceIcon(kind)),
                          const SizedBox(height: 12),
                          Copy(
                            data == null ? d[0] : LiteralText(d[0]),
                            size: 19,
                            bold: true,
                            color: color,
                          ),
                          const SizedBox(height: 8),
                          Copy(d[1], size: 14, color: secondary),
                          const SizedBox(height: 12),
                          Copy(d[2], size: 26, bold: true, color: color),
                          if (hasCaption) ...[
                            const SizedBox(height: 8),
                            Copy(d[3], size: 14, color: secondary),
                          ],
                          const SizedBox(height: 48),
                        ],
                      ),
                    )
                  : SizedBox(
                      height: large ? 197 : compactHeight + 8,
                      child: Stack(
                        children: [
                          Positioned(
                            left: large ? 18 : 15,
                            top: large ? 22 : 12,
                            width: large ? 100 : null,
                            right: large ? null : 15,
                            height: large ? 95 : 98,
                            child: CardDeviceIcon(kind),
                          ),
                          if (large) ...[
                            Positioned(
                              left: 138,
                              right: 14,
                              top: 23,
                              child: Copy(
                                data == null ? d[0] : LiteralText(d[0]),
                                maxLines: 2,
                                size: 19,
                                bold: true,
                                color: color,
                              ),
                            ),
                            Positioned(
                              left: 138,
                              right: 14,
                              top: 77,
                              child: Copy(
                                d[1],
                                size: 14,
                                color: secondary,
                                maxLines: 1,
                              ),
                            ),
                            Positioned(
                              left: 138,
                              right: 14,
                              top: 116,
                              child: Copy(
                                d[2],
                                maxLines: 1,
                                size: 26,
                                bold: true,
                                color: color,
                              ),
                            ),
                            if (hasCaption)
                              Positioned(
                                left: 138,
                                right: 14,
                                top: 154,
                                child: Copy(
                                  d[3],
                                  size: 14,
                                  color: secondary,
                                  maxLines: 2,
                                ),
                              ),
                          ] else ...[
                            textAt(
                              117,
                              Copy(
                                data == null ? d[0] : LiteralText(d[0]),
                                size: 15,
                                bold: true,
                                color: color,
                                maxLines: 1,
                              ),
                            ),
                            textAt(
                              143,
                              Copy(
                                d[1],
                                size: 11,
                                color: secondary,
                                maxLines: 1,
                              ),
                            ),
                            textAt(
                              compactHeight - 69,
                              Copy(
                                d[2],
                                size: 24,
                                bold: true,
                                color: color,
                                maxLines: 1,
                              ),
                              right:
                                  (presentation?.showPower ??
                                      (kind != 'sensor'))
                                  ? 50
                                  : 14,
                            ),
                            if (hasCaption)
                              textAt(
                                compactHeight - 36,
                                Copy(
                                  cached ? 'Cached · 2h ago' : d[3],
                                  maxLines: 3,
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
          if (presentation?.showPower ?? (kind != 'sensor'))
            Positioned(
              right: large ? 14 : 8,
              bottom: large ? 15 : 19,
              child: power(),
            ),
          Positioned(
            right: 0,
            top: 0,
            child: Semantics(
              label: localize(
                context,
                UiMessage('Menu {name}', {'name': d[0]}),
              ),
              button: true,
              child: Tooltip(
                message: localize(
                  context,
                  UiMessage('Menu {name}', {'name': d[0]}),
                ),
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
