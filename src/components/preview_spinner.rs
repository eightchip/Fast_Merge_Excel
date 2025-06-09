use egui::Ui;
use std::sync::{Arc, Mutex};

pub struct PreviewAsyncPanel {
    pub is_processing: bool,
    pub progress_shared: Arc<Mutex<f32>>,
    pub error_shared: Arc<Mutex<Option<String>>>,
    pub result_shared: Arc<Mutex<Option<(Vec<String>, Vec<Vec<String>>)>>>,
}

impl PreviewAsyncPanel {
    pub fn new() -> Self {
        Self {
            is_processing: false,
            progress_shared: Arc::new(Mutex::new(0.0)),
            error_shared: Arc::new(Mutex::new(None)),
            result_shared: Arc::new(Mutex::new(None)),
        }
    }

    /// UI描画。進捗・エラー・完了時のコールバックを受け取る
    pub fn render<F: FnMut()>(
        &mut self,
        ui: &mut Ui,
        mut on_finish: F,
    ) {
        println!("[DEBUG] is_processing = {}", self.is_processing);
        println!("[DEBUG] render_preview called");
        if self.is_processing {
            ui.add(egui::Spinner::new());
            ui.label("プレビュー生成中です。しばらくお待ちください...");
            if let Ok(progress) = self.progress_shared.lock() {
                ui.add(egui::ProgressBar::new(*progress).show_percentage());
            }
            if let Ok(error) = self.error_shared.lock() {
                if let Some(err) = &*error {
                    ui.colored_label(egui::Color32::RED, err);
                }
            }
            if let Ok(mut result) = self.result_shared.lock() {
                if result.is_some() {
                    self.is_processing = false;
                    on_finish();
                }
            }
        }
    }

    /// 非同期処理開始用ヘルパー
    pub fn start_processing(&mut self) {
        self.is_processing = true;
        *self.progress_shared.lock().unwrap() = 0.0;
        *self.error_shared.lock().unwrap() = None;
        *self.result_shared.lock().unwrap() = None;
    }
} 