#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub mod constants;
pub mod debug_panel;
pub mod financial_analysis;
pub mod frame_history;
pub mod login;
pub mod password;
pub mod rpc_client;
pub mod rpc_client_wasm;
pub mod run_mode;

#[macro_use]
extern crate rust_i18n;
i18n!("locales");
