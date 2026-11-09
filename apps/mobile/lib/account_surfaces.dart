import 'package:flutter/material.dart';

import 'account_views.dart';
import 'blocks.dart';
import 'controller.dart';
import 'feedback.dart';
import 'localization.dart';
import 'skin.dart';

/// Consumer-facing navigation, separate from saved-account management.
class PlatformPicker extends StatelessWidget {
  const PlatformPicker({
    required this.controller,
    required this.onAction,
    super.key,
  });
  final HomeController controller;
  final void Function(String) onAction;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      _AccountPanel(
        children: const [
          Copy('Choose a platform to connect', size: 22, bold: true),
          SizedBox(height: 10),
          Copy(
            'Choose the platform where your appliances are already added. Selecting a platform does not change saved accounts.',
            color: muted,
            paragraph: true,
          ),
        ],
      ),
      const SizedBox(height: 16),
      OutlinedButton(
        key: const ValueKey('platform:meiju'),
        onPressed: controller.busy ? null : () => onAction('Choose Meiju'),
        style: OutlinedButton.styleFrom(
          foregroundColor: ink,
          backgroundColor: MediaQuery.highContrastOf(context)
              ? Colors.white
              : skinOf(context).cardColor,
          padding: const EdgeInsets.all(22),
          side: BorderSide(
            color: MediaQuery.highContrastOf(context)
                ? Colors.black
                : skinAccent(context).withValues(alpha: .2),
          ),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(25),
          ),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                const _AccountIcon(Icons.home_outlined),
                const SizedBox(width: 14),
                const Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Copy('Meiju', size: 22, bold: true),
                      SizedBox(height: 3),
                      Copy('Mainland China', size: 12, color: muted),
                    ],
                  ),
                ),
                const Icon(Icons.arrow_forward_rounded, size: 24),
              ],
            ),
            const SizedBox(height: 18),
            const Copy(
              'Sign in with your existing platform account',
              size: 16,
              bold: true,
            ),
            const SizedBox(height: 7),
            const Copy('Supported devices vary by model.', color: muted),
          ],
        ),
      ),
      const SizedBox(height: 16),
      _AccountPanel(
        children: const [
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(Icons.shield_outlined, color: muted, size: 20),
              SizedBox(width: 12),
              Expanded(
                child: Copy(
                  'Only supported connection methods are shown. Browser authorization and verification codes are unavailable.',
                  size: 12,
                  color: muted,
                  paragraph: true,
                ),
              ),
            ],
          ),
        ],
      ),
      const SizedBox(height: 16),
      ActionButton(
        'View linked accounts',
        style: 'secondary',
        onPressed: controller.busy ? null : () => onAction('Back to Accounts'),
      ),
    ],
  );
}

class AccountDetails extends StatelessWidget {
  const AccountDetails({
    required this.controller,
    required this.onAction,
    this.refreshing = false,
    super.key,
  });
  final HomeController controller;
  final void Function(String) onAction;
  final bool refreshing;
  Widget _blocks(List<dynamic> values) =>
      Blocks(values, onAction: onAction, fields: {}, choices: {}, values: {});

