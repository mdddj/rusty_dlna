use anyhow::{Context, Result};
use flutter_rust_bridge::frb;
use reqwest::Client;
use socket2::{Domain, Protocol, Socket, Type};
use std::mem::MaybeUninit;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;

const AV_SERVICE: &str = "urn:schemas-upnp-org:service:AVTransport:1";
const RC_SERVICE: &str = "urn:schemas-upnp-org:service:RenderingControl:1";

// --- 数据模型 ---

#[frb(dart_metadata=("freezed"))]
#[derive(Debug, Clone)]
pub struct ProjectorInfo {
    pub friendly_name: String, // 设备名 (如 "客厅的投影仪")
    pub ip: String,
    pub location_xml_url: String, // 设备描述文件地址
    pub av_transport_url: Option<String>,
    pub rendering_control_url: Option<String>,
}

// --- ProjectorInfo 方法实现 ---
impl ProjectorInfo {
    // 获取 AV 控制 URL，如果不存在则返回错误
    fn get_av_url(&self) -> Result<&str> {
        self.av_transport_url
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("AVTransport URL not available for this device"))
    }

    // 获取 RenderingControl URL，如果不存在则返回错误
    fn get_rc_url(&self) -> Result<&str> {
        self.rendering_control_url
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("RenderingControl URL not available for this device"))
    }

    /// 投送视频 (SetAVTransportURI + Play)
    pub async fn cast_video(&self, video_url: String) -> Result<()> {
        let control_url = self.get_av_url()?;
        let args = format!(
            "<InstanceID>0</InstanceID><CurrentURI>{}</CurrentURI><CurrentURIMetaData></CurrentURIMetaData>",
            video_url
        );
        send_soap_action(control_url, AV_SERVICE, "SetAVTransportURI", &args).await?;
        self.play().await?;
        Ok(())
    }

    /// 播放
    pub async fn play(&self) -> Result<()> {
        let control_url = self.get_av_url()?;
        send_soap_action(
            control_url,
            AV_SERVICE,
            "Play",
            "<InstanceID>0</InstanceID><Speed>1</Speed>",
        )
        .await?;
        Ok(())
    }

    /// 暂停
    pub async fn pause(&self) -> Result<()> {
        let control_url = self.get_av_url()?;
        send_soap_action(
            control_url,
            AV_SERVICE,
            "Pause",
            "<InstanceID>0</InstanceID>",
        )
        .await?;
        Ok(())
    }

    /// 停止
    pub async fn stop(&self) -> Result<()> {
        let control_url = self.get_av_url()?;
        send_soap_action(
            control_url,
            AV_SERVICE,
            "Stop",
            "<InstanceID>0</InstanceID>",
        )
        .await?;
        Ok(())
    }

    /// 进度跳转 - target_time 格式为 HH:MM:SS
    pub async fn seek(&self, target_time: String) -> Result<()> {
        let control_url = self.get_av_url()?;
        let args = format!(
            "<InstanceID>0</InstanceID><Unit>REL_TIME</Unit><Target>{}</Target>",
            target_time
        );
        send_soap_action(control_url, AV_SERVICE, "Seek", &args).await?;
        Ok(())
    }

    /// 设置音量 (0-100)
    pub async fn set_volume(&self, volume: u8) -> Result<()> {
        let control_url = self.get_rc_url()?;
        let args = format!(
            "<InstanceID>0</InstanceID><Channel>Master</Channel><DesiredVolume>{}</DesiredVolume>",
            volume
        );
        send_soap_action(control_url, RC_SERVICE, "SetVolume", &args).await?;
        Ok(())
    }

    /// 获取当前音量
    pub async fn get_volume(&self) -> Result<u8> {
        let control_url = self.get_rc_url()?;
        let args = "<InstanceID>0</InstanceID><Channel>Master</Channel>";
        let xml = send_soap_action(control_url, RC_SERVICE, "GetVolume", args).await?;

        let regex = regex::Regex::new(
            r"(?i)<(?:.*:)?CurrentVolume[^>]*>(\s*\d+\s*)</(?:.*:)?CurrentVolume>",
        )
        .unwrap();

        if let Some(caps) = regex.captures(&xml) {
            let vol_str = caps.get(1).unwrap().as_str().trim();
            if let Ok(vol) = vol_str.parse::<u8>() {
                return Ok(vol);
            }
        }
        Ok(0)
    }

    /// 设置静音状态
    pub async fn set_mute(&self, mute: bool) -> Result<()> {
        let control_url = self.get_rc_url()?;
        let val = if mute { "1" } else { "0" };
        let args = format!(
            "<InstanceID>0</InstanceID><Channel>Master</Channel><DesiredMute>{}</DesiredMute>",
            val
        );
        send_soap_action(control_url, RC_SERVICE, "SetMute", &args).await?;
        Ok(())
    }

    /// 获取播放进度 (当前时长, 总时长) 格式 HH:MM:SS
    pub async fn get_position_info(&self) -> Result<(String, String)> {
        let control_url = self.get_av_url()?;
        let args = "<InstanceID>0</InstanceID>";
        let xml = send_soap_action(control_url, AV_SERVICE, "GetPositionInfo", args).await?;

        let cur_regex = regex::Regex::new(r"<RelTime>(.*?)</RelTime>").unwrap();
        let dur_regex = regex::Regex::new(r"<TrackDuration>(.*?)</TrackDuration>").unwrap();

        let current = cur_regex
            .captures(&xml)
            .map(|c| c.get(1).unwrap().as_str())
            .unwrap_or("00:00:00")
            .to_string();
        let total = dur_regex
            .captures(&xml)
            .map(|c| c.get(1).unwrap().as_str())
            .unwrap_or("00:00:00")
            .to_string();

        Ok((current, total))
    }

    /// 获取播放进度 (当前秒数, 总秒数)
    pub async fn get_position_info_sec(&self) -> Result<(u64, u64)> {
        let control_url = self.get_av_url()?;
        let args = "<InstanceID>0</InstanceID>";
        let xml = send_soap_action(control_url, AV_SERVICE, "GetPositionInfo", args).await?;

        let cur_regex = regex::Regex::new(r"(?i)<RelTime[^>]*>(.*?)</RelTime>").unwrap();
        let dur_regex =
            regex::Regex::new(r"(?i)<TrackDuration[^>]*>(.*?)</TrackDuration>").unwrap();

        let cur_str = cur_regex
            .captures(&xml)
            .map(|c| c.get(1).unwrap().as_str())
            .unwrap_or("00:00:00");
        let total_str = dur_regex
            .captures(&xml)
            .map(|c| c.get(1).unwrap().as_str())
            .unwrap_or("00:00:00");

        fn parse_time(t: &str) -> u64 {
            let parts: Vec<&str> = t.split(':').collect();
            if parts.len() == 3 {
                let h: u64 = parts[0].parse().unwrap_or(0);
                let m: u64 = parts[1].parse().unwrap_or(0);
                let s: u64 = parts[2].parse().unwrap_or(0);
                return h * 3600 + m * 60 + s;
            }
            0
        }

        Ok((parse_time(cur_str), parse_time(total_str)))
    }

    /// 获取传输状态
    pub async fn get_transport_info(&self) -> Result<TransportState> {
        let control_url = self.get_av_url()?;
        let args = "<InstanceID>0</InstanceID>";
        let xml = send_soap_action(control_url, AV_SERVICE, "GetTransportInfo", args).await?;

        let regex = regex::Regex::new(
            r"(?i)<(?:.*:)?CurrentTransportState[^>]*>(.*?)</(?:.*:)?CurrentTransportState>",
        )
        .unwrap();

        let state_str = if let Some(caps) = regex.captures(&xml) {
            caps.get(1).unwrap().as_str().trim().to_uppercase()
        } else {
            return Ok(TransportState::Unknown);
        };

        let state = match state_str.as_str() {
            "PLAYING" => TransportState::Playing,
            "PAUSED_PLAYBACK" | "PAUSED" => TransportState::Paused,
            "STOPPED" => TransportState::Stopped,
            "TRANSITIONING" => TransportState::Transitioning,
            "NO_MEDIA_PRESENT" => TransportState::NoMedia,
            _ => TransportState::Unknown,
        };

        Ok(state)
    }
}

