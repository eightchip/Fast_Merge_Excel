use egui::{Context, Ui};
use crate::app::AppState;
use std::sync::{Arc, Mutex};
use crate::components::file_selector::FileSelector;
use crate::components::key_selector::KeySelector;
use crate::components::preview_table::PreviewTable;
use crate::components::save_panel::SavePanel;
use polars::prelude::*;
use calamine::{open_workbook_auto, Reader};

pub fn render_split_save_wizard(app_state: Arc<Mutex<AppState>>, ui: &mut Ui, ctx: &Context) {
    let mut state = app_state.lock().unwrap();
    
    match state.step {
        0 => render_file_select(&mut state, ui),
        1 => render_key_select(&mut state, ui),
        2 => render_preview(&mut state, ui, ctx),
        _ => {}
    }
}

fn render_file_select(state: &mut AppState, ui: &mut Ui) {
    ui.heading("分割するExcelファイルを選択してください");
    ui.add_space(20.0);
    
    let mut on_next = || {
        state.step = 1;
    };
    
    let mut on_columns_loaded = |columns: [Vec<String>; 3]| {
        state.key_selector.available_keys = columns[0].clone();
    };
    
    state.file_selector.render(ui, &mut on_next, &mut on_columns_loaded);
    
    if state.file_selector.selected_files[0].is_some() {
        ui.add_space(20.0);
        if ui.button("次へ").clicked() {
            state.step = 1;
        }
    }
}

fn render_key_select(state: &mut AppState, ui: &mut Ui) {
    ui.heading("分割の基準となる主キーを選択してください");
    ui.add_space(20.0);
    
    let mut on_next = || {
        state.step = 2;
    };
    
    state.key_selector.render(ui, &mut on_next);
    
    if !state.key_selector.selected_keys.is_empty() {
        ui.add_space(20.0);
        if ui.button("次へ").clicked() {
            state.step = 2;
        }
    }
}

fn render_preview(state: &mut AppState, ui: &mut Ui, ctx: &Context) {
    ui.heading("分割プレビュー");
    ui.add_space(20.0);
    
    // TODO: プレビューテーブルの実装
    // TODO: 分割処理の実装
    // TODO: 保存処理の実装
} 