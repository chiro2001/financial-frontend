use std::sync::mpsc;
use rpc::api::{GuideLineResp, StockIssueResp, StockListResp, TradingHistoryItem};
use crate::financial_analysis::{MainApiClient, Token};
use crate::stock_view::TradingHistoryValueItem;

#[derive(Debug)]
pub enum Message {
    ApiClientConnect(MainApiClient),
    LoginDone(Token),
    GotStockList(StockListResp),
    GotTradingHistory((String, Vec<TradingHistoryItem>, String)),
    GotPredicts((String, Vec<TradingHistoryValueItem>, String)),
    GotStockIssue((String, StockIssueResp, String)),
    GotGuideLine((String, GuideLineResp, String)),
}

unsafe impl Send for Message {}

#[derive(Debug)]
pub struct Channel {
    pub tx: mpsc::Sender<Message>,
    pub rx: mpsc::Receiver<Message>,
}
