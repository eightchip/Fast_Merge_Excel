use std::path::PathBuf;
use rfd::FileDialog;
use egui::Ui;
use calamine::{open_workbook_auto, Reader};
use std::collections::HashSet;
use std::path::Path;
use polars::prelude::*;
use crate::components::cleaner::clean_and_infer_columns;
use crate::components::button::AppButton;
use crate::app::AppState;
use std::sync::{Arc, Mutex};
use egui::Context;

#[derive(Clone, Debug)]
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
        let labels = ["ファイルA（必須）", "ファイルB（必須）", "ファイルC（2段階結合時のみ選択）"];
        for i in 0..3 {
            ui.horizontal(|ui| {
                if AppButton::new(labels[i]).show(ui).clicked() {
                    if let Some(path) = FileDialog::new().add_filter("Excel", &["xlsx"]).pick_file() {
                        self.selected_files[i] = Some(path);
                    }
                }
                if let Some(path) = &self.selected_files[i] {
                    ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                    if i == 2 {
                        if ui.button("キャンセル").clicked() {
                            self.selected_files[i] = None;
                        }
                    }
                } else {
                    ui.label("未選択");
                }
            });
        }
        // 共通列名の抽出やcolumns_per_fileのセットはここでは行わない
        ui.add_space(10.0);
        let next_enabled = self.selected_files[0].is_some();
        if next_enabled {
            if ui.button("次へ").clicked() {
                println!("NEXT CLICKED");
                println!("CALLING ASYNC STEP TRANSITION");
                on_next();
            }
        } else {
            ui.add_enabled(false, egui::Button::new("次へ"));
        }
    }
}

pub fn get_columns_from_xlsx(path: &Path) -> Vec<String> {
    if let Ok(mut workbook) = open_workbook_auto(path) {
        let sheets = workbook.worksheets();
        if let Some((_name, sheet)) = sheets.get(0) {
            if let Some(row) = sheet.rows().next() {
                return row.iter().map(|cell| cell.to_string()).collect();
            }
        }
    }
    vec![]
}

pub fn read_xlsx_to_df(path: &Path) -> DataFrame {
    if let Ok(mut workbook) = open_workbook_auto(path) {
        let sheets = workbook.worksheets();
        if let Some((_name, sheet)) = sheets.get(0) {
            let mut rows = sheet.rows();
            let header: Vec<String> = if let Some(row) = rows.next() {
                row.iter().map(|cell| cell.to_string()).collect()
            } else { vec![] };
            let mut columns: Vec<Vec<String>> = vec![vec![]; header.len()];
            for row in rows {
                for (i, cell) in row.iter().enumerate() {
                    if i < columns.len() {
                        columns[i].push(cell.to_string());
                    }
                }
            }
            return clean_and_infer_columns(&header, &columns);
        }
    }
    DataFrame::default()
}

fn render_file_select(state: &mut AppState, ui: &mut Ui) {
    println!("[DEBUG] render_file_select step: {}, selected_files[0]: {:?}", state.step, state.file_selector.selected_files[0]);
    ui.heading("分割するExcelファイルを選択してください");
    ui.add_space(20.0);

    let mut on_next = || {};
    let mut on_columns_loaded = |columns: [Vec<String>; 3]| {
        state.key_selector.available_keys = columns[0].clone();
    };

    // デバッグ出力
    if let Some(path) = &state.file_selector.selected_files[0] {
        println!("[DEBUG] selected_files[0]: {:?}", path);
    } else {
        println!("[DEBUG] selected_files[0]: None");
    }

    // ファイル選択UIは常に表示
    state.file_selector.render(ui, &mut on_next, &mut on_columns_loaded);

    // 「次へ」ボタンだけ条件付きで有効化
    if state.file_selector.selected_files[0].is_some() {
        ui.add_space(20.0);
        if ui.button("次へ").clicked() {
            state.step = 1;
        }
    }
}

pub fn render_split_save_wizard(app_state: Arc<Mutex<AppState>>, ui: &mut Ui, ctx: &Context) {
    let mut state = app_state.lock().unwrap();
    println!("[DEBUG] split_save_wizard step: {}", state.step);
    // ...
}
