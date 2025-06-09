use egui::Ui;
use crate::app::AppState;
use std::sync::{Arc, Mutex};
use crate::components::button::AppButton;

pub fn render_complete(app_state: Arc<Mutex<AppState>>, ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(50.0);
        
        ui.heading("🎉 処理完了！");
        ui.add_space(20.0);
        
        ui.label("Excelファイルの結合・保存が正常に完了しました。");
        ui.label("📁 保存先フォルダが自動で開かれます。");
        ui.add_space(30.0);
        
        ui.horizontal(|ui| {
            if AppButton::new("最初に戻る").show(ui).clicked() {
                let mut state = app_state.lock().unwrap();
                // 状態をリセットして最初に戻る
                *state = AppState::new();
            }
            
            ui.add_space(20.0);
            
            if AppButton::new("アプリを終了").show(ui).clicked() {
                std::process::exit(0);
            }
        });
        
        ui.add_space(50.0);
        ui.separator();
        ui.add_space(20.0);
        
        ui.label("💡 他のファイルも結合したい場合は「最初に戻る」をクリックしてください。");
    });
} 