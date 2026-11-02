import 'package:flutter/material.dart';
import 'feedback.dart';

/// A failed explicit read keeps observations and offers a read-only retry.
class RefreshFeedback extends StatelessWidget {
  const RefreshFeedback({
    required this.onRetry,
    required this.onDismiss,
    super.key,
  });
  final VoidCallback? onRetry;
  final VoidCallback onDismiss;

  @override
  Widget build(BuildContext context) => FeedbackToast(
    message: 'Could not refresh. Your last readings are kept.',
    actionLabel: 'Retry',
    icon: Icons.cloud_off_outlined,
    onAction: onRetry == null
        ? null
        : () {
            onDismiss();
            onRetry!();
          },
  );
}
