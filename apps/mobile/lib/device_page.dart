import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'blocks.dart';
import 'feedback.dart';
import 'localization.dart';
import 'skin.dart';
import 'device_typography.dart';

/// The real device-detail shell shared by capability-specific appliance panels.
/// Connectivity is not inferred from a cached reading or a successful render.
class DevicePage extends StatelessWidget {
  const DevicePage({
    required this.title,
    required this.child,
    required this.onAction,
    this.busy = false,
    this.error,
    super.key,
  });
  final Object title;
  final Widget child;
  final ActionHandler onAction;
  final bool busy;
  final Object? error;

  @override
  Widget build(BuildContext context) => CallbackShortcuts(
    bindings: {
      const SingleActivator(LogicalKeyboardKey.escape): () => onAction('Back'),
    },
    child: FocusScope(
      autofocus: true,
      child: Material(
        color: Colors.transparent,
        child: LightBackground(
          device: true,
          child: SafeArea(
            child: Column(
              children: [
                Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 8,
                    vertical: 4,
                  ),
                  child: Row(
                    children: [
                      IconButton(
                        tooltip: localize(context, 'Back'),
                        onPressed: () => onAction('Back'),
                        icon: Icon(
                          Icons.chevron_left,
                          size: 28,
                          color: (MediaQuery.highContrastOf(context)
                              ? ink
                              : skinOf(context).onBackground),
                        ),
                      ),
                      Expanded(
                        child: Semantics(
                          header: true,
                          namesRoute: true,
                          child: Copy(
                            title,
                            color: (MediaQuery.highContrastOf(context)
                                ? ink
                                : skinOf(context).onBackground),
                            size: DeviceTypography.title,
                            lineHeight: DeviceTypography.lineHeight,
                            bold: true,
                            center: true,
                          ),
                        ),
                      ),
                      const SizedBox(width: 48),
                    ],
                  ),
                ),
                Expanded(
                  child: SingleChildScrollView(
                    padding: const EdgeInsets.fromLTRB(16, 4, 16, 20),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        if (busy)
                          const Padding(
                            padding: EdgeInsets.only(bottom: 10),
                            child: OperationProgress(),
                          ),
                        if (error != null)
                          FeedbackNotice(
                            title: 'Could not complete this operation',
                            message: LiteralText(
                              friendlyError(context, error!),
                            ),
                            warning: true,
                            actionLabel: 'Dismiss',
                            onAction: () => onAction('Dismiss error'),
                          ),
                        child,
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    ),
  );
}
