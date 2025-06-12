use egui::Ui;
use crate::components::button::AppButton;

pub struct CompareKeySelector {
    pub available_keys: Vec<String>,
    pub selected_keys: Vec<String>,
}

impl CompareKeySelector {
    pub fn new() -> Self {
        CompareKeySelector {
            available_keys: vec![],
            selected_keys: vec![],
        }
    }

    pub fn set_available_keys(&mut self, keys: Vec<String>) {
        self.available_keys = keys;
        self.selected_keys.clear();
    }

    pub fn render(&mut self, ui: &mut Ui, on_next: &mut dyn FnMut()) {
        ui.label("複合キー（最大5つまで）を選択してください");
        let max_keys = 5;
        egui::ScrollArea::vertical()
            .id_source("compare_key_selector_keys")
            .max_height(300.0)
            .show(ui, |ui| {
                for key in &self.available_keys {
                    let mut checked = self.selected_keys.contains(key);
                    let disabled = false; // 必要に応じて条件
                    if !disabled {
                        if ui.checkbox(&mut checked, key).changed() {
                            if checked {
                                if !self.selected_keys.contains(key) {
                                    self.selected_keys.push(key.clone());
                                }
                            } else {
                                self.selected_keys.retain(|k| k != key);
                            }
                        }
                    } else {
                        ui.add_enabled(false, egui::Checkbox::new(&mut checked, key));
                    }
                }
            });
        ui.add_space(10.0);
        let next_enabled = !self.selected_keys.is_empty();
        if next_enabled {
            if ui.button("次へ").clicked() {
                on_next();
            }
        } else {
            ui.add_enabled(false, egui::Button::new("次へ"));
        }
    }
} 