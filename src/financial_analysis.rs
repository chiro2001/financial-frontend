use std::sync::mpsc;
use crate::frame_history::FrameHistory;
use crate::run_mode::RunMode;
use egui::{FontData, FontDefinitions, FontFamily};
#[cfg(not(target_arch = "wasm32"))]
use lazy_static::lazy_static;
use rpc::API_PORT;
use tracing::info;
use crate::message::{Channel, Message};
use crate::message::Message::ApiClientConnect;
use crate::service::Service;
use rpc::api::api_rpc_client::ApiRpcClient;
use rpc::api::StockResp;
use crate::trading_history::TradingHistoryView;

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
    pub history_views: Vec<TradingHistoryView>,
}

impl Default for FinancialAnalysis {
    fn default() -> Self {
        Self {
            token: "".to_string(),
            login_done: false,
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
        // try to connect server
        let addr = format!("http://127.0.0.1:{}", API_PORT);
        #[cfg(not(target_arch = "wasm32"))]
        {
            let tx = channel_resp_tx;
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
                // let r = client.login(rpc::api::LoginRegisterRequest { username: "".to_string(), password: "".to_string() }).await;
                // info!("r: {:?}", r);
                // let client = rpc::api::api_rpc_client::ApiRpcClient::new(tonic::transport::Endpoint::new(addr).unwrap().connect().await.unwrap());
                info!("got api client: {:?}", client);
                tx.send(ApiClientConnect(client)).unwrap();
            });
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
        }
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
            }
            Message::GotStockList(stock) => {
                self.stock_list = stock.data;
                self.stock_list_requesting = false;
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
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod test {
    use rpc::api::LoginRegisterRequest;
    use rpc::api::register_client::RegisterClient;
    use rpc::API_PORT;
    use tonic::transport::Channel;
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