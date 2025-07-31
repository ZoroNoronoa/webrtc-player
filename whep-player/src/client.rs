use bytes::Bytes;
use local_ip_address::list_afinet_netifas;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderValue, USER_AGENT};
use serde::Deserialize;
use std::{
    error::Error,
    io::ErrorKind,
    net::{IpAddr, SocketAddr, SocketAddrV4},
    str::FromStr,
    time::{Duration, Instant},
};
use str0m::{
    Candidate, Event, IceConnectionState, Input, Output, Rtc,
    change::{SdpAnswer, SdpOffer},
    format::Codec,
    media::{Direction as RtcDirection, MediaData, MediaKind, MediaTime, Mid},
    net::{Protocol, Receive},
};
use tokio::net::UdpSocket;
use tracing::{debug, error, info, trace, warn};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct WhipClaims {
    pub whip_url: String,
    pub jti: String,
}

#[derive(Debug)]
pub enum WebrtcEvent {
    Continue,
    Media(MediaData),
    Disconnected,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum WebrtcError {
    ServerError(Box<dyn Error + Send + Sync>),
    SdpError,
    WebrtcError(Box<dyn Error + Send + Sync>),
    NetworkError(Box<dyn Error + Send + Sync>),
    SendError(String),
    NoCandidates,
}

pub struct Client {
    rtc: Rtc, // WebRTC 连接的核心对象
    socket: UdpSocket,
    local_socket_addr: SocketAddr,
    buf: [u8; 1500], // udp 数据包缓冲区 (1500 字节, 标准 MTU (Maximum Transmission Unit) )
    video_mid: Option<Mid>, // 媒体视频流的标识符
    _audio_mid: Option<Mid>,
}

impl Client {
    pub async fn new() -> Result<Self, WebrtcError> {
        // 在系统上分配一个 UDP socket, 并绑定到所有网卡的任意可用端口
        let socket = UdpSocket::bind("0.0.0.0:0".parse::<SocketAddrV4>().unwrap())
            .await
            .expect("Should bind udp socket");

        // 构建一个 WebRTC 对象
        let mut rtc = Rtc::builder()
            .clear_codecs() // 清除默认的音视频编解码器列表, 后续可以只启用你需要的编解码器, 避免不必要的协商
            .enable_h264(true) // 启用 H264 视频编解码器
            .set_stats_interval(Some(Duration::from_secs(2))) // 设置每 2 秒手机一次连接的统计数据
            .set_reordering_size_video(1) // 设置视频流的乱序缓冲区为 1 (保证低延迟, 但是网络不佳时会丢帧)
            .set_reordering_size_audio(1) // 设置音频流的乱序缓冲区为 1
            .build();

        // 本地监听的 UDP 端口
        info!("local socket address: {:?}", socket.local_addr());

        // Discover host candidates
        // 获取系统的网络接口列表, 为每个有效的 IPV4 接口创建 WebRTC ICE 候选者
        let mut local_socket_addr = None;
        if let Ok(network_interfaces) = list_afinet_netifas() {
            for (name, ip) in network_interfaces {
                debug!("iface: {} / {:?}", name, ip);
                match ip {
                    IpAddr::V4(ip4) => {
                        if !ip4.is_loopback() && !ip4.is_link_local() {
                            let socket_addr =
                                SocketAddr::new(ip, socket.local_addr().unwrap().port());
                            local_socket_addr = Some(socket_addr.clone());
                            info!("Discover local candidate: [{} / {:?}]", name, ip);
                            rtc.add_local_candidate(
                                Candidate::host(socket_addr, str0m::net::Protocol::Udp)
                                    .expect("Fail to create local candidate"),
                            );
                        }
                    }
                    IpAddr::V6(_ip6) => {}
                }
            }
        } else {
            return Err(WebrtcError::NoCandidates);
        }

        let Some(local_socket_addr) = local_socket_addr else {
            return Err(WebrtcError::NoCandidates);
        };

        Ok(Self {
            socket,
            local_socket_addr,
            rtc,
            buf: [0; 1500],
            video_mid: None,
            _audio_mid: None,
        })
    }

