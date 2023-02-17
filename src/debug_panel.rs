use crate::financial_analysis::FinancialAnalysis;
use crate::run_mode::RunMode;
use egui::Ui;

impl FinancialAnalysis {
    pub fn debug_panel(&mut self, ui: &mut Ui) {
        let run_mode = &mut self.run_mode;
        ui.label(t!("debug.mode"));
        ui.radio_value(run_mode, RunMode::Reactive, t!("debug.reactive.text"))
            .on_hover_text(t!("debug.reactive.hover"));
        ui.radio_value(run_mode, RunMode::Continuous, t!("debug.continuous.text"))
            .on_hover_text(t!("debug.continuous.hover"));
        if self.run_mode == RunMode::Continuous {
            ui.label(t!(
                "debug.fps",
                fps = format!("{:.1}", self.frame_history.fps()).as_str()
            ));
        } else {
            self.frame_history.ui(ui);
        }
        let mut debug_on_hover = ui.ctx().debug_on_hover();
        ui.checkbox(
            &mut debug_on_hover,
            format!("🐛 {}", t!("debug.debug_mode")),
        );
        ui.ctx().set_debug_on_hover(debug_on_hover);
        ui.horizontal(|ui| {
            if ui
                .button(t!("debug.reset_egui.text"))
                .on_hover_text(t!("debug.reset_egui.hover"))
                .clicked()
            {
                ui.ctx().memory_mut(|mem| *mem = Default::default());
            }

            if ui.button(t!("debug.reset_everything")).clicked() {
                ui.ctx().memory_mut(|mem| *mem = Default::default());
            }
        });
        egui::warn_if_debug_build(ui);
    }
}
