use egui::Window;
use crate::financial_analysis::FinancialAnalysis;
use crate::password::password;

impl FinancialAnalysis {
    pub fn login_window(&mut self, ctx: &egui::Context) {
        Window::new("账户登录").show(ctx, |ui| {
            egui::Grid::new("login_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("账户名");
                    ui.text_edit_singleline(&mut self.input_username);
                    ui.end_row();
                    ui.label("密码");
                    ui.add(password(&mut self.input_password));
                });
            ui.vertical_centered_justified(|ui| {
                if ui.button("登录").clicked() {}
                if ui.button("注册").clicked() {}
            });
        });
    }
}