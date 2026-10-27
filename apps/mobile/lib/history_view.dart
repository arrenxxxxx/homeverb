import 'dart:async';
import 'package:flutter/material.dart';
import 'client_contract.dart';
import 'controller.dart';
import 'controls.dart';
import 'device_detail_controls.dart' show componentLabel;
import 'localization.dart';
import 'numeric_history.dart';
import 'skin.dart';

bool supportsNumericHistory(CoreClient core) =>
    core is NativeSessionClient &&
    core.apiAtLeast(33) &&
    core.methods.containsAll({'inspect_history_options', 'read_history'});

/// One foreground dialog owns ephemeral history. Opening only inspects local options.
class NumericHistoryDialog extends StatefulWidget {
  const NumericHistoryDialog({required this.controller, super.key});
  final HomeController controller;
  @override
  State<NumericHistoryDialog> createState() => _NumericHistoryDialogState();
}

class _NumericHistoryDialogState extends State<NumericHistoryDialog>
    with WidgetsBindingObserver {
  late final HomeController c = widget.controller;
  late final CoreClient core = c.core;
  late final int epoch = c.controlEpoch;
  late final Json descriptor = c.description!;
  late final String device = c.selectedDevice!;
  late final List<Json> connections =
      (descriptor['device']['connections'] as List).cast<Json>();
  final start = TextEditingController(), end = TextEditingController();
  String? connection;
  List<NumericHistoryOption> options = [];
  NumericHistoryOption? selected;
  NumericHistoryResult? result;
  Object? error;
  bool loading = false, finished = false;
  bool get current =>
      mounted &&
      !finished &&
      c.isForeground &&
      c.controlEpoch == epoch &&
      identical(c.core, core) &&
      c.selectedDevice == device &&
      identical(c.description, descriptor);

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    c.accountLoginCancellation.addListener(cancel);
    c.addListener(checkCurrent);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (current && connections.isNotEmpty) inspect(connections.first['id']);
    });
  }

  void checkCurrent() {
    if (!current) cancel();
  }

  void cancel() {
    if (finished) return;
    finished = true;
    result = null;
    options = [];
    selected = null;
    if (loading) core.cancelAll();
    loading = false;
    start.clear();
    end.clear();
    scheduleMicrotask(() {
      if (!mounted) return;
      final route = ModalRoute.of(context);
      if (route == null || !route.isActive) return;
      if (route.isCurrent) {
        Navigator.of(context).pop();
      } else {
        Navigator.of(context).removeRoute(route);
      }
    });
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) cancel();
  }

  Future<void> run(Future<void> Function() work) async {
    if (!current || loading || c.busy) return;
    setState(() {
      loading = true;
      error = null;
      result = null;
    });
    final ok = await c.perform(() async {
      if (!current) return;
      try {
        await work();
      } catch (e) {
        if (current) error = e;
      }
    });
    if (!current) return;
    setState(() {
      loading = false;
      if (!ok) error = c.error;
    });
  }

  Future<void> inspect(String id) => run(() async {
    connection = id;
    options = [];
    selected = null;
    final raw = await core.call('inspect_history_options', {
      'device_id': device,
      'connection_id': id,
    });
    if (!current) return;
    options = NumericHistoryOption.parse(raw, descriptor, id);
    if (options.isNotEmpty) choose(options.first);
  });

  void choose(NumericHistoryOption option) {
    selected = option;
    result = null;
    error = null;
    final now = DateTime.now().toUtc();
    end.text = historyUtc(now);
    start.text = historyUtc(
      now.subtract(
        Duration(
          seconds: option.maxWindowSeconds < 604800
              ? option.maxWindowSeconds
              : 604800,
        ),
      ),
    );
  }

  Future<void> read() => run(() async {
    final started = DateTime.now().toUtc();
    final query = selected!.query(start.text, end.text, started);
    final raw = await core.call('read_history', {'history': query.json});
    if (!current) return;
    result = NumericHistoryResult.parse(
      raw,
      query,
      started,
      DateTime.now().toUtc(),
    );
  });

  @override
  void dispose() {
    c.accountLoginCancellation.removeListener(cancel);
    c.removeListener(checkCurrent);
    WidgetsBinding.instance.removeObserver(this);
    finished = true;
    if (loading) core.cancelAll();
    result = null;
    options = [];
    selected = null;
    start.dispose();
    end.dispose();
    super.dispose();
  }

  Object optionLabel(NumericHistoryOption option) {
    final part = c.components.firstWhere(
      (p) => p['id'] == option.target['component_id'],
    );
    return UiMessage('{component}: {property} ({unit})', {
      'component': UiMessage(componentLabel(part, siblings: c.components)),
      'property': UiMessage(
        sourceForKey(
              '${option.target['capability']}.${option.target['property']}',
            ) ??
            controlLabel(option.target['property']!),
      ),
      'unit': option.unit,
    });
  }

  @override
  Widget build(BuildContext context) => PopScope<void>(
    onPopInvokedWithResult: (didPop, _) {
      if (didPop) cancel();
    },
    child: AlertDialog(
      title: const Copy('Historical readings', control: true),
      content: SizedBox(
        width: 520,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Copy(
                'Choose a series and a UTC time range, then read once. The end time is excluded.',
                control: true,
              ),
              if (connections.length > 1)
                Wrap(
                  spacing: 8,
                  children: [
                    for (var i = 0; i < connections.length; i++)
                      ChoiceChip(
                        label: Copy(
                          UiMessage('Connection {number}: {transport}', {
                            'number': i + 1,
                            'transport': UiMessage(
                              controlLabel(connections[i]['transport']),
                            ),
                          }),
                          control: true,
                        ),
                        selected: connection == connections[i]['id'],
                        onSelected: loading
                            ? null
                            : (_) => inspect(connections[i]['id']),
                      ),
                  ],
                ),
              if (loading) const LinearProgressIndicator(),
              for (final option in options)
                RadioListTile<NumericHistoryOption>(
                  title: Copy(optionLabel(option), control: true),
                  value: option,
                  // Kept compatible with the pinned Flutter SDK.
                  // ignore: deprecated_member_use
                  groupValue: selected,
                  // ignore: deprecated_member_use
                  onChanged: loading
                      ? null
                      : (value) => setState(() => choose(value!)),
                ),
              if (!loading && options.isEmpty && error == null)
                const Copy(
                  'No historical series is available on this connection.',
                  control: true,
                ),
              if (selected != null) ...[
                Copy(
                  UiMessage(
                    'Maximum range: {seconds} seconds. Maximum records: {count}.',
                    {
                      'seconds': selected!.maxWindowSeconds,
                      'count': selected!.maxRecords,
                    },
                  ),
                  control: true,
                ),
                const Copy('UTC format: YYYY-MM-DDTHH:MM:SSZ', control: true),
                TextField(
                  key: const ValueKey('history-start'),
                  controller: start,
                  enabled: !loading,
                  autocorrect: false,
                  decoration: InputDecoration(
                    labelText: localize(context, 'Start time (UTC)'),
                  ),
                  onChanged: (_) => setState(() {
                    result = null;
                    error = null;
                  }),
                ),
                TextField(
                  key: const ValueKey('history-end'),
                  controller: end,
                  enabled: !loading,
                  autocorrect: false,
                  decoration: InputDecoration(
                    labelText: localize(context, 'End time (UTC, excluded)'),
                  ),
                  onChanged: (_) => setState(() {
                    result = null;
                    error = null;
                  }),
                ),
                TextButton(
                  key: const ValueKey('history-read'),
                  onPressed: loading ? null : read,
                  child: const Copy('Read history', control: true),
                ),
              ],
              if (error != null)
                Copy(
                  LiteralText(localizedError(context, error!)),
                  control: true,
                ),
              if (result case final value?) ...[
                const Divider(),
                Copy(
                  UiMessage('Source: {source}', {
                    'source': UiMessage(
                      controlLabel(value.query.option.source),
                    ),
                  }),
                  control: true,
                ),
                const Copy(
                  'Completeness is unproven. Missing records are not zero. The provider calendar-day timezone is unknown.',
                  control: true,
                ),
                Copy(
                  UiMessage('Received at {time}', {
                    'time': value.receivedAt.toUtc().toIso8601String(),
                  }),
                  control: true,
                ),
                Copy(
                  UiMessage('Rejected records: {count}', {
                    'count': value.rejected,
                  }),
                  control: true,
                ),
                if (value.limitReached)
                  const Copy(
                    'The record limit was reached. More records may exist.',
                    control: true,
                  ),
                if (value.records.isEmpty)
                  const Copy(
                    'No valid records were returned for this range.',
                    control: true,
                  ),
                for (final row in value.records)
                  Padding(
                    padding: const EdgeInsets.symmetric(vertical: 8),
                    child: Copy(
                      LiteralText(
                        '${historyUtc(row.time)}\n${row.value} ${value.query.option.unit}',
                      ),
                      control: true,
                    ),
                  ),
              ],
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: cancel,
          child: const Copy('Close', control: true),
        ),
      ],
    ),
  );
}
