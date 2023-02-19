use std::sync::mpsc;
use crate::frame_history::FrameHistory;
use crate::run_mode::RunMode;
use egui::{FontData, FontDefinitions, FontFamily};
use tracing::info;
use crate::message::{Channel, Message};
use crate::message::Message::ApiClientConnect;
use crate::service::Service;
use crate::utils::execute;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct FinancialAnalysis {
    pub token: String,
    #[serde(skip)]
    pub login_done: bool,
    #[serde(skip)]
    pub run_mode: RunMode,
    #[serde(skip)]
    pub frame_history: FrameHistory,
    pub enable_debug_panel: bool,

    #[serde(skip)]
    pub channel: Option<Channel>,

    // login inputs
    pub input_username: String,
    #[serde(skip)]
    pub input_password: String,
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    pub client: Option<rpc::api::api_rpc_client::ApiRpcClient<tonic_web_wasm_client::Client>>,
    #[cfg(not(target_arch = "wasm32"))]
    #[serde(skip)]
    // pub client: Option<rpc::api::api_rpc_client::ApiRpcClient<tonic::client::Grpc<>>>,
    pub client: Option<rpc::api::api_rpc_client::ApiRpcClient<tonic_web_wasm_client::Client>>,
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
            input_username: "test".to_string(),
            input_password: "test".to_string(),
            // client: None,
            client: None,
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
        // try to connect server
        let tx = channel_resp_tx;
        execute(async move {
            use tonic_web_wasm_client::Client;
            let base_url = format!("http://127.0.0.1:{}", 51411);
            let query_client = rpc::api::api_rpc_client::ApiRpcClient::new(Client::new(base_url));
            // let response = query_client.register(RegisterRequest { username: "".to_string(), password: "".to_string() }).await;
            tx.send(ApiClientConnect(query_client)).unwrap();
        });
        self
    }

    pub fn message_handler(&mut self, msg: Message) {
        match msg {
            ApiClientConnect(client) => {
                info!("set client: {:?}", client);
                self.client = Some(client);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use rpc::api::RegisterRequest;
    use tracing::info;

    #[test]
    fn client_native() {
        tracing_subscriber::fmt::init();
        tokio::runtime::Runtime::new().unwrap().block_on(async move {
            let addr = "http://127.0.0.1:51411";
            //  why it did not generate `connect`?
            let mut client = rpc::api::api_rpc_client::ApiRpcClient::new(tonic::transport::Endpoint::new(addr).unwrap().connect().await.unwrap());
            info!("got client: {:?}", client);
            let _r = client.register(RegisterRequest::default()).await.unwrap();
        });
    }
}