#![warn(clippy::all, rust_2018_idioms)]

pub mod app;
pub mod constants;
pub mod financial_analysis;
pub mod debug_panel;
pub mod run_mode;
pub mod frame_history;
pub mod login;
pub mod password;

#[macro_use]
extern crate rust_i18n;
i18n!("locales");