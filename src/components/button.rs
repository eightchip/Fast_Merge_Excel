use egui::{Ui, Button, Color32, RichText, Image, Response};

pub struct AppButton {
    pub label: String,
    pub icon: Option<egui::Image<'static>>,
    pub fill: Color32,
    pub text_color: Color32,
    pub size: egui::Vec2,
}

impl AppButton {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            icon: None,
            fill: Color32::from_rgb(33, 150, 243), // 青系
            text_color: Color32::WHITE,
            size: egui::vec2(70.0, 20.0), // 大きめ
        }
    }
    pub fn with_icon(mut self, icon: egui::Image<'static>) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn with_fill(mut self, color: Color32) -> Self {
        self.fill = color;
        self
    }
    pub fn with_text_color(mut self, color: Color32) -> Self {
        self.text_color = color;
        self
    }
    pub fn with_size(mut self, size: egui::Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn show(self, ui: &mut Ui) -> Response {
        let mut button = Button::new(RichText::new(self.label).color(self.text_color).size(10.0))
            .fill(self.fill)
            .min_size(self.size)
            .rounding(8.0)
            .frame(true);
        if let Some(icon) = self.icon {
            // アイコン付き（左側）
            button = button;
            // アイコン描画はカスタム実装可
        }
        let resp = ui.add(button);
        resp
    }
} 