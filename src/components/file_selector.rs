use std::path::PathBuf;
use rfd::FileDialog;
use egui::Ui;
use calamine::{open_workbook_auto, Reader};
use std::collections::HashSet;

pub struct FileSelector {
    pub selected_files: [Option<PathBuf>; 3], // A, B, C
    pub columns_per_file: [Vec<String>; 3],  // 各ファイルの列名
}

impl FileSelector {
    pub fn new() -> Self {
        FileSelector {
            selected_files: [None, None, None], // 3ファイル分
            columns_per_file: [vec![], vec![], vec![]],
        }
    }

    pub fn render(&mut self, ui: &mut Ui, on_next: &mut dyn FnMut(), on_columns_loaded: &mut dyn FnMut([Vec<String>; 3])) {
        let labels = ["ファイルA（必須）", "ファイルB（必須）", "ファイルC（任意）"];
        for i in 0..3 {
            ui.horizontal(|ui| {
                if ui.button(labels[i]).clicked() {
                    if let Some(path) = FileDialog::new().add_filter("Excel", &["xlsx"]).pick_file() {
                        // 列名取得
                        let mut columns = Vec::new();
                        if let Ok(mut workbook) = open_workbook_auto(&path) {
                            let sheets = workbook.worksheets();
                            if let Some((_name, sheet)) = sheets.get(0) {
                                if let Some(row) = sheet.rows().next() {
                                    columns = row.iter().map(|cell| cell.to_string()).collect();
                                }
                            }
                        }
                        self.selected_files[i] = Some(path);
                        self.columns_per_file[i] = columns;
                    }
                }
                if let Some(path) = &self.selected_files[i] {
                    ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                } else {
                    ui.label("未選択");
                }
            });
        }
        // 共通列名の抽出
        let selected_columns: Vec<&Vec<String>> = self.selected_files.iter().enumerate()
            .filter_map(|(i, f)| if f.is_some() { Some(&self.columns_per_file[i]) } else { None })
            .collect();
        let common_columns = if selected_columns.len() >= 2 {
            let mut iter = selected_columns.iter();
            if let Some(first) = iter.next() {
                let mut set: HashSet<String> = first.iter().cloned().collect();
                for cols in iter {
                    set = set.intersection(&cols.iter().cloned().collect()).cloned().collect();
                }
                set.into_iter().collect::<Vec<_>>()
            } else {
                vec![]
            }
        } else if selected_columns.len() == 1 {
            selected_columns[0].clone()
        } else {
            vec![]
        };
        if self.selected_files.iter().any(|f| f.is_some()) {
            on_columns_loaded(self.columns_per_file.clone());
        }
        ui.add_space(10.0);
        let selected_count = self.selected_files[0..2].iter().filter(|f| f.is_some()).count();
        let next_enabled = selected_count >= 2;
        if ui.add_enabled(next_enabled, egui::Button::new("次へ")).clicked() {
            if next_enabled {
                on_next();
            }
        }
    }
}
