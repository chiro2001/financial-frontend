use crate::financial_analysis::FinancialAnalysis;
use crate::password::password;
use egui::Window;
use rpc::api::RegisterRequest;
use crate::utils::execute;

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
                if ui.button("注册").clicked() {
                    // execute(async {
                    //     self.register();
                    // })
                    tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(self.register());

                    // if let Some(client) = &mut self.client {
                    //     // execute(async move {
                    //     //     let r = client.register(RegisterRequest { username: "".to_string(), password: "".to_string() }).await;
                    //     // });
                    //     // let mut client = client.clone();
                    //     let r = client.register(RegisterRequest { username: "".to_string(), password: "".to_string() });
                    //     // std::thread::spawn(move || futures::executor::block_on(async move {
                    //     //     let r = r.await;
                    //     // }));
                    //     // tokio::spawn(async {
                    //     //     // let r = client.register(RegisterRequest { username: "".to_string(), password: "".to_string() });
                    //     //     let r = r.await;
                    //     // });
                    //     // execute(async move {
                    //     //     let r = r.await;
                    //     // });
                    // }
                }
            });
        });
    }
    pub async fn register(&self) {
        if let Some(client) = &self.client {
            let mut client = client.clone();
            let r = client.register(RegisterRequest { username: "".to_string(), password: "".to_string() }).await;
        }
    }
}
