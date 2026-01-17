# rusty_dlna

A Flutter plugin for DLNA/UPnP device discovery and media casting, powered by Rust.

[![pub package](https://img.shields.io/pub/v/rusty_dlna.svg)](https://pub.dev/packages/rusty_dlna)

## Features

- 🔍 **Device Discovery** - Scan for DLNA/UPnP devices on your local network
- 📺 **Media Casting** - Cast video URLs to smart TVs and projectors
- ▶️ **Playback Control** - Play, pause, stop, and seek
- 🔊 **Volume Control** - Get/set volume and mute
- 📊 **Status Monitoring** - Get playback position and transport state
- 🌐 **Wake on LAN** - Remote wake devices by MAC address

## Supported Platforms

- Android
- iOS
- macOS
- Linux
- Windows

## Installation

Add to your `pubspec.yaml`:

```yaml
dependencies:
  rusty_dlna: ^0.0.1
```

## Setup

### Initialize the library

```dart
import 'package:rusty_dlna/api/cast.dart';
import 'package:rusty_dlna/frb_generated.dart';

void main() async {
  await RustLib.init();
  runApp(MyApp());
}
```

### Platform-specific configuration

#### Android

Add network permissions to `android/app/src/main/AndroidManifest.xml`:

```xml
<uses-permission android:name="android.permission.INTERNET"/>
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE"/>
<uses-permission android:name="android.permission.CHANGE_WIFI_MULTICAST_STATE"/>
```

#### macOS

Add network entitlements to `macos/Runner/DebugProfile.entitlements` and `macos/Runner/Release.entitlements`:

```xml
<key>com.apple.security.network.client</key>
<true/>
<key>com.apple.security.network.server</key>
<true/>
```

#### iOS

Add to `ios/Runner/Info.plist`:

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

**Important for iOS 14+**: The local network permission dialog will appear automatically when you first attempt to scan for devices. To ensure the permission is properly requested, you can use `network_info_plus` package to trigger the permission dialog before scanning:

```dart
import 'package:network_info_plus/network_info_plus.dart';

// Trigger permission dialog by accessing WiFi info
final info = NetworkInfo();
await info.getWifiName(); // This triggers the local network permission dialog

// Now you can scan for devices
final devices = await scanProjectors(timeoutSecs: BigInt.from(5));
```

## Usage

### Scan for devices

```dart
final devices = await scanProjectors(timeoutSecs: BigInt.from(5));

for (final device in devices) {
  print('Found: ${device.friendlyName} at ${device.ip}');
}
```

### Cast a video

```dart
final device = devices.first;
await device.castVideo(videoUrl: 'http://example.com/video.mp4');
```

### Playback control

```dart
await device.play();
await device.pause();
await device.stop();
await device.seek(targetTime: '00:05:00'); // HH:MM:SS format
```

### Volume control

```dart
await device.setVolume(volume: 50);
final currentVolume = await device.getVolume();
await device.setMute(mute: true);
```

### Get playback status

```dart
// Get position as formatted strings (HH:MM:SS)
final (current, total) = await device.getPositionInfo();
print('Progress: $current / $total');

// Get position in seconds
final (currentSec, totalSec) = await device.getPositionInfoSec();

// Get transport state
final state = await device.getTransportInfo();
// TransportState: Playing, Paused, Stopped, Transitioning, NoMedia, Unknown
```

### Wake on LAN

```dart
await wakeOnLan(macAddress: 'AA:BB:CC:11:22:33');
```

## API Reference

### ProjectorInfo

| Property | Type | Description |
|----------|------|-------------|
| `friendlyName` | `String` | Device display name |
| `ip` | `String` | Device IP address |
| `locationXmlUrl` | `String` | UPnP description URL |
| `avTransportUrl` | `String?` | AVTransport control URL |
| `renderingControlUrl` | `String?` | RenderingControl URL |

### ProjectorInfo Methods

| Method | Description |
|--------|-------------|
| `castVideo(videoUrl)` | Cast video and start playback |
| `play()` | Resume playback |
| `pause()` | Pause playback |
| `stop()` | Stop playback |
| `seek(targetTime)` | Seek to position (HH:MM:SS) |
| `setVolume(volume)` | Set volume (0-100) |
| `getVolume()` | Get current volume |
| `setMute(mute)` | Set mute state |
| `getPositionInfo()` | Get position as strings |
| `getPositionInfoSec()` | Get position in seconds |
| `getTransportInfo()` | Get playback state |

### Global Functions

| Function | Description |
|----------|-------------|
| `scanProjectors(timeoutSecs)` | Discover DLNA devices |
| `wakeOnLan(macAddress)` | Send Wake-on-LAN packet |

## Troubleshooting

### "No route to host" error (os error 65)

This error typically occurs when:

1. **Device not connected to network** - Ensure your device is connected to WiFi or Ethernet
2. **Missing network permissions** - Check platform-specific setup above
3. **Firewall blocking** - System firewall may block UDP multicast traffic
4. **VPN interference** - Try disabling VPN temporarily
5. **Different networks** - Device and DLNA devices must be on the same local network

**Solutions:**

- **iOS 14+**: The local network permission dialog must be triggered before scanning. See [iOS Setup Guide](IOS_SETUP.md) for detailed instructions. Quick fix:
  ```dart
  // Add network_info_plus to pubspec.yaml
  // Trigger permission dialog before scanning
  if (Platform.isIOS) {
    await NetworkInfo().getWifiName(); // Triggers permission dialog
  }
  final devices = await scanProjectors(timeoutSecs: BigInt.from(5));
  ```
- **Android**: Verify all permissions in AndroidManifest.xml are added
- **macOS**: Ensure network entitlements are properly configured
- **All platforms**: Try connecting to the same WiFi network as your DLNA device

### No devices found

- Increase scan timeout: `scanProjectors(timeoutSecs: BigInt.from(10))`
- Ensure DLNA device is powered on and connected to network
- Check if device supports UPnP/DLNA protocol
- Try scanning multiple times (some devices respond slowly)

### Cast fails but device is found

- Verify the video URL is accessible from the DLNA device's network
- Some devices only support specific video formats (MP4, MKV, etc.)
- Check if device's `avTransportUrl` is not null

## Example

See the [example](example/) directory for a complete demo app.

## License

MIT License - see [LICENSE](LICENSE) for details.
