use egui::Ui;
use crate::app::AppState;
use std::sync::{Arc, Mutex};
use crate::steps::async_step::async_step_transition;
use crate::components::file_selector::get_columns_from_xlsx;
use crate::components::file_selector::FileSelector;

pub fn render_file_select(app_state: Arc<Mutex<AppState>>, ui: &mut Ui) {
    let next_step = {
        let state = app_state.lock().unwrap();
        state.step + 1
    };
    let app_state_clone = app_state.clone();
    let app_state_prev = app_state.clone();
    let mut file_selector = {
        let state = app_state.lock().unwrap();
        state.file_selector.clone()
    };
    file_selector.render(
        ui,
        &mut move || {
            let files = {
                let state = app_state_clone.lock().unwrap();
                println!("selected_files: {:?}", state.file_selector.selected_files);
                let a = state.file_selector.selected_files[0].clone();
                let b = state.file_selector.selected_files[1].clone();
                let c = state.file_selector.selected_files[2].clone();
                [a, b, c]
            };
            async_step_transition(app_state_clone.clone(), next_step, move || {
                let mut columns: [Vec<String>; 3] = [vec![], vec![], vec![]];
                let mut error = None;
                let mut file_count = 0;
                for (i, file_opt) in files.iter().enumerate() {
                    if let Some(ref path) = file_opt {
                        columns[i] = get_columns_from_xlsx(path);
                        file_count += 1;
                    }
                }
                if file_count < 2 {
                    error = Some("2ファイル以上選択してください".to_string());
                }
                let col_lens: Vec<_> = columns.iter().map(|c| c.len()).filter(|&l| l > 0).collect();
                if col_lens.len() >= 2 {
                    let min = *col_lens.iter().min().unwrap();
                    let max = *col_lens.iter().max().unwrap();
                    if max > min * 2 {
                        error = Some("ファイル間で列数が極端に異なります".to_string());
                    }
                }
                let selected_columns: Vec<&Vec<String>> = files.iter().enumerate()
                    .filter_map(|(i, f)| if f.is_some() { Some(&columns[i]) } else { None })
                    .collect();
                let common_columns = if selected_columns.len() >= 2 {
                    let mut iter = selected_columns.iter();
                    if let Some(first) = iter.next() {
                        let mut set: std::collections::HashSet<String> = first.iter().cloned().collect();
                        for cols in iter {
                            set = set.intersection(&cols.iter().cloned().collect()).cloned().collect();
                        }
                        set.into_iter().collect::<Vec<_>>()
                    } else {
                        vec![]
                    }
                } else if selected_columns.len() == 1 {
                    selected_columns[0].clone()
                } else {
                    vec![]
                };
                move |state: &mut AppState| {
                    // columns は後で move するためクローンを保持
                    let columns_clone = columns.clone();
                    state.file_selector.columns_per_file = columns;
                    state.processing_error = error;
                    state.key_selector.available_keys = common_columns;
                    // MultiStageJoin 用に AB, BC 共通キーを算出
                    if matches!(state.mode, crate::app::MergeMode::MultiStageJoin) {
                        // AB 共通キー
                        let ab_keys: Vec<String> = if !columns_clone[0].is_empty() && !columns_clone[1].is_empty() {
                            let set_a: std::collections::HashSet<String> = columns_clone[0].iter().cloned().collect();
                            let set_b: std::collections::HashSet<String> = columns_clone[1].iter().cloned().collect();
                            set_a.intersection(&set_b).cloned().collect()
                        } else { vec![] };

                        // BC 共通キー
                        let bc_keys: Vec<String> = if !columns_clone[1].is_empty() && !columns_clone[2].is_empty() {
                            let set_b: std::collections::HashSet<String> = columns_clone[1].iter().cloned().collect();
                            let set_c: std::collections::HashSet<String> = columns_clone[2].iter().cloned().collect();
                            set_b.intersection(&set_c).cloned().collect()
                        } else { vec![] };

                        state.ab_common_keys = ab_keys.clone();
                        state.bc_common_keys = bc_keys.clone();
                        state.multi_stage_columns = (ab_keys.clone(), bc_keys.clone());
                        
                        // MultiStageKeySelector に利用可能なキーを設定
                        state.multi_stage_key_selector.available_keys_stage1 = ab_keys;
                        state.multi_stage_key_selector.available_keys_stage2 = bc_keys;
                    }
                }
            });
        },
        &mut |_cols| {
            // 列情報の初期化など必要ならここで
        },
    );
    
    // 「前へ」ボタンを追加
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        if ui.button("← 前へ（モード選択）").clicked() {
            let mut state = app_state_prev.lock().unwrap();
            state.step = 0; // モード選択画面に戻る
        }
    });
    
    // UI描画後、file_selectorの内容をAppStateに戻す
    {
        let mut state = app_state.lock().unwrap();
        state.file_selector = file_selector;
    }
} 