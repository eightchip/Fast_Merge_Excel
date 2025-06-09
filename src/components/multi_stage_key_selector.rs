use egui::Ui;
use crate::components::button::AppButton;

pub struct MultiStageKeySelector {
    pub available_keys_stage1: Vec<String>,
    pub selected_keys_stage1: Vec<String>,
    pub available_keys_stage2: Vec<String>,
    pub selected_keys_stage2: Vec<String>,
}

impl MultiStageKeySelector {
    pub fn new() -> Self {
        MultiStageKeySelector {
            available_keys_stage1: vec![],
            selected_keys_stage1: vec![],
            available_keys_stage2: vec![],
            selected_keys_stage2: vec![],
        }
    }

    pub fn set_available_keys(&mut self, stage1: Vec<String>, stage2: Vec<String>) {
        self.available_keys_stage1 = stage1;
        self.selected_keys_stage1.clear();
        self.available_keys_stage2 = stage2;
        self.selected_keys_stage2.clear();
    }

    pub fn render(&mut self, ui: &mut Ui, on_next: &mut dyn FnMut()) {
        ui.label("[1段階目] 複合キー（最大3つまで）を選択してください");
        let max_keys = 3;
        egui::ScrollArea::vertical()
            .id_source("multi_stage_key_selector_stage1")
            .max_height(200.0)
            .show(ui, |ui| {
                for key in &self.available_keys_stage1 {
                    let mut checked = self.selected_keys_stage1.contains(key);
                    let disabled = false;
                    if !disabled {
                        if ui.checkbox(&mut checked, key).changed() {
                            if checked {
                                if !self.selected_keys_stage1.contains(key) {
                                    self.selected_keys_stage1.push(key.clone());
                                }
                            } else {
                                self.selected_keys_stage1.retain(|k| k != key);
                            }
                        }
                    } else {
                        ui.add_enabled(false, egui::Checkbox::new(&mut checked, key));
                    }
                }
            });
        ui.add_space(10.0);
        ui.label("[2段階目] 複合キー（最大3つまで）を選択してください");
        egui::ScrollArea::vertical()
            .id_source("multi_stage_key_selector_stage2")
            .max_height(200.0)
            .show(ui, |ui| {
                for key in &self.available_keys_stage2 {
                    let mut checked = self.selected_keys_stage2.contains(key);
                    let disabled = !checked && self.selected_keys_stage2.len() >= max_keys;
                    ui.horizontal(|ui| {
                        if ui.add_enabled(!disabled, egui::Checkbox::new(&mut checked, key)).changed() {
                            if checked {
                                if !self.selected_keys_stage2.contains(key) && self.selected_keys_stage2.len() < max_keys {
                                    self.selected_keys_stage2.push(key.clone());
                                }
                            } else {
                                self.selected_keys_stage2.retain(|k| k != key);
                            }
                        }
                    });
                }
            });
        ui.add_space(10.0);
        let next_enabled = !self.selected_keys_stage1.is_empty() && !self.selected_keys_stage2.is_empty();
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