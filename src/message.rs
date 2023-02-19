use std::sync::mpsc;

#[derive(Debug)]
pub enum Message {
    ApiClientConnect(rpc::api::api_rpc_client::ApiRpcClient<tonic_web_wasm_client::Client>),
}

unsafe impl Send for Message {}

#[derive(Debug)]
pub struct Channel {
    pub tx: mpsc::Sender<Message>,
    pub rx: mpsc::Receiver<Message>,
}
