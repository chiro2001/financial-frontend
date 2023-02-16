use egui::{CentralPanel, SidePanel, TopBottomPanel};
use crate::constants::REPAINT_AFTER_SECONDS;
use crate::financial_analysis::FinancialAnalysis;
use crate::run_mode::RunMode;

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
        TopBottomPanel::top("global_menu").show(ctx, |ui| {
            ui.checkbox(&mut self.enable_debug_panel, "调试面板");
        });
        if self.enable_debug_panel {
            SidePanel::left("debug_panel").show(ctx, |ui| {
                self.debug_panel(ui);
            });
        }
        CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(self.login_done, |ui| {
                ui.label("主界面");
            });
        });
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}
