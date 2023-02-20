use crate::constants::REPAINT_AFTER_SECONDS;
use crate::financial_analysis::FinancialAnalysis;
use crate::run_mode::RunMode;
use egui::{CentralPanel, Direction, Layout, SidePanel, TopBottomPanel, Window};
use egui_extras::{Column, TableBuilder};
use num_traits::Float;
use regex::Regex;
use rpc::api::StockListResp;
use tonic::Request;
use tracing::info;
use crate::message::Message;
use crate::utils::execute;

impl eframe::App for FinancialAnalysis {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.frame_history
            .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
        match self.run_mode {
            RunMode::Continuous => {
                // Tell the backend to repaint as soon as possible
                ctx.request_repaint();
            }
            RunMode::Reactive => {
                // let the computer rest for a bit
                ctx.request_repaint_after(std::time::Duration::from_secs_f32(
                    REPAINT_AFTER_SECONDS,
                ));
            }
        }

        if let Some(channel) = &self.channel {
            let mut messages = vec![];
            while let Ok(rx) = channel.rx.try_recv() {
                messages.push(rx);
            }
            for rx in messages {
                self.message_handler(rx);
            }
        }
        if !self.stock_list_requesting && self.stock_list.is_empty() && !self.token.is_empty() {
            if let Some(mut client) = self.client.clone() {
                let tx = self.loop_tx.as_ref().map(|x| x.clone());
                self.stock_list_requesting = true;
                execute(async move {
                    let r = client.stock_list(Request::new(())).await;
                    if let Ok(stock) = r {
                        let stock: StockListResp = stock.into_inner();
                        info!("got stock_list: {}", stock.data.len());
                        if let Some(tx) = tx {
                            tx.send(Message::GotStockList(stock)).unwrap();
                        }
                    }
                });
            }
        }

        TopBottomPanel::top("global_menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.checkbox(&mut self.enable_debug_panel, "调试面板");
            });
        });
        if self.enable_debug_panel {
            SidePanel::left("debug_panel").show(ctx, |ui| {
                self.debug_panel(ui);
            });
        }
        CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(self.login_done && !self.token.is_empty(), |ui| {
                // ui.label("主界面");
                // ui.label(format!("stocks: {}, requesting: {}", self.stock_list.len(), self.stock_list_requesting));
                TopBottomPanel::top("search-result").show_inside(ui, |ui| {
                    // ui.centered_and_justified(|ui| {
                    //     ui.vertical_centered_justified(|ui| {
                    //         ui.with_layout(Layout::right_to_left(Align::Center).with_main_justify(true), |ui| {
                    // ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("🔍搜索");
                        if ui.text_edit_singleline(&mut self.search_text).changed() || self.search_text != self.stock_list_select_text
                            || (self.stock_list_select_text.is_empty() && !self.stock_list.is_empty() && self.stock_list_select.is_empty()) {
                            let filter =
                                if let Ok(re) = Regex::new(self.search_text.as_str()) {
                                    let re = re.clone();
                                    Some(move |text: &str| {
                                        re.is_match(text)
                                    })
                                } else { None };
                            let stock_list_select = self.stock_list.iter().filter(|s| {
                                if let Some(filter) = &filter {
                                    filter(&s.code) ||
                                        filter(&s.symbol) ||
                                        filter(&s.name)
                                } else {
                                    let filter = |text: &str| self.search_text.contains(text);
                                    filter(&s.code) ||
                                        filter(&s.symbol) ||
                                        filter(&s.name)
                                }
                            });
                            self.stock_list_select = stock_list_select.map(|x| x.clone()).collect();
                            self.stock_list_select_text = self.search_text.to_string();
                        }
                    });
                    //         });
                    //     });
                    // });
                });
                CentralPanel::default().show_inside(ui, |ui| {
                    pub const SIGNAL_HEIGHT_DEFAULT: f32 = 30.0;
                    let rect_max = ui.max_rect();
                    let label_width = 64.0;
                    let table = TableBuilder::new(ui)
                        .striped(true)
                        .resizable(false)
                        // .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .cell_layout(Layout::centered_and_justified(Direction::TopDown))
                        .column(Column::exact(label_width).resizable(false))
                        .column(Column::exact(label_width).resizable(false))
                        .column(Column::exact(rect_max.width() - label_width - label_width).resizable(false))
                        .min_scrolled_height(0.0)
                        .max_scroll_height(f32::infinity());
                    table.header(SIGNAL_HEIGHT_DEFAULT, |mut header| {
                        header.col(|ui| {
                            ui.label("代码");
                        });
                        header.col(|ui| {
                            ui.label("代号");
                        });
                        header.col(|ui| {
                            ui.label("名称");
                        });
                    })
                        .body(|mut body| {
                            for stock in &self.stock_list_select {
                                body.row(SIGNAL_HEIGHT_DEFAULT, |mut row| {
                                    row.col(|ui| {
                                        ui.label(stock.code.to_string());
                                    });
                                    row.col(|ui| {
                                        ui.label(stock.symbol.to_string());
                                    });
                                    row.col(|ui| {
                                        ui.label(stock.name.to_string());
                                    });
                                });
                            }
                        });
                });
            });
        });
        if !self.login_done {
            if self.client.is_some() {
                self.login_window(ctx);
            } else {
                Window::new("连接中...")
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("正在连接后端...");
                            ui.spinner();
                        });
                    });
            }
        }
        if self.stock_list_requesting {
            Window::new("加载数据...")
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("正在拉取最新信息...");
                        ui.spinner();
                    });
                });
        }
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
