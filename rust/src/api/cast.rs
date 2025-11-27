use anyhow::{Context, Result};
use flutter_rust_bridge::frb;
use futures::StreamExt;
use reqwest::Client;
use ssdp_client::SearchTarget;
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
    let search_target = SearchTarget::RootDevice;
    let timeout = Duration::from_secs(timeout_secs);

    // 修复点 1: search 函数需要 4 个参数，最后补一个 None
    // 修复点 2: 这返回的是一个 Stream，不是 Vec
    let mut stream = ssdp_client::search(&search_target, timeout, 2, None)
        .await
        .map_err(|e| anyhow::anyhow!("SSDP search failed: {}", e))?;

    let mut devices = Vec::new();

    // 修复点 3: 使用异步流的方式遍历结果
    while let Some(response_result) = stream.next().await {
        // 流中的每一项也是一个 Result，需要解包
        if let Ok(response) = response_result {
            let location = response.location().to_string();

            // 为了防止重复扫描同一个设备，可以先简单的根据 location 去重 (可选)
            // 这里直接解析
            if let Ok(info) = parse_device_xml(&location).await {
                // 简单的去重逻辑：如果列表中还没这个 IP
                if !devices.iter().any(|d: &ProjectorInfo| d.ip == info.ip) {
                    devices.push(info);
                }
            }
        }
    }

    Ok(devices)
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
