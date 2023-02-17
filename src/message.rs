use std::sync::mpsc;

#[derive(Debug)]
pub enum Message {
    // ApiClientConnect(rpc::ApiClient),
}

unsafe impl Send for Message {}

#[derive(Debug)]
pub struct Channel {
    pub tx: mpsc::Sender<Message>,
    pub rx: mpsc::Receiver<Message>,
}
