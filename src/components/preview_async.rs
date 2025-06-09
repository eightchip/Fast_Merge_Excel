use egui::Ui;
use std::sync::{Arc, Mutex};
use polars::prelude::*;

pub struct PreviewAsyncComponent {
    pub is_processing: bool,
    pub progress_shared: Arc<Mutex<f32>>,
    pub error_shared: Arc<Mutex<Option<String>>>,
    pub result_shared: Arc<Mutex<Option<(Vec<String>, Vec<Vec<String>>)>>>,
    pub preview_data: Vec<Vec<String>>,
    pub columns: Vec<String>,
    pub is_full_ready: bool,
}

impl PreviewAsyncComponent {
    pub fn new() -> Self {
        Self {
            is_processing: false,
            progress_shared: Arc::new(Mutex::new(0.0)),
            error_shared: Arc::new(Mutex::new(None)),
            result_shared: Arc::new(Mutex::new(None)),
            preview_data: vec![],
            columns: vec![],
            is_full_ready: false,
        }
    }

    /// プレビューUI描画。即時20行 or 全件切り替え、進捗・エラー表示
    pub fn render<F: FnMut()>(&mut self, ui: &mut Ui, mut on_full_ready: F) {
        if self.is_processing {
            ui.add(egui::Spinner::new());
            ui.label("最初の20件を表示中です。全件プレビューを計算中...");
            if !self.preview_data.is_empty() {
                // 20行即時プレビュー
                self.render_table(ui, 20);
            }
            if let Ok(progress) = self.progress_shared.lock() {
                ui.add(egui::ProgressBar::new(*progress).show_percentage());
            }
            if let Ok(error) = self.error_shared.lock() {
                if let Some(err) = &*error {
                    ui.colored_label(egui::Color32::RED, err);
                }
            }
            if let Ok(mut result) = self.result_shared.lock() {
                if let Some((columns, preview_data)) = result.take() {
                    self.columns = columns;
                    self.preview_data = preview_data;
                    self.is_processing = false;
                    self.is_full_ready = true;
                    on_full_ready();
                }
            }
        } else if self.is_full_ready {
            self.render_table(ui, usize::MAX); // 全件
        }
    }

    /// テーブル描画（最大rows件）
    pub fn render_table(&self, ui: &mut Ui, rows: usize) {
        let show_rows = self.preview_data.iter().take(rows).collect::<Vec<_>>();
        let col_widths = self.auto_column_widths(&show_rows);
        egui::ScrollArea::horizontal().show(ui, |ui| {
            let mut table = egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .min_scrolled_height(800.0);
            for w in &col_widths {
                table = table.column(egui_extras::Column::initial(*w));
            }
            table
                .header(20.0, |mut header| {
                    for col in &self.columns {
                        header.col(|ui| { ui.label(col); });
                    }
                })
                .body(|mut body| {
                    for row in &show_rows {
                        body.row(18.0, |mut row_ui| {
                            for cell in row.iter() {
                                row_ui.col(|ui| {
                                    ui.label(cell);
                                });
                            }
                        });
                    }
                });
        });
    }

    /// ヘッダー＋最初の数行の最大文字数からカラム幅を自動計算
    pub fn auto_column_widths(&self, rows: &Vec<&Vec<String>>) -> Vec<f32> {
        let mut widths = vec![];
        for (i, col) in self.columns.iter().enumerate() {
            let mut max_len = col.chars().count();
            for row in rows.iter().take(10) {
                if let Some(cell) = row.get(i) {
                    max_len = max_len.max(cell.chars().count());
                }
            }
            widths.push((max_len as f32) * 14.0 + 24.0);
        }
        widths
    }

    /// 非同期処理開始用ヘルパー
    pub fn start_processing(&mut self, columns: Vec<String>, preview_data: Vec<Vec<String>>) {
        self.is_processing = true;
        *self.progress_shared.lock().unwrap() = 0.0;
        *self.error_shared.lock().unwrap() = None;
        *self.result_shared.lock().unwrap() = None;
        self.columns = columns;
        self.preview_data = preview_data;
        self.is_full_ready = false;
    }
} 