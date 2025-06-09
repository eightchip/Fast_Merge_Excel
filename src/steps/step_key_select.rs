use egui::Ui;
use crate::steps::async_step::async_step_transition;
use std::sync::{Arc, Mutex};
use crate::app::AppState;

pub fn render_key_select(app_state: Arc<Mutex<AppState>>, ui: &mut Ui) {
    let next_step = {
        let state = app_state.lock().unwrap();
        state.step + 1
    };
    let app_state_clone = app_state.clone();
    let mut key_selector = {
        let state = app_state.lock().unwrap();
        state.key_selector.clone()
    };
    key_selector.render(
        ui,
        &mut move || {
            async_step_transition(app_state_clone.clone(), next_step, move || {
                // ここでキー抽出・検証など重い処理を行う
                move |state: &mut AppState| {
                    // ファイル選択済み列名の和集合をavailable_columnsにセット
                    let mut all_columns = std::collections::HashSet::new();
                    for cols in &state.file_selector.columns_per_file {
                        for col in cols {
                            all_columns.insert(col.clone());
                        }
                    }
                    state.column_selector.available_columns = all_columns.into_iter().collect();
                }
            });
        },
    );
    // UI描画後、key_selectorの内容をAppStateに戻す
    {
        let mut state = app_state.lock().unwrap();
        state.key_selector = key_selector;
    }
} 