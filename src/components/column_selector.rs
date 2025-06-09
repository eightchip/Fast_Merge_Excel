use egui::Ui;
use polars::prelude::*;
use crate::components::button::AppButton;

#[derive(Clone, Debug)]
pub struct ColumnSelector {
    pub selected_columns: Vec<String>, // 選択された列を保持
    pub available_columns: Vec<String>, // 選択肢（ダミー）
}

impl ColumnSelector {
    pub fn new() -> Self {
        ColumnSelector {
            selected_columns: Vec::new(),
            available_columns: vec![],
        }
    }

    pub fn render(&mut self, ui: &mut Ui, on_next: &mut dyn FnMut()) {
        ui.label("出力対象の列を選択してください（複数可）");
        ui.horizontal(|ui| {
            if AppButton::new("すべて選択").show(ui).clicked() {
                self.selected_columns = self.available_columns.clone();
            }
            if AppButton::new("すべて解除").show(ui).clicked() {
                self.selected_columns.clear();
            }
        });
        // 列リスト部分だけ縦スクロール
        egui::ScrollArea::vertical().id_source("column_list").max_height(300.0).show(ui, |ui| {
            for col in &self.available_columns {
                let mut checked = self.selected_columns.contains(col);
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut checked, col).changed() {
                        if checked {
                            if !self.selected_columns.contains(col) {
                                self.selected_columns.push(col.clone());
                            }
                        } else {
                            self.selected_columns.retain(|c| c != col);
                        }
                    }
                });
            }
        });
        // 選択順～ソートキー部分も縦スクロール
        egui::ScrollArea::vertical().id_source("column_selected").max_height(300.0).show(ui, |ui| {
            if !self.selected_columns.is_empty() {
                ui.add_space(10.0);
                ui.label("選択順:");
                for (i, col) in self.selected_columns.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}", i + 1));
                        ui.label(col);
                    });
                }
            }
            ui.add_space(10.0);
            if !self.selected_columns.is_empty() {
                if AppButton::new("次へ").show(ui).clicked() {
                    on_next();
                }
            } else {
                AppButton::new("次へ")
                    .with_fill(egui::Color32::from_gray(180))
                    .with_text_color(egui::Color32::from_gray(80))
                    .show(ui);
            }
            if AppButton::new("前へ").show(ui).clicked() {
                // 前へコールバックまたはself.step -= 1
            }
            // ソートキーUIはapp.rs側で呼ぶのでここでは省略
        });
    }
}
