import 'common.just'

export OPENSSL_NO_VENDOR := "1"

FFmpegZipFileName := "ffmpeg-n7.1.1-54-g6400860b9d-linux64-gpl-shared-7.1"

[no-cd]
install-deps:
    #!/bin/sh
    set -e

    mkdir -p "{{ExtDir}}"
    cd {{ExtDir}}

    # FFMPEG

    if [ ! -d "{{FFmpeg}}" ]; then
        echo "Downloading {{FFmpeg}}"
        wget "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2025-06-27-14-04/{{FFmpegZipFileName}}.tar.xz"
        tar -xf {{FFmpegZipFileName}}.tar.xz
        mv {{FFmpegZipFileName}} {{FFmpeg}}
        rm {{FFmpegZipFileName}}.tar.xz
    fi

    # OpenSSL

    if [ ! -d "openssl" ]; then
        echo "Downloading OpenSSL"
        git clone --depth 1 https://github.com/openssl/openssl.git
        cd openssl
        ./config
        make
        cd ..
    fi

[no-cd]
build *param:
    cargo build -vvv --release -- {{param}}

[no-cd]
build-debug *param:
    cargo build -- {{param}}

[no-cd]
run *param:
    cargo run --release -- {{param}}

[no-cd]
test *param:
    cargo test -- {{param}}

[no-cd]
clippy:
    cargo clippy

[no-cd]
debug *param:
    cargo run -- {{param}}

[no-cd]
profile *param:
    cargo run --profile profile -- {{param}}
