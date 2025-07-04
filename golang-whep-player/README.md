# golang-webrtc-player

## 地址

<https://github.com/rprata/musasaurus-player.git>

## 升级 golang 版本

```bash
# linux
# @see https://go.dev/doc/install
wget "https://go.dev/dl/go1.24.4.linux-amd64.tar.gz"
sudo rm -rf /usr/local/go && sudo tar -C /usr/local -xzf go1.24.4.linux-amd64.tar.gz
export PATH=$PATH:/usr/local/go/bin
go version
```

## 初始化

```bash
go mod init golang-whep-player
```

## 运行

```bash
go run main.go
```