// --- 1. 扫描功能 (服务发现) ---

pub async fn scan_projectors(timeout_secs: u64) -> Result<Vec<ProjectorInfo>> {
    // SSDP 多播地址和端口
    const SSDP_ADDR: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);
    const SSDP_PORT: u16 = 1900;

    // 构造 M-SEARCH 请求
    let search_request = format!(
        "M-SEARCH * HTTP/1.1\r\n\
         HOST: {}:{}\r\n\
         MAN: \"ssdp:discover\"\r\n\
         MX: {}\r\n\
         ST: upnp:rootdevice\r\n\
         \r\n",
        SSDP_ADDR, SSDP_PORT, timeout_secs
    );

    let mut devices = Vec::new();
    
    // 尝试方法1: 标准组播 SSDP
    println!("SSDP: Trying multicast method...");
    match try_multicast_ssdp(&search_request, timeout_secs).await {
        Ok(found_devices) => {
            println!("SSDP: Multicast succeeded, found {} devices", found_devices.len());
            for dev in found_devices {
                if !devices.iter().any(|d: &ProjectorInfo| d.ip == dev.ip) {
                    devices.push(dev);
                }
            }
        }
        Err(e) => {
            println!("SSDP: Multicast failed: {}, trying broadcast method...", e);
            
            // 方法2: 使用广播发送到子网
            match try_broadcast_ssdp(&search_request, timeout_secs).await {
                Ok(found_devices) => {
                    println!("SSDP: Broadcast succeeded, found {} devices", found_devices.len());
                    for dev in found_devices {
                        if !devices.iter().any(|d: &ProjectorInfo| d.ip == dev.ip) {
                            devices.push(dev);
                        }
                    }
                }
                Err(e2) => {
                    println!("SSDP: Broadcast also failed: {}", e2);
                    // 如果两种方法都失败，返回原始错误
                    return Err(e.context("Both multicast and broadcast SSDP methods failed"));
                }
            }
        }
    }

    Ok(devices)
}

