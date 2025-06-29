fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    match target_os.as_str() {
        "macos" | "ios" => {
            println!(
                "cargo:rustc-link-search={}/lib",
                std::env::var("FFMPEG_DIR").expect("FFMPEG_DIR")
            );
        }
        "linux" => {
            println!(
                "cargo:rustc-link-search={}/lib/amd64",
                std::env::var("FFMPEG_DIR").expect("FFMPEG_DIR")
            );
            println!(
                "cargo:rustc-link-search={}/lib",
                std::env::var("FFMPEG_DIR").expect("FFMPEG_DIR")
            );
        }
        "windows" => {
            const FFMPEG_DIR: &str = "ext\\ffmpeg-gpl-shared-7.1";

            {
                // 在 windows PATH 里追加 ffmpeg bin 路径, 避免查找不到 ffmpeg 的动态库
                // 在 .cargo/config.toml 里无法给 windows 设置 PATH 环境变量, 暂时没找到原因
                let current_path = std::env::var("PATH").unwrap();
                let ffmpeg_bin_path = format!("{}\\bin", FFMPEG_DIR);
                let combined_path = format!("{};{}", ffmpeg_bin_path, current_path);
                println!("cargo:rustc-env=PATH={}", combined_path);
            }

            // 导出 NvOptimusEnablement 符号, 告诉 NVIDIA Optimus 系统优先使用独立显卡而非集成显卡
            println!("cargo:rustc-link-arg=/EXPORT:NvOptimusEnablement");
            // AMD 双显卡切换技术 (类似 NVIDIA Optimus), 启用硬件加速并优先使用独立显卡运行程序
            println!("cargo:rustc-link-arg=/EXPORT:AmdPowerXpressRequestHighPerformance");
            println!("cargo:rustc-link-search={}\\lib\\x64", FFMPEG_DIR);
            println!("cargo:rustc-link-search={}\\lib", FFMPEG_DIR);
        }
        tos => panic!("unknown target os {:?}!", tos),
    }
}
