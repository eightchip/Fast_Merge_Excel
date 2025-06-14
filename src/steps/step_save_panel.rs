use egui::Ui;
use crate::app::AppState;
use std::sync::{Arc, Mutex};
use crate::steps::async_step::async_step_transition;
use crate::components::save_panel::{save_to_xlsx, SaveError};
use crate::components::button::AppButton;
use std::path::Path;

pub fn render_save_panel(app_state: Arc<Mutex<AppState>>, ui: &mut Ui) {
    // デバッグログを削除
    let (current_step, next_step) = {
        let state = app_state.lock().unwrap();
        (state.step, state.step + 1)
    };
    let app_state_clone = app_state.clone();
    let mut save_panel = {
        let state = app_state.lock().unwrap();
        state.save_panel.clone()
    };
    // デバッグログを削除
    save_panel.render(
        ui,
        &mut move |file_name: &Path| -> Result<(), SaveError> {
            let file_name = file_name.to_string_lossy().to_string();
            // 保存を開始する前に結果をリセット
            {
                let mut state = app_state_clone.lock().unwrap();
                state.save_result = None;
                state.save_error_message = None;
            }
            
            // 非同期処理を開始
            async_step_transition(app_state_clone.clone(), next_step, move || {
                move |state: &mut AppState| {
                    println!("[SAVE] Starting save process for file: {}", file_name);
                    
                    // 最新のデータを取得（ソート済みプレビューデータを優先）
                    let (save_columns, save_data) = if state.preview_table.has_sorted_data() {
                        // プレビューテーブルにソート済みデータがある場合はそれを使用
                        let (sorted_cols, sorted_data) = state.preview_table.get_sorted_data();
                        println!("[SAVE] Using sorted preview data: {} columns, {} rows", sorted_cols.len(), sorted_data.len());
                        if state.preview_table.is_sorted() {
                            println!("[SAVE] Data has been sorted by user");
                        }
                        (sorted_cols, sorted_data)
                    } else if let Some((columns, complete_data)) = &state.complete_result_data {
                        // フォールバック：元の完全なデータを使用
                        println!("[SAVE] Using original complete data: {} columns, {} rows", columns.len(), complete_data.len());
                        (columns.clone(), complete_data.clone())
                    } else {
                        println!("[SAVE] No data available for saving");
                        // データがない場合
                        state.save_result = Some(false);
                        state.save_error_message = Some("保存用のデータが見つかりません".to_string());
                        return;
                    };
                    
                    // 実際のファイル保存
                    println!("[SAVE] Attempting to save to: {}", file_name);
                    match save_to_xlsx(&file_name, &save_columns, &save_data) {
                        Ok(()) => {
                            println!("✅ [SAVE] File saved successfully: {}", file_name);
                            println!("[SAVE] Save operation completed without errors");
                            
                            // 保存成功をAppStateに記録
                            state.save_result = Some(true);
                            state.save_error_message = None;
                            state.save_panel.clear_error(); // エラーをクリア
                            
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
                        Err(save_error) => {
                            println!("❌ [SAVE] Save error occurred: {:?}", save_error);
                            println!("[SAVE] Error details: {}", save_error.user_friendly_message());
                            
                            // SavePanelにエラー情報を設定（UIで表示される）
                            state.save_panel.set_error(save_error.clone());
                            
                            // レガシーサポートのためAppStateにも設定
                            state.save_result = Some(false);
                            state.save_error_message = Some(save_error.user_friendly_message());
                        }
                    }
                }
            });
            
            // 保存処理の結果を返す（非同期処理の結果はAppStateに保存される）
            Ok(())
        },
    );
    // 「前へ」ボタンを追加
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        if ui.button("← 前へ（プレビュー）").clicked() {
            let mut state = app_state.lock().unwrap();
            state.step = 4; // プレビュー画面に戻る
        }
    });
    
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