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

List<dynamic> linkedCardRows(
  HomeController c,
  List<String> ids, {
  bool open = false,
}) => [
  for (final id in ids)
    [
      c.cards
              .where((card) => card['device']['id'] == id)
              .firstOrNull?['device']['alias'] ??
          'Saved card · ${id.split('-').last}',
      c.cards.any((card) => card['device']['id'] == id)
          ? c.roomName(
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
        c.cards
                .where(
                  (card) =>
                      card['device']['id'] == binding['device_id'] &&
                      (card['device']['connections'] as List).any(
                        (connection) =>
                            connection['id'] == binding['connection_id'],
                      ),
                )
                .firstOrNull?['device']['alias'] ??
            'Saved binding without a matching card',
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

List<dynamic> accountBlocks(HomeController c, String id) {
  final account = c.account;
  final state = account?['credential_state'];
  final actions = <dynamic>[
    if (state == 'stored' && c.supports('refresh_account'))
      ['button', 'Retrieve Devices'],
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
      return [
        if (c.isMock)
          [
            'status',
            'Mock profile',
            'Platform authorization is unavailable in Mock. This profile contains fictional devices only.',
            'neutral',
          ],
        if (c.accountRemoval?['status'] == 'removed')
          [
            'status',
            'Removed from this device',
            'Other installations and provider authorization are unchanged.',
            'neutral',
          ],
        if (c.accounts.isEmpty)
          [
            'note',
            c.busy
                ? 'Loading local accounts…'
                : 'No platform account is saved on this installation.',
          ],
        if (c.accounts.isNotEmpty)
          [
            'card',
            'Meiju · Mainland China',
            [
              for (final a in c.accounts)
                [
                  a['label'],
                  '${credentialLabel(a['credential_state'])}${c.accounts.where((other) => other['label'] == a['label']).length > 1 ? ' · ${a['id'].split('-').last}' : ''}',
                  c.supports('get_account') ? 'account:${a['id']}' : null,
                ],
            ],
          ],
        ['button', 'Add Platform'],
        if (c.supports('list_accounts'))
          ['button', 'Reload Accounts', 'secondary']
        else
          [
            'note',
            'Local account management requires a native library update.',
          ],
        [
          'note',
          'Opening this page only reads local account metadata. Saved authorization does not prove current access.',
        ],
      ];
    case 'P02':
      return [
        [
          'card',
          'Appliance platforms',
          [
            ['Meiju', 'Mainland China · experimental', 'Choose Meiju'],
          ],
        ],
        [
          'note',
          c.isMock
              ? 'Account connection is unavailable in this Mock build.'
              : 'Direct cloud connection. Supported devices use read-only access in this build.',
        ],
        ['button', 'Compatibility Details', 'secondary'],
      ];
    case 'P03':
      return [
        ['hero', 'platform', 'Meiju', 'Mainland China'],
        ['field', 'Account name', account?['label'] ?? 'Meiju account'],
        [
          'note',
          c.selectedAccount == null
              ? 'Use your existing Meiju account. After sign-in, saved authorization is used to retrieve bound devices automatically. No appliance controls are granted.'
              : 'Sign in to the original account. Identity must match; local device names, rooms and favorites are preserved. This does not grant new controls.',
        ],
        [
          'note',
          'Only mainland China is supported. Browser OAuth, code challenges and appliance approval are unavailable.',
        ],
        ['button', 'Continue to Sign In', c.canSignIn ? '' : 'disabled'],
        ['button', 'Cancel', 'secondary'],
      ];
    case 'P08':
      final report = c.accountReport;
      final retrieval = report?['retrieval'] as Json?;
      final status = retrieval?['status'];
      return [
        [
          'status',
          status == 'complete'
              ? 'Device retrieval complete'
              : status == 'partial'
              ? 'Device retrieval incomplete'
              : 'Device retrieval failed',
          report == null
              ? 'No completed account report is available.'
              : '${report['authentication'] == 'succeeded' ? 'Sign-in succeeded.' : 'No new sign-in was attempted.'} ${credentialLabel(report['account']['credential_state'])}.',
          status == 'complete' ? 'neutral' : 'warning',
        ],
        if (status == 'complete')
          [
            'note',
            '${(retrieval!['device_ids'] as List).length} devices retrieved. Unsupported models remain visible. An empty result does not delete older saved cards.',
          ],
        if (retrieval?['error'] != null)
          [
            'note',
            '${retrieval!['error']['code']}: ${retrieval['error']['message']}',
          ],
        [
          'note',
          'Existing cards are reloaded from local storage. Retrieval does not establish physical state or grant appliance controls.',
        ],
        ...actions,
        ['button', 'View Devices', 'secondary'],
        ['button', 'Back to Accounts', 'secondary'],
      ];
    case 'P09':
    case 'P10':
      if (account == null)
        return [
          ['note', 'Select a saved account.'],
          ['button', 'Reload Accounts'],
        ];
      return [
        ['hero', 'platform', account['label'], 'Meiju · ${account['region']}'],
        [
          'status',
          credentialLabel(state),
          'Current provider authorization has not been checked by this local view.',
          state == 'stored' ? 'neutral' : 'warning',
        ],
        [
          'rows',
          [
            ['Devices', '${c.accountDeviceIds.length} saved cards', null],
            if (state != 'removing' && c.supports('rename_account'))
              ['Account name', account['label'], 'Rename Account'],
          ],
        ],
        if (c.accountDeviceIds.isNotEmpty)
          [
            'card',
            'Saved devices',
            linkedCardRows(c, c.accountDeviceIds, open: true),
          ],
        if (state == 'stored' && c.supports('inspect_account_inventory')) ...[
          ['button', 'Inspect Saved Inventory', 'secondary'],
          ...savedInventoryBlocks(c),
        ],
        if (c.accountRemoval?['status'] == 'pending')
          [
            'status',
            'Local cleanup pending',
            '${c.accountRemoval!['error']?['code']}: ${c.accountRemoval!['error']?['message']}',
            'warning',
          ],
        [
          'note',
          account['last_complete_inventory_at'] == null
              ? 'No complete device-list time recorded.'
              : 'Last complete device list: ${account['last_complete_inventory_at']}. Recorded using this device’s clock; not current authorization or appliance status.',
        ],
        if (c.accountConflicts != null) ...[
          [
            'note',
            'Saved records may refer to the same appliance. Physical identity is unverified. No devices are merged and no permission is transferred.',
          ],
          [
            'note',
            'Selected device: ${c.accountConflicts!['selected']['device_id']}',
          ],
          if ((c.accountConflicts!['matches'] as List).isEmpty)
            ['note', 'No equal saved device identifiers were found.'],
          for (final match in c.accountConflicts!['matches'] as List)
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
        ...actions,
        ['button', 'Revoke Access · if supported', 'secondary'],
      ];
    case 'P11':
      return [
        [
          'status',
          'Remove from this device?',
          'Remove ${account?['label'] ?? 'the selected account'}, its local credentials and ${c.removalScope?.length ?? c.accountDeviceIds.length} linked cards, preferences and saved readings.',
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
          'Rooms, operation receipts and replay protection remain. Other installations and provider authorization are unaffected. Reconnecting creates new local identities without restoring previous organization.',
        ],
        ['note', 'This does not securely erase backups or storage pages.'],
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
    super.key,
  });
  final Future<void> Function(Uint8List login, Uint8List password) onSubmit;
  final VoidCallback onCancel, onReload;
  final bool needsReload, busy;
  @override
  State<AccountSignIn> createState() => _AccountSignInState();
}

class _AccountSignInState extends State<AccountSignIn>
    with WidgetsBindingObserver {
  final login = TextEditingController(), password = TextEditingController();
  String? error;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  void clear() {
    login.clear();
    password.clear();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state != AppLifecycleState.resumed) clear();
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    clear();
    login.dispose();
    password.dispose();
    super.dispose();
  }

  Future<void> submit() async {
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
    setState(() => error = null);
    try {
      await widget.onSubmit(loginBytes, passwordBytes);
    } finally {
      loginBytes.fillRange(0, loginBytes.length, 0);
      passwordBytes.fillRange(0, passwordBytes.length, 0);
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
            label: label,
            child: ValueListenableBuilder<TextEditingValue>(
              valueListenable: controller,
              builder: (context, value, _) => TextField(
                key: ValueKey('credential:$label'),
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
      const Copy('Sign in to Meiju', size: 23, bold: true),
      const SizedBox(height: 12),
      const Copy(
        'Sign-in connects directly to the admitted mainland provider and automatically retrieves devices. Saved authorization uses the system secure store.',
        color: muted,
      ),
      const SizedBox(height: 16),
      field('Meiju account', login),
      field('Password', password, secret: true),
      if (error != null) Copy(error!, color: const Color(0xff9e3421)),
      if (widget.busy)
        const Copy('Connecting and retrieving devices…')
      else if (widget.needsReload) ...[
        const Copy(
          'Reload saved accounts before starting another connection. An interrupted sign-in may have saved authorization.',
        ),
        ActionButton('Reload Accounts', onPressed: widget.onReload),
      ] else
        ActionButton('Sign In and Retrieve Devices', onPressed: submit),
      ActionButton('Cancel', style: 'secondary', onPressed: widget.onCancel),
    ],
  );
}
