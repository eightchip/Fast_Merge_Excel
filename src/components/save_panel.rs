use std::path::{Path, PathBuf};
use rfd::FileDialog;
use egui::Ui;
use umya_spreadsheet::{Spreadsheet, writer::xlsx, new_file};
use umya_spreadsheet::writer::xlsx::XlsxError;

pub struct SavePanel {
    pub save_path: Option<PathBuf>, // 保存先のパスを保持
    pub file_name: String, // 入力中のファイル名
}

impl SavePanel {
    pub fn new() -> Self {
        SavePanel {
            save_path: None,
            file_name: "merged_output.xlsx".to_string(),
        }
    }

    pub fn set_save_path(&mut self, path: PathBuf) {
        self.save_path = Some(path.clone());
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            self.file_name = name.to_string();
        }
    }

    pub fn render(&mut self, ui: &mut Ui, on_save: &mut dyn FnMut(&str)) {
        ui.label("保存先ファイル名（.xlsx）");
        ui.horizontal(|ui| {
            if ui.button("参照...").clicked() {
                if let Some(path) = FileDialog::new().add_filter("XLSX", &["xlsx"]).set_directory(".").save_file() {
                    self.set_save_path(path);
                }
            }
            ui.text_edit_singleline(&mut self.file_name);
        });
        let valid = self.file_name.ends_with(".xlsx") && !self.file_name.trim().is_empty();
        if !self.file_name.ends_with(".xlsx") {
            ui.colored_label(egui::Color32::RED, ".xlsx形式のみ許可されます");
        }
        ui.add_space(10.0);
        if ui.add_enabled(valid, egui::Button::new("保存")).clicked() {
            if valid {
                on_save(&self.file_name);
            }
        }
    }
}

pub fn save_to_xlsx(file_name: &str, columns: &[String], data: &[Vec<String>]) -> Result<(), XlsxError> {
    let mut book = new_file();
    let sheet = book.get_sheet_by_name_mut("Sheet1").unwrap();

    // ヘッダー
    for (c, h) in columns.iter().enumerate() {
        sheet.get_cell_mut((c as u32 + 1, 1)).set_value(h);
    }
    // データ
    for (r, row) in data.iter().enumerate() {
        for (c, val) in row.iter().enumerate() {
            sheet.get_cell_mut((c as u32 + 1, (r + 2) as u32)).set_value(val);
        }
    }
    xlsx::write(&book, Path::new(file_name))?;
    Ok(())
}

