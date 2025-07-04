# Player for WebRTC HTTP Egress Protocol

## Build from source

### OSX

Requirements:

- XCode command line tools installed
- Install additional dependencies using homebrew

```bash
brew install gstreamer gst-plugins-bad libsoup@2 icu4c cmake

# 查看 libsoup 安装位置
brew list libsoup@2

# 验证是否安装成功
pkg-config --libs gstreamer-1.0

# 模糊查找 libsoup
pkg-config --list-all | grep libsoup

# Q: homebrew 安装完 libsoup@2 后, pkg-config 查找不到
# A: https://blog.csdn.net/dotphoenix/article/details/127494393
export PKG_CONFIG_PATH=$(find /usr/local/Cellar -name 'pkgconfig' -type d | grep lib/pkgconfig | tr '\n' ':' | sed s/.$//)
```

```bash
cmake -DCMAKE_BUILD_TYPE=Release -G "Unix Makefiles" .
make
sudo make install
```

### Windows

Requirements:

- choco
- ninja： cmake 编译工具
- msys2：提供了 Windows 上的 Unix 工具链和库
- pkg-config

```bash
# 安装 choco
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# 安装 Ninja
choco install ninja

# 安装 msys2
choco install msys2

# 在 msys2 中安装 pkg-config

```

```bash
cmake -DCMAKE_BUILD_TYPE=Release -G "Ninja" .
```
