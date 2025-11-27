import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rusty_dlna/api/cast.dart';
import 'package:rusty_dlna/frb_generated.dart';

void main() {
  // 注意：真实设备扫描需要在真机环境运行，模拟器网络通常是隔离的
  // 单元测试中通常 mock Rust 返回，或者在集成测试中跑

  test('Projector Control Flow', () async {
    // 初始化 Rust (FRB v2 自动处理，v1 需要手动 init)
    await RustLib.init();

    debugPrint("Step 1: 扫描设备...");
    // 扫描 3 秒
    final devices = await scanProjectors(timeoutSecs: BigInt.from(3));

    if (devices.isEmpty) {
      debugPrint("未发现设备，请检查网络。");
      return;
    }

    final projector = devices.first;
    debugPrint("发现设备: ${projector.friendlyName} (${projector.ip})");
    debugPrint("控制地址: ${projector.avTransportUrl}");

    // 必须有控制 URL 才能操作
    if (projector.avTransportUrl == null) {
      debugPrint("该设备不支持 AVTransport 协议");
      return;
    }

    final avUrl = projector.avTransportUrl!;
    final rcUrl = projector.renderingControlUrl!;

    // --- 播放测试 ---
    debugPrint("Step 2: 投送视频...");
    // 确保这个 URL 局域网可访问，或者公网可访问
    const videoUrl =
        "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";

    try {
      await castVideo(controlUrl: avUrl, videoUrl: videoUrl);
      debugPrint("投送指令已发送");
    } catch (e) {
      debugPrint("投送失败: $e");
    }

    // 等待一会看效果
    await Future.delayed(const Duration(seconds: 10));

    // --- 暂停 ---
    debugPrint("Step 3: 暂停");
    await pauseVideo(controlUrl: avUrl);

    await Future.delayed(const Duration(seconds: 3));

    // --- 继续播放 ---
    debugPrint("Step 4: 继续播放");
    await playVideo(controlUrl: avUrl);

    // --- 调节音量 ---
    if (projector.renderingControlUrl != null) {
      debugPrint("Step 5: 设置音量为 20");
      await setVolume(controlUrl: rcUrl, volume: 20);

      final currentVol = await getVolume(controlUrl: rcUrl);
      debugPrint("当前音量: $currentVol");
    }

    // --- 快进 (Seek) ---
    debugPrint("Step 6: 快进到 00:05:00");
    await seekVideo(controlUrl: avUrl, targetTime: "00:05:00");

    // --- 停止 ---
    debugPrint("Step 7: 停止");
    await stopVideo(controlUrl: avUrl);
  });

  test('Remote Power On (WoL)', () async {
    // 这里的 MAC 地址需要你之前通过其他方式获取并存储在本地
    // DLNA 扫描不一定返回 MAC，通常需要读取 ARP 表或用户手动输入
    const targetMac = "AA:BB:CC:DD:EE:FF";

    debugPrint("尝试唤醒设备: $targetMac");
    try {
      await wakeOnLan(macAddress: targetMac);
      debugPrint("魔术包发送成功");
    } catch (e) {
      debugPrint("唤醒失败: $e");
    }
  });
}
