# iOS Local Network Permission Setup

## Problem

On iOS 14+, apps need explicit user permission to access the local network for device discovery (SSDP/mDNS). Additionally, iOS 14.5+ has stricter enforcement of multicast permissions, which can cause SSDP discovery to fail even with proper Info.plist configuration.

Common errors:
```
AnyhowException(SSDP search failed: io error: No route to host (os error 65))
```

## Solution (v0.2.0+)

Starting from version 0.2.0, this plugin uses a custom SSDP implementation with `socket2` for better iOS compatibility, avoiding the issues with the `ssdp-client` crate on iOS.

### 1. Configure Info.plist

Add these keys to your `ios/Runner/Info.plist`:

```xml
<key>NSLocalNetworkUsageDescription</key>
<string>This app needs local network access to discover DLNA devices</string>

<key>NSBonjourServices</key>
<array>
    <string>_ssdp._udp</string>
</array>

<key>NSLocationWhenInUseUsageDescription</key>
<string>Location permission is required to access WiFi information</string>
```

### 2. Trigger Permission Dialog

The system permission dialog doesn't appear automatically. You need to trigger it by accessing network information before scanning:

#### Option A: Using network_info_plus (Recommended)

Add to `pubspec.yaml`:
```yaml
dependencies:
  network_info_plus: ^6.0.0
  permission_handler: ^11.3.1
```

In your code:
```dart
import 'dart:io';
import 'package:network_info_plus/network_info_plus.dart';
import 'package:permission_handler/permission_handler.dart';

Future<bool> requestLocalNetworkPermission() async {
  if (!Platform.isIOS) return true;
  
  try {
    // This triggers the local network permission dialog
    final info = NetworkInfo();
    await info.getWifiName();
    
    // Also request location permission (needed for WiFi info)
    final status = await Permission.locationWhenInUse.request();
    return status.isGranted;
  } catch (e) {
    print('Permission request failed: $e');
    return false;
  }
}

// Use before scanning
Future<void> scanDevices() async {
  if (Platform.isIOS) {
    final granted = await requestLocalNetworkPermission();
    if (!granted) {
      // Show error to user
      return;
    }
  }
  
  // Now safe to scan
  final devices = await scanProjectors(timeoutSecs: BigInt.from(5));
}
```

#### Option B: Let first scan trigger it

The permission dialog will appear on the first SSDP scan attempt, but the scan will fail. The user needs to:
1. Allow the permission
2. Scan again

This is less user-friendly but requires no additional packages.

### 3. Testing

1. Delete the app from device/simulator
2. Reinstall and run
3. Tap scan button
4. Permission dialog should appear
5. Tap "OK" to allow
6. Devices should be discovered

### 4. Troubleshooting

**Dialog doesn't appear:**
- Check Info.plist has all required keys
- Ensure `NSBonjourServices` includes `_ssdp._udp`
- Try accessing WiFi name first: `await NetworkInfo().getWifiName()`

**Still getting "No route to host":**
- Check device is connected to WiFi (not cellular)
- Verify app has permission in Settings > Privacy > Local Network
- Disable VPN if enabled
- Ensure device and DLNA devices are on same network

**Permission denied:**
- User can re-enable in: Settings > [Your App] > Local Network
- Or: Settings > Privacy & Security > Local Network > [Your App]

## Why This Happens

iOS 14 introduced privacy protections for local network access. Apps must:
1. Declare usage in Info.plist
2. Trigger permission dialog at runtime
3. Get user approval

The permission dialog only appears when the app actually attempts to use local network features (like accessing WiFi info or sending multicast packets).

## References

- [Apple Documentation: Local Network Privacy](https://developer.apple.com/videos/play/wwdc2020/10110/)
- [network_info_plus package](https://pub.dev/packages/network_info_plus)
- [permission_handler package](https://pub.dev/packages/permission_handler)