    pub async fn send_whip_request(
        &mut self,
        url: &str,
        token: &Option<String>,
        direction: RtcDirection,
    ) -> Result<(), WebrtcError> {
        // Add receive tracks and generate an offer
        // * stream-id: 也被叫做 Media Stream ID, 用于标识一个媒体流 (MediaStream), 用于同步一个流下的多个轨道 (比如音频和视频同步播放)
        // * track-id: 也被叫做 Media Stream Track ID, 用于标识流中的某一个具体轨道 (比如音频轨道、视频轨道)
        let mut change = self.rtc.sdp_api();
        self.video_mid = Some(change.add_media(
            MediaKind::Video,
            direction,
            Some("video_0".to_string()),
            Some("video_0".to_string()),
        ));

        // 创建 SDP Offer
        // * 如果此方法返回 SDPOffer, 说明更改不会立即生效, 调用者需要与 remote peer 进行协商, 并在获得 answer 后使用 SdpPendingOffer 应答
        // * 如果返回 None, 要么没有更改, 要么更改可以直接应用无须协商
        //
        // 当你调用 `sdp_api().add_media` 后更改不会立即生效, 而是需要进行 SDP 协商
        // `apply()` 方法会尝试应用这些更改, 如果更改需要协商, 那么 `apply()` 会发挥一个 SdpOffer, 你需要把 Offer 发送给 Remote Peer
        // 此时 str0m 内部会生成一个 SdpPendingOffer, 它表示当前有一个待处理的 Offer, 等远端回复
        let (offer, pending) = change.apply().ok_or(WebrtcError::SdpError)?;

        let offer_str = offer.to_sdp_string();
        info!("offer: {}", offer_str);
        info!("token: {:?}", token);
        info!("url: {}", url);

        let mut headers = reqwest::header::HeaderMap::new();

        // 构造 WHEP 协议中 SDP 交换请求的 header
        // 如果有 token 的话需要附加到 Bearer 中
        if let Some(token) = &token {
            let authorization_value = HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|e| WebrtcError::ServerError(e.into()))?;
            headers.append(AUTHORIZATION, authorization_value);
        }

        headers.append(
            CONTENT_TYPE,
            HeaderValue::from_str("application/sdp").unwrap(),
        );
        headers.append(ACCEPT, HeaderValue::from_str("application/sdp").unwrap());
        headers.append(USER_AGENT, HeaderValue::from_str("bitwhip").unwrap());

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| WebrtcError::ServerError(e.into()))?;

        // 处理重定向的问题
        let mut next_url =
            reqwest::Url::from_str(&url).map_err(|e| WebrtcError::ServerError(e.into()))?;
        let res = loop {
            let response = client
                .post(next_url.clone())
                .body(offer_str.clone())
                .send()
                .await
                .map_err(|e| WebrtcError::ServerError(e.into()))?;
            if response.status().is_redirection() {
                if let Some(location) = response
                    .headers()
                    .get(reqwest::header::HeaderName::from_static("location"))
                {
                    next_url = reqwest::Url::from_str(
                        location
                            .to_str()
                            .map_err(|e| WebrtcError::ServerError(e.into()))?,
                    )
                    .map_err(|e| WebrtcError::ServerError(e.into()))?;
                    info!("Redirect! Next URL: {:?}", next_url);
                    continue;
                }
            } else {
                break response;
            }
        };

        // get answer sdp from body
        // 从返回值中解析出来 SDP
        let http_code = res.status();
        info!("status: {}", http_code);
        if http_code != reqwest::StatusCode::CREATED {
            return Err(WebrtcError::ServerError(
                format!("POST failed with status: {}", http_code).into(),
            ));
        }

