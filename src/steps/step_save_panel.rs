use egui::Ui;
use crate::app::AppState;
use std::sync::{Arc, Mutex};
use crate::steps::async_step::async_step_transition;
use crate::components::save_panel::save_to_xlsx;
use crate::components::button::AppButton;

pub fn render_save_panel(app_state: Arc<Mutex<AppState>>, ui: &mut Ui) {
    println!("[DEBUG] Rendering save panel");
    let (current_step, next_step) = {
        let state = app_state.lock().unwrap();
        (state.step, state.step + 1)
    };
    let app_state_clone = app_state.clone();
    let mut save_panel = {
        let state = app_state.lock().unwrap();
        state.save_panel.clone()
    };
    println!("[DEBUG] Save panel initialized");
    save_panel.render(
        ui,
        &mut move |file_name| {
            let file_name = file_name.to_string();
            // 保存を開始する前に結果をリセット
            {
                let mut state = app_state_clone.lock().unwrap();
                state.save_result = None;
                state.save_error_message = None;
            }
            
            async_step_transition(app_state_clone.clone(), next_step, move || {
                move |state: &mut AppState| {
                    println!("[SAVE] Starting save process for file: {}", file_name);
                    
                    // 完全なデータを取得（保存用）
                    if let Some((columns, complete_data)) = &state.complete_result_data {
                        println!("[SAVE] Saving {} columns and {} rows", columns.len(), complete_data.len());
                        
                        // 実際のファイル保存
                        match save_to_xlsx(&file_name, columns, complete_data) {
                            Ok(()) => {
                                println!("[SAVE] File saved successfully: {}", file_name);
                                
                                // 保存成功をAppStateに記録
                                state.save_result = Some(true);
                                state.save_error_message = None;
                                
                                // 保存先フォルダを開く（絶対パスで処理）
                                let file_path = std::path::Path::new(&file_name);
                                let dir_to_open = if file_path.is_absolute() {
                                    // 絶対パスの場合は親ディレクトリを取得
                                    if let Some(parent_dir) = file_path.parent() {
                                        parent_dir.to_path_buf()
                                    } else {
                                        std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
                                    }
                                } else {
                                    // 相対パスの場合は現在のディレクトリの絶対パスを取得
                                    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                                    if let Some(parent_dir) = file_path.parent() {
                                        current_dir.join(parent_dir)
                                    } else {
                                        current_dir
                                    }
                                };
                                
                                // Windows用のexplorerコマンドで絶対パスを指定
                                let dir_str = dir_to_open.to_string_lossy().to_string();
                                println!("[SAVE] Opening folder: {}", dir_str);
                                let _ = std::process::Command::new("explorer")
                                    .arg(&dir_str)
                                    .spawn()
                                    .map_err(|e| println!("[SAVE] Failed to open folder: {:?}", e));
                            }
                            Err(e) => {
                                println!("[SAVE] Error saving file: {:?}", e);
                                
                                // 保存失敗をAppStateに記録
                                state.save_result = Some(false);
                                state.save_error_message = Some(format!("保存に失敗しました: {:?}", e));
                            }
                        }
                    } else {
                        println!("[SAVE] No complete data available for saving");
                        // データがない場合も失敗として記録
                        state.save_result = Some(false);
                        state.save_error_message = Some("保存用のデータが見つかりません".to_string());
                    }
                }
            });
        },
    );
    // UI描画後、save_panelの内容をAppStateに戻す
    {
        let mut state = app_state.lock().unwrap();
        state.save_panel = save_panel;
    }
    
    // 保存結果をUIに表示（一時的に無効化）
    /*
    {
        let state = app_state.lock().unwrap();
        ui.add_space(10.0);
        match state.save_result {
            Some(true) => {
                ui.horizontal(|ui| {
                    ui.label("✅");
                    ui.colored_label(egui::Color32::from_rgb(0, 150, 0), "保存が完了しました！");
                });
                ui.label("📁 保存先フォルダが自動で開かれます");
                ui.add_space(10.0);
                
                // 保存成功後の「次へ」ボタン
                if AppButton::new("完了").show(ui).clicked() {
                    let mut state = app_state.lock().unwrap();
                    state.step = next_step;
                }
            }
            Some(false) => {
                ui.horizontal(|ui| {
                    ui.label("❌");
                    ui.colored_label(egui::Color32::RED, "保存に失敗しました");
                });
                if let Some(error_msg) = &state.save_error_message {
                    ui.colored_label(egui::Color32::RED, error_msg);
                }
                ui.add_space(10.0);
                
                // 保存失敗後の「再試行」と「スキップ」ボタン
                ui.horizontal(|ui| {
                    if AppButton::new("再試行").show(ui).clicked() {
                        let mut state = app_state.lock().unwrap();
                        state.save_result = None;
                        state.save_error_message = None;
                    }
                    if AppButton::new("スキップして完了").show(ui).clicked() {
                        let mut state = app_state.lock().unwrap();
                        state.step = next_step;
                    }
                });
            }
            None => {
                // まだ保存が実行されていない、または処理中
            }
        }
    }
    */
} 