// 尝试使用组播发送 SSDP
async fn try_multicast_ssdp(search_request: &str, timeout_secs: u64) -> Result<Vec<ProjectorInfo>> {
    const SSDP_ADDR: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);
    const SSDP_PORT: u16 = 1900;
    
    let socket = create_ssdp_socket()?;
    
    // 发送搜索请求到组播地址
    let target_addr = SocketAddrV4::new(SSDP_ADDR, SSDP_PORT);
    socket
        .send_to(search_request.as_bytes(), &target_addr.into())
        .context("Failed to send SSDP multicast request")?;
    
    println!("SSDP: Multicast request sent successfully");
    
    collect_ssdp_responses(socket, timeout_secs).await
}

// 尝试使用广播发送 SSDP（iOS 不需要特殊权限）
async fn try_broadcast_ssdp(search_request: &str, timeout_secs: u64) -> Result<Vec<ProjectorInfo>> {
    use std::net::UdpSocket;
    
    const SSDP_PORT: u16 = 1900;
    
    // 获取本地 IP
    let local_ip = get_local_ip()
        .ok_or_else(|| anyhow::anyhow!("Failed to detect local IP address"))?;
    
    // 计算广播地址 (假设 /24 子网)
    let octets = local_ip.octets();
    let broadcast_ip = Ipv4Addr::new(octets[0], octets[1], octets[2], 255);
    
    println!("SSDP: Using broadcast address: {}", broadcast_ip);
    
    // 创建 UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.set_read_timeout(Some(Duration::from_secs(timeout_secs)))?;
    
    // 发送到广播地址
    let target_addr = SocketAddrV4::new(broadcast_ip, SSDP_PORT);
    socket.send_to(search_request.as_bytes(), target_addr)
        .context("Failed to send SSDP broadcast request")?;
    
    println!("SSDP: Broadcast request sent successfully");
    
    // 也尝试发送到 255.255.255.255
    let _ = socket.send_to(search_request.as_bytes(), "255.255.255.255:1900");
    
    // 收集响应
    let mut devices = Vec::new();
    let mut buffer = [0u8; 2048];
    let start_time = std::time::Instant::now();
    
    socket.set_nonblocking(true)?;
    
    while start_time.elapsed() < Duration::from_secs(timeout_secs) {
        match socket.recv_from(&mut buffer) {
            Ok((size, _addr)) => {
                if let Ok(response) = String::from_utf8(buffer[..size].to_vec()) {
                    if let Some(location) = extract_location(&response) {
                        if let Ok(info) = parse_device_xml(&location).await {
                            if !devices.iter().any(|d: &ProjectorInfo| d.ip == info.ip) {
                                devices.push(info);
                            }
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(_) => break,
        }
    }
    
    Ok(devices)
}

// 收集 SSDP 响应
async fn collect_ssdp_responses(socket: Socket, timeout_secs: u64) -> Result<Vec<ProjectorInfo>> {
    socket.set_read_timeout(Some(Duration::from_secs(timeout_secs)))?;
    
    let mut devices = Vec::new();
    let mut buffer: Vec<MaybeUninit<u8>> = vec![MaybeUninit::uninit(); 2048];
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < Duration::from_secs(timeout_secs) {
        match socket.recv_from(&mut buffer) {
            Ok((size, _addr)) => {
                let data: Vec<u8> = buffer[..size]
                    .iter()
                    .map(|b| unsafe { b.assume_init() })
                    .collect();

                if let Ok(response) = String::from_utf8(data) {
                    if let Some(location) = extract_location(&response) {
                        if let Ok(info) = parse_device_xml(&location).await {
                            if !devices.iter().any(|d: &ProjectorInfo| d.ip == info.ip) {
                                devices.push(info);
                            }
                        }
                    }
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(e) => {
                return Err(anyhow::anyhow!("SSDP receive error: {}", e));
            }
        }
    }

    Ok(devices)
}

// 创建 SSDP socket
fn create_ssdp_socket() -> Result<Socket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))
        .context("Failed to create socket")?;

    // 设置 socket 选项
    socket.set_reuse_address(true)?;

    // 绑定到任意端口
    let bind_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0);
    socket.bind(&bind_addr.into())?;

    // iOS 特定：设置多播选项
    const SSDP_ADDR: Ipv4Addr = Ipv4Addr::new(239, 255, 255, 250);

    // 获取本地 IP 地址用于加入多播组
    let local_ip = get_local_ip();
    println!("SSDP: Detected local IP: {:?}", local_ip);
    
    let local_ip = local_ip.ok_or_else(|| anyhow::anyhow!("Failed to detect local IP address"))?;

    // 加入多播组（iOS 需要，即使只是发送）
    println!("SSDP: Joining multicast group {} on interface {}", SSDP_ADDR, local_ip);
    socket.join_multicast_v4(&SSDP_ADDR, &local_ip)?;

    // 设置多播 TTL
    socket.set_multicast_ttl_v4(2)?;

    // 设置多播接口（iOS 需要明确指定）
    println!("SSDP: Setting multicast interface to {}", local_ip);
    socket.set_multicast_if_v4(&local_ip)?;

    // 允许多播回环（接收自己发送的包）
    socket.set_multicast_loop_v4(true)?;

    // 设置非阻塞模式
    socket.set_nonblocking(true)?;

    Ok(socket)
}

// 获取本地 IP 地址（过滤 VPN/代理接口）
fn get_local_ip() -> Option<Ipv4Addr> {
    use std::net::UdpSocket;

    // 判断是否是 VPN/代理软件使用的虚拟 IP 地址
    fn is_vpn_ip(ip: &Ipv4Addr) -> bool {
        let octets = ip.octets();
        // 198.18.0.0/15 - 通常被 Surge、Shadowrocket 等代理软件使用
        if octets[0] == 198 && (octets[1] == 18 || octets[1] == 19) {
            return true;
        }
        // 100.64.0.0/10 - CGNAT，有时也被 VPN 使用
        if octets[0] == 100 && octets[1] >= 64 && octets[1] <= 127 {
            return true;
        }
        // 10.8.0.0/24, 10.9.0.0/24 - 常见 OpenVPN 地址段
        if octets[0] == 10 && (octets[1] == 8 || octets[1] == 9) && octets[2] == 0 {
            return true;
        }
        false
    }

    // 判断是否是有效的局域网 IP
    fn is_valid_lan_ip(ip: &Ipv4Addr) -> bool {
        if ip.is_loopback() || ip.is_unspecified() || ip.is_link_local() {
            return false;
        }
        // 过滤 VPN IP
        if is_vpn_ip(ip) {
            return false;
        }
        // 允许私有地址（常见局域网）
        let octets = ip.octets();
        // 192.168.x.x
        if octets[0] == 192 && octets[1] == 168 {
            return true;
        }
        // 10.x.x.x (但排除上面已过滤的 VPN 段)
        if octets[0] == 10 {
            return true;
        }
        // 172.16.0.0 - 172.31.255.255
        if octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31 {
            return true;
        }
        false
    }

    // 方法1: 尝试连接到局域网网关来获取 WiFi IP（避免经过 VPN）
    // 使用常见的局域网网关地址
    let gateway_addrs = ["192.168.1.1:80", "192.168.0.1:80", "10.0.0.1:80", "172.16.0.1:80"];
    
    for gateway in &gateway_addrs {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect(*gateway).is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    if let std::net::SocketAddr::V4(addr_v4) = addr {
                        let ip = *addr_v4.ip();
                        if is_valid_lan_ip(&ip) {
                            println!("SSDP: Found valid LAN IP via gateway {}: {}", gateway, ip);
                            return Some(ip);
                        }
                    }
                }
            }
        }
    }

    // 方法2: 回退到原始方法，但验证结果
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                if let std::net::SocketAddr::V4(addr_v4) = addr {
                    let ip = *addr_v4.ip();
                    if is_valid_lan_ip(&ip) {
                        println!("SSDP: Found valid LAN IP via external connect: {}", ip);
                        return Some(ip);
                    } else {
                        println!("SSDP: Detected IP {} appears to be VPN/proxy interface, skipping", ip);
                    }
                }
            }
        }
    }

    // 方法3: 遍历所有网络接口寻找有效的局域网 IP
    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "linux", target_os = "android"))]
    {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            // 尝试一些常见的局域网广播地址
            let broadcast_addrs = ["192.168.1.255:1900", "192.168.0.255:1900", "10.0.0.255:1900"];
            for addr in &broadcast_addrs {
                if socket.connect(*addr).is_ok() {
                    if let Ok(local) = socket.local_addr() {
                        if let std::net::SocketAddr::V4(addr_v4) = local {
                            let ip = *addr_v4.ip();
                            if is_valid_lan_ip(&ip) {
                                println!("SSDP: Found valid LAN IP via broadcast probe: {}", ip);
                                return Some(ip);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("SSDP: Warning - Could not find a valid LAN IP address. VPN/proxy may be blocking local network access.");
    None
}

// 从 HTTP 响应中提取 LOCATION 头
fn extract_location(response: &str) -> Option<String> {
    for line in response.lines() {
        if line.to_lowercase().starts_with("location:") {
            return line
                .split(':')
                .skip(1)
                .collect::<Vec<_>>()
                .join(":")
                .trim()
                .to_string()
                .into();
        }
    }
    None
}

// 辅助：获取并解析设备描述 XML
async fn parse_device_xml(url: &str) -> Result<ProjectorInfo> {
    let client = Client::new();
    let xml_content = client
        .get(url)
        .timeout(Duration::from_secs(2))
        .send()
        .await?
        .text()
        .await?;

    // 这里使用简单的正则提取，生产环境建议用 xml-rs 做严谨解析
    let name_regex = regex::Regex::new(r"<friendlyName>(.*?)</friendlyName>").unwrap();
    let av_regex = regex::Regex::new(r"<serviceType>urn:schemas-upnp-org:service:AVTransport:1</serviceType>[\s\S]*?<controlURL>(.*?)</controlURL>").unwrap();
    let rc_regex = regex::Regex::new(r"<serviceType>urn:schemas-upnp-org:service:RenderingControl:1</serviceType>[\s\S]*?<controlURL>(.*?)</controlURL>").unwrap();

    let friendly_name = name_regex
        .captures(&xml_content)
        .map(|c| c.get(1).unwrap().as_str().to_string())
        .unwrap_or("未知设备".to_string());

    // 解析控制 URL (通常是相对路径，需要拼接到 Base URL)
    let base_url = url.rsplit_once('/').map(|(base, _)| base).unwrap_or(url);

    let resolve_url = |path: &str| -> String {
        if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}/{}", base_url, path.trim_start_matches('/'))
        }
    };

    let av_transport_url = av_regex
        .captures(&xml_content)
        .map(|c| resolve_url(c.get(1).unwrap().as_str()));
    let rendering_control_url = rc_regex
        .captures(&xml_content)
        .map(|c| resolve_url(c.get(1).unwrap().as_str()));

    // 简单从 URL 提取 IP
    let ip = url
        .split("://")
        .nth(1)
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_string();

    Ok(ProjectorInfo {
        friendly_name,
        ip,
        location_xml_url: url.to_string(),
        av_transport_url,
        rendering_control_url,
    })
}

// --- 播放状态枚举 ---
#[frb(dart_metadata=("freezed"))]
#[derive(Debug, Clone, PartialEq)]
pub enum TransportState {
    Stopped,
    Playing,
    Paused,
    Transitioning, // 正在缓冲或加载中
    NoMedia,       // 没有投送内容
    Unknown,       // 无法识别的状态
}

// --- 辅助函数 ---

// 发送 SOAP 请求
async fn send_soap_action(
    control_url: &str,
    service_type: &str,
    action: &str,
    args: &str,
) -> Result<String> {
    let client = Client::new();
    let body = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
        <s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
            <s:Body>
                <u:{action} xmlns:u="{service_type}">
                    {args}
                </u:{action}>
            </s:Body>
        </s:Envelope>"#
    );

    let soap_action_header = format!("\"{}#{}\"", service_type, action);

    let res = client
        .post(control_url)
        .header("Content-Type", "text/xml; charset=\"utf-8\"")
        .header("SOAPACTION", soap_action_header)
        .body(body)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(anyhow::anyhow!("SOAP Request failed: {}", res.status()));
    }

    Ok(res.text().await?)
}

// --- 远程开机 (Wake on LAN) ---

pub fn wake_on_lan(mac_address: String) -> Result<()> {
    // 1. 处理 MAC 格式 (允许 AA:BB.. 或 AA-BB..)
    let mac_clean = mac_address.replace([':', '-'], "");
    let mac_bytes = hex::decode(&mac_clean).context("Invalid MAC address format")?;

    if mac_bytes.len() != 6 {
        return Err(anyhow::anyhow!("MAC address must be 6 bytes"));
    }

    // 2. 构造魔术包 (6个FF + 16次MAC)
    let mut packet = vec![0xFF; 6];
    for _ in 0..16 {
        packet.extend_from_slice(&mac_bytes);
    }

    // 3. 广播
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    socket.send_to(&packet, "255.255.255.255:9")?;

    Ok(())
}
