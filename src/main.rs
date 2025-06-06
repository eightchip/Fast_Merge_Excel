mod components;
mod app;
mod font_util;

use app::App;
use eframe::egui;
use eframe::CreationContext;
use std::fs;

fn main() {
    let app_name = "Magic Merge Excel 2.0";
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        app_name,
        options,
        Box::new(|cc: &CreationContext| {
            font_util::set_japanese_font(&cc.egui_ctx);
            Box::new(App::new())
        })
    );
}
