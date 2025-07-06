import 'common.just'

VcpkgArch := "x64-windows-static-md"

export VCPKG_ROOT := ProjectDir / "ext/vcpkg"

# Use our vcpkg openssl
export OPENSSL_NO_VENDOR := "1"

FFmpegZipFileName := "ffmpeg-n7.1.1-54-g6400860b9d-win64-gpl-shared-7.1"

[no-cd]
install-deps:
    #!powershell
    $ProgressPreference = 'SilentlyContinue'
    $ErrorActionPreference = 'Stop'

    mkdir "{{ExtDir}}" -ErrorAction SilentlyContinue
    cd {{ExtDir}}

    # FFMPEG
    if (-not (Test-Path -Path "{{FFmpeg}}")) {
        echo "Downloading {{FFmpeg}}"
        wget "https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2025-06-27-14-04/{{FFmpegZipFileName}}.zip" -outfile "{{FFmpegZipFileName}}.zip"
        Expand-Archive {{FFmpegZipFileName}}.zip temp-nested-folder
        Move-Item temp-nested-folder/{{FFmpegZipFileName}} {{FFmpeg}}
        del {{FFmpegZipFileName}}.zip
        del temp-nested-folder
    }

    # OpenSSL

    if (-not (Test-Path -Path "vcpkg/installed/{{VcpkgArch}}/lib/openssl")) {
        echo "Downloading OpenSSL"
        rm -Recurse -Force \vcpkg -ErrorAction SilentlyContinue
        git clone --depth 1 https://github.com/Microsoft/vcpkg.git
        .\vcpkg\bootstrap-vcpkg.bat -disableMetrics
        .\vcpkg\vcpkg install "openssl[core]:{{VcpkgArch}}"
    }

[no-cd]
build *param:
    cargo build -vvv --release -p whep-player -- {{param}}

[no-cd]
build-debug *param:
    cargo build -p whep-player -- {{param}}

[no-cd]
run *param:
    cargo run --release -p whep-player -- {{param}}

[no-cd]
test *param:
    cargo test -p whep-player -- {{param}}

[no-cd]
clippy:
    cargo clippy

[no-cd]
debug *param:
    cargo run -p whep-player -- {{param}}

[no-cd]
profile *param:
    cargo run --profile profile -p whep-player -- {{param}}
