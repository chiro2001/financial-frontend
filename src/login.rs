use crate::financial_analysis::FinancialAnalysis;
use crate::password::password;
use egui::{RichText, Window};
use rpc::api::{LoginRegisterRequest, ReasonResp};
use rpc::API_PORT;
use tracing::{error, info};
use crate::message::Message::{LoginDone, LoginError};
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
            if !self.login_error.is_empty() {
                ui.label(RichText::new(format!("登录失败：{}", self.login_error)).color(ui.visuals().warn_fg_color));
            }
            ui.vertical_centered_justified(|ui| {
                if ui.button("登录").clicked() {
                    // block_on(self.login());
                    if let Some(client) = &self.client {
                        let mut client = client.clone();
                        let username = self.input_username.clone();
                        let password = self.input_password.clone();
                        let tx = self.loop_tx.as_ref().map(|x| x.clone());
                        execute(async move {
                            info!("login, client: {:?}", client);
                            let r = client.login(LoginRegisterRequest { username, password }).await;
                            info!("login resp: {:?}", r);
                            match r {
                                Ok(r) => {
                                    let data = r.into_inner();
                                    if let Some(tx) = tx {
                                        if data.err {
                                            error!("{}", data.reason);
                                            tx.send(LoginError(data.reason)).unwrap();
                                        } else {
                                            tx.send(LoginDone(data.token)).unwrap();
                                        }
                                    }
                                }
                                Err(e) => error!("{}", e.to_string())
                            }
                        });
                    }
                }
                if ui.button("注册").clicked() {
                    let username = self.input_username.to_string();
                    let password = self.input_password.to_string();
                    block_on(Self::register(username, password));
                }
            });
        });
    }
    pub async fn register(username: String, password: String) {
        let addr = format!("http://127.0.0.1:{}", API_PORT);
        let res =
            match if cfg!(target_arch = "wasm32") {
                use tonic_web_wasm_client::Client;
                let mut client = rpc::api::register_client::RegisterClient::new(Client::new(addr));
                client.register(LoginRegisterRequest { username, password }).await
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let mut client = rpc::api::register_client::RegisterClient::new(tonic::transport::Endpoint::new(addr).unwrap().connect().await.unwrap());
                    client.register(LoginRegisterRequest { username, password }).await
                }
                #[cfg(target_arch = "wasm32")]
                {
                    panic!("unsupported")
                }
            } {
                Ok(r) => r.into_inner(),
                Err(e) => {
                    error!("{}", e);
                    ReasonResp { err: true, reason: e.to_string() }
                }
            };
        info!("register resp: {:?}", res);
    }
}
