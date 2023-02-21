use std::sync::mpsc;
use crate::frame_history::FrameHistory;
use crate::run_mode::RunMode;
use egui::{Direction, FontData, FontDefinitions, FontFamily, Label, Layout, Sense, Ui};
use egui_extras::{Column, TableBuilder};
#[cfg(not(target_arch = "wasm32"))]
use lazy_static::lazy_static;
use num_traits::Float;
use rpc::API_PORT;
use tracing::{info, warn};
use crate::message::{Channel, Message};
use crate::message::Message::ApiClientConnect;
use crate::service::Service;
use rpc::api::api_rpc_client::ApiRpcClient;
use rpc::api::{StockListResp, StockResp};
use tonic::Request;
use crate::stock_view::StockView;
use crate::utils::{execute, get_random_u32};

#[cfg(not(target_arch = "wasm32"))]
lazy_static! {
    pub static ref RT: std::sync::Arc<tokio::runtime::Runtime> = std::sync::Arc::new(tokio::runtime::Builder::new_multi_thread()
    .enable_io().build().unwrap());
}

#[cfg(not(target_arch = "wasm32"))]
pub type ApiChannel = tonic::transport::Channel;
#[cfg(target_arch = "wasm32")]
pub type ApiChannel = tonic_web_wasm_client::Client;

