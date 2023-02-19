use crate::financial_analysis::FinancialAnalysis;
use crate::password::password;
use egui::Window;
use rpc::api::LoginRegisterRequest;
use crate::utils::block_on;

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
                    block_on(self.register());
                }
            });
        });
    }
    pub async fn login(&self) {
        if let Some(client) = &self.client {
            let mut client = client.clone();
            let _r = client.login(LoginRegisterRequest { username: "".to_string(), password: "".to_string() }).await;
        }
    }
    pub async fn register(&self) {
        let addr = format!("http://127.0.0.1:{}", 51411);
        if cfg!(target_arch = "wasm32") {
            use tonic_web_wasm_client::Client;
            let mut client = rpc::api::register_client::RegisterClient::new(Client::new(addr));
            let _r = client.register(LoginRegisterRequest { username: "".to_string(), password: "".to_string() }).await;
        } else {
            let mut client = rpc::api::register_client::RegisterClient::new(tonic::transport::Endpoint::new(addr).unwrap().connect().await.unwrap());
            let _r = client.register(LoginRegisterRequest { username: "".to_string(), password: "".to_string() }).await;
        }
    }
}
