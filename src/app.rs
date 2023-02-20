use crate::constants::REPAINT_AFTER_SECONDS;
use crate::financial_analysis::FinancialAnalysis;
use crate::run_mode::RunMode;
use egui::{CentralPanel, SidePanel, TopBottomPanel, Window};
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
                ui.label("主界面");
                ui.label(format!("stocks: {}, requesting: {}", self.stock_list.len(), self.stock_list_requesting));
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
