import 'package:flutter/material.dart';
import 'localization.dart';
import 'skin.dart';

/// A failed foreground read does not erase the last device observations.
class RefreshFeedback extends StatelessWidget {
  const RefreshFeedback({
    required this.onRetry,
    required this.onDismiss,
    super.key,
  });
  final VoidCallback? onRetry;
  final VoidCallback onDismiss;

  @override
  Widget build(BuildContext context) => Semantics(
    liveRegion: true,
    child: Padding(
      padding: const EdgeInsets.only(bottom: 16),
      child: Surface(
        radius: 20,
        padding: const EdgeInsets.all(16),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Icon(Icons.cloud_off_outlined, color: skinAccent(context)),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Row(
                    children: [
                      const Expanded(
                        child: Copy(
                          'Some devices could not update',
                          size: 16,
                          bold: true,
                        ),
                      ),
                      IconButton(
                        tooltip: localize(context, 'Dismiss'),
                        onPressed: onDismiss,
                        icon: const Icon(Icons.close_rounded, size: 20),
                        constraints: const BoxConstraints(
                          minWidth: 40,
                          minHeight: 40,
                        ),
                        padding: EdgeInsets.zero,
                      ),
                    ],
                  ),
                  const Copy(
                    'Your last readings are kept. Check your connection and try again.',
                    color: muted,
                  ),
                  Align(
                    alignment: Alignment.centerRight,
                    child: TextButton(
                      onPressed: onRetry,
                      child: Copy(
                        'Retry',
                        bold: true,
                        color: skinAccent(context),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    ),
  );
}