// pub type MainApiClient = rpc::api::api_rpc_client::ApiRpcClient<ApiChannel>;
pub type MainApiClient = ApiRpcClient<tonic::codegen::InterceptedService<ApiChannel, fn(tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status>>>;
pub type RegisterApiClient = rpc::api::register_client::RegisterClient<ApiChannel>;

pub type Token = String;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FinancialAnalysis {
    pub token: Token,
    #[serde(skip)]
    pub login_done: bool,
    #[serde(skip)]
    pub login_error: String,
    #[serde(skip)]
    pub run_mode: RunMode,
    #[serde(skip)]
    pub frame_history: FrameHistory,
    pub enable_debug_panel: bool,

    #[serde(skip)]
    pub channel: Option<Channel>,
    #[serde(skip)]
    pub loop_tx: Option<mpsc::Sender<Message>>,

    // login inputs
    pub input_username: String,
    #[serde(skip)]
    pub input_password: String,
    #[serde(skip)]
    pub client: Option<MainApiClient>,
    #[serde(skip)]
    pub stock_list: Vec<StockResp>,
    #[serde(skip)]
    pub stock_list_requesting: bool,
    #[serde(skip)]
    pub stock_list_select: Vec<StockResp>,
    #[serde(skip)]
    pub stock_list_select_text: String,

    pub search_text: String,
    #[serde(skip)]
    pub history_views: Vec<StockView>,

    #[serde(skip)]
    pub stock_list_popular: Vec<StockResp>,

    pub api_host: String,
}

impl Default for FinancialAnalysis {
    fn default() -> Self {
        Self {
            token: "".to_string(),
            login_done: false,
            login_error: "".to_string(),
            run_mode: Default::default(),
            frame_history: Default::default(),
            enable_debug_panel: true,
            channel: None,
            loop_tx: None,
            input_username: "test".to_string(),
            input_password: "test".to_string(),
            // client: None,
            client: None,
            stock_list: vec![],
            stock_list_requesting: false,
            stock_list_select: vec![],
            stock_list_select_text: "".to_string(),
            search_text: "".to_string(),
            history_views: vec![],
            stock_list_popular: vec![],
            api_host: "localhost".to_string(),
        }
    }
}

impl FinancialAnalysis {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // load chinese font
        let mut fonts = FontDefinitions::default();
        let font_name = "MiLanTing";
        fonts.font_data.insert(
            font_name.to_owned(),
            FontData::from_static(include_bytes!("../assets/MiLanTing.ttf")),
        );
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, font_name.to_owned());
        cc.egui_ctx.set_fonts(fonts);

        rust_i18n::set_locale("zh-CN");

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let def: FinancialAnalysis = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        // def.client = Some(crate::rpc_client::get_client());
        def.init()
    }
    pub fn refresh_client(&mut self) {
        // try to connect server
        let addr = format!("http://{}:{}", self.api_host, API_PORT);
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(tx) = self.loop_tx.clone() {
                RT.spawn(async move {
                    info!("preparing main api client...");
                    // let client = rpc::api::api_rpc_client::ApiRpcClient::new(tonic::transport::Endpoint::new(addr).unwrap().connect().await.unwrap());
                    let client: MainApiClient = rpc::api::api_rpc_client::ApiRpcClient::with_interceptor(
                        tonic::transport::Endpoint::new(addr).unwrap().connect().await.unwrap(),
                        // tonic::transport::Channel::from_static("http://127.0.0.1:51411").connect().await.unwrap(),
                        move |mut req: tonic::Request<()>| {
                            let token: tonic::metadata::MetadataValue<_> = "token".parse().unwrap();
                            req.metadata_mut().insert("authorization", token.clone());
                            Ok(req)
                        },
                    );
                    info!("got api client: {:?}", client);
                    tx.send(ApiClientConnect(client)).unwrap();
                });
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            // let client = rpc::api::api_rpc_client::ApiRpcClient::new(tonic_web_wasm_client::Client::new(addr));
            let inner = tonic_web_wasm_client::Client::new(addr);
            let client: MainApiClient = rpc::api::api_rpc_client::ApiRpcClient::with_interceptor(
                inner,
                move |mut req: tonic::Request<()>| {
                    let token: tonic::metadata::MetadataValue<_> = "token".parse().unwrap();
                    req.metadata_mut().insert("authorization", token.clone());
                    Ok(req)
                },
            );
            info!("got api client: {:?}", client);
            self.client = Some(client);
            self.load_stock_list();
        }
    }
    pub fn init(mut self) -> Self {
        let (channel_req_tx, channel_req_rx) = mpsc::channel();
        let (channel_resp_tx, channel_resp_rx) = mpsc::channel();

        // launch service
        Service::start(Channel {
            tx: channel_resp_tx.clone(),
            rx: channel_req_rx,
        });
        self.channel = Some(Channel {
            tx: channel_req_tx,
            rx: channel_resp_rx,
        });
        self.loop_tx = Some(channel_resp_tx.clone());
        self.refresh_client();
        // TODO: dynamic check token
        if !self.token.is_empty() {
            self.login_done = true;
        }
        self
    }

    pub fn message_handler(&mut self, msg: Message) {
        match msg {
            ApiClientConnect(client) => {
                info!("set client: {:?}", client);
                self.client = Some(client);
            }
            Message::LoginDone(token) => {
                info!("token: {:?}", token);
                self.login_done = true;
                self.token = token.into();
                self.load_stock_list();
            }
            Message::LoginError(reason) => {
                self.login_done = false;
                self.login_error = reason.into();
            }
            Message::GotStockList(stock) => {
                info!("GotStockList");
                self.stock_list = stock.data;
                self.stock_list_requesting = false;
                // randomly select some stocks to popular
                if self.stock_list.len() > 0 {
                    let n = 6;
                    let mut data = vec![];
                    for _ in 0..n {
                        data.push(self.stock_list.get(get_random_u32() as usize % self.stock_list.len()).unwrap().clone());
                    }
                    self.stock_list_popular = data;
                }
            }
            Message::GotTradingHistory(d) => {
                // dispatch messages
                let mut target = None;
                for view in &mut self.history_views {
                    if view.stock.symbol == d.0 {
                        target = Some(view);
                    }
                }
                if let Some(target) = target {
                    target.message_handler(Message::GotTradingHistory(d));
                }
            }
            Message::GotPredicts(d) => {
                // dispatch messages
                let mut target = None;
                for view in &mut self.history_views {
                    if view.stock.symbol == d.0 {
                        target = Some(view);
                    }
                }
                if let Some(target) = target {
                    target.message_handler(Message::GotPredicts(d));
                }
            }
            Message::GotStockIssue(d) => {
                // dispatch messages
                let mut target = None;
                for view in &mut self.history_views {
                    if view.stock.code == d.0 {
                        target = Some(view);
                    }
                }
                if let Some(target) = target {
                    target.message_handler(Message::GotStockIssue(d));
                }
            }
            Message::GotGuideLine(d) => {
                // dispatch messages
                let mut target = None;
                for view in &mut self.history_views {
                    if view.stock.code == d.0 {
                        target = Some(view);
                    }
                }
                if let Some(target) = target {
                    target.message_handler(Message::GotGuideLine(d));
                }
            }
            Message::GotIncomeAnalysis(d) => {
                // dispatch messages
                let mut target = None;
                for view in &mut self.history_views {
                    if view.stock.code == d.0 {
                        target = Some(view);
                    }
                }
                if let Some(target) = target {
                    target.message_handler(Message::GotIncomeAnalysis(d));
                }
            }
        }
    }
    pub fn stock_list(&self, ui: &mut Ui, data: &Vec<StockResp>, on_click: impl FnOnce(StockResp), expand: bool) {
        pub const SIGNAL_HEIGHT_DEFAULT: f32 = 30.0;
        let rect_max = ui.max_rect();
        let label_width = 64.0;
        let mut set_stock = None;
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            // .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .cell_layout(Layout::centered_and_justified(Direction::TopDown))
            .column(Column::exact(label_width).resizable(false))
            .column(Column::exact(label_width).resizable(false))
            .column(Column::exact(if expand { rect_max.width() - label_width - label_width } else { label_width }).resizable(false))
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
            .body(|body| {
                body.heterogeneous_rows((0..data.len()).map(|_| SIGNAL_HEIGHT_DEFAULT), |row_index, mut row| {
                    if let Some(stock) = data.get(row_index) {
                        let mut r = None;
                        let mut add_label = |text: &str, ui: &mut Ui| {
                            let resp = ui.add(Label::new(text).sense(Sense::click()));
                            if resp.double_clicked() {
                                r = Some(resp);
                            }
                        };
                        row.col(|ui| add_label(stock.code.as_str(), ui));
                        row.col(|ui| add_label(stock.symbol.as_str(), ui));
                        row.col(|ui| add_label(stock.name.as_str(), ui));
                        if let Some(r) = r {
                            if r.double_clicked() {
                                if !self.history_views.iter().any(|x| x.stock.symbol == stock.symbol) {
                                    set_stock = Some(stock.clone());
                                    // self.history_views.push(TradingHistoryView::new(stock.clone(), self.client.clone(), self.loop_tx.clone()));
                                }
                            }
                        }
                    }
                });
            });
        if let Some(stock) = set_stock {
            on_click(stock);
        }
    }
    pub fn stock_list_select_view(&mut self, ui: &mut Ui) {
        let mut set_stock = None;
        self.stock_list(ui, &self.stock_list_select, |stock| {
            set_stock = Some(stock);
        }, true);
        if let Some(stock) = set_stock {
            self.history_views.push(StockView::new(stock, self.client.clone(), self.loop_tx.clone()));
        }
    }
    pub fn stock_list_popular_view(&mut self, ui: &mut Ui) {
        ui.set_max_width(64.0 * 4.0);
        let mut set_stock = None;
        self.stock_list(ui, &self.stock_list_popular, |stock| {
            set_stock = Some(stock);
        }, false);
        if let Some(stock) = set_stock {
            self.history_views.push(StockView::new(stock, self.client.clone(), self.loop_tx.clone()));
        }
    }
    pub fn load_stock_list(&mut self) {
        if let Some(mut client) = self.client.clone() {
            let tx = self.loop_tx.as_ref().map(|x| x.clone());
            self.stock_list_requesting = true;
            execute(async move {
                info!("requesting stock list");
                let r = client.stock_list(Request::new(())).await;
                if let Ok(stock) = r {
                    let stock: StockListResp = stock.into_inner();
                    info!("got stock_list: {}", stock.data.len());
                    if let Some(tx) = tx {
                        tx.send(Message::GotStockList(stock)).unwrap();
                    }
                }
            });
        } else {
            warn!("no client when updating stock!");
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod test {
    use rpc::api::LoginRegisterRequest;
    use rpc::API_PORT;
    use tracing::info;
    use crate::financial_analysis::RegisterApiClient;

    #[test]
    fn client_native() {
        tracing_subscriber::fmt::init();
        tokio::runtime::Runtime::new().unwrap().block_on(async move {
            let addr = format!("http://127.0.0.1:{}", API_PORT);
            //  why it did not generate `connect`?
            let mut client: RegisterApiClient = rpc::api::register_client::RegisterClient::new(tonic::transport::Endpoint::new(addr).unwrap().connect().await.unwrap());
            info!("got client: {:?}", client);
            let _r = client.register(LoginRegisterRequest::default()).await.unwrap();
        });
    }
}