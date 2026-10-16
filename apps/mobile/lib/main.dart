import 'dart:convert';
import 'dart:io';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'localization.dart';
import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:path_provider/path_provider.dart';
import 'package:file_selector/file_selector.dart';
import 'blocks.dart';
import 'appliance_controls.dart';
import 'climate_controls.dart';
import 'account_views.dart';
import 'approval_review.dart';
import 'compartment_controls.dart';
import 'controller.dart';
import 'diagnostics_export.dart';
import 'diagnostics_view.dart';
import 'controls.dart';
import 'native_client.dart';
import 'preferences.dart';
import 'session_permissions.dart';
import 'receipts.dart';
import 'screens.dart' hide Json;
import 'skin.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await applyLocalePreference(null);
  LicenseRegistry.addLicense(() async* {
    yield LicenseEntryWithLineBreaks([
      'Inter',
    ], await rootBundle.loadString('assets/fonts/LICENSE.txt'));
  });
  try {
    final catalog = await ScreenCatalog.load();
    final directory = await getApplicationSupportDirectory();
    final prefs = LocalPreferences(File('${directory.path}/presentation.json'));
    await prefs.load();
    await applyLocalePreference(prefs.locale);
    const library = String.fromEnvironment('HOMEVERB_LIBRARY');
    const adapter = String.fromEnvironment(
      'HOMEVERB_ADAPTER',
      defaultValue: 'homeverb.mock',
    );
    Future<CoreClient> openSession(
      List<SessionPermission> permissions,
    ) => NativeClient.open(
      dataDir:
          '${directory.path}/${adapter == 'homeverb.mock' ? 'native' : 'native-meiju'}',
      adapter: adapter,
      libraryPath: library.isEmpty ? null : library,
      allowMockWrites: adapter == 'homeverb.mock',
      permissions: permissions,
    );
    runApp(
      TranslationProvider(
        child: NativeApp(
          catalog: catalog,
          controller: HomeController(
            await openSession(const []),
            prefs,
            reopen: adapter == 'midea.meiju' ? openSession : null,
          ),
        ),
      ),
    );
  } catch (e) {
    runApp(TranslationProvider(child: StartupFailure(error: e)));
  }
}

class StartupFailure extends StatelessWidget {
  const StartupFailure({required this.error, super.key});
  final Object error;
  @override
  Widget build(BuildContext context) => MaterialApp(
    locale: translationsOf(context).$meta.locale.flutterLocale,
    supportedLocales: AppLocaleUtils.supportedLocales,
    localizationsDelegates: GlobalMaterialLocalizations.delegates,
    theme: windowTheme(),
    highContrastTheme: windowTheme(highContrast: true),
    home: Scaffold(
      body: LightBackground(
        child: SafeArea(
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const Copy('HomeVerb', size: 30, bold: true),
                const SizedBox(height: 24),
                const Copy('Native Core unavailable', size: 23, bold: true),
                const SizedBox(height: 16),
                Copy(localizedError(context, error)),
                const SizedBox(height: 20),
                const Copy(
                  'Install the matching native library and restart. No device or account operation was started.',
                ),
              ],
            ),
          ),
        ),
      ),
    ),
  );
}

class NativeApp extends StatefulWidget {
  const NativeApp({required this.catalog, required this.controller, super.key});
  final ScreenCatalog catalog;
  final HomeController controller;
  @override
  State<NativeApp> createState() => NativeAppState();
}

