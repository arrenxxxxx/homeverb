import 'localization.dart';
import 'credential_access.dart';
import 'dart:typed_data';
import 'package:flutter/material.dart';
import 'controller.dart';
import 'native_client.dart';
import 'skin.dart';

String credentialLabel(dynamic state) => switch (state) {
  'stored' => 'Saved authorization · not checked',
  'pending' => 'Authorization needs recovery',
  'removing' => 'Local cleanup pending',
  _ => 'Unsupported account state',
};

Object accountDisplayName(HomeController c, Json account) {
  final name = c.accountDisplayName(account);
  return name == null ? const UiMessage('Meiju account') : LiteralText(name);
}

class PlatformAccounts extends StatelessWidget {
  const PlatformAccounts({
    required this.controller,
    required this.onAction,
    super.key,
  });
  final HomeController controller;
  final void Function(String) onAction;

  @override
  Widget build(BuildContext context) {
    final c = controller;
    final highContrast = MediaQuery.highContrastOf(context);
    Widget accountCard(Json account) {
      final id = account['id'] as String;
      final duplicate =
          c.accounts
              .where(
                (other) =>
                    c.accountDisplayName(other) ==
                    c.accountDisplayName(account),
              )
              .length >
          1;
      return Padding(
        padding: const EdgeInsets.only(bottom: 12),
        child: OutlinedButton(
          key: ValueKey('account:$id'),
          onPressed: !c.busy && c.supports('get_account')
              ? () => onAction('account:$id')
              : null,
          style: OutlinedButton.styleFrom(
            foregroundColor: highContrast ? Colors.black : ink,
            backgroundColor: Colors.white,
            padding: const EdgeInsets.all(20),
            side: BorderSide(
              color: highContrast ? Colors.black : const Color(0xffe1e3de),
            ),
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(22),
            ),
          ),
          child: Row(
            children: [
              Container(
                width: 48,
                height: 48,
                decoration: BoxDecoration(
                  color: highContrast ? Colors.white : const Color(0xfff7eee6),
                  borderRadius: BorderRadius.circular(15),
                ),
                child: Icon(
                  Icons.person_outline_rounded,
                  color: highContrast ? Colors.black : accent,
                  size: 28,
                ),
              ),
              const SizedBox(width: 14),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Copy(accountDisplayName(c, account), size: 18, bold: true),
                    const SizedBox(height: 6),
                    const Copy('Meiju', size: 12, color: muted),
                    if (duplicate)
                      Copy(
                        LiteralText(id.split('-').last),
                        size: 11,
                        color: muted,
                      ),
                    if (account['credential_state'] != 'stored') ...[
                      const SizedBox(height: 6),
                      Copy(
                        credentialLabel(account['credential_state']),
                        size: 12,
                        color: highContrast ? Colors.black : accent,
                      ),
                    ],
                  ],
                ),
              ),
              const SizedBox(width: 8),
              const Icon(Icons.chevron_right_rounded, size: 22),
            ],
          ),
        ),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            const Expanded(child: Copy('Accounts', size: 14, bold: true)),
            if (c.accountNeedsReload && c.supports('list_accounts'))
              IconButton(
                tooltip: localize(context, 'Reload Accounts'),
                onPressed: c.busy ? null : () => onAction('Reload Accounts'),
                icon: const Icon(Icons.refresh_rounded, color: muted),
              ),
          ],
        ),
        const SizedBox(height: 8),
        if (c.isMock)
          const Padding(
            padding: EdgeInsets.only(bottom: 16),
            child: Copy(
              'Account connection is unavailable in this Mock build.',
              color: muted,
            ),
          ),
        if (c.accounts.isEmpty && !c.isMock)
          const Padding(
            padding: EdgeInsets.only(bottom: 20),
            child: Copy(
              'Add a platform to bring your appliances here.',
              color: muted,
            ),
          ),
        for (final account in c.accounts) accountCard(account),
        ActionButton(
          'Add Platform',
          onPressed: c.busy ? null : () => onAction('Add Platform'),
        ),
      ],
    );
  }
}

Object roomPresentation(HomeController c, String? id) {
  final room = c.rooms.where((r) => r['id'] == id).firstOrNull;
  return room == null
      ? const UiMessage('Unassigned')
      : LiteralText(room['name']);
}

Object savedCardName(HomeController c, String id) {
  final card = c.cards.where((v) => v['device']['id'] == id).firstOrNull;
  return card == null
      ? UiMessage('Saved card · {suffix}', {'suffix': id.split('-').last})
      : LiteralText(card['device']['alias']);
}

