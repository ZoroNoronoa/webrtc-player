# BitWHIP

- [BitWHIP](#bitwhip)
  - [什么是 BitWHIP](#什么是-bitwhip)
  - [构建](#构建)
    - [安装 Just](#安装-just)
    - [安装依赖](#安装依赖)
  - [使用方法](#使用方法)
    - [播放 WHIP](#播放-whip)
    - [播放 WHEP](#播放-whep)
    - [推流](#推流)
  - [TODO](#todo)
  - [更多信息](#更多信息)

## 什么是 BitWHIP

BitWHIP 是一个用 Rust 编写的命令行 WebRTC Agent。你可以用它做以下事情：

- 以 30ms 延迟发布你的桌面
- 在本地播放器中播放流
- 从其他来源拉取 WebRTC 视频并播放
  - [Broadcast Box](https://github.com/glimesh/broadcast-box)
  - [IVS](https://aws.amazon.com/ivs/)
  - [Cloudflare](https://developers.cloudflare.com/stream/webrtc-beta/)
  - [Dolby.io](https://docs.dolby.io/streaming-apis/reference/whip_whippublish)
  - [Red5](https://www.red5.net/docs/special/user-guide/whip-whep-configuration/)
  - [Nimble Streamer](https://softvelum.com/nimble/)
  - 任何支持 [WHIP](https://datatracker.ietf.org/doc/draft-ietf-wish-whip/)/[WHEP](https://datatracker.ietf.org/doc/draft-murillo-whep/) 的服务！

BitWHIP 基于开放协议构建，因此几乎可以在任何地方使用。它还可以与 OBS、FFmpeg 或 GStreamer 等你喜欢的工具和库互操作。

## 构建

BitWHIP 使用 [just](https://github.com/casey/just) 简化依赖安装和构建流程。要构建本项目，首先安装 `just`，然后执行 `install-deps`。

### 安装 Just

`cargo install just`

### 安装依赖

`just install-deps`

## 使用方法

构建完成后，你有三种不同的使用方式。

### 播放 WHIP

播放 WHIP 会启动一个本地 WHIP 服务器，客户端可以推流到这里。你可以用 BitWHIP 或其他 WHIP 客户端（如 [OBS](https://obsproject.com/) 或 [GStreamer](https://gstreamer.freedesktop.org/)）推送视频。

```bash
just run play whip
```

WHIP 客户端应使用 `http://localhost:1337/` 作为 URL，并可使用任意 Bearer Token。你可以通过运行 `just run stream http://localhost:1337/ bitwhip` 用 BitWHIP 推流。

### 播放 WHEP

播放 WHEP 会连接到 WHEP 服务器并播放视频。下面是一个从 <https://b.siobud.com/> 拉流并使用 Bearer Token `bitwhip` 的示例：

```bash
just run play-whep https://b.siobud.com/api/whep bitwhip
```

运行后，打开 <https://b.siobud.com/publish/bitwhip>，你的视频会在本地播放器中打开。

### 推流

*目前仅支持带有 NVIDIA 显卡的 Windows，后续会支持更多平台。*

推流会捕获你的本地桌面并通过 WHIP 发布。运行时需要一个 URL 和 Bearer Token。下面是一个推送到 <https://b.siobud.com/> 并使用 Bearer Token `bitwhip` 的示例：

```bash
just run stream https://b.siobud.com/api/whip bitwhip
```

## TODO

- [ ] 创建二进制文件
- [ ] 改进构建系统
- 支持更多采集方式
  - [ ] gdigrab（Windows）
  - [ ] x11grab（Linux）
- 支持更多编码方式
  - [ ] QuickSync
  - [ ] x264

## 更多信息

[Selkies-GStreamer](https://github.com/selkies-project/selkies-gstreamer) 是一个 WebRTC 远程桌面流媒体实现，已实现 0-16ms 的延迟。