        info!("headers: {:?}", res.headers());
        let answer = res
            .text()
            .await
            .map_err(|e| WebrtcError::ServerError(e.into()))?;
        info!("answer:\n{}", answer);
        // for (i, line) in answer.lines().enumerate() {
        //     println!("{}: {}", i, line);
        // }
        let modified_answer = answer
            .lines()
            .map(|line| {
                if line.starts_with("o=") {
                    // o (Origin) 行描述会话的创建信息, SRS 目前传递的是 `o=SRS/5.0.217(Bee)` 无法解析
                    let mut words: Vec<&str> = line.split_whitespace().collect();
                    if !words.is_empty() {
                        words[0] = "o=-";
                    }
                    words.join(" ")
                } else if line.starts_with("s=") {
                    // s (Session Name) 行描述会话的名称, SRS 目前传递的是 `s=SRSPlaySession` 无法解析
                    "s=-".to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("\n");
        info!("modified_answer:\n{}", modified_answer);

        let sdp_answer = match SdpAnswer::from_sdp_string(&modified_answer) {
            Ok(answer) => answer,
            Err(e) => {
                error!("Failed to parse SDP answer: {:?}", e);
                return Err(WebrtcError::SdpError);
            }
        };

        match self.rtc.sdp_api().accept_answer(pending, sdp_answer) {
            Ok(_) => {}
            Err(e) => {
                error!("Accept_answer failed: {:?}", e);
            }
        }

        Ok(())
    }

    pub fn accept_whip_request(&mut self, offer: String) -> Result<String, WebrtcError> {
        let offer = SdpOffer::from_sdp_string(&offer).map_err(|_| WebrtcError::SdpError)?;
        if let Ok(answer) = self.rtc.sdp_api().accept_offer(offer) {
            return Ok(answer.to_sdp_string());
        }

        return Err(WebrtcError::SdpError);
    }

    pub async fn recv<'a>(&mut self) -> Result<WebrtcEvent, WebrtcError> {
        trace!("recv poll_output()");
        let timeout = match self
            .rtc
            .poll_output()
            .map_err(|e| WebrtcError::WebrtcError(e.into()))?
        {
            Output::Event(event) => match event {
                Event::Connected => {
                    info!("connected");
                    return Ok(WebrtcEvent::Continue);
                }
                Event::IceConnectionStateChange(state) => {
                    info!("ice connection state change: {:?}", state);
                    match state {
                        IceConnectionState::Disconnected => return Ok(WebrtcEvent::Disconnected),
                        _ => return Ok(WebrtcEvent::Continue),
                    }
                }
                Event::MediaIngressStats(stats) => {
                    info!("egress stats: {:?}", stats);
                    return Ok(WebrtcEvent::Continue);
                }
                Event::MediaEgressStats(stats) => {
                    info!("egress stats: {:?}", stats);
                    return Ok(WebrtcEvent::Continue);
                }
                Event::PeerStats(stats) => {
                    info!("stats: {:?}", stats);
                    return Ok(WebrtcEvent::Continue);
                }
                Event::MediaData(media) => {
                    return Ok(WebrtcEvent::Media(media));
                }
                Event::MediaAdded(media) => {
                    info!("Media Added: {:?}", media);
                    info!("Codec Config: {:?}", self.rtc.codec_config());
                    return Ok(WebrtcEvent::Continue);
                }
                _ => {
                    return Ok(WebrtcEvent::Continue);
                }
            },
            Output::Timeout(timeout) => timeout,
            Output::Transmit(send) => {
                // Apply random packet loss to outbound traffic
                if let Err(e) = self.socket.send_to(&send.contents, send.destination).await {
                    debug!(
                        "sending to {} => {}, len {} error {:?}",
                        send.source,
                        send.destination,
                        send.contents.len(),
                        e
                    );
                };
                return Ok(WebrtcEvent::Continue);
            }
        };

        let duration = timeout - Instant::now();
        if duration.is_zero() {
            // Drive time forwards in rtc straight away.
            return match self.rtc.handle_input(Input::Timeout(Instant::now())) {
                Ok(_) => Ok(WebrtcEvent::Continue),
                Err(e) => {
                    error!("error handle input rtc: {:?}", e);
                    Ok(WebrtcEvent::Continue)
                }
            };
        }

        let input = match tokio::time::timeout(duration, self.socket.recv_from(&mut self.buf)).await
        {
            Ok(Ok((n, source))) => {
                // UDP data received.
                info!(
                    "received from {} => {}, len {}",
                    source,
                    SocketAddr::new(
                        self.local_socket_addr.ip(),
                        self.socket.local_addr().unwrap().port(),
                    ),
                    n
                );
                Input::Receive(
                    Instant::now(),
                    Receive {
                        proto: Protocol::Udp,
                        source,
                        destination: SocketAddr::new(
                            self.local_socket_addr.ip(),
                            self.socket.local_addr().unwrap().port(),
                        ),
                        contents: (&self.buf[..n]).try_into().expect("should webrtc"),
                    },
                )
            }
            Ok(Err(e)) => match e.kind() {
                ErrorKind::ConnectionReset => return Ok(WebrtcEvent::Continue),
                _ => {
                    error!("[TransportWebrtc] network error {:?}", e);
                    return Err(WebrtcError::NetworkError(e.into()));
                }
            },
            Err(_e) => {
                // Expected error for set_read_timeout().
                // One for windows, one for the rest.
                Input::Timeout(Instant::now())
            }
        };

        // Input is either a Timeout or Receive of data. Both drive the state forward.
        self.rtc
            .handle_input(input)
            .map_err(|e| WebrtcError::WebrtcError(e.into()))?;
        return Ok(WebrtcEvent::Continue);
    }

    pub fn send_video(&mut self, frame_data: Bytes, pts: Duration) -> Result<(), WebrtcError> {
        if let Some(mid) = self.video_mid {
            // TODO = maybe look this up once?
            let params = &self
                .rtc
                .codec_config()
                .find(|p| {
                    debug!("payload: {:?}", p);
                    p.spec().codec == Codec::H264
                        && p.spec().format.profile_level_id.unwrap_or(0) == 4382751
                })
                .cloned()
                .unwrap();
            if let Some(writer) = self.rtc.writer(mid) {
                let freq = params.spec().clock_rate;
                let media_time: MediaTime = pts.into();
                writer
                    .write(
                        params.pt(),
                        Instant::now(),
                        media_time.rebase(freq),
                        frame_data,
                    )
                    .map_err(|e| WebrtcError::SendError(e.to_string()))?;
            }
        } else {
            warn!("trying to send video without mid");
        }
        Ok(())
    }
}
