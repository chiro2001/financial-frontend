#![allow(dead_code)]

use egui::{Align2, Color32, FontId, Pos2, Ui, Vec2};
use std::future::Future;

#[cfg(not(target_arch = "wasm32"))]
pub fn execute<F: Future<Output=()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    // std::thread::spawn(move || futures::executor::block_on(f));
    crate::financial_analysis::RT.spawn(f);
    // tokio::runtime::Builder::new_multi_thread()
    //     .enable_io()
    //     // .enable_time()
    //     .build().unwrap().spawn(f);
    // tokio::spawn(f);
}

#[cfg(target_arch = "wasm32")]
pub fn execute<F: Future<Output=()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn block_on<F: Future<Output=()>>(f: F) {
    // this is stupid... use any executor of your choice instead
    // tokio::runtime::Builder::new_multi_thread()
    //     .enable_io()
    //     // .enable_time()
    //     .build().unwrap().block_on(f);
    crate::financial_analysis::RT.block_on(f);
}

#[cfg(target_arch = "wasm32")]
pub fn block_on<F: Future<Output=()> + 'static>(f: F) {
    // not blocked really...
    wasm_bindgen_futures::spawn_local(f);
}

pub async fn sleep_ms(mills: u64) {
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::sleep(std::time::Duration::from_millis(mills));
    #[cfg(target_arch = "wasm32")]
    {
        // #[wasm_bindgen]
        pub fn sleep(ms: i32) -> js_sys::Promise {
            js_sys::Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
                    .unwrap();
            })
        }
        let promise = sleep(mills as i32);
        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
    }
}

pub const LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

pub const LOREM_IPSUM_LONG: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

Curabitur pretium tincidunt lacus. Nulla gravida orci a odio. Nullam varius, turpis et commodo pharetra, est eros bibendum elit, nec luctus magna felis sollicitudin mauris. Integer in mauris eu nibh euismod gravida. Duis ac tellus et risus vulputate vehicula. Donec lobortis risus a elit. Etiam tempor. Ut ullamcorper, ligula eu tempor congue, eros est euismod turpis, id tincidunt sapien risus a quam. Maecenas fermentum consequat mi. Donec fermentum. Pellentesque malesuada nulla a mi. Duis sapien sem, aliquet nec, commodo eget, consequat quis, neque. Aliquam faucibus, elit ut dictum aliquet, felis nisl adipiscing sapien, sed malesuada diam lacus eget erat. Cras mollis scelerisque nunc. Nullam arcu. Aliquam consequat. Curabitur augue lorem, dapibus quis, laoreet et, pretium ac, nisi. Aenean magna nisl, mollis quis, molestie eu, feugiat in, orci. In hac habitasse platea dictumst.";

#[allow(dead_code)]
pub fn lorem_ipsum(ui: &mut egui::Ui) {
    ui.with_layout(
        egui::Layout::top_down(egui::Align::LEFT).with_cross_justify(true),
        |ui| {
            ui.label(
                egui::RichText::new(crate::utils::LOREM_IPSUM_LONG)
                    .small()
                    .weak(),
            );
        },
    );
}

pub fn get_text_size(ui: &Ui, text: &str, font: FontId) -> Vec2 {
    ui.painter()
        .text(
            Pos2::ZERO,
            Align2::RIGHT_BOTTOM,
            text,
            font,
            Color32::TRANSPARENT,
        )
        .size()
}

pub fn get_random_buf() -> Result<[u8; 4], getrandom::Error> {
    let mut buf = [0u8; 4];
    getrandom::getrandom(&mut buf)?;
    Ok(buf)
}

pub fn get_random_u32() -> u32 {
    u32::from_le_bytes(match get_random_buf() {
        Ok(r) => r,
        Err(_) => [0; 4],
    })
}