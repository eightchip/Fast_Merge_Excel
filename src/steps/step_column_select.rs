use egui::Ui;
use crate::app::AppState;
use std::sync::{Arc, Mutex};
use crate::steps::async_step::async_step_transition;

pub fn render_column_select(app_state: Arc<Mutex<AppState>>, ui: &mut Ui) {
    let next_step = {
        let state = app_state.lock().unwrap();
        state.step + 1
    };
    let app_state_clone = app_state.clone();
    let mut column_selector = {
        let state = app_state.lock().unwrap();
        state.column_selector.clone()
    };
    column_selector.render(
        ui,
        &mut move || {
            async_step_transition(app_state_clone.clone(), next_step, move || {
                // ここで列抽出・検証など重い処理を行う
                move |state: &mut AppState| {
                    println!("[DEBUG] join_type_picker.selected_join_type at column_select: {:?}", state.join_type_picker.selected_join_type);
                    // Ensure join type is properly set in AppState
                    if state.join_type_picker.selected_join_type.is_none() {
                        // Set default join type based on mode if not set
                        match state.mode {
                            crate::app::MergeMode::ZennenTaihi => {
                                state.join_type_picker.selected_join_type = Some(crate::components::join_type_picker::JoinType::FullOuter);
                            },
                            crate::app::MergeMode::MultiStageJoin => {
                                state.join_type_picker.selected_join_type = Some(crate::components::join_type_picker::JoinType::Left);
                            },
                            crate::app::MergeMode::KanzenIcchi => {
                                state.join_type_picker.selected_join_type = Some(crate::components::join_type_picker::JoinType::Inner);
                            },
                            crate::app::MergeMode::Hikaku => {
                                state.join_type_picker.selected_join_type = Some(crate::components::join_type_picker::JoinType::Left);
                            },
                            crate::app::MergeMode::TateRenketsu => {
                                state.join_type_picker.selected_join_type = Some(crate::components::join_type_picker::JoinType::Concat);
                            },
                            crate::app::MergeMode::None => {},
                        }
                    }
                }
            });
        },
    );
    // UI描画後、column_selectorの内容をAppStateに戻す
    {
        let mut state = app_state.lock().unwrap();
        state.column_selector = column_selector;
    }
} 