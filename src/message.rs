use std::sync::mpsc;
use crate::financial_analysis::{MainApiClient, Token};

#[derive(Debug)]
pub enum Message {
    ApiClientConnect(MainApiClient),
    LoginDone(Token),
}

unsafe impl Send for Message {}

#[derive(Debug)]
pub struct Channel {
    pub tx: mpsc::Sender<Message>,
    pub rx: mpsc::Receiver<Message>,
}
