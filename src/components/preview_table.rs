use egui::{Ui, Color32, RichText};

pub struct PreviewTable {
    pub preview_data: Vec<Vec<String>>, // プレビューデータを保持
    pub columns: Vec<String>, // 列名
    pub page: usize, // 現在のページ番号
}

impl PreviewTable {
    pub fn new() -> Self {
        PreviewTable {
            columns: vec![],
            preview_data: vec![],
            page: 0,
        }
    }

    pub fn set_preview_data(&mut self, data: Vec<Vec<String>>) {
        self.preview_data = data; // プレビューデータを設定
        self.page = 0;
    }

    pub fn render(&mut self, ui: &mut Ui, on_next: &mut dyn FnMut()) {
        ui.label("プレビュー（ページ表示）");
        let rows_per_page = 20;
        let total = self.preview_data.len();
        let total_pages = (total + rows_per_page - 1) / rows_per_page;
        let start = self.page * rows_per_page;
        let end = ((self.page + 1) * rows_per_page).min(total);
        let preview_rows = if start < end {
            &self.preview_data[start..end]
        } else {
            &[]
        };
        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .columns(egui_extras::Column::remainder(), self.columns.len())
                .min_scrolled_height(800.0)
                .header(20.0, |mut header| {
                    for col in &self.columns {
                        header.col(|ui| { ui.label(col); });
                    }
                })
                .body(|mut body| {
                    for row in preview_rows {
                        body.row(18.0, |mut row_ui| {
                            for cell in row {
                                row_ui.col(|ui| {
                                    ui.label(cell);
                                });
                            }
                        });
                    }
                });
        });
        ui.horizontal(|ui| {
            if ui.add_enabled(self.page > 0, egui::Button::new("前へ")).clicked() {
                self.page -= 1;
            }
            ui.label(format!("{}/{}ページ", self.page + 1, total_pages.max(1)));
            if ui.add_enabled(self.page + 1 < total_pages, egui::Button::new("次へ")).clicked() {
                self.page += 1;
            }
        });
        ui.add_space(10.0);
        if ui.button("次へ").clicked() {
            on_next();
        }
    }
}
