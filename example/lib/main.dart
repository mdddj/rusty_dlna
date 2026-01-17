import 'dart:io';

import 'package:flutter/material.dart';
import 'package:network_info_plus/network_info_plus.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:rusty_dlna/api/cast.dart';
import 'package:rusty_dlna/frb_generated.dart';

Future<void> main() async {
  // 1. 初始化 Rust 库 (FRB v2)
  await RustLib.init();
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Lan Cast Demo',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.deepPurple),
        useMaterial3: true,
      ),
      home: const HomePage(),
    );
  }
}

class HomePage extends StatefulWidget {
  const HomePage({super.key});

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> {
  // 存储扫描到的设备列表
  List<ProjectorInfo> _devices = [];
  bool _isScanning = false;

  // 用于测试 WoL 的 MAC 输入框
  final TextEditingController _macController = TextEditingController();

  // 扫描设备的方法
  Future<void> _handleScan() async {
    // iOS: 触发本地网络权限对话框
    if (Platform.isIOS) {
      final granted = await _requestLocalNetworkPermission();
      if (!granted) {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
              content: Text('需要本地网络权限才能扫描设备，请在设置中允许'),
              duration: Duration(seconds: 5),
            ),
          );
        }
        return;
      }
    }

    setState(() => _isScanning = true);
    _devices.clear(); // 清空旧列表

    try {
      // 调用 Rust: 扫描 13 秒
      final results = await scanProjectors(timeoutSecs: BigInt.from(3));
      setState(() {
        _devices = results;
      });

      if (results.isEmpty && mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(const SnackBar(content: Text('未发现设备，请检查是否在同一 WiFi')));
      }
    } catch (e) {
      debugPrint('扫描出错: $e');
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('扫描出错: $e')));
      }
    } finally {
      setState(() => _isScanning = false);
    }
  }

  // 请求 iOS 本地网络权限
  Future<bool> _requestLocalNetworkPermission() async {
    try {
      // 方法1: 尝试访问 WiFi 信息，这会触发系统权限对话框
      final info = NetworkInfo();
      final wifiName = await info.getWifiName();
      debugPrint('WiFi Name: $wifiName');

      // 方法2: 检查位置权限（某些情况下需要）
      if (Platform.isIOS) {
        final status = await Permission.locationWhenInUse.status;
        if (!status.isGranted) {
          final result = await Permission.locationWhenInUse.request();
          if (!result.isGranted) {
            return false;
          }
        }
      }

      return true;
    } catch (e) {
      debugPrint('权限请求失败: $e');
      return false;
    }
  }

  // 远程开机测试
  Future<void> _handleWol() async {
    final mac = _macController.text.trim();
    if (mac.isEmpty) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('请输入 MAC 地址')));
      return;
    }
    try {
      await wakeOnLan(macAddress: mac);
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('魔术包已发送至 $mac')));
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('WoL 失败: $e')));
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('投影仪扫描器')),
      body: Column(
        children: [
          // --- 顶部操作区 (WoL) ---
          Padding(
            padding: const EdgeInsets.all(8.0),
            child: Card(
              child: Padding(
                padding: const EdgeInsets.all(8.0),
                child: Row(
                  children: [
                    Expanded(
                      child: TextField(
                        controller: _macController,
                        decoration: const InputDecoration(
                          labelText: '测试远程开机 (MAC地址)',
                          hintText: '例如 AA:BB:CC:11:22:33',
                          border: OutlineInputBorder(),
                        ),
                      ),
                    ),
                    const SizedBox(width: 8),
                    ElevatedButton(
                      onPressed: _handleWol,
                      child: const Text('唤醒'),
                    ),
                  ],
                ),
              ),
            ),
          ),

          // --- 扫描进度条 ---
          if (_isScanning) const LinearProgressIndicator(),

          // --- 设备列表 ---
          Expanded(
            child: _devices.isEmpty
                ? const Center(child: Text("点击右下角按钮开始扫描"))
                : ListView.builder(
                    itemCount: _devices.length,
                    itemBuilder: (context, index) {
                      final device = _devices[index];
                      debugPrint("info:$device");
                      return ListTile(
                        leading: const Icon(Icons.tv, size: 32),
                        title: Text(device.friendlyName),
                        subtitle: Text(device.ip),
                        trailing: const Icon(Icons.arrow_forward_ios),
                        onTap: () {
                          // 点击跳转到控制页面
                          if (device.avTransportUrl == null) {
                            ScaffoldMessenger.of(context).showSnackBar(
                              const SnackBar(
                                content: Text('该设备不支持 AVTransport，无法控制'),
                              ),
                            );
                            return;
                          }
                          Navigator.of(context).push(
                            MaterialPageRoute(
                              builder: (_) => ControlPage(device: device),
                            ),
                          );
                        },
                      );
                    },
                  ),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _isScanning ? null : _handleScan,
        child: _isScanning
            ? const SizedBox(
                width: 24,
                height: 24,
                child: CircularProgressIndicator(color: Colors.white),
              )
            : const Icon(Icons.search),
      ),
    );
  }
}

