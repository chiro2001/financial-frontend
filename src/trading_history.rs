use std::sync::mpsc;
use egui::{Label, RichText, Window};
use rpc::api::{StockResp, TradingHistoryItem, TradingHistoryRequest, TradingHistoryType};
use tracing::{error, info};
use crate::financial_analysis::MainApiClient;
use crate::message::Message;
use crate::utils::execute;

pub struct TradingHistoryView {
    pub stock: StockResp,
    pub data: Vec<TradingHistoryItem>,
    pub client: Option<MainApiClient>,
    pub tx: Option<mpsc::Sender<Message>>,
    requesting: bool,
    error: String,
    pub typ: TradingHistoryType,
    pub valid: bool,
}

impl TradingHistoryView {
    pub fn new(stock: StockResp, client: Option<MainApiClient>, tx: Option<mpsc::Sender<Message>>) -> Self {
        Self {
            stock,
            data: vec![],
            client,
            requesting: false,
            tx,
            typ: TradingHistoryType::Week,
            error: "".to_string(),
            valid: true,
        }
    }
    pub fn window(&mut self, ctx: &egui::Context) {
        if !self.requesting && self.data.is_empty() && self.error.is_empty() {
            self.requesting = true;
            let symbol = self.stock.symbol.to_string();
            let typ = match self.typ {
                TradingHistoryType::Daily => 0,
                TradingHistoryType::Week => 1,
                TradingHistoryType::Month => 2,
            };
            let mut client = self.client.clone();
            let tx = self.tx.clone();
            execute(async move {
                if let Some(tx) = tx {
                    if let Some(client) = &mut client {
                        let r = client.trading_history(TradingHistoryRequest { symbol: symbol.clone(), typ }).await;
                        match r {
                            Ok(r) => {
                                let resp = r.into_inner();
                                info!("get trading history done: {}", symbol);
                                tx.send(Message::GotTradingHistory((symbol, resp.data, "".to_string()))).unwrap();
                            }
                            Err(e) => {
                                error!("{}", e);
                                tx.send(Message::GotTradingHistory((symbol, vec![], e.to_string()))).unwrap();
                            }
                        }
                    }
                }
            });
        }
        Window::new(format!("[{}]{}", self.stock.code, self.stock.name))
            .open(&mut self.valid)
            .show(ctx, |ui| {
                ui.set_min_width(480.0);
                if self.requesting {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("正在加载...");
                    });
                } else {
                    if self.data.is_empty() {
                        ui.centered_and_justified(|ui| {
                            if self.error.is_empty() {
                                ui.label("无数据");
                            } else {
                                ui.add(Label::new(RichText::new(format!("错误: {}", self.error)).color(ui.visuals().warn_fg_color)));
                            }
                        });
                    } else {
                        ui.label(format!("{:?}", self.stock));
                    }
                }
            });
    }
    pub fn message_handler(&mut self, msg: Message) {
        match msg {
            Message::GotTradingHistory((symbol, data, error)) => {
                if symbol == self.stock.symbol {
                    self.data = data;
                    self.requesting = false;
                    self.error = error;
                }
            }
            _ => {}
        }
    }
}