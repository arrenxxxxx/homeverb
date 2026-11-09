import 'package:flutter/material.dart';

import 'skin.dart';

/// Persistent actionable information, never a selectable setting or whole-row link.
class FeedbackNotice extends StatelessWidget {
  const FeedbackNotice({
    required this.message,
    this.title,
    this.actionLabel,
    this.onAction,
    this.icon = Icons.info_outline_rounded,
    this.warning = false,
    super.key,
  });
  final Object message;
  final Object? title;
  final String? actionLabel;
  final VoidCallback? onAction;
  final IconData icon;
  final bool warning;

  @override
  Widget build(BuildContext context) {
    final contrast = MediaQuery.highContrastOf(context);
    final color = contrast
        ? Colors.black
        : warning
        ? const Color(0xff986218)
        : const Color(0xff306887);
    final inlineAction =
        title != null &&
        MediaQuery.sizeOf(context).width >= 360 &&
        MediaQuery.textScalerOf(context).scale(14) < 20;
    final action = actionLabel == null
        ? null
        : OutlinedButton(
            onPressed: onAction,
            style: OutlinedButton.styleFrom(
              foregroundColor: color,
              minimumSize: const Size(48, 48),
              padding: const EdgeInsets.symmetric(horizontal: 10),
              side: BorderSide(color: color.withValues(alpha: .45)),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(8),
              ),
            ),
            child: Copy(actionLabel!, size: 13, color: color, bold: true),
          );
    return Semantics(
      liveRegion: true,
      child: Container(
        key: const ValueKey('feedback-notice'),
        margin: const EdgeInsets.only(bottom: 12),
        padding: const EdgeInsets.fromLTRB(12, 12, 10, 10),
        decoration: BoxDecoration(
          color: contrast
              ? Colors.white
              : warning
              ? const Color(0xfffff8e9)
              : const Color(0xffedf5fa),
          border: Border(left: BorderSide(color: color, width: 3)),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Padding(
              padding: EdgeInsets.only(
                top: inlineAction && action != null ? 13.5 : 0,
              ),
              child: ExcludeSemantics(
                child: Icon(icon, color: color, size: 21),
              ),
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (title != null) ...[
                    Row(
                      children: [
                        Expanded(
                          child: Copy(
                            title!,
                            bold: true,
                            size: 14,
                            color: color,
                          ),
                        ),
                        if (inlineAction && action != null) ...[
                          const SizedBox(width: 8),
                          action,
                        ],
                      ],
                    ),
                    const SizedBox(height: 4),
                  ],
                  Copy(message, size: 13, color: ink, paragraph: true),
                  if (!inlineAction && action != null)
                    Align(alignment: Alignment.centerRight, child: action),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Dark non-interactive surface; only the separately delineated action is a button.
class FeedbackToast extends StatelessWidget {
  const FeedbackToast({
    required this.message,
    this.actionLabel,
    this.onAction,
    this.icon = Icons.info_outline_rounded,
    this.iconColor = const Color(0xffffd586),
    super.key,
  });
  final Object message;
  final String? actionLabel;
  final VoidCallback? onAction;
  final IconData icon;
  final Color iconColor;

  @override
  Widget build(BuildContext context) => Semantics(
    liveRegion: true,
    child: Row(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        ExcludeSemantics(child: Icon(icon, color: iconColor, size: 22)),
        const SizedBox(width: 12),
        Flexible(child: Copy(message, color: Colors.white, size: 14)),
        if (actionLabel != null) ...[
          const SizedBox(width: 12),
          Container(width: 1, height: 30, color: const Color(0xff687383)),
          const SizedBox(width: 4),
          ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 120),
            child: TextButton(
              onPressed: onAction,
              style: TextButton.styleFrom(
                minimumSize: const Size(48, 48),
                foregroundColor: const Color(0xffb8e2ff),
              ),
              child: Copy(
                actionLabel!,
                color: const Color(0xffb8e2ff),
                bold: true,
              ),
            ),
          ),
        ],
      ],
    ),
  );
}

/// A small deduplicated queue scoped to the current foreground page.
/// Call clear on navigation/background; callbacks capture the caller's epoch.
class FeedbackQueue {
  final Set<String> _pending = {};
  ScaffoldMessengerState? _messenger;
  int _generation = 0;

  void show(
    BuildContext context, {
    required String key,
    required Widget content,
    bool hasAction = false,
    bool aboveNavigation = false,
  }) {
    if (_pending.contains(key) || _pending.length >= 3) return;
    final messenger = ScaffoldMessenger.of(context);
    _messenger = messenger;
    _pending.add(key);
    final generation = _generation;
    final features = MediaQuery.of(context);
    final navigationInset = aboveNavigation && features.size.width < 800
        ? 74 +
              (features.textScaler.scale(11) - 11).clamp(0, double.infinity) *
                  1.4 +
              features.padding.bottom
        : 16.0;
    messenger
        .showSnackBar(
          SnackBar(
            backgroundColor: Colors.transparent,
            elevation: 0,
            behavior: SnackBarBehavior.floating,
            margin: EdgeInsets.fromLTRB(24, 8, 24, navigationInset),
            padding: EdgeInsets.zero,
            duration: Duration(seconds: features.accessibleNavigation ? 30 : 8),
            // Keep actionable feedback available until explicitly dismissed by a
            // screen-reader user; do not race the spoken announcement.
            persist: hasAction && features.accessibleNavigation,
            content: LayoutBuilder(
              builder: (context, constraints) => UnconstrainedBox(
                constrainedAxis: Axis.vertical,
                child: ConstrainedBox(
                  constraints: BoxConstraints(maxWidth: constraints.maxWidth),
                  child: Container(
                    key: const ValueKey('feedback-toast-surface'),
                    padding: const EdgeInsets.symmetric(
                      horizontal: 14,
                      vertical: 10,
                    ),
                    decoration: BoxDecoration(
                      color: const Color(0xff263241),
                      borderRadius: BorderRadius.circular(12),
                      boxShadow: const [
                        BoxShadow(
                          color: Color(0x33000000),
                          blurRadius: 10,
                          offset: Offset(0, 4),
                        ),
                      ],
                    ),
                    child: content,
                  ),
                ),
              ),
            ),
          ),
          snackBarAnimationStyle: features.disableAnimations
              ? AnimationStyle.noAnimation
              : null,
        )
        .closed
        .then((_) {
          if (_generation == generation) _pending.remove(key);
        });
  }

  void clear() {
    _generation++;
    _pending.clear();
    if (_messenger?.mounted ?? false) {
      _messenger!.clearSnackBars();
      _messenger!.removeCurrentSnackBar();
    }
    _messenger = null;
  }
}
