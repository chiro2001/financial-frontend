#![cfg(target_arch = "wasm32")]

use async_io_stream::IoStream;
use std::marker::Unpin;
use tarpc::serde::{Deserialize, Serialize};
use tarpc::serde_transport::Transport;
// use tokio_serde::*;
use tokio_serde::{Serializer, Deserializer};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;
use rpc::API_PORT;
// use ws_stream_wasm::*;
use ws_stream_wasm::{WsMeta, WsStreamIo};

pub async fn connect<Item, SinkItem, Codec, CodecFn>(
    codec_fn: CodecFn,
) -> Result<Transport<IoStream<WsStreamIo, Vec<u8>>, Item, SinkItem, Codec>, std::io::Error>
where
    Item: for<'de> Deserialize<'de>,
    SinkItem: Serialize,
    Codec: Serializer<SinkItem> + Deserializer<Item>,
    CodecFn: Fn() -> Codec,
{
    info!("Starting connect2");
    let url = format!("ws://127.0.0.1:{}", API_PORT);
    info!("Connecting to server: {}", url);
    match WsMeta::connect(&url, None).await {
        Ok((_ws, _wsio)) => {
            //let session = WebSocketSession::connect(url);
            info!("Creating the frame");
            let frame = Framed::new(_wsio.into_io(), LengthDelimitedCodec::new());
            info!("Creating the Transport");
            let tmp = tarpc::serde_transport::new(frame, codec_fn());
            info!("Returning Transport");
            Ok(tmp)
        }
        Err(e) => {
            info!("Errored on WsMeta connect\n{:?}", e);
            Err(std::io::Error::from(std::io::ErrorKind::ConnectionRefused))
        }
    }
}

pub async fn build_client<Item, SinkItem>(
) -> Result<impl tarpc::Transport<SinkItem, Item>, std::io::Error>
where
    Item: for<'de> Deserialize<'de> + Unpin,
    SinkItem: Serialize + Unpin,
{
    info!("In build client");
    Ok(
        connect(tokio_serde::formats::Json::<Item, SinkItem>::default)
            .await
            .unwrap(),
    )
    //self.server_handles.push(handle);
}
