use std::sync::mpsc;
use rpc::api::{StockListResp, TradingHistoryItem};
use crate::financial_analysis::{MainApiClient, Token};
use crate::trading_history::TradingHistoryValueItem;

#[derive(Debug)]
pub enum Message {
    ApiClientConnect(MainApiClient),
    LoginDone(Token),
    GotStockList(StockListResp),
    GotTradingHistory((String, Vec<TradingHistoryItem>, String)),
    GotPredicts((String, Vec<TradingHistoryValueItem>, String)),
}

unsafe impl Send for Message {}

#[derive(Debug)]
pub struct Channel {
    pub tx: mpsc::Sender<Message>,
    pub rx: mpsc::Receiver<Message>,
}
