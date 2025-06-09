use egui::Ui;
use crate::components::button::AppButton;

#[derive(Clone, Debug)]
pub struct KeySelector {
    pub selected_keys: Vec<String>, // 選択されたキーを保持
    pub available_keys: Vec<String>, // 選択肢（ダミー）
}

impl KeySelector {
    pub fn new() -> Self {
        KeySelector {
            selected_keys: Vec::new(),
            available_keys: vec![],
        }
    }

    pub fn render(&mut self, ui: &mut Ui, on_next: &mut dyn FnMut()) {
        ui.label("結合キーを選択してください（複数可）");
        let mut changed = false;
        egui::ScrollArea::vertical()
            .id_source("key_selector_keys")
            .max_height(300.0)
            .show(ui, |ui| {
                for key in &self.available_keys {
                    let mut checked = self.selected_keys.contains(key);
                    if ui.checkbox(&mut checked, key).changed() {
                        changed = true;
                        if checked {
                            if !self.selected_keys.contains(key) {
                                self.selected_keys.push(key.clone());
                            }
                        } else {
                            self.selected_keys.retain(|k| k != key);
                        }
                    }
                }
            });
        if self.available_keys.is_empty() {
            ui.colored_label(egui::Color32::RED, "共通列がありません");
        }
        ui.add_space(10.0);
        let next_enabled = !self.selected_keys.is_empty();
        if next_enabled {
            if AppButton::new("次へ").show(ui).clicked() {
                on_next();
            }
        } else {
            AppButton::new("次へ")
                .with_fill(egui::Color32::from_gray(180))
                .with_text_color(egui::Color32::from_gray(80))
                .show(ui);
        }
    }
}
