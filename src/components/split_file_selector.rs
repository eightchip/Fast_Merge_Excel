use egui::Ui;
use rfd::FileDialog;
use std::path::PathBuf;
use crate::components::button::AppButton;

pub struct SplitFileSelector {
    pub selected_file: Option<PathBuf>,
}

impl SplitFileSelector {
    pub fn new() -> Self {
        Self { selected_file: None }
    }

    pub fn render(&mut self, ui: &mut Ui, on_file_selected: &mut dyn FnMut(Option<PathBuf>)) {
        ui.horizontal(|ui| {
            if AppButton::new("ファイルを選択").show(ui).clicked() {
                if let Some(path) = FileDialog::new().add_filter("Excel", &["xlsx"]).pick_file() {
                    self.selected_file = Some(path.clone());
                    on_file_selected(Some(path));
                }
            }
            if let Some(path) = &self.selected_file {
                ui.label(path.file_name().unwrap_or_default().to_string_lossy());
            } else {
                ui.label("未選択");
            }
        });
    }
} 