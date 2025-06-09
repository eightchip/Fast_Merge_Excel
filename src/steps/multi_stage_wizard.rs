use egui::Ui;
use crate::app::AppState;
use std::sync::{Arc, Mutex};
use crate::steps::{step_file_select, step_key_select, step_column_select, step_preview, step_save_panel, step_complete};

pub fn render_multi_stage_wizard(app_state: Arc<Mutex<AppState>>, ui: &mut Ui, ctx: &egui::Context) {
    let step = app_state.lock().unwrap().step;
    match step {
        1 => step_file_select::render_file_select(app_state.clone(), ui),
        2 => step_key_select::render_key_select(app_state.clone(), ui),
        3 => step_column_select::render_column_select(app_state.clone(), ui),
        4 => step_preview::render_preview(app_state.clone(), ui, ctx),
        5 => step_save_panel::render_save_panel(app_state.clone(), ui),
        6 => step_complete::render_complete(app_state.clone(), ui),
        _ => {},
    }
} 