List<dynamic> linkedCardRows(
  HomeController c,
  List<String> ids, {
  bool open = false,
}) => [
  for (final id in ids)
    [
      savedCardName(c, id),
      c.cards.any((card) => card['device']['id'] == id)
          ? roomPresentation(
              c,
              c.cards.firstWhere(
                (card) => card['device']['id'] == id,
              )['preferences']['room_id'],
            )
          : 'Details unavailable',
      open && c.cards.any((card) => card['device']['id'] == id)
          ? 'open:$id'
          : null,
    ],
];

Object savedBindingName(HomeController c, Json binding) {
  final card = c.cards
      .where(
        (card) =>
            card['device']['id'] == binding['device_id'] &&
            (card['device']['connections'] as List).any(
              (connection) => connection['id'] == binding['connection_id'],
            ),
      )
      .firstOrNull;
  return card == null
      ? const UiMessage('Saved binding without a matching card')
      : LiteralText(card['device']['alias']);
}

List<dynamic> savedInventoryBlocks(HomeController c) {
  final result = c.accountInventory;
  if (result == null) return [];
  final devices = (result['devices'] as List).cast<Json>();
  return [
    [
      'note',
      'Saved inventory only. This does not check current access, online state, model support or control permission. Historical inventory time is shown separately when recorded.',
    ],
    if (devices.isEmpty)
      [
        'note',
        'No saved bindings were returned. This does not prove the cloud account is empty. Use Retrieve Devices to refresh explicitly.',
      ],
    for (final binding in devices)
      [
        'card',
        savedBindingName(c, binding),
        [
          [
            'Saved membership',
            switch (binding['membership']) {
              'listed' => 'Listed in the last complete inventory',
              'not_listed' => 'Not listed · live requests blocked',
              'unconfirmed' => 'Membership unconfirmed',
              _ => 'Unknown membership',
            },
            null,
          ],
          ['Device ID', binding['device_id'], null],
          ['Connection ID', binding['connection_id'], null],
          if (c.supports('inspect_device_conflicts'))
            [
              'Inspect duplicate device records',
              'Saved records only',
              'conflicts:${binding['device_id']}',
            ],
          if (binding['membership'] != 'listed')
            [
              'Next step',
              'Retrieve Devices explicitly to check membership.',
              null,
            ],
        ],
      ],
  ];
}

List<dynamic> accountRefreshBlocks(HomeController c) {
  final retrieval = c.accountReport!['retrieval'] as Json;
  final status = retrieval['status'];
  if (status == 'complete')
    return [
      ['note', 'Device list updated'],
      if ((retrieval['device_ids'] as List).isEmpty)
        [
          'note',
          'Check that your appliances are added to this account in Meiju.',
        ],
    ];
  return [
    [
      'status',
      status == 'partial'
          ? 'Some devices could not be updated'
          : 'Could not update devices',
      'Your saved devices are still here. Try again or sign in again.',
      'warning',
    ],
    if (retrieval['error'] != null)
      ['note', UiError(retrieval['error']['code'])],
    if (c.account?['credential_state'] == 'stored' && c.canSignIn)
      ['button', 'Reauthorize', 'secondary'],
  ];
}

