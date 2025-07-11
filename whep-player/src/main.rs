use crate::player::render_video;
use anyhow::{Error, Result};
use axum::{Router, response::Response, routing::post};
use clap::Parser;
use encoder::Encoder;
use ffmpeg_next::{
    Packet, Rational,
    ffi::{AVBufferRef, av_buffer_ref},
    format::Pixel,
};
use source::Source;
use std::{collections::HashMap, sync::mpsc, time::Instant};
use whep_player::{Cli, Commands};

mod client;
mod encoder;
mod player;
mod source;
mod whip;

// no_mangle: 防止 Rust 编译器对符号名进行名称修饰 (name mangling)
// 用于 NVIDIA 和 AMD 驱动精确匹配这个符号名
#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
pub static NvOptimusEnablement: i32 = 1;
#[unsafe(no_mangle)]
#[allow(non_upper_case_globals)]
pub static AmdPowerXpressRequestHighPerformance: i32 = 1;

struct EncodedPacket(Packet, Instant);

fn create_encoder(width: u32, height: u32, hw_frames: *mut AVBufferRef) -> Result<Encoder> {
    let encoder = Encoder::new(
        "h264_nvenc",
        Some(HashMap::from([
            ("preset".into(), "p6".into()),
            ("tune".into(), "ull".into()),
        ])),
        |encoder| {
            let frame_rate = Rational::new(60, 1);
            encoder.set_bit_rate(5000 * 1000);
            encoder.set_width(width);
            encoder.set_height(height);
            encoder.set_time_base(frame_rate.invert());
            encoder.set_frame_rate(Some(frame_rate));
            encoder.set_gop(120);
            encoder.set_max_b_frames(0);
            encoder.set_format(Pixel::D3D11);
            unsafe {
                let encoder = &mut *encoder.as_mut_ptr();
                encoder.hw_frames_ctx = av_buffer_ref(hw_frames);
            }

            Ok(())
        },
    )?;

    Ok(encoder)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 初始化 ffmpeg
    ffmpeg_next::init()?;

    // 初始化日志
    let args = Cli::parse();
    let _guard = whep_player::util::init_logger(args.verbose);

    match args.commands {
        Commands::Stream { url, token } => stream(url, token).await?,
        Commands::PlayWHIP {} => play_whip().await,
        Commands::PlayWHEP { url, token } => play_whep(url, token).await?,
    }

    Ok(())
}

async fn stream(url: String, token: Option<String>) -> Result<()> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let join_handle = tokio::task::spawn_blocking(move || -> Result<()> {
        let mut encoder: Option<Encoder> = None;
        let mut source: Box<dyn Source + Send + Sync> =
            Box::new(source::dxdup::DisplayDuplicator::new()?);

        let ensure_encoder = |encoder: &mut Option<Encoder>,
                              width: u32,
                              height: u32,
                              hw_frames: *mut AVBufferRef|
         -> Result<()> {
            if let Some(enc) = encoder {
                if enc.dimensions() != (width, height) {
                    encoder.replace(create_encoder(width, height, hw_frames)?);
                }
            } else {
                encoder.replace(create_encoder(width, height, hw_frames)?);
            }

            Ok(())
        };
        let start = Instant::now();
        loop {
            // Pull frame from duplicator
            let frame = source.get_frame()?;
            let hw_frames = unsafe { (*frame.as_ptr()).hw_frames_ctx };
            // Fetch encoder or create it
            ensure_encoder(&mut encoder, frame.width(), frame.height(), hw_frames)?;
            if let Some(encoder) = &mut encoder {
                // Encode frame
                if let Some(packet) = encoder.encode(&frame)? {
                    tx.send(EncodedPacket(packet, start)).unwrap();
                }
            }
        }
    });

    tokio::select! {
        _ = whip::publish(&url, token, rx) => {},
        res = join_handle => {
            res??
        }
    }

    Ok(())
}

async fn whip_handler(
    tx: mpsc::Sender<ffmpeg_next::frame::Video>,
    offer: String,
) -> Response<String> {
    let answer = whip::subscribe_as_server(tx, offer);
    Response::builder()
        .status(201)
        .header("Location", "/")
        .body(answer)
        .unwrap()
}

async fn play_whip() {
    println!("Listening for WHIP Requests on 0.0.0.0:1337");
    let (tx, rx): (
        mpsc::Sender<ffmpeg_next::frame::Video>,
        mpsc::Receiver<ffmpeg_next::frame::Video>,
    ) = mpsc::channel();

    tokio::task::spawn(async move {
        axum::serve(
            tokio::net::TcpListener::bind("0.0.0.0:1337").await.unwrap(),
            Router::new().route("/", post(move |offer: String| whip_handler(tx, offer))),
        )
        .await
        .unwrap();
    });

    render_video(rx);
}

async fn play_whep(url: String, token: Option<String>) -> Result<()> {
    // mpsc: Multi-Producer Single-Consumer
    // 多生产者, 单消费者, 用于在不同的线程之间传递数据
    let (tx, rx): (
        mpsc::Sender<ffmpeg_next::frame::Video>,
        mpsc::Receiver<ffmpeg_next::frame::Video>,
    ) = mpsc::channel();

    whip::subscribe_as_client(tx, &url, token).await;
    render_video(rx);

    Ok(())
}
