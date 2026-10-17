import 'client_contract.dart';

/// Local card presentation, never an authority or a device command.
class DevicePresentation {
  const DevicePresentation({
    required this.artwork,
    required this.power,
    required this.showPower,
    required this.powerEnabled,
    required this.updating,
  });
  final String artwork;
  final bool? power;
  final bool showPower, powerEnabled, updating;
}

String deviceArtwork(Json device) {
  final components = (device['components'] as List? ?? []).cast<Json>();
  final ids = components.map((part) => part['id']).toSet();
  final capabilities = {
    for (final part in components)
      for (final capability in part['capabilities'] as List? ?? [])
        capability.toString().split('.').last,
  };
  // Namespaced family names select decorative art only, never control behavior.
  if (ids.contains('fridge') || ids.contains('freezer')) return 'refrigerator';
  if (capabilities.contains('dishwasher')) return 'dishwasher';
  if (capabilities.contains('climate') && components.length == 1) return 'ac';
  if (capabilities.contains('light')) return 'lamp';
  if (capabilities.contains('media')) return 'tv';
  if (capabilities.contains('fan')) return 'fan';
  if (capabilities.contains('sensor')) return 'sensor';
  return 'appliance';
}