List<dynamic> accountBlocks(
  HomeController c,
  String id, {
  bool refreshing = false,
}) {
  final account = c.account;
  final state = account?['credential_state'];
  final actions = <dynamic>[
    if (state == 'stored' && c.supports('refresh_account'))
      ['button', 'Refresh Devices'],
    if (state == 'pending' && c.supports('recover_account')) ...[
      [
        'note',
        'Recovery validates saved authorization and retrieves cloud devices. It preserves local organization and sends no appliance controls.',
      ],
      ['button', 'Recover Saved Authorization'],
    ],
    if (c.canSignIn && account != null)
      [
        'button',
        state == 'pending' ? 'Sign In to the Original Account' : 'Reauthorize',
        'secondary',
      ],
    if (state == 'pending' && !c.canSignIn)
      [
        'note',
        'Signing in to a pending account requires native API 1.8 or later.',
      ],
    if (c.supports('remove_account') && account != null)
      [
        'button',
        state == 'removing' ? 'Retry local cleanup' : 'Remove From This Device',
        'secondary',
      ],
    if (c.accountNeedsReload) ['button', 'Reload Accounts', 'secondary'],
  ];
  switch (id) {
    case 'P01':
      return [];
    case 'P02':
      return [
        ['note', 'Choose the app you already use for your appliances.'],
        [
          'card',
          'Available platforms',
          [
            ['Meiju', 'Mainland China', 'Choose Meiju'],
          ],
        ],
        ['note', 'Supported devices vary by model.'],
      ];
    case 'P03':
      return [
        ['hero', 'platform', 'Meiju', 'Mainland China'],
        if (c.selectedAccount == null) ...[
          ['field', 'Account name', ''],
          ['note', 'Optional. Uses your sign-in account when blank.'],
        ],
        [
          'note',
          c.selectedAccount == null
              ? 'Use your existing Meiju account. Sign-in retrieves bound devices automatically. Open a supported device to control it.'
              : 'Sign in to the original account. Identity must match; local device names, rooms and favorites are preserved. Open a supported device to control it.',
        ],
        [
          'note',
          'Only mainland China is supported. Browser OAuth, code challenges and appliance approval are unavailable.',
        ],
        ['button', 'Continue to Sign In', c.canSignIn ? '' : 'disabled'],
        ['button', 'Cancel', 'secondary'],
      ];
    case 'P08':
      final retrieval = c.accountReport?['retrieval'] as Json?;
      final status = retrieval?['status'];
      return [
        [
          'status',
          status == 'complete'
              ? 'Device list updated'
              : status == 'partial'
              ? 'Some devices could not be updated'
              : 'Could not update devices',
          status == 'complete'
              ? UiMessage('{count} devices found.', {
                  'count': (retrieval!['device_ids'] as List).length,
                })
              : 'Your saved devices are still here. Try again or sign in again.',
          status == 'complete' ? 'success' : 'warning',
        ],
        if (status == 'complete' && (retrieval!['device_ids'] as List).isEmpty)
          [
            'note',
            'Check that your appliances are added to this account in Meiju.',
          ],
        if (status != 'complete') ...[
          if (retrieval?['error'] != null)
            ['note', UiError(retrieval!['error']['code'])],
          ...actions.where((action) => action[1] != 'Remove From This Device'),
        ],
        ['button', 'View Devices'],
        ['button', 'Account details', 'secondary'],
      ];
    case 'P09':
    case 'P10':
      if (account == null)
        return [
          ['note', 'Select a saved account.'],
          ['button', 'Reload Accounts'],
        ];
      return [
        ['summary', accountDisplayName(c, account), 'Meiju'],
        if (state != 'stored')
          [
            'status',
            credentialLabel(state),
            state == 'removing'
                ? 'Finish removing this account from this device.'
                : 'Restore access to update your devices.',
            'warning',
          ],
        if (c.accountReport?['account']['id'] == c.selectedAccount &&
            c.accountReport != null)
          ...accountRefreshBlocks(c),
        if (c.accountDeviceIds.isNotEmpty)
          [
            'card',
            'Devices',
            linkedCardRows(c, c.accountDeviceIds, open: true),
          ],
        if (c.accountDeviceIds.isEmpty && state == 'stored')
          [
            'note',
            'No devices yet. Update the list after adding appliances in Meiju.',
          ],
        if (state == 'stored' && c.supports('refresh_account'))
          [
            'button',
            'Refresh Devices',
            c.busy ? 'disabled' : 'secondary',
            refreshing,
          ],
        if (state != 'stored') ...actions,
        if (c.accountRemoval?['status'] == 'pending')
          ['note', UiError(c.accountRemoval!['error']?['code'] ?? 'UNKNOWN')],
        if (state == 'stored')
          [
            'disclosure',
            'Manage account',
            [
              [
                'rows',
                [
                  if (c.supports('rename_account'))
                    [
                      'Account name',
                      accountDisplayName(c, account),
                      'Rename Account',
                    ],
                  if (c.canSignIn &&
                      (c.accountReport == null ||
                          c.accountReport?['retrieval']['status'] ==
                              'complete'))
                    ['Reauthorize', '', 'Reauthorize'],
                  if (c.supports('remove_account'))
                    ['Remove From This Device', '', 'Remove From This Device'],
                ],
              ],
              if (c.supports('inspect_account_inventory'))
                [
                  'disclosure',
                  'Troubleshooting',
                  [
                    ['button', 'Inspect Saved Inventory', 'secondary'],
                    ...savedInventoryBlocks(c),
                    [
                      'note',
                      account['last_complete_inventory_at'] == null
                          ? 'No complete device-list time recorded.'
                          : UiMessage(
                              'Last complete device list: {time}. Recorded using this device’s clock; not current authorization or appliance status.',
                              {'time': account['last_complete_inventory_at']},
                            ),
                    ],
                    if (c.accountConflicts != null) ...[
                      [
                        'note',
                        'Saved records may refer to the same appliance. Physical identity is unverified. No devices are merged and no permission is transferred.',
                      ],
                      [
                        'note',
                        UiMessage('Selected device: {device}', {
                          'device':
                              c.accountConflicts!['selected']['device_id'],
                        }),
                      ],
                      if ((c.accountConflicts!['matches'] as List).isEmpty)
                        [
                          'note',
                          'No equal saved device identifiers were found.',
                        ],
                      for (final match
                          in c.accountConflicts!['matches'] as List)
                        [
                          'card',
                          'Possible duplicate',
                          [
                            ['Account ID', match['account_id'], null],
                            ['Device ID', match['device_id'], null],
                            ['Connection ID', match['connection_id'], null],
                            ['Saved membership', match['membership'], null],
                          ],
                        ],
                    ],
                  ],
                  c.selectedAccount,
                ],
            ],
            c.selectedAccount,
          ],
        if (state == 'stored' && c.accountNeedsReload)
          ['button', 'Reload Accounts', 'secondary'],
      ];
    case 'P11':
      return [
        [
          'status',
          'Remove from this device?',
          UiMessage(
            'Remove {account}, its local credentials and {count} linked cards, preferences and saved readings.',
            {
              'account': account == null
                  ? const UiMessage('the selected account')
                  : accountDisplayName(c, account),
              'count': c.removalScope?.length ?? c.accountDeviceIds.length,
            },
          ),
          'warning',
        ],
        if ((c.removalScope ?? c.accountDeviceIds).isNotEmpty)
          [
            'card',
            'Linked cards',
            linkedCardRows(c, c.removalScope ?? c.accountDeviceIds),
          ],
        [
          'note',
          'This only removes the account and its devices from HomeVerb on this device. Your Meiju account is unchanged. Reconnecting will not restore device names or room assignments.',
        ],
        if (c.removalScope != null && !c.accountNeedsReload)
          ['button', 'Remove Locally']
        else
          ['button', 'Review removal scope'],
        ['button', 'Cancel', 'secondary'],
      ];
    case 'P12':
      return [
        [
          'status',
          'Provider revocation unavailable',
          'Local removal does not revoke provider access or copies on another installation.',
          'warning',
        ],
        ['button', 'Back', 'secondary'],
      ];
    default:
      return [
        [
          'status',
          'Flow unavailable',
          'This authorization or approval method is not supported by the current native adapter.',
          'warning',
        ],
        ['button', 'Back to Accounts', 'secondary'],
      ];
  }
}