class NativeAppState extends State<NativeApp> with WidgetsBindingObserver {
  HomeController get c => widget.controller;
  String id = 'D01';
  final List<String> history = [];
  final List<Json> receiptParents = [];
  final Map<String, TextEditingController> fields = {};
  final Map<String, int> choices = {};
  final Map<String, double> values = {};
  final navigator = GlobalKey<NavigatorState>();
  String? selectedMode;
  Json? actionCapability;
  String? actionName;
  String? formError;
  LocalCleanup? cleanupScope;
  bool editingAccountName = false;
  bool exporting = false;
  String connectionLabel = 'Meiju account';
  bool _disposed = false;
  int navigation = 0;
  int activityEpoch = 0;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    c.addListener(changed);
    id = c.preferences.root;
    c.initialize();
  }

  void changed() {
    if (!_disposed && mounted) setState(() {});
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) activityEpoch++;
    c.setForeground(state == AppLifecycleState.resumed);
  }

  @override
  void dispose() {
    _disposed = true;
    WidgetsBinding.instance.removeObserver(this);
    c.removeListener(changed);
    c.cancelPending();
    c.core.close();
    for (final field in fields.values) {
      field.dispose();
    }
    super.dispose();
  }

  void clearFields() {
    for (final f in fields.values) {
      f.dispose();
    }
    fields.clear();
    choices.clear();
    values.clear();
    formError = null;
  }

  Future<void> go(String target, {bool back = false}) async {
    if (!mounted) return;
    final navContext = navigator.currentContext;
    if (navContext != null) {
      ScaffoldMessenger.of(navContext).removeCurrentSnackBar();
    }
    final deviceHistory = ['C01', 'C08', 'D06'].contains(id);
    c.cancelPending();
    setState(() {
      navigation++;
      receiptParents.clear();
      if (!back) history.add(id);
      id = target;
      clearFields();
      if (target != 'D03') c.search = '';
    });
    if (['D01', 'P01', 'S01'].contains(target))
      await c.savePreferences({'root': target});
    if (target == 'P01' && c.supports('list_accounts')) await c.loadAccounts();
    if (target == 'S05') await c.loadHistory(deviceOnly: deviceHistory);
    if (target == 'S07') await c.loadDiagnostics();
    if (target == 'S04' && c.cleanupNeedsReload) await c.initialize();
  }

  Future<void> finish(Future<bool> work, String destination) async {
    final epoch = navigation;
    if (await work && mounted && epoch == navigation) await go(destination);
  }

  Future<bool> executeOnDevice(
    String capability,
    String actionName,
    Json arguments,
  ) async {
    final epoch = navigation;
    final device = c.selectedDevice;
    final component = c.component;
    final foreground = activityEpoch;
    final success = await c.execute(capability, actionName, arguments);
    if (success &&
        mounted &&
        epoch == navigation &&
        c.isForeground &&
        foreground == activityEpoch &&
        device == c.selectedDevice &&
        component == c.component) {
      // One foreground read updates observations; it never changes the receipt.
      await c.refresh();
      if (mounted &&
          epoch == navigation &&
          id == 'D01' &&
          c.isForeground &&
          foreground == activityEpoch) {
        final context = navigator.currentContext;
        if (context != null)
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text(
                localize(context, outcomeSummary(c.receipt?['status'])),
              ),
              action: SnackBarAction(
                label: localize(context, 'View Details'),
                onPressed: () => action('View Details'),
              ),
            ),
          );
      }
    }
    return success &&
        mounted &&
        epoch == navigation &&
        foreground == activityEpoch &&
        c.isForeground &&
        device == c.selectedDevice &&
        component == c.component;
  }

  Future<void> executeClimateEdit(Json arguments) async {
    if (c.capability('power') == null) {
      await finish(c.execute('climate', 'set', arguments), 'S06');
      return;
    }
    final epoch = navigation;
    if (await executeOnDevice('climate', 'set', arguments) &&
        mounted &&
        epoch == navigation &&
        c.isForeground &&
        id != 'C01') {
      await go('C01');
    }
  }

  String input(String label) => fields[label]?.text ?? '';
  int choice() => choices.values.firstOrNull ?? 0;
  String get selectedRoute => ['auto', 'lan', 'cloud'][choice().clamp(0, 2)];
  Future<void> action(String label) async {
    if (label == 'Back' ||
        label == 'Cancel' ||
        label == 'Close' ||
        label == 'Not Now') {
      if (id == 'S06' && receiptParents.isNotEmpty) {
        c.receipt = receiptParents.removeLast();
        changed();
        return;
      }
      await go(history.isEmpty ? 'D01' : history.removeLast(), back: true);
      return;
    }
    if (label.startsWith('account:')) {
      await finish(c.selectAccount(label.substring('account:'.length)), 'P09');
      return;
    }
    if ([
      'Choose Meiju',
      'Reauthorize',
      'Sign In to the Original Account',
    ].contains(label)) {
      if (c.busy) return;
      editingAccountName = false;
      if (label == 'Choose Meiju') {
        c.selectedAccount = null;
        c.accountDetails = null;
        c.accountReport = null;
        c.accountRemoval = null;
      }
      await go(c.canSignIn ? 'P03' : 'P13');
      return;
    }
    if (label == 'Continue to Sign In' || label == 'Save Account Name') {
      if (c.busy) return;
      final name = input('Account name').trim();
      if (name.isEmpty ||
          name.runes.length > 80 ||
          RegExp(r'[\x00-\x1f\x7f]').hasMatch(name)) {
        setState(
          () => formError = 'Use 1–80 characters without control characters.',
        );
        return;
      }
      if (label == 'Save Account Name') {
        await finish(c.renameAccount(name), 'P09');
      } else {
        connectionLabel = name;
        await go('P04');
      }
      return;
    }
    if (label == 'Rename Account') {
      if (c.busy || c.account?['credential_state'] == 'removing') return;
      editingAccountName = true;
      await go('P03');
      return;
    }
    if (label == 'Reload Accounts') {
      await finish(c.loadAccounts(), 'P01');
      return;
    }
    if (label.startsWith('conflicts:')) {
      await c.inspectDeviceConflicts(label.substring('conflicts:'.length));
      return;
    }
    if (label == 'Inspect Saved Inventory') {
      await c.inspectAccountInventory();
      return;
    }
    if (label == 'Retrieve Devices' || label == 'Recover Saved Authorization') {
      await finish(
        c.retrieveAccount(recover: label == 'Recover Saved Authorization'),
        'P08',
      );
      return;
    }
    if ([
      'Remove From This Device',
      'Retry local cleanup',
      'Review removal scope',
    ].contains(label)) {
      await finish(c.reviewAccountRemoval(), 'P11');
      return;
    }
    if (label == 'Remove Locally') {
      final epoch = navigation;
      if (await c.removeAccount() && mounted && navigation == epoch) {
        await go(c.accountRemoval?['status'] == 'removed' ? 'P01' : 'P09');
      }
      return;
    }
    if (label == 'Back to Accounts') {
      await go('P01');
      return;
    }
    if (label == 'View Devices') {
      await go('D01');
      return;
    }
    if (label == 'Revoke Access · if supported') {
      await go('P12');
      return;
    }
    if (label.startsWith('receipt:')) {
      await finish(c.loadReceipt(label.substring('receipt:'.length)), 'S06');
      return;
    }
    if (label.startsWith('receipt-item:') || label.startsWith('batch-item:')) {
      final nested = label.startsWith('receipt-item:');
      final source = nested ? c.receipt : c.batchResult;
      final index = int.tryParse(label.substring(label.indexOf(':') + 1));
      final items = source?['items'] as List? ?? [];
      if (index == null || index < 0 || index >= items.length || c.busy) return;
      if (nested) receiptParents.add(c.receipt!);
      c.receipt = items[index] as Json;
      if (nested) {
        changed();
      } else {
        await go('S06');
      }
      return;
    }
    if (label == 'Dismiss error') {
      c.error = null;
      changed();
      return;
    }
    if (label == 'Reload local data') {
      await c.initialize();
      return;
    }
    if (label.startsWith('control:')) {
      if (c.busy) return;
      final parts = label.split(':');
      final index = int.tryParse(parts[1]);
      if (index == null || index < 0 || index >= c.capabilities.length) return;
      actionCapability = c.capabilities[index];
      actionName = Uri.decodeComponent(parts[2]);
      await go('C10');
      return;
    }
    if (label.startsWith('open-component:') ||
        label.startsWith('edit-component:')) {
      final componentId = Uri.decodeComponent(
        label.substring(label.indexOf(':') + 1),
      );
      final epoch = navigation;
      if (await c.selectComponent(componentId) &&
          mounted &&
          epoch == navigation) {
        final edit = label.startsWith('edit-component:');
        final setter = c.capability('climate')?['actions']?['set'] as Json?;
        if (edit && setter != null && !hasDirectTarget(setter)) {
          actionCapability = c.capability('climate');
          actionName = 'set';
          await go('C10');
        } else {
          await go(edit ? 'C14' : 'C01');
        }
      }
      return;
    }
    if (label.startsWith('component:')) {
      if (c.busy) return;
      clearFields();
      await c.selectComponent(label.substring('component:'.length));
      return;
    }
    if (label.startsWith('room:')) {
      final name = label.substring(5);
      c.filter = name == 'All' || name == 'Unassigned'
          ? name
          : c.rooms.firstWhere((r) => r['name'] == name)['id'];
      changed();
      return;
    }
    if (label.startsWith('open:') ||
        label.startsWith('power:') ||
        label.startsWith('menu:')) {
      final epoch = navigation,
          device = label.substring(label.indexOf(':') + 1);
      if (!await c.selectDevice(device)) return;
      if (!mounted || epoch != navigation) return;
      if (label.startsWith('power:')) {
        final state = c.observed('power.state');
        if (state is! bool) {
          c.error = NativeFailure(
            'STATE_UNAVAILABLE',
            'Refresh state before using a quick power action.',
          );
          changed();
          return;
        }
        await executeOnDevice('power', 'set', {'state': !state});
      } else {
        await go(
          label.startsWith('menu:')
              ? 'D05'
              : c.hasCompartmentOverview
              ? 'C08'
              : 'C01',
        );
      }
      return;
    }
    if (label == 'Refresh State') {
      if (['D01', 'D11'].contains(id))
        await c.refreshCards();
      else if (id == 'S06')
        await c.refreshReceipt();
      else if (id == 'C08')
        await c.refreshComponents();
      else
        await c.refresh();
      return;
    }
    if (label == 'Discover Mock device' ||
        label == 'Retrieve Saved Accounts' ||
        label == 'Search Again') {
      await finish(c.discover(), 'D01');
      return;
    }
    if (label == 'Save Room') {
      formError = null;
      final name = input('Room name').trim();
      if (name.isEmpty ||
          name.runes.length > 80 ||
          RegExp(r'[\x00-\x1f\x7f]').hasMatch(name)) {
        setState(
          () => formError = 'Use 1–80 characters without control characters.',
        );
        return;
      }
      await finish(c.saveRoom(name, id: c.editingRoom), 'R01');
      return;
    }
    if (label == 'Delete Room' && c.editingRoom != null) {
      await finish(c.deleteRoom(c.editingRoom!), 'R01');
      return;
    }
    if (label == 'Save Name') {
      formError = null;
      if (id == 'S10') {
        final name = input('Sharing name').trim();
        if (name.isEmpty || name.length > 80) {
          setState(() => formError = 'Use 1–80 characters.');
          return;
        }
        await finish(c.savePreferences({'sharing_name': name}), 'S01');
      } else {
        await finish(
          c.saveDevice({
            'alias': input('Local name').trim().isEmpty
                ? null
                : input('Local name').trim(),
          }),
          'D06',
        );
      }
      return;
    }
    if (label == 'Pin to Top' || label == 'Unpin') {
      await finish(
        c.saveDevice({
          'favorite': !(c.currentCard?['preferences']['favorite'] ?? false),
        }),
        'D01',
      );
      return;
    }
    if (label == 'Move Devices') {
      if (c.selection.isEmpty) {
        c.error = NativeFailure(
          'INVALID_ARGUMENT',
          'Select at least one device to move.',
        );
        changed();
        return;
      }
      final index = choice();
      await finish(
        c.assign(index < c.rooms.length ? c.rooms[index]['id'] : null),
        'D01',
      );
      return;
    }
    if (label == 'Select Devices') {
      c.selection.clear();
      c.batchResult = null;
      await go('C12');
      return;
    }
    if (label == 'Apply to Selected Devices') {
      await c.executeSelectedPower(choice() == 0);
      return;
    }
    if (label == 'Save Preference') {
      final changes = id == 'D08'
          ? {
              'device_routes': {
                ...(c.preferences.values['device_routes'] as Map? ?? {}),
                c.selectedDevice!: selectedRoute,
              },
            }
          : {'route': selectedRoute};
      await finish(c.savePreferences(changes), id == 'D08' ? 'D06' : 'S01');
      return;
    }
    if (label == 'Power') {
      final state = c.observed('power.state');
      if (state is bool)
        await executeOnDevice('power', 'set', {'state': !state});
      else {
        c.error = NativeFailure(
          'STATE_UNAVAILABLE',
          'Read state before changing power.',
        );
        changed();
      }
      return;
    }
    if (label == 'Increase target' || label == 'Decrease target') {
      formError = null;
      final schema =
          c.capability(
                'climate',
              )?['actions']?['set']?['parameters']?['properties']?['target_temperature']
              as Json?;
      final current = c.observed('climate.target_temperature');
      if (schema == null || current is! num) {
        c.error = NativeFailure(
          'STATE_UNAVAILABLE',
          'Read a supported target first.',
        );
        changed();
        return;
      }
      final value =
          current +
          (schema['multipleOf'] ?? 1) * (label == 'Increase target' ? 1 : -1);
      final error = validateValue(value, schema);
      if (error != null) {
        setState(() => formError = error);
        return;
      }
      await executeClimateEdit({'target_temperature': value});
      return;
    }
    if (label == 'Save Mode') {
      final modes =
          c.capability(
                'climate',
              )?['actions']?['set']?['parameters']?['properties']?['mode']?['enum']
              as List? ??
          [];
      if (modes.isNotEmpty)
        await finish(
          c.execute('climate', 'set', {
            'mode': modes[choice().clamp(0, modes.length - 1)],
          }),
          'S06',
        );
      return;
    }
    if (label == 'Save target') {
      formError = null;
      final value = num.tryParse(input('Target temperature'));
      final schema =
          c.capability(
                'climate',
              )?['actions']?['set']?['parameters']?['properties']?['target_temperature']
              as Json?;
      final error = schema == null
          ? 'Control unavailable.'
          : validateValue(value, schema);
      if (error != null) {
        setState(() => formError = error);
        return;
      }
      await executeClimateEdit({'target_temperature': value});
      return;
    }
    if (label == 'Clear Cached State' || label == 'Clear Recent Operations') {
      cleanupScope = label == 'Clear Cached State'
          ? LocalCleanup.cachedState
          : LocalCleanup.executionHistory;
      await go('S09');
      return;
    }
    if (label == 'Confirm Clear' && cleanupScope != null) {
      receiptParents.clear();
      await finish(c.clearLocalData(cleanupScope!), 'S04');
      return;
    }
    if (label == 'Remove Local Data') cleanupScope = null;
    if (label == 'Review Full Report') {
      if (c.diagnostics == null || c.busy) return;
      final exact = const JsonEncoder.withIndent('  ').convert(c.diagnostics);
      await showDialog<void>(
        context: navigator.currentContext!,
        builder: (context) => AlertDialog(
          title: Text(localize(context, 'Diagnostic report')),
          backgroundColor: Theme.of(context).colorScheme.surface,
          surfaceTintColor: Colors.transparent,
          scrollable: true,
          content: SelectableText(exact),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: Text(localize(context, 'Close')),
            ),
          ],
        ),
      );
      return;
    }
    if (label == 'Export Redacted Report') {
      await export();
      return;
    }
    if (label.startsWith('choice:')) return;
    if (label == 'Privacy' || label == 'Licenses') {
      if (label == 'Licenses') {
        showLicensePage(
          context: navigator.currentContext!,
          applicationName: 'HomeVerb',
          applicationVersion: '0.2.0',
        );
      } else {
        await showDialog<void>(
          context: navigator.currentContext!,
          builder: (context) => AlertDialog(
            title: Text(localize(context, 'Privacy')),
            content: Text(
              localize(
                context,
                'Explicit account connection and device retrieval can contact the admitted provider directly. No telemetry or diagnostic uploads. Credentials use the system secure store. Diagnostics are exported only when requested. Nearby sharing is unavailable.',
              ),
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context),
                child: Text(localize(context, 'Close')),
              ),
            ],
          ),
        );
      }
      return;
    }
    if (label == 'Add Room') {
      c.editingRoom = null;
      await go('R02');
      return;
    }
    const routes = {
      'Devices': 'D01',
      'Platforms': 'P01',
      'Settings': 'S01',
      'Language': 'L01',
      'Search': 'D03',
      'Add device': 'A01',
      'Add Device': 'A01',
      'Manage rooms': 'R01',
      'Rooms': 'R01',
      'Appearance': 'S02',
      'This Device': 'S10',
      'Connections': 'S03',
      'Recent Operations': 'S05',
      'Local Data': 'S04',
      'Diagnostics': 'S07',
      'About': 'S08',
      'Rename': 'D07',
      'Name': 'D07',
      'Move to Room': 'R03',
      'Room': 'R03',
      'Move Existing Devices': 'R03',
      'Reorder Devices': 'D10',
      'Device Information': 'D06',
      'Device information': 'D06',
      'Connection': 'D08',
      'Mode': 'C02',
      'Target temperature': 'C14',
      'View Details': 'S06',
      'Preview Skin': 'S11',
      'Back to Appearance': 'S02',
      'Compatibility': 'S12',
      'Compatibility Details': 'S12',
      'Add Platform': 'P02',
      'Add Another Platform': 'P02',
      'Receive Authorization': 'H05',
      'Send Authorization': 'H01',
      'Back to Platforms': 'P02',
      'Text size and accessibility': 'D12',
      'Inspect cached state': 'D11',
      'Remove Local Data': 'S09',
      'View Operations': 'S05',
      'Done': 'D01',
    };
    if (routes[label] != null) {
      if (routes[label] == 'R03') {
        c.selectedRoom = c.currentCard?['preferences']?['room_id'];
        c.selection.clear();
        if (['D05', 'D06'].contains(id) && c.selectedDevice != null) {
          c.selection.add(c.selectedDevice!);
        }
      }
      await go(routes[label]!);
      return;
    }
    if (label == 'Balcony' || label == 'Nursery') {
      fields['Room name']?.text = label;
      changed();
      return;
    }
    c.error = NativeFailure(
      'UNAVAILABLE',
      'This operation is unavailable in the current native session.',
    );
    changed();
  }

  Future<void> export() async {
    if (c.diagnostics == null || c.busy || exporting) return;
    exporting = true;
    c.error = null;
    changed();
    final exact = const JsonEncoder.withIndent('  ').convert(c.diagnostics);
    try {
      if (Platform.isAndroid) {
        await exportAndroidDiagnostics(exact);
        return;
      }
      final location = await getSaveLocation(
        suggestedName: 'homeverb-diagnostics.json',
        acceptedTypeGroups: [
          const XTypeGroup(label: 'JSON', extensions: ['json']),
        ],
      );
      if (location == null) return;
      await XFile.fromData(
        utf8.encode(exact),
        mimeType: 'application/json',
        name: 'homeverb-diagnostics.json',
      ).saveTo(location.path);
    } catch (_) {
      c.error = NativeFailure(
        'EXPORT_FAILED',
        'Could not save the previewed report.',
      );
      changed();
    } finally {
      exporting = false;
      changed();
    }
  }

  Json get screen => screenFor(id);
  Json screenFor(String id) {
    final original = widget.catalog.ui(id == 'L01' ? 'S01' : id);
    final ui = Map<String, dynamic>.from(original);
    List<dynamic>? b;
    if (id.startsWith('P')) {
      b = accountBlocks(c, id);
      if (id == 'P03') ui['header'] = 'Connect Meiju';
      if (id == 'P13') ui['header'] = 'Platform connection';
      if (id == 'P03' && editingAccountName) {
        ui['header'] = 'Account name';
        b = [
          ['field', 'Account name', c.account?['label'] ?? ''],
          [
            'note',
            'This changes only the local display name. Identity, credentials and permissions stay unchanged.',
          ],
          ['button', 'Save Account Name'],
          ['button', 'Cancel', 'secondary'],
        ];
      }
      if (id == 'P04') ui['header'] = 'Meiju sign-in';
      if (id == 'P09' || id == 'P10') ui['header'] = 'Account details';
      if (id == 'P08') ui['header'] = 'Account result';
    }
    if (id.startsWith('H')) {
      b = [
        [
          'status',
          'Unavailable',
          'Nearby sharing is unavailable in the current native contract.',
          'warning',
        ],
        ['note', 'No discovery, transfer or authorization has started.'],
        ['button', 'Compatibility Details', 'secondary'],
      ];
    }
    final card = c.currentCard,
        device = card?['device'] as Json?,
        prefs = card?['preferences'] as Json?;
    switch (id) {
      case 'L01':
        ui.remove('root');
        ui['header'] = 'Language';
        b = [];
        break;
      case 'S01':
        b = [
          [
            'rows',
            [
              [
                'Language',
                c.preferences.locale == null
                    ? 'System default'
                    : c.preferences.locale == 'zh-CN'
                    ? 'Simplified Chinese'
                    : 'English (United States)',
                'Language',
              ],
            ],
          ],
          ...original['blocks'] as List,
        ];
        break;
      case 'D01':
      case 'D11':
      case 'D12':
        if (c.cards.isEmpty) {
          b = c.busy
              ? [
                  ['note', 'Loading local devices…'],
                ]
              : c.error != null
              ? [
                  ['note', 'The local device list could not be loaded.'],
                  ['button', 'Reload local data', 'secondary'],
                ]
              : widget.catalog.ui('D04')['blocks'] as List;
        }
        break;
      case 'D03':
        b = [
          ['field', 'Search devices', c.search],
          ['note', 'Searches local device names, models and rooms.'],
        ];
        break;
      case 'D05':
        ui['header'] = device?['alias'] == null
            ? 'Device'
            : LiteralText(device!['alias']);
        b = [
          [
            'rows',
            [
              ['Rename', '›'],
              ['Move to Room', '›'],
              [prefs?['favorite'] == true ? 'Unpin' : 'Pin to Top', '›'],
              ['Reorder Devices', '›'],
              ['Select Devices', '›'],
              ['Device Information', '›'],
            ],
          ],
          ['button', 'Close', 'secondary'],
        ];
        break;
      case 'D06':
        b = [
          [
            'hero',
            'ac',
            device?['alias'] == null ? 'Device' : LiteralText(device!['alias']),
            roomPresentation(c, prefs?['room_id']),
          ],
          [
            'rows',
            [
              ['Name', LiteralText(device?['alias'] ?? '—')],
              ['Room', roomPresentation(c, prefs?['room_id'])],
              ['Model', LiteralText(device?['model'] ?? '—')],
              ['Connection', c.preferences.route(c.selectedDevice)],
            ],
          ],
          [
            'note',
            UiMessage('Supported: {capabilities}', {
              'capabilities': c.capabilities.map((v) => v['id']).join(' · '),
            }),
          ],
          ['button', 'Recent Operations', 'secondary'],
        ];
        break;
      case 'D07':
        b = [
          ['field', 'Local name', prefs?['alias'] ?? ''],
          ['note', 'A blank name restores the adapter name.'],
          ['button', 'Save Name'],
          ['button', 'Cancel', 'secondary'],
        ];
        break;
      case 'D08':
      case 'S03':
        final connections = (device?['connections'] as List? ?? [])
            .map((v) => v['transport'])
            .toSet();
        b = [
          [
            'choices',
            [
              'Auto — prefer local when available',
              'Local network only — no cloud fallback',
              'Platform cloud — requires internet',
            ],
            ['auto', 'lan', 'cloud'].indexOf(
              c.preferences.route(id == 'D08' ? c.selectedDevice : null),
            ),
          ],
          [
            'note',
            id == 'D08'
                ? UiMessage(
                    'Available routes: {routes}. Core validates every selected route.',
                    {
                      'routes': connections
                          .map((v) => localize(context, routeLabel(v)))
                          .join(', '),
                    },
                  )
                : 'Device-specific preferences take priority. No background scanning.',
          ],
          ['button', 'Save Preference'],
        ];
        break;
      case 'R02':
      case 'R05':
        b = [
          [
            'field',
            'Room name',
            c.rooms
                    .where((r) => r['id'] == c.editingRoom)
                    .firstOrNull?['name'] ??
                '',
          ],
          [
            'chips',
            ['Balcony', 'Nursery'],
          ],
          ['button', 'Save Room'],
          ['button', 'Cancel', 'secondary'],
        ];
        break;
      case 'R04':
        ui['header'] = UiMessage('Delete {room}?', {
          'room': c.roomName(c.editingRoom),
        });
        break;
      case 'R03':
        b = [
          [
            'note',
            'Select devices and a destination room. Access stays unchanged.',
          ],
          [
            'choices',
            [...c.rooms.map((r) => LiteralText(r['name'])), 'Unassigned'],
            c.selectedRoom == null
                ? 0
                : c.rooms
                      .indexWhere((room) => room['id'] == c.selectedRoom)
                      .clamp(0, c.rooms.length),
          ],
          ['button', 'Move Devices'],
          ['button', 'Add Room', 'secondary'],
        ];
        break;
      case 'C01':
        ui['header'] = device?['alias'] == null
            ? 'Device'
            : LiteralText(device!['alias']);
        if (device?['support_status'] == 'unsupported') {
          b = [
            [
              'status',
              'Unsupported model',
              'This device was retrieved from the platform, but this model has no admitted readings or controls.',
              'warning',
            ],
            ['button', 'Device Information', 'secondary'],
          ];
          break;
        }
        ui['subtitle'] = applianceFamily(c) == null
            ? roomPresentation(c, prefs?['room_id'])
            : '';
        final temperature = c.observed('climate.target_temperature');
        final targetAction = c.capability('climate')?['actions']?['set'];
        final power = c.capability('power');
        final powerAction = power?['actions']?['set'] as Json?;
        final canEditPower =
            powerAction != null &&
            actionUnavailable(power!, powerAction) == null &&
            powerAction['parameters']?['properties']?['state'] != null;

        final canEditTarget =
            targetAction?['risk'] == 'low' &&
            targetAction?['parameters']?['properties']?['target_temperature'] !=
                null;
        values['observed_target'] = temperature is num
            ? temperature.toDouble()
            : double.nan;
        fields
                .putIfAbsent('observed_mode', () => TextEditingController())
                .text =
            '${c.observed('climate.mode') ?? 'Not available'}';
        b = [
          if (c.components.length > 1)
            [
              'tabs',
              c.components.map((part) => '${part['id']}').toList(),
              c.components.indexWhere((part) => part['id'] == c.component),
              c.components.map((part) => 'component:${part['id']}').toList(),
            ],
          if (c.capability('climate') != null)
            [
              'climate',
              c.snapshot == null
                  ? 'No reading'
                  : localize(
                      context,
                      UiMessage('{source} observation', {
                        'source': UiMessage(
                          c.snapshot!['from_cache'] == true ? 'Cached' : 'Read',
                        ),
                      }),
                    ),
              canEditTarget,
            ],
          if (c.capability('climate') != null && !canEditTarget)
            ['note', 'Temperature is read-only here.'],
          if (power != null ||
              targetAction?['parameters']?['properties']?['mode'] != null ||
              c.capability(
                    'climate',
                  )?['properties']?['current_temperature']?['readable'] ==
                  true)
            [
              'rows',
              [
                if (c.capability('power') != null)
                  [
                    'Power',
                    c.observed('power.state') == null
                        ? 'Unknown'
                        : c.observed('power.state') == true
                        ? 'On'
                        : 'Off',
                    canEditPower ? 'Power' : null,
                  ],
                if (c.capability(
                      'climate',
                    )?['actions']?['set']?['parameters']?['properties']?['mode'] !=
                    null)
                  [
                    'Mode',
                    UiMessage('{mode}  ›', {
                      'mode': UiMessage(
                        controlLabel(
                          '${c.observed('climate.mode') ?? 'Unknown'}',
                        ),
                      ),
                    }),
                    targetAction?['risk'] == 'low' ? 'Mode' : null,
                  ],
                if (c.capability(
                      'climate',
                    )?['properties']?['current_temperature']?['readable'] ==
                    true)
                  [
                    'Current temperature',
                    propertyValue(
                      c.observed('climate.current_temperature'),
                      c.capability(
                        'climate',
                      )!['properties']['current_temperature'],
                    ),
                    null,
                  ],
              ],
            ],
          ...describedControls(c),
          ['button', 'Refresh State', 'secondary'],
          ['button', 'Device information', 'secondary'],
          ['button', 'Recent Operations', 'secondary'],
        ];
        break;
      case 'C08':
        ui['header'] = device?['alias'] == null
            ? 'Device'
            : LiteralText(device!['alias']);
        ui['subtitle'] = '';
        b = [];
        break;
      case 'C02':
        b = [
          [
            'choices',
            c.capability(
                  'climate',
                )?['actions']?['set']?['parameters']?['properties']?['mode']?['enum'] ??
                [],
            0,
          ],
          ['button', 'Save Mode'],
          ['button', 'Cancel', 'secondary'],
        ];
        break;
      case 'C10':
        ui['base'] = history.isNotEmpty && history.last == 'C08'
            ? 'C08'
            : 'C01';
        ui['overlay'] = 'sheet';
        ui['header'] = actionName == null
            ? 'Action unavailable'
            : widget.catalog.labels['${actionCapability!['id']}.$actionName'] ??
                  '${controlLabel(actionCapability!['id'])} · ${controlLabel(actionName!)}';
        b = [
          ['note', 'Choose a supported action from the device page.'],
        ];
        break;
      case 'C12':
        ui['header'] = c.batchResult == null
            ? 'Select devices'
            : 'Operation results';
        b = c.batchResult == null
            ? [
                [
                  'note',
                  'Choose exact devices and a power action. Each outcome is recorded separately.',
                ],
                [
                  'choices',
                  ['Turn on', 'Turn off'],
                  0,
                ],
                ['button', 'Apply to Selected Devices'],
                ['button', 'Cancel', 'secondary'],
              ]
            : [
                [
                  'status',
                  outcomeLabel(c.batchResult!['status']),
                  'Inspect each item. Completed actions are not rolled back.',
                  outcomeColor(c.batchResult!['status']),
                ],
                [
                  'rows',
                  [
                    for (
                      var i = 0;
                      i < (c.batchResult!['items'] as List).length;
                      i++
                    )
                      [
                        LiteralText(
                          receiptDeviceName(c.batchResult!['items'][i]),
                        ),
                        outcomeLabel(c.batchResult!['items'][i]['status']),
                        'batch-item:$i',
                      ],
                  ],
                ],
                ['button', 'View Operations', 'secondary'],
                ['button', 'Done'],
              ];
        break;
      case 'C14':
        ui['base'] = history.isNotEmpty && history.last == 'C08'
            ? 'C08'
            : 'C01';
        b = [
          [
            'note',
            UiMessage('Observed target remains {value}.', {
              'value':
                  c.observed('climate.target_temperature') ??
                  const UiMessage('Unknown'),
            }),
          ],
          [
            'field',
            'Target temperature',
            '${c.observed('climate.target_temperature') ?? ''}',
          ],
          [
            'note',
            '${c.capability('climate')?['actions']?['set']?['parameters']?['properties']?['target_temperature']}',
          ],
          ['button', 'Save target'],
          ['button', 'Cancel', 'secondary'],
        ];
        break;
      case 'A01':
      case 'A02':
      case 'A03':
      case 'A04':
      case 'A06':
      case 'A07':
        b = [
          [
            'status',
            c.isMock ? 'Mock discovery' : 'Saved-account retrieval',
            c.isMock
                ? 'Only a fictional appliance can be discovered by this host.'
                : 'Explicitly retrieve cloud inventory from saved Meiju accounts. No LAN scan or pairing is started.',
            'neutral',
          ],
          [
            'button',
            c.isMock ? 'Discover Mock device' : 'Retrieve Saved Accounts',
          ],
          ['button', 'Add Platform', 'secondary'],
          [
            'note',
            c.isMock
                ? 'No physical appliance or local network is scanned.'
                : 'New accounts connect from Add Platform and retrieve their devices automatically.',
          ],
        ];
        break;
      case 'A05':
        b = [
          [
            'status',
            'Identity merge unavailable',
            'Names alone do not prove identity. No native merge method exists.',
            'warning',
          ],
        ];
        break;
      case 'S04':
        b = [
          if (c.cleanupCount != null)
            [
              'status',
              'Local cleanup completed',
              UiMessage(
                '{count} {kind} removed. New operations can add data again.',
                {
                  'count': c.cleanupCount!,
                  'kind': UiMessage(
                    c.cleanupResult == LocalCleanup.cachedState
                        ? 'saved snapshots'
                        : 'detailed receipts',
                  ),
                },
              ),
              'neutral',
            ],
          [
            'note',
            'Choose what to clear. Accounts, credentials, devices, rooms and preferences stay on this installation.',
          ],
          if (c.supports('clear_cached_state'))
            ['button', 'Clear Cached State', 'secondary'],
          if (c.supports('clear_execution_history'))
            ['button', 'Clear Recent Operations', 'secondary'],
          if (!c.supports('clear_cached_state') ||
              !c.supports('clear_execution_history'))
            ['note', 'Some cleanup actions require a native library update.'],
          if (c.cleanupNeedsReload) ...[
            [
              'note',
              'Cleanup was not fully confirmed. Reload local data before deciding what to do next.',
            ],
            ['button', 'Reload local data', 'secondary'],
          ],
          ['button', 'Remove Local Data', 'secondary'],
          [
            'note',
            'Full-profile deletion is unavailable. Backups and storage pages are not securely erased.',
          ],
        ];
        break;
      case 'S09':
        ui['header'] = cleanupScope == LocalCleanup.cachedState
            ? 'Clear cached state?'
            : cleanupScope == LocalCleanup.executionHistory
            ? 'Clear recent operations?'
            : 'Remove local data';
        b = cleanupScope == null
            ? [
                [
                  'status',
                  'Full cleanup unavailable',
                  'Full-profile deletion is not available. Use the separate cache or history action.',
                  'warning',
                ],
                ['button', 'Back', 'secondary'],
              ]
            : [
                [
                  'status',
                  'Review the scope',
                  cleanupScope == LocalCleanup.cachedState
                      ? 'Remove saved device and component readings from this installation. Missing readings become unknown. No device refresh will start.'
                      : 'Remove detailed operation receipts, including uncertain outcomes and their evidence. Request IDs remain reserved: clearing history never permits replay. Active operations prevent cleanup.',
                  'warning',
                ],
                [
                  'note',
                  cleanupScope == LocalCleanup.cachedState
                      ? 'Operation receipts are retained.'
                      : 'Saved device readings are retained.',
                ],
                [
                  'note',
                  'Accounts, credentials, devices, rooms, preferences and permissions remain. This does not securely erase backups or storage pages.',
                ],
                if (c.cleanupNeedsReload) ...[
                  [
                    'note',
                    'Reload local data before confirming another cleanup.',
                  ],
                  ['button', 'Reload local data', 'secondary'],
                ] else
                  ['button', 'Confirm Clear'],
                ['button', 'Cancel', 'secondary'],
              ];
        break;
      case 'S10':
        b = [
          ['field', 'Sharing name', c.preferences.sharingName],
          ['note', 'Shown only during an explicit nearby sharing session.'],
          ['button', 'Save Name'],
        ];
        break;
      case 'S06':
        final r = c.receipt;
        b = r == null
            ? [
                ['note', 'No retained receipt selected.'],
              ]
            : receiptBlocks(r, receiptDeviceName(r));
        break;
      case 'S07':
        b = diagnosticsBlocks(c.diagnostics, exporting: exporting || c.busy);
        break;
      case 'S08':
        b = [
          ['hero', 'home', 'HomeVerb', 'One language for your appliances.'],
          [
            'note',
            c.isMock
                ? '0.2.0 · Native Mock host · Fictional devices'
                : '0.2.0 · Experimental Meiju · Device controls',
          ],
          [
            'rows',
            [
              ['Compatibility', '›'],
              ['Privacy', '›'],
              ['Licenses', '›'],
            ],
          ],
        ];
        break;
      case 'S12':
        b = [
          [
            'status',
            'Experimental device support',
            'Supported controls are available when you open a device. Compatibility depends on its exact model and region.',
            'neutral',
          ],
          [
            'note',
            'Support requires model, region and physical-device evidence.',
          ],
        ];
        break;
    }
    if (b != null) ui['blocks'] = b;
    if (formError != null && id == this.id)
      ui['blocks'] = [
        ['status', 'Check this value', formError, 'error'],
        ...ui['blocks'] as List,
      ];
    return ui;
  }

  Widget deviceCards() => DeviceGrid(
    keys: c.visibleCards.map((v) => v['device']['id'] as String).toList(),
    data: c.visibleCards.map((v) {
      final d = v['device'] as Json,
          p = v['preferences'] as Json,
          s = c.snapshots[d['id']];
      return [
        '${d['alias']}',
        roomPresentation(c, p['room_id']),
        s?['observations']?['climate.target_temperature']?['value'] == null
            ? '—'
            : '${s!['observations']['climate.target_temperature']['value']}°C',
        d['support_status'] == 'unsupported'
            ? 'Unsupported model'
            : s == null
            ? 'Not read'
            : 'Saved observation',
      ];
    }).toList(),
    onAction: action,
  );
  Widget? special() => specialFor(id);
  Object roomDeviceCount(String? room) {
    final count = c.cards
        .where((card) => card['preferences']['room_id'] == room)
        .length;
    return UiMessage(count == 1 ? '{count} device' : '{count} devices', {
      'count': count,
    });
  }

  Widget? specialFor(String id) {
    if (id == 'L01') {
      final t = translationsOf(context);
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          for (final locale in <String?>[null, 'en-US', 'zh-CN'])
            ListTile(
              key: ValueKey('locale:${locale ?? 'system'}'),
              title: Text(
                locale == null
                    ? localize(context, 'System default')
                    : locale == 'en-US'
                    ? 'English'
                    : AppLocale.zhCn.translations.mobileLanguageName,
              ),
              subtitle: locale == null ? null : Text(locale),
              trailing: c.preferences.locale == locale
                  ? const Icon(Icons.check, color: accent)
                  : null,
              onTap: c.busy
                  ? null
                  : () async {
                      if (await c.savePreferences({'locale': locale})) {
                        await applyLocalePreference(locale);
                        changed();
                      }
                    },
            ),
          Copy(t.mobileLanguageNotice),
        ],
      );
    }
    if (id == 'P04') {
      final epoch = navigation;
      return AccountSignIn(
        key: ValueKey('signin:$epoch'),
        needsReload: c.accountNeedsReload,
        busy: c.busy,
        onSubmit: (login, password) async {
          final success = await c.connectAccount(
            login,
            password,
            connectionLabel,
          );
          if (success && mounted && epoch == navigation) await go('P08');
        },
        onCancel: () => action('Cancel'),
        onReload: () => action('Reload Accounts'),
      );
    }

    if (['D01', 'D11', 'D12'].contains(id) && c.cards.isNotEmpty)
      return DeviceHome(
        mode: 'all',
        onAction: action,
        cards: deviceCards(),
        roomNames: [
          'All',
          ...c.rooms.map((v) => LiteralText(v['name'])),
          'Unassigned',
        ],
        filter: ['All', 'Unassigned'].contains(c.filter)
            ? c.filter
            : c.roomName(c.filter),
      );
    if (id == 'C08') {
      return CompartmentControls(
        controller: c,
        onAction: action,
        onTarget: (component, value) => finish(
          c.execute('climate', 'set', {
            'target_temperature': value,
          }, componentId: component),
          'S06',
        ),
      );
    }
    if (id == 'C01' &&
        c.capability('climate') != null &&
        c.capability('power') != null &&
        c.currentCard?['device']['support_status'] != 'unsupported') {
      return ClimateControls(
        key: ValueKey('climate:${c.selectedDevice}:${c.component}'),
        controller: c,
        onAction: action,
        onExecute: executeOnDevice,
      );
    }
    if (id == 'C01' && applianceFamily(c) != null) {
      final epoch = navigation;
      return ApplianceControls(
        key: ValueKey(
          '${c.selectedDevice}:${c.component}:${applianceFamily(c)}',
        ),
        controller: c,
        family: applianceFamily(c)!,
        onAction: action,
        onExecute: (capability, action, arguments) async {
          final success = await c.execute(capability, action, arguments);
          if (success && mounted && epoch == navigation) await go('S06');
          return success;
        },
      );
    }
    if (id == 'C10' && actionCapability != null && actionName != null) {
      final epoch = navigation;
      if (c.approvalNeedsReceipt) {
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Copy(
              'The confirmation reply was interrupted. Look up the original receipt before considering another command.',
            ),
            Copy(
              UiMessage('Request: {request}', {
                'request': c.approvalRequestId ?? '',
              }),
            ),
            ActionButton(
              'Look Up Receipt',
              onPressed: c.busy
                  ? null
                  : () async {
                      if (await c.loadReceipt(c.approvalRequestId!) &&
                          mounted &&
                          epoch == navigation) {
                        c.approvalNeedsReceipt = false;
                        await go('S06');
                      }
                    },
            ),
            ActionButton(
              'Cancel',
              style: 'secondary',
              onPressed: () => action('Cancel'),
            ),
          ],
        );
      }
      if (c.approvalPreview case final preview?) {
        return ApprovalReview(
          preview: preview,
          labels: widget.catalog.labels,
          onCancel: () => action('Cancel'),
          onConfirm: () async {
            if (await c.confirmApproval() && mounted && epoch == navigation)
              await go('S06');
          },
        );
      }
      return CapabilityActionForm(
        key: ValueKey('$epoch:${actionCapability!['id']}:$actionName'),
        capability: actionCapability!,
        action: actionName!,
        onPrepare: c.supportsTrustedApproval
            ? (arguments, route) => c.prepareApproval(
                actionCapability!['id'],
                actionName!,
                arguments,
                route,
              )
            : null,
        observation: (property) =>
            c.snapshot?['observations']?[property] as Json?,
        error: () => c.error,
        onCancel: () => action('Cancel'),
        onSubmit: (arguments) async {
          final success = await c.execute(
            actionCapability!['id'],
            actionName!,
            arguments,
          );
          if (success && mounted && epoch == navigation) await go('S06');
          return success;
        },
      );
    }
    if (id == 'D03')
      return Column(
        children: [
          Blocks(
            screenFor(id)['blocks'] as List,
            onAction: action,
            fields: fields,
            choices: choices,
            values: values,
            onChanged: () {
              c.search = input('Search devices');
              changed();
            },
          ),
          const SizedBox(height: 12),
          deviceCards(),
        ],
      );
    if (id == 'R01')
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (c.rooms.isNotEmpty)
            Surface(
              radius: 18,
              child: ReorderableListView.builder(
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                padding: EdgeInsets.zero,
                buildDefaultDragHandles: false,
                itemCount: c.rooms.length,
                onReorderItem: (from, to) async {
                  if (c.busy) return;
                  final ids = c.rooms
                      .map((room) => room['id'] as String)
                      .toList();
                  ids.insert(to, ids.removeAt(from));
                  await c.reorderRooms(ids);
                },
                itemBuilder: (context, index) {
                  final r = c.rooms[index];
                  return Column(
                    key: ValueKey('room:${r['id']}'),
                    children: [
                      if (index > 0)
                        const Padding(
                          padding: EdgeInsets.symmetric(horizontal: 16),
                          child: Divider(
                            height: 0,
                            thickness: .6,
                            color: Color(0xffdeded7),
                          ),
                        ),
                      Padding(
                        padding: const EdgeInsets.only(left: 16, right: 6),
                        child: Row(
                          children: [
                            Expanded(
                              child: Copy(
                                LiteralText(r['name']),
                                control: true,
                              ),
                            ),
                            const SizedBox(width: 10),
                            Copy(
                              roomDeviceCount(r['id']),
                              control: true,
                              size: 12,
                              color: muted,
                            ),
                            ReorderableDragStartListener(
                              index: index,
                              child: SizedBox(
                                width: 34,
                                height: 44,
                                child: PopupMenuButton<String>(
                                  tooltip: localize(
                                    context,
                                    UiMessage('Room actions {name}', {
                                      'name': r['name'],
                                    }),
                                  ),
                                  padding: EdgeInsets.zero,
                                  icon: const Glyph(
                                    'drag',
                                    size: 11,
                                    color: muted,
                                  ),
                                  onSelected: (v) async {
                                    c.editingRoom = r['id'];
                                    if (v == 'Rename')
                                      await go('R05');
                                    else if (v == 'Delete')
                                      await go('R04');
                                    else if (v == 'Move devices') {
                                      c.selection.clear();
                                      c.selectedRoom = r['id'];
                                      await go('R03');
                                    } else {
                                      final ids = c.rooms
                                              .map((e) => e['id'] as String)
                                              .toList(),
                                          i = c.rooms.indexOf(r),
                                          j = v == 'Move up' ? i - 1 : i + 1;
                                      if (j >= 0 && j < ids.length) {
                                        ids.removeAt(i);
                                        ids.insert(j, r['id']);
                                        await c.reorderRooms(ids);
                                      }
                                    }
                                  },
                                  itemBuilder: (_) =>
                                      [
                                            'Rename',
                                            'Delete',
                                            'Move devices',
                                            'Move up',
                                            'Move down',
                                          ]
                                          .map(
                                            (v) => PopupMenuItem(
                                              value: v,
                                              child: Text(localize(context, v)),
                                            ),
                                          )
                                          .toList(),
                                ),
                              ),
                            ),
                          ],
                        ),
                      ),
                    ],
                  );
                },
              ),
            ),
          const SizedBox(height: 12),
          Rows(
            [
              ['Unassigned', roomDeviceCount(null)],
            ],
            onAction: (_) async {
              c.filter = 'Unassigned';
              await go('D01');
            },
          ),
          ActionButton('Add Room', onPressed: () => action('Add Room')),
          const Copy(
            'Drag to reorder. Use a row menu to rename or delete.',
            color: muted,
          ),
        ],
      );
    if (id == 'R03' || (id == 'C12' && c.batchResult == null))
      return Column(
        children: [
          for (final card in c.cards)
            ChoiceTile(
              key: ValueKey('select-device:${card['device']['id']}'),
              label: LiteralText(card['device']['alias']),
              selected: c.selection.contains(card['device']['id']),
              multiple: true,
              onTap: c.busy
                  ? null
                  : () {
                      setState(() {
                        final id = card['device']['id'] as String;
                        if (!c.selection.remove(id)) c.selection.add(id);
                      });
                    },
            ),
          Blocks(
            screenFor(id)['blocks'] as List,
            onAction: action,
            fields: fields,
            choices: choices,
            values: values,
            onChanged: changed,
          ),
        ],
      );
    if (id == 'D10') {
      final cards = c.visibleCards;
      Future<void> move(int from, int to) async {
        if (c.busy || to < 0 || to >= cards.length) return;
        final ids = cards
            .map((card) => card['device']['id'] as String)
            .toList();
        ids.insert(to, ids.removeAt(from));
        await c.reorderDevices(ids);
      }

      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Copy(
            UiMessage('{room} · Drag handles change the visible order.', {
              'room': c.filter == 'All' || c.filter == 'Unassigned'
                  ? UiMessage(c.filter)
                  : LiteralText(c.roomName(c.filter)),
            }),
            color: muted,
          ),
          const SizedBox(height: 12),
          if (cards.isEmpty)
            const Copy('No devices in this view.')
          else
            Surface(
              radius: 18,
              child: ReorderableListView.builder(
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                padding: EdgeInsets.zero,
                buildDefaultDragHandles: false,
                itemCount: cards.length,
                onReorderItem: move,
                itemBuilder: (context, index) {
                  final card = cards[index],
                      deviceId = card['device']['id'] as String;
                  return Column(
                    key: ValueKey(deviceId),
                    children: [
                      if (index > 0)
                        const Padding(
                          padding: EdgeInsets.symmetric(horizontal: 16),
                          child: Divider(
                            height: 0,
                            thickness: .6,
                            color: Color(0xffdeded7),
                          ),
                        ),
                      Padding(
                        padding: const EdgeInsets.only(left: 16, right: 4),
                        child: Row(
                          children: [
                            Expanded(
                              child: Copy(
                                LiteralText(card['device']['alias']),
                                control: true,
                              ),
                            ),
                            if (card['preferences']['favorite'] == true)
                              const Copy(
                                'Pinned',
                                size: 12,
                                color: muted,
                                control: true,
                              ),
                            ReorderableDragStartListener(
                              index: index,
                              enabled: !c.busy,
                              child: PopupMenuButton<int>(
                                key: ValueKey('order-menu:$deviceId'),
                                tooltip: localize(
                                  context,
                                  UiMessage('Reorder {name}', {
                                    'name': card['device']['alias'],
                                  }),
                                ),
                                enabled: !c.busy,
                                icon: const Glyph(
                                  'drag',
                                  size: 11,
                                  color: muted,
                                ),
                                onSelected: (offset) =>
                                    move(index, index + offset),
                                itemBuilder: (_) => [
                                  PopupMenuItem(
                                    value: -1,
                                    enabled: index > 0,
                                    child: Text(localize(context, 'Move up')),
                                  ),
                                  PopupMenuItem(
                                    value: 1,
                                    enabled: index + 1 < cards.length,
                                    child: Text(localize(context, 'Move down')),
                                  ),
                                ],
                              ),
                            ),
                          ],
                        ),
                      ),
                    ],
                  );
                },
              ),
            ),
          const SizedBox(height: 12),
          const Copy(
            'Room membership stays unchanged. Hidden devices retain their position.',
            color: muted,
          ),
          const SizedBox(height: 12),
          ActionButton('Done', onPressed: () => action('Back')),
        ],
      );
    }
    if (id == 'S05')
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (c.history.isEmpty)
            const Copy(
              'No retained operations. This does not prove that no past action occurred.',
            )
          else
            Rows([
              for (final receipt in c.history)
                [
                  LiteralText(receiptDeviceName(receipt)),
                  UiMessage('{outcome} · {time}', {
                    'outcome': UiMessage(outcomeLabel(receipt['status'])),
                    'time': receiptTime(receipt['completed_at'], compact: true),
                  }),
                  'receipt:${receipt['request_id']}',
                ],
            ], onAction: action),
          const Copy(
            'Select an operation to inspect its retained evidence. Sharing history is unavailable.',
            color: muted,
          ),
        ],
      );
    return null;
  }

  String receiptDeviceName(Json receipt) {
    for (final card in c.cards) {
      if (card['device']['id'] == receipt['device_id']) {
        return card['preferences']['alias'] ?? card['device']['alias'];
      }
    }
    return 'Retained device';
  }

  @override
  Widget build(BuildContext context) => MaterialApp(
    navigatorKey: navigator,
    locale: translationsOf(context).$meta.locale.flutterLocale,
    supportedLocales: AppLocaleUtils.supportedLocales,
    localizationsDelegates: GlobalMaterialLocalizations.delegates,
    debugShowCheckedModeBanner: false,
    theme: windowTheme(),
    highContrastTheme: windowTheme(highContrast: true),
    home: PopScope(
      canPop: false,
      onPopInvokedWithResult: (popped, _) {
        if (!popped) action('Back');
      },
      child: Scaffold(
        body: ScreenView(
          key: id == 'S06' ? ObjectKey(c.receipt) : null,
          id: id,
          catalog: widget.catalog,
          screenOverride: screen,
          home: special(),
          resolveScreen: screenFor,
          resolveHome: specialFor,
          onAction: action,
          fields: fields,
          choices: choices,
          values: values,
          onChanged: changed,
          busy: c.busy,
          error: id == 'C10' && actionCapability != null ? null : c.error,
        ),
      ),
    ),
  );
}
