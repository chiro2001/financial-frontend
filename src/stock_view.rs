use std::ops::RangeInclusive;
use std::sync::mpsc;
use egui::{Align2, CentralPanel, Color32, ComboBox, DragValue, FontId, Grid, Label, Painter, pos2, Rect, RichText, Sense, TopBottomPanel, Ui, vec2, Widget, Window};
use rpc::api::{PredictRequest, StockIssueRequest, StockIssueResp, StockResp, TradingHistoryItem, TradingHistoryRequest, TradingHistoryType};
use tracing::{error, info};
use crate::constants::LINE_WIDTH;
use crate::financial_analysis::MainApiClient;
use crate::message::Message;
use crate::utils::{execute, get_text_size};

#[derive(Debug, Clone)]
pub struct TradingHistoryValueItem {
    pub date: String,
    pub open: f32,
    pub close: f32,
    pub high: f32,
    pub low: f32,
    pub volume: usize,
}

impl From<TradingHistoryItem> for TradingHistoryValueItem {
    fn from(value: TradingHistoryItem) -> Self {
        Self {
            date: value.date,
            open: value.open.parse().unwrap_or(-1.0),
            close: value.close.parse().unwrap_or(-1.0),
            high: value.high.parse().unwrap_or(-1.0),
            low: value.low.parse().unwrap_or(-1.0),
            volume: value.volume.parse().unwrap_or(0),
        }
    }
}

impl TradingHistoryValueItem {
    pub fn valid(&self) -> bool {
        self.volume != 0 && self.low > 0.0 && self.open > 0.0 && self.close > 0.0 && self.high > 0.0
            && self.high >= self.low
    }
    pub fn new(date: &str) -> Self {
        Self {
            date: date.to_string(),
            open: 0.0,
            close: 0.0,
            high: 0.0,
            low: 0.0,
            volume: 0,
        }
    }
    pub fn force_valid(&mut self) {
        if self.volume == 0 {
            self.volume = 1;
        }
        self.low = f32::min(f32::min(self.high, self.open), self.close);
        self.high = f32::max(f32::max(self.low, self.open), self.close);
    }
}

pub struct StockView {
    pub stock: StockResp,
    pub data: Vec<TradingHistoryValueItem>,
    pub client: Option<MainApiClient>,
    pub tx: Option<mpsc::Sender<Message>>,
    requesting: bool,
    error: String,
    pub typ: TradingHistoryType,
    pub valid: bool,
    pub predicts: Vec<TradingHistoryValueItem>,
    pub predict_len: u32,
    pub predicting: bool,
    predict_error: String,

    pub issue: Option<StockIssueResp>,
    requesting_issue: bool,
}

