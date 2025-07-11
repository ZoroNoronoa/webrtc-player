use crate::EncodedPacket;
use crate::client::{Client, WebrtcEvent};
use bytes::Bytes;
use ffmpeg_next;
use futures::executor;
use std::{sync::mpsc, time::Instant};
use str0m::media::Direction as RtcDirection;
use tokio::sync::mpsc::{UnboundedReceiver, error::TryRecvError};
use tracing::{error, info};

pub async fn publish(
    publish_url: &str,
    token: Option<String>,
    mut packet_rx: UnboundedReceiver<EncodedPacket>,
) {
    info!(
        "creating client to push to {} with token: {:?}",
        publish_url, token
    );

    let mut client = Client::new().await.unwrap();
    client
        .send_whip_request(&publish_url, &token, RtcDirection::SendOnly)
        .await
        .expect("should connect");

    loop {
        match client.recv().await {
            Ok(event) => match event {
                WebrtcEvent::Disconnected => {
                    info!("disconnected");
                    break;
                }
                WebrtcEvent::Media(_) => {
                    panic!("Publisher incorrectly has incoming media");
                }
                WebrtcEvent::Continue => loop {
                    let packet = packet_rx.try_recv();
                    match packet {
                        Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
                        Ok(packet) => {
                            let pts = Instant::now() - packet.1;
                            if let Some(data) = packet.0.data() {
                                client
                                    .send_video(Bytes::copy_from_slice(data), pts)
                                    .unwrap();
                            }
                        }
                    }
                },
            },
            Err(err) => {
                error!("error: {:?}", err);
                break;
            }
        }
    }
}

pub async fn decode_recv_loop(mut client: Client, tx: mpsc::Sender<ffmpeg_next::frame::Video>) {
    let codec = ffmpeg_next::decoder::find_by_name("h264").expect("H264 Decoder Available");
    let context = ffmpeg_next::codec::context::Context::new_with_codec(codec);
    let mut decoder = context.decoder().video().expect("Decoder init correctly");

    loop {
        match client.recv().await {
            Ok(event) => match event {
                WebrtcEvent::Disconnected => {
                    info!("disconnected");
                    break;
                }
                WebrtcEvent::Media(media) => {
                    // Decoder failures may happen, ignore them
                    match decoder.send_packet(&ffmpeg_next::Packet::borrow(&media.data)) {
                        Err(_) => continue,
                        Ok(_) => {}
                    };

                    let mut frame = ffmpeg_next::frame::Video::empty();
                    while decoder.receive_frame(&mut frame).is_ok() {
                        tx.send(frame).expect("pushed");
                        frame = ffmpeg_next::frame::Video::empty();
                    }
                }
                WebrtcEvent::Continue => {
                    info!("Continue");
                }
            },
            Err(err) => {
                error!("error: {:?}", err);
                break;
            }
        }
    }
}

pub async fn subscribe_as_client(
    tx: mpsc::Sender<ffmpeg_next::frame::Video>,
    publish_url: &str,
    token: Option<String>,
) {
    let mut client = Client::new().await.unwrap();
    client
        .send_whip_request(&publish_url, &token, RtcDirection::RecvOnly)
        .await
        .expect("should connect");

    tokio::task::spawn(async move {
        decode_recv_loop(client, tx).await;
    });
}

pub fn subscribe_as_server(tx: mpsc::Sender<ffmpeg_next::frame::Video>, offer: String) -> String {
    let mut client = executor::block_on(Client::new()).expect("Ok");
    let answer = client.accept_whip_request(offer).expect("Ok");
    tokio::task::spawn(async move {
        decode_recv_loop(client, tx).await;
    });

    answer
}
