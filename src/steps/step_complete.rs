use egui::Ui;
use crate::app::AppState;
use std::sync::{Arc, Mutex};
// use crate::components::button::AppButton;

pub fn render_complete(app_state: Arc<Mutex<AppState>>, ui: &mut Ui) {
    let (save_result, save_error_message) = {
        let state = app_state.lock().unwrap();
        (state.save_result, state.save_error_message.clone())
    };
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        match save_result {
            Some(true) => {
                ui.heading("処理完了！");
                ui.add_space(20.0);
                ui.label("Excelファイルの結合・保存が正常に完了しました。");
                ui.label("📁 保存先フォルダが自動で開かれます。");
                ui.add_space(30.0);
                ui.horizontal(|ui| {
                    if ui.button("最初に戻る").clicked() {
                        let mut state = app_state.lock().unwrap();
                        *state = AppState::new();
                    }
                    ui.add_space(20.0);
                    if ui.button("アプリを終了").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.add_space(50.0);
                ui.separator();
                ui.add_space(20.0);
                ui.label("💡 他のファイルも結合したい場合は「最初に戻る」をクリックしてください。");
            }
            Some(false) => {
                ui.heading("❌ 保存に失敗しました");
                ui.add_space(20.0);
                if let Some(msg) = &save_error_message {
                    ui.colored_label(egui::Color32::RED, msg);
                } else {
                    ui.colored_label(egui::Color32::RED, "保存に失敗しました。詳細不明");
                }
                ui.add_space(30.0);
                ui.horizontal(|ui| {
                    if ui.button("最初に戻る").clicked() {
                        let mut state = app_state.lock().unwrap();
                        *state = AppState::new();
                    }
                    ui.add_space(20.0);
                    if ui.button("再試行").clicked() {
                        let mut state = app_state.lock().unwrap();
                        state.step = 5; // 保存画面に戻る
                        state.save_result = None;
                        state.save_error_message = None;
                    }
                    ui.add_space(20.0);
                    if ui.button("アプリを終了").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.add_space(50.0);
                ui.separator();
                ui.add_space(20.0);
                ui.label("💡 対処方法を参考に再度保存をお試しください。");
            }
            None => {
                // 通常ここには来ないが、念のため
                ui.heading("処理結果が不明です");
            }
        }
    });
} 