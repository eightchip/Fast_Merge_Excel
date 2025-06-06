use egui::Ui;

pub struct KeySelector {
    pub selected_keys: Vec<String>, // 選択されたキーを保持
    pub available_keys: Vec<String>, // 選択肢（ダミー）
}

impl KeySelector {
    pub fn new() -> Self {
        KeySelector {
            selected_keys: Vec::new(),
            available_keys: vec![
                "顧客ID".to_string(),
                "商品コード".to_string(),
                "日付".to_string(),
            ],
        }
    }

    pub fn render(&mut self, ui: &mut Ui, on_next: &mut dyn FnMut()) {
        ui.label("結合キーを選択してください（複数可）");
        let mut changed = false;
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
        if self.available_keys.is_empty() {
            ui.colored_label(egui::Color32::RED, "共通列がありません");
        }
        ui.add_space(10.0);
        let next_enabled = !self.selected_keys.is_empty();
        if ui.add_enabled(next_enabled, egui::Button::new("次へ")).clicked() {
            if next_enabled {
                on_next();
            }
        }
    }
}
