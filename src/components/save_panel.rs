// src/components/save_panel.rs

use std::path::{Path, PathBuf};
use rfd::FileDialog;
use egui::Ui;
use umya_spreadsheet::{Spreadsheet, writer::xlsx, new_file};
use umya_spreadsheet::writer::xlsx::XlsxError;
use crate::components::button::AppButton;
use std::io::{Error as IoError, ErrorKind};
use std::fs;
use magic_merge_excel_2::utils::excel_style;

#[derive(Clone, Debug)]
pub enum SaveError {
    FileInUse(String),
    PermissionDenied(String),
    PathNotFound(String),
    Other(String),
}

impl From<XlsxError> for SaveError {
    fn from(error: XlsxError) -> Self {
        match error {
            XlsxError::Io(e) => {
                if e.kind() == ErrorKind::PermissionDenied {
                    SaveError::PermissionDenied(e.to_string())
                } else if e.kind() == ErrorKind::NotFound {
                    SaveError::PathNotFound(e.to_string())
                } else {
                    SaveError::Other(e.to_string())
                }
            }
            _ => SaveError::Other(error.to_string()),
        }
    }
}

impl SaveError {
    pub fn user_friendly_message(&self) -> String {
        match self {
            SaveError::FileInUse(path) => format!("ファイルが使用中です: {}", path),
            SaveError::PermissionDenied(path) => format!("アクセス権限がありません: {}", path),
            SaveError::PathNotFound(path) => format!("ファイルが見つかりません: {}", path),
            SaveError::Other(msg) => format!("保存エラー: {}", msg),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SavePanel {
    pub error: Option<SaveError>,
    pub save_path: std::path::PathBuf,
    pub last_saved_path: Option<std::path::PathBuf>,
}

impl SavePanel {
    pub fn new() -> Self {
        let default = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("merged_output.xlsx");
        Self {
            error: None,
            save_path: default,
            last_saved_path: None,
        }
    }

    pub fn render(&mut self, ui: &mut Ui, on_save: impl FnOnce(&std::path::Path) -> Result<(), SaveError>) {
        ui.horizontal(|ui| {
            if ui.button("参照...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Excel", &["xlsx"])
                    .set_directory(self.save_path.parent().unwrap_or(std::path::Path::new(".")))
                    .set_file_name(
                        self.save_path
                            .file_name()
                            .and_then(|os_str| os_str.to_str())
                            .unwrap_or("merged_output.xlsx")
                    )
                    .save_file()
                {
                    self.save_path = path;
                }
            }
            ui.label(format!("保存先: {}", self.save_path.display()));
        });
        ui.horizontal(|ui| {
            if ui.button("保存").clicked() {
                let path = &self.save_path;
                match on_save(path) {
                    Ok(_) => {
                        self.error = None;
                        self.last_saved_path = Some(path.to_path_buf());
                        // エクスプローラーでファイルを選択状態で開く
                        let _ = std::process::Command::new("explorer")
                            .arg("/select,")
                            .arg(path)
                            .spawn();
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
            }
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::RED, error.user_friendly_message());
            }
        });
        if let Some(path) = &self.last_saved_path {
            ui.label(format!("保存先: {}", path.display()));
        }
    }

    pub fn set_error(&mut self, error: SaveError) {
        self.error = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }
}

pub fn save_to_xlsx(file_name: &str, columns: &[String], data: &[Vec<String>]) -> Result<(), SaveError> {
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
    // スタイル適用
    excel_style::apply_common_style(sheet, 1, columns.len() as u32, data.len() as u32);
    // 保存実行
    xlsx::write(&book, Path::new(file_name)).map_err(SaveError::from)
}
