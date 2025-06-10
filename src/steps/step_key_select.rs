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
    let mode = {
        let s = app_state.lock().unwrap();
        s.mode.clone()
    };

    if matches!(mode, crate::app::MergeMode::MultiStageJoin) {
        // ----- MultiStageKeySelector -----
        let mut ms_key_selector = {
            let state = app_state.lock().unwrap();
            state.multi_stage_key_selector.clone()
        };
        
        let ab_keys = ms_key_selector.selected_keys_stage1.clone();
        let bc_keys = ms_key_selector.selected_keys_stage2.clone();
        
        ms_key_selector.render(ui, &mut move || {
            let ab_keys = ab_keys.clone();
            let bc_keys = bc_keys.clone();
            
            async_step_transition(app_state_clone.clone(), next_step, move || {
                move |state: &mut AppState| {
                    state.selected_ab_keys = ab_keys;
                    state.selected_bc_keys = bc_keys;
                    
                    // 全列を column_selector に供給（次のステップで列選択を行う）
                    let mut all_columns = std::collections::HashSet::new();
                    
                    // MultiStageJoinモードでは結合後の列名を予測して追加
                    if state.file_selector.columns_per_file.len() >= 2 {
                        let file_a_cols = &state.file_selector.columns_per_file[0];
                        let file_b_cols = &state.file_selector.columns_per_file[1];
                        
                        if matches!(state.mode, crate::app::MergeMode::ZennenTaihi) {
                            // 前年対比モードでは_right列を除外し、ユーザーにとって分かりやすい選択肢のみ表示
                            
                            // ファイルAの列（基本列）
                            for col in file_a_cols {
                                all_columns.insert(col.clone());
                            }
                            
                            // ファイルBで新しい列のみ追加（重複する列は除外）
                            for col in file_b_cols {
                                if !file_a_cols.contains(col) {
                                    all_columns.insert(col.clone());
                                }
                            }
                            
                            println!("[COLUMN_SELECT] Zennen Taihi mode - excluding _right columns from selection");
                        } else {
                            // 通常モードでは結合後の列名を予測して表示
                            
                            // ファイルAの列（そのまま）
                            for col in file_a_cols {
                                all_columns.insert(col.clone());
                            }
                            
                            // ファイルBの列（重複する場合は_rightサフィックス）
                            for col in file_b_cols {
                                if file_a_cols.contains(col) {
                                    // 重複する列は_rightサフィックス付きで追加
                                    all_columns.insert(format!("{}_right", col));
                                } else {
                                    // 重複しない列はそのまま追加
                                    all_columns.insert(col.clone());
                                }
                            }
                        }
                    } else {
                        // 通常モード：すべてのファイルの列を追加
                        for cols in &state.file_selector.columns_per_file {
                            for col in cols {
                                all_columns.insert(col.clone());
                            }
                        }
                    }
                    
                    state.column_selector.available_columns = all_columns.into_iter().collect();
                    println!("[COLUMN_SELECT] Available columns for selection: {:?}", state.column_selector.available_columns);
                }
            });
        });
        
        // 状態を更新
        {
            let mut state = app_state.lock().unwrap();
            state.multi_stage_key_selector = ms_key_selector;
        }
    } else {
        // ----- 既存 KeySelector -----
        let mut key_selector = {
            let state = app_state.lock().unwrap();
            state.key_selector.clone()
        };
        key_selector.render(ui, &mut move || {
            async_step_transition(app_state_clone.clone(), next_step, move || {
                move |state: &mut AppState| {
                    // ファイル選択済み列名の和集合をavailable_columnsにセット
                    let mut all_columns = std::collections::HashSet::new();
                    
                    // 通常モードでも結合後の列名を予測
                    if state.file_selector.columns_per_file.len() >= 2 {
                        let file_a_cols = &state.file_selector.columns_per_file[0];
                        let file_b_cols = &state.file_selector.columns_per_file[1];
                        
                        if matches!(state.mode, crate::app::MergeMode::ZennenTaihi) {
                            // 前年対比モードでは_right列を除外し、ユーザーにとって分かりやすい選択肢のみ表示
                            
                            // ファイルAの列（基本列）
                            for col in file_a_cols {
                                all_columns.insert(col.clone());
                            }
                            
                            // ファイルBで新しい列のみ追加（重複する列は除外）
                            for col in file_b_cols {
                                if !file_a_cols.contains(col) {
                                    all_columns.insert(col.clone());
                                }
                            }
                            
                            println!("[COLUMN_SELECT] Zennen Taihi mode - excluding _right columns from selection");
                        } else {
                            // 通常モードでも_right列を分かりやすくするため、基本列のみ表示
                            
                            // ファイルAの列（そのまま）
                            for col in file_a_cols {
                                all_columns.insert(col.clone());
                            }
                            
                            // ファイルBで新しい列のみ追加（重複する列は除外）
                            for col in file_b_cols {
                                if !file_a_cols.contains(col) {
                                    all_columns.insert(col.clone());
                                }
                            }
                            
                            println!("[COLUMN_SELECT] Normal mode - showing user-friendly column selection");
                        }
                    } else {
                        // ファイルが1つまたは0個の場合
                        for cols in &state.file_selector.columns_per_file {
                            for col in cols {
                                all_columns.insert(col.clone());
                            }
                        }
                    }
                    
                    state.column_selector.available_columns = all_columns.into_iter().collect();
                    println!("[COLUMN_SELECT] Available columns for normal mode: {:?}", state.column_selector.available_columns);
                }
            });
        });
        {
            let mut state = app_state.lock().unwrap();
            state.key_selector = key_selector;
        }
    }
} 