impl StockView {
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
            predicts: vec![],
            predict_len: 0,
            predicting: false,
            predict_error: "".to_string(),
            issue: None,
            requesting_issue: false,
        }
    }
    pub fn window(&mut self, ctx: &egui::Context) {
        if self.issue.is_none() && !self.requesting_issue {
            self.requesting_issue = true;
            let symbol = self.stock.symbol.to_string();
            let client = self.client.clone();
            let tx = self.tx.clone();
            execute(async move {
                if let Some(mut client) = client {
                    if let Some(tx) = tx {
                        let r = client.stock_issue(StockIssueRequest { symbol: symbol.clone() }).await;
                        if let Ok(r) = r {
                            let data = r.into_inner();
                            tx.send(Message::GotStockIssue((symbol, data, "".to_string()))).unwrap();
                        }
                    }
                }
            });
        }
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
        let mut valid = self.valid;
        Window::new(format!("[{}]{}", self.stock.code, self.stock.name))
            .open(&mut valid)
            .show(ctx, |ui| {
                ui.set_min_width(32.0);
                ui.set_min_height(64.0);
                if self.requesting {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("正在加载...");
                    });
                } else {
                    TopBottomPanel::top(format!("{}-banner", self.stock.symbol))
                        .resizable(false)
                        .show_inside(ui, |ui| {
                            ui.horizontal(|ui| {
                                let type_last = self.typ.clone();
                                ComboBox::new(format!("{}-combo-box", self.stock.symbol), "数据单位")
                                    .selected_text(match self.typ {
                                        TradingHistoryType::Daily => "日线",
                                        TradingHistoryType::Week => "周线",
                                        TradingHistoryType::Month => "月线",
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.style_mut().wrap = Some(false);
                                        ui.set_min_width(60.0);
                                        ui.selectable_value(&mut self.typ, TradingHistoryType::Daily, "日线");
                                        ui.selectable_value(&mut self.typ, TradingHistoryType::Week, "周线");
                                        ui.selectable_value(&mut self.typ, TradingHistoryType::Month, "月线");
                                    });
                                if type_last != self.typ {
                                    // change request option, reload
                                    self.requesting = false;
                                    self.error.clear();
                                    self.data.clear();
                                }
                                ui.label("预测新数据范围");
                                ui.add_enabled_ui(!self.predicting, |ui| {
                                    DragValue::new(&mut self.predict_len)
                                        .clamp_range(0..=(self.data.len() / 4))
                                        .ui(ui);
                                    ui.add_enabled_ui(self.predict_len != 0, |ui| {
                                        if ui.button(if self.predicting { "正在预测" } else { "预测" }).clicked() {
                                            let tx = self.tx.clone();
                                            let client = self.client.clone();
                                            let length = self.predict_len;
                                            self.predicting = true;
                                            let raw_data = self.data.clone();
                                            let symbol = self.stock.symbol.to_string();
                                            execute(async move {
                                                if let Some(tx) = tx {
                                                    if let Some(client) = &client {
                                                        let clients = (0..4).map(|_| client.clone()).collect::<Vec<_>>();
                                                        let data_list: Vec<Vec<f32>> = vec![
                                                            raw_data.iter().map(|x| x.high).collect(),
                                                            raw_data.iter().map(|x| x.low).collect(),
                                                            raw_data.iter().map(|x| x.open).collect(),
                                                            raw_data.iter().map(|x| x.close).collect(),
                                                        ];
                                                        let mut results: Vec<Vec<f32>> = (0..4).map(|_| vec![]).collect();
                                                        let mut errors = vec![];
                                                        let mut features = vec![];
                                                        for (i, (data, mut client)) in data_list.into_iter().zip(clients.into_iter()).enumerate() {
                                                            info!("requesting new predict... {}/4", i);
                                                            let r = async move {
                                                                client.predict_data(PredictRequest { data, length }).await
                                                            };
                                                            features.push((i, r));
                                                        }
                                                        #[cfg(not(target_arch = "wasm32"))]
                                                            let features = features.into_iter().map(|r|
                                                            (r.0, tokio::spawn(r.1))
                                                        ).collect::<Vec<_>>();
                                                        for (i, r) in features {
                                                            let r = r.await;
                                                            match r {
                                                                Ok(r) => match r {
                                                                    Ok(r) => {
                                                                        results[i] = r.into_inner().data;
                                                                    }
                                                                    Err(e) => {
                                                                        errors.push(e.to_string())
                                                                    }
                                                                },
                                                                Err(e) => errors.push(e.to_string()),
                                                            }
                                                        }
                                                        let len = results.iter().map(|x| x.len()).min();
                                                        if results.len() != 4 || len.is_none() || len.unwrap_or(0) == 0 {
                                                            let e = format!("Errors: {:?}", errors);
                                                            error!("{}", e);
                                                            tx.send(Message::GotPredicts((symbol, vec![], e))).unwrap();
                                                        } else {
                                                            info!("got {:?} predicts", len);
                                                            let mut predicts = vec![];
                                                            for i in 0..len.unwrap() {
                                                                let mut p = TradingHistoryValueItem::new("");
                                                                p.high = results[0][i];
                                                                p.low = results[1][i];
                                                                p.open = results[2][i];
                                                                p.close = results[3][i];
                                                                p.volume = 1;
                                                                predicts.push(p);
                                                            }
                                                            tx.send(Message::GotPredicts((symbol, predicts, "".to_string()))).unwrap();
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                        if self.predicting {
                                            ui.spinner();
                                        }
                                    });
                                });
                            });
                        });
                    TopBottomPanel::bottom(format!("{}-history", self.stock.symbol))
                        // .resizable(true)
                        // .min_height(64.0)
                        .resizable(false)
                        .show_inside(ui, |ui| {
                            Grid::new(format!("{}-info-grid", self.stock.symbol))
                                .num_columns(2)
                                .spacing([40.0, 4.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    if let Some(issue) = &self.issue {
                                        ui.label("上市地");
                                        ui.label(issue.market.as_str());
                                        ui.end_row();
                                        ui.label("主承销商");
                                        ui.label(issue.consignee.as_str());
                                        ui.end_row();
                                        ui.label("承销方式");
                                        ui.label(issue.underwriting.as_str());
                                        ui.end_row();
                                        ui.label("上市推荐人");
                                        ui.label(issue.sponsor.as_str());
                                        ui.end_row();
                                        ui.label("每股发行价(元)");
                                        ui.label(issue.issue_price.as_str());
                                        ui.end_row();
                                        ui.label("发行方式");
                                        ui.label(issue.issue_mode.as_str());
                                        ui.end_row();
                                        ui.label("发行市盈率（按发行后总股本）");
                                        ui.label(issue.issue_pe.as_str());
                                        ui.end_row();
                                        ui.label("首发前总股本（万股）");
                                        ui.label(issue.pre_capital.as_str());
                                        ui.end_row();
                                        ui.label("首发后总股本（万股）");
                                        ui.label(issue.capital.as_str());
                                        ui.end_row();
                                        ui.label("实际发行量（万股）");
                                        ui.label(issue.issue_volume.as_str());
                                        ui.end_row();
                                        ui.label("预计募集资金（万元）");
                                        ui.label(issue.expected_fundraising.as_str());
                                        ui.end_row();
                                        ui.label("最实际募集资金合计（万元）");
                                        ui.label(issue.fundraising.as_str());
                                        ui.end_row();
                                        ui.label("发行费用总额（万元）");
                                        ui.label(issue.issue_cost.as_str());
                                        ui.end_row();
                                        ui.label("募集资金净额（万元）");
                                        ui.label(issue.net_amount_raised.as_str());
                                        ui.end_row();
                                        ui.label("承销费用（万元）");
                                        ui.label(issue.underwriting_fee.as_str());
                                        ui.end_row();
                                        ui.label("招股公告日");
                                        ui.label(issue.announcement_date.as_str());
                                        ui.end_row();
                                        ui.label("上市日期");
                                        ui.label(issue.launch_date.as_str());
                                        ui.end_row();
                                    }
                                });
                        });
                    CentralPanel::default().show_inside(ui, |ui| {
                        if self.data.is_empty() {
                            ui.centered_and_justified(|ui| {
                                if self.error.is_empty() {
                                    ui.label("无数据");
                                } else {
                                    ui.add(Label::new(RichText::new(format!("错误: {}", self.error)).color(ui.visuals().warn_fg_color)));
                                }
                            });
                        } else {
                            // ui.label(format!("{:?}", self.stock));
                            self.paint_data(ui);
                        }
                    });
                }
            });
        self.valid = valid;
    }
    fn paint_item(rect: Rect, ui: &Ui, painter: &Painter, item: &TradingHistoryValueItem, allow_invalid: bool) {
        if !item.valid() && !allow_invalid {
            painter.text(rect.center(), Align2::CENTER_CENTER, "无效数据", Default::default(), ui.visuals().text_color());
            return;
        }
        // a <= b
        let (a, b, increase) = if item.open <= item.close {
            (item.open, item.close, true)
        } else {
            (item.close, item.open, false)
        };
        painter.vline(rect.center_top().x, rect.y_range(), (LINE_WIDTH, ui.visuals().text_color()));
        let y_top = rect.top() + (rect.height() * (item.high - b) / (item.high - item.low));
        let y_bottom = y_top + (rect.height() * (b - a) / (item.high - item.low));
        painter.rect_filled(Rect::from_x_y_ranges(rect.x_range(), RangeInclusive::new(y_top, y_bottom)), 0.0,
                            if increase { Color32::RED } else { Color32::GREEN });
    }
    fn paint_data(&mut self, ui: &mut Ui) {
        let len_data = self.data.len() + self.predicts.len();
        if len_data == 0 { return; }
        let font: FontId = Default::default();
        let text_height = get_text_size(ui, "T", font.clone()).y;
        let rect_max = ui.available_rect_before_wrap();
        let rect_data_max = Rect::from_x_y_ranges(rect_max.x_range(), RangeInclusive::new(rect_max.top(), rect_max.bottom() - text_height));
        let width = rect_data_max.width() / len_data as f32;
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::hover());
        let value_max = self.data.iter().map(|x| x.high).reduce(|a, b| if a > b { a } else { b }).unwrap_or(1.0);
        let value_min = self.data.iter().map(|x| x.low).reduce(|a, b| if a < b { a } else { b }).unwrap_or(0.0);
        let value_range = value_max - value_min;
        let height = rect_data_max.height();
        let mut last_date_rect: Option<Rect> = None;
        for i in 0..len_data {
            let item = self.data.get(i);
            let item = if item.is_none() {
                self.predicts.get(i - self.data.len())
            } else {
                item
            };
            if item.is_none() {
                continue;
            }
            let mut item = item.unwrap().clone();
            item.force_valid();
            let p = i as f32;
            let range_x = RangeInclusive::new(rect_data_max.left() + p * width, rect_data_max.left() + (p + 1.0) * width);
            let rect = Rect::from_x_y_ranges(
                range_x.clone(),
                RangeInclusive::new(rect_data_max.top() + height * (value_max - item.high) / value_range, rect_data_max.top() + height * (value_max - item.low) / value_range));
            Self::paint_item(rect, ui, &painter, &item, i >= self.data.len());
            if let Some(pos) = response.hover_pos() {
                if range_x.contains(&pos.x) {
                    painter.text(pos - vec2(0.0, text_height * 2.0), Align2::RIGHT_BOTTOM, item.date.as_str(), font.clone(), ui.visuals().strong_text_color());
                    painter.text(pos - vec2(0.0, text_height * 0.0), Align2::RIGHT_BOTTOM, format!("收盘{}", item.close), font.clone(), ui.visuals().strong_text_color());
                    painter.text(pos - vec2(0.0, text_height * 1.0), Align2::RIGHT_BOTTOM, format!("开盘{}", item.open), font.clone(), ui.visuals().strong_text_color());
                }
            }
            let paint_date = |color: Color32|
                painter.text(
                    pos2(rect.center_bottom().x, rect_data_max.bottom()),
                    Align2::CENTER_TOP,
                    format!("  {}  ", item.date),
                    font.clone(), color);
            let date_rect = paint_date(Color32::TRANSPARENT);
            let real_paint =
                if let Some(last_date_rect) = last_date_rect {
                    !last_date_rect.intersects(date_rect)
                } else {
                    true
                };
            if real_paint {
                paint_date(ui.visuals().text_color());
                last_date_rect = Some(date_rect);
            }
        }
    }
    pub fn message_handler(&mut self, msg: Message) {
        match msg {
            Message::GotTradingHistory((symbol, data, error)) => {
                if symbol == self.stock.symbol {
                    self.data = data.into_iter().map(|x| x.into()).collect();
                    self.requesting = false;
                    self.error = error;
                }
            }
            Message::GotPredicts((symbol, data, error)) => {
                if symbol == self.stock.symbol {
                    info!("{} set predicts", symbol);
                    self.predicts = data;
                    self.predicting = false;
                    self.predict_error = error;
                }
            }
            Message::GotStockIssue((symbol, data, _error)) => {
                if symbol == self.stock.symbol {
                    info!("{} set issue", symbol);
                    self.issue = Some(data);
                    self.requesting_issue = false;
                }
            }
            _ => {}
        }
    }
}