  @override
  Widget build(BuildContext context) {
    final c = controller, account = controller.account;
    if (account == null) return _blocks(accountBlocks(c, 'P09'));
    if (c.accountDetails == null) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Copy(accountDisplayName(c, account), size: 24, bold: true),
          Padding(
            padding: const EdgeInsets.all(32),
            child: Center(
              child: c.busy
                  ? const CircularProgressIndicator()
                  : ActionButton(
                      'Retry',
                      onPressed: () => onAction('Retry account details'),
                    ),
            ),
          ),
        ],
      );
    }
    final stored = account['credential_state'] == 'stored';
    final report = c.accountReport?['account']['id'] == c.selectedAccount
        ? c.accountReport
        : null;
    final original = accountBlocks(c, 'P09', refreshing: refreshing);
    final needsSignIn = accountRequiresSignIn(c);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        _AccountPanel(
          children: [
            Row(
              children: [
                const _AccountIcon(Icons.person_outline_rounded),
                const SizedBox(width: 14),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Copy(
                        accountDisplayName(c, account),
                        size: 24,
                        bold: true,
                      ),
                      const SizedBox(height: 4),
                      const Copy(
                        'Meiju · Mainland China',
                        size: 12,
                        color: muted,
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 18),
            Copy(
              stored
                  ? 'Saved on this device'
                  : credentialLabel(account['credential_state']),
              color: skinAccent(context),
            ),
            const SizedBox(height: 8),
            const Copy(
              'Saved locally does not mean currently online or still authorized.',
              size: 12,
              color: muted,
              paragraph: true,
            ),
          ],
        ),
        const SizedBox(height: 16),
        _AccountPanel(
          children: [
            const Copy('Device synchronization', size: 18, bold: true),
            const SizedBox(height: 10),
            if (stored && report != null)
              if (needsSignIn)
                FeedbackNotice(
                  title: 'Sign in again',
                  message:
                      'Your authorization has expired. Sign in again to update devices. Your saved devices are kept.',
                  warning: true,
                  icon: Icons.lock_outline_rounded,
                  actionLabel: c.canSignIn ? 'Go to sign in' : null,
                  onAction: c.busy || !c.canSignIn
                      ? null
                      : () => onAction('Reauthorize'),
                )
              else if (report['retrieval']['status'] != 'complete')
                const Copy(
                  'Your saved devices are kept. Check your connection and refresh when ready.',
                  color: muted,
                  paragraph: true,
                )
              else
                _blocks(accountRefreshBlocks(c))
            else if (stored)
              const Copy(
                'No synchronization result checked on this page yet. Refresh the platform device list when needed.',
                color: muted,
                paragraph: true,
              ),
            if (stored && !needsSignIn && c.supports('refresh_account')) ...[
              const SizedBox(height: 16),
              ActionButton(
                'Refresh Devices',
                style: 'secondary',
                loading: refreshing,
                onPressed: c.busy ? null : () => onAction('Refresh Devices'),
              ),
            ],
            if (!stored)
              _blocks(
                original
                    .where(
                      (b) =>
                          b[0] == 'button' ||
                          b[0] == 'note' ||
                          b[0] == 'status',
                    )
                    .toList(),
              ),
          ],
        ),
        const SizedBox(height: 16),
        _AccountPanel(
          children: [
            const Copy('Devices', size: 18, bold: true),
            const SizedBox(height: 4),
            Copy(
              UiMessage('{count} devices associated locally', {
                'count': c.accountDeviceIds.length,
              }),
              size: 12,
              color: muted,
            ),
            if (c.accountDeviceIds.isEmpty) ...[
              const SizedBox(height: 12),
              const Copy(
                'No devices yet. Update the list after adding appliances in Meiju.',
                color: muted,
                paragraph: true,
              ),
            ],
            for (final id in c.accountDeviceIds)
              Padding(
                padding: const EdgeInsets.only(top: 14),
                child: TextButton(
                  key: ValueKey('account-device:$id'),
                  style: TextButton.styleFrom(
                    padding: const EdgeInsets.symmetric(vertical: 12),
                    foregroundColor: ink,
                  ),
                  onPressed:
                      !c.busy &&
                          c.cards.any((card) => card['device']['id'] == id)
                      ? () => onAction('open:$id')
                      : null,
                  child: Row(
                    children: [
                      Icon(
                        Icons.devices_other_rounded,
                        size: 24,
                        color: skinAccent(context),
                      ),
                      const SizedBox(width: 12),
                      Expanded(
                        child: Copy(savedCardName(c, id), size: 16, bold: true),
                      ),
                      const SizedBox(width: 8),
                      const Icon(Icons.chevron_right_rounded, size: 22),
                    ],
                  ),
                ),
              ),
          ],
        ),
        const SizedBox(height: 16),
        // Keep the existing management and troubleshooting contracts, including
        // explicit removal and identity-recovery confirmation, unchanged.
        _blocks(original.where((b) => b[0] == 'disclosure').toList()),
        if (stored && c.accountNeedsReload)
          ActionButton(
            'Reload Accounts',
            style: 'secondary',
            onPressed: c.busy ? null : () => onAction('Reload Accounts'),
          ),
      ],
    );
  }
}

class _AccountPanel extends StatelessWidget {
  const _AccountPanel({required this.children});
  final List<Widget> children;
  @override
  Widget build(BuildContext context) => Container(
    decoration: BoxDecoration(
      color: MediaQuery.highContrastOf(context)
          ? Colors.white
          : skinOf(context).cardColor,
      borderRadius: BorderRadius.circular(25),
      border: MediaQuery.highContrastOf(context)
          ? Border.all(color: Colors.black, width: 1.5)
          : null,
    ),
    padding: const EdgeInsets.all(20),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: children,
    ),
  );
}

class _AccountIcon extends StatelessWidget {
  const _AccountIcon(this.icon);
  final IconData icon;
  @override
  Widget build(BuildContext context) => Container(
    width: 48,
    height: 48,
    decoration: BoxDecoration(
      color: MediaQuery.highContrastOf(context)
          ? Colors.white
          : skinOf(context).secondaryColor,
      borderRadius: BorderRadius.circular(16),
    ),
    child: Icon(icon, color: skinAccent(context), size: 28),
  );
}
