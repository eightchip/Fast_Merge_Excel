use egui::Ui;

#[derive(Clone, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Clone)]
pub struct SortKey {
    pub column: String,
    pub order: SortOrder,
}

pub struct SortSettings {
    pub candidates: Vec<String>, // 列名候補
    pub sort_keys: Vec<SortKey>, // 選択したソートキー（最大5つ、順序付き）
}

impl SortSettings {
    pub fn new(candidates: Vec<String>) -> Self {
        SortSettings {
            candidates,
            sort_keys: vec![],
        }
    }

    pub fn render(&mut self, ui: &mut Ui) {
        ui.label("ソートキー（順序付き、昇順/降順）");
        egui::ScrollArea::vertical().id_source("sort_keys").max_height(200.0).show(ui, |ui| {
            let max_keys = self.candidates.len();
            for i in 0..max_keys {
                ui.horizontal(|ui| {
                    if let Some(sort_key) = self.sort_keys.get_mut(i) {
                        egui::ComboBox::from_id_source(format!("sort_col_{}", i))
                            .selected_text(&sort_key.column)
                            .show_ui(ui, |ui| {
                                for col in &self.candidates {
                                    ui.selectable_value(&mut sort_key.column, col.clone(), col);
                                }
                            });
                        egui::ComboBox::from_id_source(format!("sort_order_{}", i))
                            .selected_text(match sort_key.order {
                                SortOrder::Ascending => "昇順",
                                SortOrder::Descending => "降順",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut sort_key.order, SortOrder::Ascending, "昇順");
                                ui.selectable_value(&mut sort_key.order, SortOrder::Descending, "降順");
                            });
                        if ui.button("削除").clicked() {
                            self.sort_keys.remove(i);
                        }
                    } else {
                        // 新規追加ボタン
                        if ui.button("＋ソートキー追加").clicked() {
                            if self.sort_keys.len() < self.candidates.len() {
                                let next_col = self.candidates.iter().find(|c| !self.sort_keys.iter().any(|k| &k.column == *c)).cloned().unwrap_or_default();
                                self.sort_keys.push(SortKey { column: next_col, order: SortOrder::Ascending });
                            }
                        }
                    }
                });
            }
        });
    }
}
