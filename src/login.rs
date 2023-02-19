use crate::financial_analysis::FinancialAnalysis;
use crate::password::password;
use egui::Window;
use rpc::api::{LoginRegisterRequest, ReasonResp};
use rpc::API_PORT;
use tracing::{error, info};
use crate::message::Message::LoginDone;
use crate::utils::{block_on, execute};

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
                if ui.button("登录").clicked() {
                    // block_on(self.login());
                    if let Some(client) = &self.client {
                        let mut client = client.clone();
                        let tx = self.channel.as_ref().map(|x| x.tx.clone());
                        execute(async move {
                            info!("login, client: {:?}", client);
                            let r = client.login(LoginRegisterRequest { username: "".to_string(), password: "".to_string() }).await;
                            info!("login resp: {:?}", r);
                            match r {
                                Ok(r) => {
                                    let data = r.into_inner();
                                    if data.err {
                                        error!("{}", data.reason);
                                    } else {
                                        if let Some(tx) = tx {
                                            tx.send(LoginDone("".to_string())).unwrap();
                                        }
                                    }
                                }
                                Err(e) => error!("{}", e.to_string())
                            }
                        });
                    }
                }
                if ui.button("注册").clicked() {
                    block_on(self.register());
                }
            });
        });
    }
    pub async fn register(&self) {
        let addr = format!("http://127.0.0.1:{}", API_PORT);
        let res =
            match if cfg!(target_arch = "wasm32") {
                use tonic_web_wasm_client::Client;
                let mut client = rpc::api::register_client::RegisterClient::new(Client::new(addr));
                client.register(LoginRegisterRequest { username: "".to_string(), password: "".to_string() }).await
            } else {
                let mut client = rpc::api::register_client::RegisterClient::new(tonic::transport::Endpoint::new(addr).unwrap().connect().await.unwrap());
                client.register(LoginRegisterRequest { username: "".to_string(), password: "".to_string() }).await
            } {
                Ok(r) => r,
                Err(e) => {
                    error!("{}", e);
                    tonic::Response::new(ReasonResp { err: true, reason: e.to_string() })
                }
            };
        let data = res.into_inner();
        info!("register resp: {:?}", data);
    }
}
