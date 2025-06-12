use egui::Ui;
use crate::components::button::AppButton;

#[derive(Clone)]
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
        ui.heading("2段階結合設定");
        
        // Stage 1: A↔B キー選択 (フリガナ、顧客コード等)
        ui.group(|ui| {
            ui.label("Step 1: 入金 ↔ 履歴 の結合キー:");
            
            egui::ScrollArea::vertical()
                .max_height(120.0)
                .id_source("stage1_keys")
                .show(ui, |ui| {
                for key in &self.available_keys_stage1 {
                    let selected = self.selected_keys_stage1.contains(key);
                    let mut new_selected = selected;
                    
                    if ui.checkbox(&mut new_selected, key).changed() {
                        if new_selected {
                            if !self.selected_keys_stage1.contains(key) {
                                self.selected_keys_stage1.push(key.clone());
                            }
                        } else {
                            self.selected_keys_stage1.retain(|k| k != key);
                        }
                    }
                }
             });
            
            if !self.selected_keys_stage1.is_empty() {
                ui.label(format!("選択済み: {:?}", self.selected_keys_stage1));
            }
        });
        
        ui.add_space(10.0);
        
        // Stage 2: B↔C キー選択 (売上先等)
        ui.group(|ui| {
            ui.label("Step 2: 履歴 ↔ 売上 の結合キー:");
            
            egui::ScrollArea::vertical()
                .max_height(120.0)
                .id_source("stage2_keys")
                .show(ui, |ui| {
                for key in &self.available_keys_stage2 {
                    let selected = self.selected_keys_stage2.contains(key);
                    let mut new_selected = selected;
                    
                    if ui.checkbox(&mut new_selected, key).changed() {
                        if new_selected {
                            if !self.selected_keys_stage2.contains(key) {
                                self.selected_keys_stage2.push(key.clone());
                            }
                        } else {
                            self.selected_keys_stage2.retain(|k| k != key);
                        }
                    }
                }
             });
            
            if !self.selected_keys_stage2.is_empty() {
                ui.label(format!("選択済み: {:?}", self.selected_keys_stage2));
            }
        });
        
        ui.add_space(10.0);
        
        // 次へボタン (全ての設定が完了した場合のみ有効)
        let ready = !self.selected_keys_stage1.is_empty() 
            && !self.selected_keys_stage2.is_empty();
            
        if ready {
            if ui.button("次へ").clicked() {
                on_next();
            }
        } else {
            ui.add_enabled(false, egui::Button::new("次へ (設定を完了してください)"));
        }
    }
} 