use std::ops::RangeInclusive;
use std::sync::mpsc;
use egui::{Align2, Color32, ComboBox, FontId, Label, Painter, pos2, Rect, RichText, Sense, Ui, vec2, Window};
use rpc::api::{StockResp, TradingHistoryItem, TradingHistoryRequest, TradingHistoryType};
use tracing::{error, info};
use crate::constants::LINE_WIDTH;
use crate::financial_analysis::MainApiClient;
use crate::message::Message;
use crate::utils::{execute, get_text_size};

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
}

pub struct TradingHistoryView {
    pub stock: StockResp,
    pub data: Vec<TradingHistoryValueItem>,
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
                }
            });
        self.valid = valid;
    }
    fn paint_item(rect: Rect, ui: &Ui, painter: &Painter, item: &TradingHistoryValueItem) {
        if !item.valid() {
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
    fn paint_data(&self, ui: &mut Ui) {
        let len_data = self.data.len();
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
            let item = self.data.get(i).unwrap();
            let p = i as f32;
            let range_x = RangeInclusive::new(rect_data_max.left() + p * width, rect_data_max.left() + (p + 1.0) * width);
            let rect = Rect::from_x_y_ranges(
                range_x.clone(),
                RangeInclusive::new(rect_data_max.top() + height * (value_max - item.high) / value_range, rect_data_max.top() + height * (value_max - item.low) / value_range));
            Self::paint_item(rect, ui, &painter, item);
            if let Some(pos) = response.hover_pos() {
                if range_x.contains(&pos.x) {
                    painter.text(pos, Align2::RIGHT_BOTTOM, item.date.as_str(), font.clone(), ui.visuals().strong_text_color());
                    painter.text(pos - vec2(0.0, text_height * 1.0), Align2::RIGHT_BOTTOM, format!("收盘{}", item.close), font.clone(), ui.visuals().strong_text_color());
                    painter.text(pos - vec2(0.0, text_height * 2.0), Align2::RIGHT_BOTTOM, format!("开盘{}", item.open), font.clone(), ui.visuals().strong_text_color());
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
            _ => {}
        }
    }
}