set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
PathSep := if os() == "windows" { ";" } else { ":" }

ProjectVersion := "1.5.4"
ProjectDir := if path_exists(justfile_directory() / "Cargo.toml") == "true" { justfile_directory()
       } else if path_exists(justfile_directory() / ".." / "Cargo.toml") == "true" { justfile_directory() / "../"
       } else { justfile_directory() }
ExtDir := ProjectDir / "ext"

# FFmpeg := if os() == "windows" { "ffmpeg-n7.1.1-54-g6400860b9d-win64-gpl-shared-7.1"
#    } else if os() + arch() == "macosaarch64" { "ffmpeg-arm64"
#    } else if os() + arch() == "macosx86_64"  { "ffmpeg-x64_64"
#    } else if os() + arch() == "linuxaarch64" { "ffmpeg-n7.0-latest-linuxarm64-gpl-shared-7.0"
#    } else if os() + arch() == "linuxx86_64"  { "ffmpeg-n7.0-latest-linux64-gpl-shared-7.0"
#    } else                      { "ffmpeg-7.0-linux-clang-gpl-lite" }
# 不同操作系统都统一放到同一个目录, 方便后续在 .cargo/config.toml 里设置环境变量
FFmpeg := "ffmpeg-gpl-shared-7.1"
export FFMPEG_DIR := ProjectDir / "ext" / FFmpeg

export LD_LIBRARY_PATH := ProjectDir / "target/release:" + FFMPEG_DIR + "/lib:" + FFMPEG_DIR + "/lib/amd64"
export PATH := FFMPEG_DIR + "/bin" + PathSep + FFMPEG_DIR + "/bin/amd64" + PathSep + env_var('PATH')
