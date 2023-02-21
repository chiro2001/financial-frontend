use crate::constants::{AVAILABLE_API_HOSTS, REPAINT_AFTER_SECONDS};
use crate::financial_analysis::FinancialAnalysis;
use crate::run_mode::RunMode;
use egui::{CentralPanel, ComboBox, Label, RichText, SidePanel, TopBottomPanel, Window};
use regex::Regex;

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
        if !self.stock_list_requesting && self.client.is_some() && self.stock_list.is_empty() && !self.token.is_empty() {
            self.load_stock_list();
        }

        TopBottomPanel::top("global_menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                egui::widgets::global_dark_light_mode_switch(ui);
                ui.checkbox(&mut self.enable_debug_panel, "调试面板");
                ui.add_enabled_ui(!self.token.is_empty(), |ui| {
                    if ui.button("退出登录").clicked() {
                        self.token.clear();
                        self.login_done = false;
                    }
                });
                if ui.button("重新连接").clicked() {
                    self.refresh_client();
                }
                let last_api = self.api_host.clone();
                ComboBox::new("api-select", "远程服务器地址")
                    .selected_text(&self.api_host)
                    .show_ui(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        for api in AVAILABLE_API_HOSTS {
                            ui.selectable_value(&mut self.api_host, api.to_string(), api.to_string());
                        }
                    });
                if last_api != self.api_host {
                    self.refresh_client();
                }
            });
        });
        if self.enable_debug_panel {
            SidePanel::left("debug_panel").show(ctx, |ui| {
                self.debug_panel(ui);
            });
        }
        let enabled = self.login_done && !self.token.is_empty();
        CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(enabled, |ui| {
                TopBottomPanel::top("search-result").show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("🔍搜索");
                        let re = Regex::new(self.search_text.as_str());
                        if !self.stock_list.is_empty() && (ui.text_edit_singleline(&mut self.search_text).changed() || self.search_text != self.stock_list_select_text
                            || (self.stock_list_select_text.is_empty() && self.search_text != self.stock_list_select_text)) {
                            let filter =
                                if let Ok(re) = re.clone() {
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
                        ui.add(Label::new(RichText::new(if self.search_text.is_empty() {
                            "支持正则表达式检索"
                        } else {
                            if re.is_ok() {
                                "正确的正则表达式"
                            } else {
                                "无效的正则表达式"
                            }
                        }).color(ui.visuals().weak_text_color())))
                    });
                });
                CentralPanel::default().show_inside(ui, |ui| {
                    SidePanel::left("popular-stocks")
                        .resizable(false)
                        .show_inside(ui, |ui| {
                            ui.heading("热门股票");
                            self.stock_list_popular_view(ui);
                        });
                    CentralPanel::default().show_inside(ui, |ui| {
                        self.stock_list_select_view(ui);
                    });
                });
            });
            // ui.label(format!("windows: {}", self.history_views.len()));
        });
        if !self.login_done {
            if self.client.is_some() {
                self.login_window(ctx);
            }
        }
        if self.client.is_none() {
            Window::new("连接中...")
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("正在连接后端...");
                        ui.spinner();
                    });
                });
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
        // remove invalid windows inplace
        self.history_views.retain(|x| x.valid);
        if enabled {
            for view in &mut self.history_views {
                if view.valid { view.window(ctx); }
            }
        }
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
