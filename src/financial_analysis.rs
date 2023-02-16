use egui::{FontData, FontDefinitions, FontFamily};
use crate::frame_history::FrameHistory;
use crate::run_mode::RunMode;

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
}

impl Default for FinancialAnalysis {
    fn default() -> Self {
        Self {
            token: "".to_string(),
            login_done: false,
            run_mode: Default::default(),
            frame_history: Default::default(),
            enable_debug_panel: true,
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
        let def = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        def
    }
}