// --- 控制面板页面 ---

class ControlPage extends StatefulWidget {
  final ProjectorInfo device;

  const ControlPage({super.key, required this.device});

  @override
  State<ControlPage> createState() => _ControlPageState();
}

class _ControlPageState extends State<ControlPage> {
  // 默认测试视频
  final TextEditingController _urlController = TextEditingController(
    text:
        "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4",
  );

  String _statusLog = "准备就绪";

  void _log(String msg) {
    setState(() => _statusLog = msg);
    debugPrint(msg);
  }

  // 通用的错误处理包装器
  Future<void> _performAction(
    String name,
    Future<void> Function() action,
  ) async {
    try {
      _log("正在执行: $name...");
      await action();
      _log("$name 成功");
    } catch (e) {
      _log("$name 失败: $e");
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Error: $e')));
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final hasRenderingControl = widget.device.renderingControlUrl != null;

    return Scaffold(
      appBar: AppBar(title: Text(widget.device.friendlyName)),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // 1. 视频源输入
            TextField(
              controller: _urlController,
              decoration: const InputDecoration(
                labelText: '视频 URL',
                border: OutlineInputBorder(),
              ),
              maxLines: 2,
            ),
            const SizedBox(height: 16),

            // 2. 投屏按钮
            ElevatedButton.icon(
              icon: const Icon(Icons.cast),
              label: const Text("开始投屏 (Cast)"),
              style: ElevatedButton.styleFrom(
                backgroundColor: Colors.deepPurple,
                foregroundColor: Colors.white,
                padding: const EdgeInsets.symmetric(vertical: 16),
              ),
              onPressed: () => _performAction("投屏", () async {
                await widget.device.castVideo(videoUrl: _urlController.text);
              }),
            ),

            const SizedBox(height: 24),
            const Text("播放控制", style: TextStyle(fontWeight: FontWeight.bold)),
            const Divider(),

            // 3. 播放控制按钮组
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                IconButton.filledTonal(
                  icon: const Icon(Icons.play_arrow),
                  tooltip: "播放",
                  onPressed: () =>
                      _performAction("播放", () => widget.device.play()),
                ),
                IconButton.filledTonal(
                  icon: const Icon(Icons.pause),
                  tooltip: "暂停",
                  onPressed: () =>
                      _performAction("暂停", () => widget.device.pause()),
                ),
                IconButton.filledTonal(
                  icon: const Icon(Icons.stop),
                  color: Colors.red,
                  tooltip: "停止",
                  onPressed: () =>
                      _performAction("停止", () => widget.device.stop()),
                ),
              ],
            ),

            const SizedBox(height: 16),

            // 4. 进度跳转 (Seek)
            Row(
              children: [
                const Text("跳转: "),
                Expanded(
                  child: Wrap(
                    spacing: 8,
                    alignment: WrapAlignment.center,
                    children: [
                      OutlinedButton(
                        child: const Text("00:00"),
                        onPressed: () => _performAction(
                          "跳转到开头",
                          () => widget.device.seek(targetTime: "00:00:00"),
                        ),
                      ),
                      OutlinedButton(
                        child: const Text("05:00"),
                        onPressed: () => _performAction(
                          "跳转到 5分",
                          () => widget.device.seek(targetTime: "00:05:00"),
                        ),
                      ),
                      OutlinedButton(
                        child: const Text("10:00"),
                        onPressed: () => _performAction(
                          "跳转到 10分",
                          () => widget.device.seek(targetTime: "00:10:00"),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),

            const SizedBox(height: 24),
            const Text("音量控制", style: TextStyle(fontWeight: FontWeight.bold)),
            const Divider(),

            // 5. 音量控制 (检查是否支持 RC 服务)
            if (!hasRenderingControl)
              const Center(child: Text("此设备不支持音量控制 (无 RenderingControl)"))
            else
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: [
                  IconButton(
                    icon: const Icon(Icons.volume_down),
                    onPressed: () => _performAction(
                      "音量 10",
                      () => widget.device.setVolume(volume: 10),
                    ),
                  ),
                  IconButton(
                    icon: const Icon(Icons.volume_up),
                    onPressed: () => _performAction(
                      "音量 30",
                      () => widget.device.setVolume(volume: 30),
                    ),
                  ),
                  OutlinedButton(
                    onPressed: () async {
                      try {
                        final vol = await widget.device.getVolume();
                        _log("当前音量: $vol");
                      } catch (e) {
                        _log("获取音量失败: $e");
                      }
                    },
                    child: const Text("获取当前音量"),
                  ),
                ],
              ),

            const SizedBox(height: 24),
            Container(
              padding: const EdgeInsets.all(8),
              color: Colors.grey[200],
              width: double.infinity,
              child: Text(
                "状态日志: $_statusLog",
                style: const TextStyle(fontFamily: "monospace"),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
