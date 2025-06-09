use egui::Ui;
use rfd::FileDialog;
use std::path::PathBuf;
use crate::components::button::AppButton;

pub struct MultiStageFileSelector {
    pub selected_files: [Option<PathBuf>; 3], // A, B, C
}

impl MultiStageFileSelector {
    pub fn new() -> Self {
        MultiStageFileSelector {
            selected_files: [None, None, None],
        }
    }

    pub fn render(&mut self, ui: &mut Ui, on_next: &mut dyn FnMut()) {
        let labels = ["ファイルA", "ファイルB", "ファイルC"];
        for i in 0..3 {
            ui.horizontal(|ui| {
                if AppButton::new(labels[i]).show(ui).clicked() {
                    if let Some(path) = FileDialog::new().add_filter("Excel", &["xlsx"]).pick_file() {
                        self.selected_files[i] = Some(path);
                    }
                }
                if let Some(path) = &self.selected_files[i] {
                    ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                } else {
                    ui.label("未選択");
                }
            });
        }
        ui.add_space(10.0);
        let next_enabled = self.selected_files.iter().all(|f| f.is_some());
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