class AccountSignIn extends StatefulWidget {
  const AccountSignIn({
    required this.onSubmit,
    required this.onCancel,
    required this.onReload,
    required this.needsReload,
    required this.busy,
    this.reauthorizing = false,
    this.onRecover,
    this.credentialAccess = const PlatformCredentialAccess(),
    super.key,
  });
  final Future<void> Function(Uint8List login, Uint8List password) onSubmit;
  final Future<void> Function(Uint8List login, Uint8List password)? onRecover;
  final VoidCallback onCancel, onReload;
  final bool needsReload, busy, reauthorizing;
  final CredentialAccess credentialAccess;
  @override
  State<AccountSignIn> createState() => _AccountSignInState();
}

class _AccountSignInState extends State<AccountSignIn>
    with WidgetsBindingObserver {
  final login = TextEditingController(), password = TextEditingController();
  String? error;
  CredentialReadiness? readiness;
  bool foreground = true, submitting = false;
  int checkEpoch = 0;
  bool get canEnter =>
      readiness == CredentialReadiness.ready ||
      readiness == CredentialReadiness.notApplicable;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    checkReadiness();
  }

  Future<void> checkReadiness() async {
    final epoch = ++checkEpoch;
    setState(() => readiness = null);
    final result = await widget.credentialAccess.check();
    if (!mounted || !foreground || epoch != checkEpoch) return;
    setState(() => readiness = result);
  }

  void clear() {
    login.clear();
    password.clear();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    foreground = state == AppLifecycleState.resumed;
    if (!foreground) {
      checkEpoch++;
      clear();
      setState(() => readiness = null);
    } else {
      checkReadiness();
    }
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    checkEpoch++;
    clear();
    login.dispose();
    password.dispose();
    super.dispose();
  }

  Future<void> submit({bool recovery = false}) async {
    if (!foreground ||
        !canEnter ||
        submitting ||
        widget.busy ||
        widget.needsReload)
      return;
    if (!RegExp(r'^[\x21-\x7e]{1,256}$').hasMatch(login.text) ||
        !RegExp(r'^[\x20-\x7e]{1,256}$').hasMatch(password.text)) {
      setState(
        () => error =
            'Use 1–256 ASCII characters in each field; the account cannot contain spaces.',
      );
      return;
    }
    final loginBytes = Uint8List.fromList(login.text.codeUnits);
    final passwordBytes = Uint8List.fromList(password.text.codeUnits);
    clear();
    final epoch = ++checkEpoch;
    setState(() {
      error = null;
      submitting = true;
    });
    try {
      // Recheck immediately before dispatch; backgrounding invalidates this result.
      final result = await widget.credentialAccess.check();
      if (!mounted || !foreground || epoch != checkEpoch) return;
      setState(() => readiness = result);
      if (!canEnter) return;
      await (recovery ? widget.onRecover! : widget.onSubmit)(
        loginBytes,
        passwordBytes,
      );
    } finally {
      loginBytes.fillRange(0, loginBytes.length, 0);
      passwordBytes.fillRange(0, passwordBytes.length, 0);
      if (mounted) setState(() => submitting = false);
    }
  }

  Future<void> openSecuritySettings() async {
    clear();
    final opened = await widget.credentialAccess.openSecuritySettings();
    if (!mounted || !foreground) return;
    if (!opened) {
      setState(
        () => error =
            'Could not open Android security settings. Open them from the system Settings app.',
      );
    }
  }

  Widget field(
    String label,
    TextEditingController controller, {
    bool secret = false,
  }) => Padding(
    padding: const EdgeInsets.only(bottom: 16),
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
          child: Semantics(
            label: localize(context, label),
            child: ValueListenableBuilder<TextEditingValue>(
              valueListenable: controller,
              builder: (context, value, _) => TextField(
                key: ValueKey('credential:$label'),
                enabled: !widget.busy && !submitting,
                controller: controller,
                obscureText: secret,
                autocorrect: false,
                enableSuggestions: false,
                enableIMEPersonalizedLearning: false,
                smartDashesType: SmartDashesType.disabled,
                smartQuotesType: SmartQuotesType.disabled,
                autofillHints: null,
                keyboardType: TextInputType.visiblePassword,
                style: windowFieldTextStyle,
                textAlignVertical: TextAlignVertical.center,
                decoration: InputDecoration(
                  border: InputBorder.none,
                  contentPadding: fieldTextPadding(
                    context,
                    secret && value.text.isNotEmpty ? '•' : value.text,
                  ),
                ),
              ),
            ),
          ),
        ),
      ],
    ),
  );
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      const Copy('Meiju · Mainland China', size: 14, color: muted),
      const SizedBox(height: 20),
      Copy(
        widget.reauthorizing
            ? 'Sign in to the same Meiju account. Your device names and rooms will stay.'
            : 'Sign in with your Meiju account to add your appliances.',
        color: muted,
      ),
      const SizedBox(height: 16),
      if (canEnter) ...[
        field('Meiju account', login),
        field('Password', password, secret: true),
      ] else ...[
        Copy(switch (readiness) {
          null => 'Checking Android credential protection…',
          CredentialReadiness.screenLockRequired =>
            'Set a screen-lock PIN, pattern or password in Android Settings, then unlock this device and return. HomeVerb cannot save authorization without a secure screen lock.',
          CredentialReadiness.locked =>
            'Unlock this device, then return to sign in.',
          _ =>
            'Android credential support is unavailable. Restart the app or install a matching native build before trying again.',
        }),
        if (readiness == CredentialReadiness.screenLockRequired)
          ActionButton(
            'Open Android Security Settings',
            onPressed: openSecuritySettings,
          ),
        if (readiness != null)
          ActionButton(
            'Check Again',
            style: 'secondary',
            onPressed: checkReadiness,
          ),
      ],
      if (error != null) Copy(error!, color: const Color(0xff9e3421)),
      if (widget.busy)
        const Copy('Connecting and retrieving devices…')
      else if (widget.needsReload) ...[
        const Copy(
          'Reload saved accounts before starting another connection. An interrupted sign-in may have saved authorization.',
        ),
        ActionButton('Reload Accounts', onPressed: widget.onReload),
      ] else if (canEnter)
        ActionButton(
          'Sign In and Retrieve Devices',
          onPressed: submitting ? null : submit,
        ),
      if (canEnter && !widget.needsReload && widget.onRecover != null) ...[
        const Copy(
          'If the same account was refused because its identity changed, restore its saved binding after reviewing the matching devices.',
          color: muted,
        ),
        ActionButton(
          'Restore Saved Account',
          style: 'secondary',
          onPressed: submitting || widget.busy
              ? null
              : () => submit(recovery: true),
        ),
      ],
      ActionButton('Cancel', style: 'secondary', onPressed: widget.onCancel),
    ],
  );
}
