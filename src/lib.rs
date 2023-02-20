#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub mod constants;
pub mod debug_panel;
pub mod financial_analysis;
pub mod frame_history;
pub mod login;
pub mod password;
pub mod run_mode;
pub mod utils;
pub mod message;
pub mod service;
pub mod trading_history;

#[macro_use]
extern crate rust_i18n;
i18n!("locales");
