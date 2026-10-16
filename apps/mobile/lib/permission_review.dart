import 'package:flutter/material.dart';
import 'native_client.dart';
import 'session_permissions.dart';
import 'skin.dart';

class PermissionReview extends StatefulWidget {
  const PermissionReview({
    required this.inspection,
    required this.deviceName,
    required this.busy,
    required this.onConfirm,
    super.key,
  });
  final Json inspection;
  final String deviceName;
  final bool busy;
  final Future<bool> Function(Set<String>) onConfirm;
  @override
  State<PermissionReview> createState() => _PermissionReviewState();
}

class _PermissionReviewState extends State<PermissionReview> {
  final selected = <String>{};
  bool reviewing = false;
  @override
  Widget build(BuildContext context) {
    final choices = SessionPermission.choices(widget.inspection);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Copy('Allow controls for this session', size: 23, bold: true),
        Copy(widget.deviceName, size: 18, bold: true),
        const SizedBox(height: 12),
        const Copy(
          'Choose exact values for this device over the cloud. These experimental controls can affect the real appliance. Mode activation also turns it on. Permission lasts until the app process closes or you revoke it.',
        ),
        const SizedBox(height: 12),
        if (choices.isEmpty)
          const Copy(
            'No reviewed controls can be enabled for this connection.',
          ),
        if (!reviewing)
          for (final choice in choices)
            CheckboxListTile(
              key: ValueKey('permission:${choice.identity}'),
              contentPadding: EdgeInsets.zero,
              title: Copy(permissionLabel(choice)),
              value: selected.contains(choice.identity),
              onChanged: widget.busy
                  ? null
                  : (value) => setState(() {
                      if (value == true) {
                        selected.add(choice.identity);
                      } else {
                        selected.remove(choice.identity);
                      }
                    }),
            )
        else ...[
          const Copy('Only these selected values will be allowed:', bold: true),
          for (final choice in choices.where(
            (v) => selected.contains(v.identity),
          ))
            Copy(permissionLabel(choice)),
          const SizedBox(height: 12),
          const Copy(
            'This changes local permission only. After confirming, explicitly retrieve the account devices to publish enabled controls. Nothing is sent to the appliance now.',
          ),
        ],
        const SizedBox(height: 12),
        if (!reviewing)
          ActionButton(
            'Review Selected Permissions',
            onPressed: selected.isEmpty || widget.busy
                ? null
                : () => setState(() => reviewing = true),
          )
        else ...[
          ActionButton(
            'Allow Selected Controls',
            onPressed: widget.busy
                ? null
                : () async {
                    await widget.onConfirm(Set.unmodifiable(selected));
                  },
          ),
          ActionButton(
            'Change Selection',
            style: 'secondary',
            onPressed: widget.busy
                ? null
                : () => setState(() => reviewing = false),
          ),
        ],
      ],
    );
  }
}

String permissionLabel(SessionPermission choice) =>
    switch ((choice.capability, choice.action)) {
      ('power', 'set') => choice.value == true ? 'Power on' : 'Power off',
      ('climate', 'set') => 'Target temperature: ${choice.value}°C',
      ('vendor.midea.ac', 'activate_mode') =>
        'Turn on and select mode: ${choice.value}',
      ('vendor.midea.ac', 'set_fan_speed') => 'Fan speed: ${choice.value}',
      _ => '${choice.capability}.${choice.action}: ${choice.value}',
    };
