use egui::{Context, Ui};
use crate::components::file_selector::FileSelector;
use crate::components::key_selector::KeySelector;
use crate::components::join_type_picker::{JoinTypePicker, JoinType};
use crate::components::column_selector::ColumnSelector;
use crate::components::preview_table::PreviewTable;
use crate::components::save_panel::{SavePanel, save_to_xlsx};
use calamine::{open_workbook_auto, DataType};
use calamine::Reader;
use polars::prelude::*;
use crate::components::join_type_picker::to_polars_join_type;
use std::collections::HashSet;
use crate::components::sort_settings::{SortSettings, SortOrder, SortKey};
use umya_spreadsheet::Spreadsheet;
use umya_spreadsheet::writer::xlsx;
use std::cell::RefCell;
use std::process::Command;
use crate::components::cleaner::clean_and_infer_columns;

pub struct App {
    file_selector: FileSelector,
    key_selector: KeySelector,
    join_type_picker: JoinTypePicker,
    column_selector: ColumnSelector,
    preview_table: PreviewTable,
    save_panel: SavePanel,
    step: u8, // 0:ファイル選択, 1:キー選択, ...
    ab_common_keys: Vec<String>,
    bc_common_keys: Vec<String>,
    selected_ab_keys: Vec<String>,
    selected_bc_keys: Vec<String>,
    sort_settings: SortSettings,
}

impl App {
    pub fn new() -> Self {
        App {
            file_selector: FileSelector::new(),
            key_selector: KeySelector::new(),
            join_type_picker: JoinTypePicker::new(),
            column_selector: ColumnSelector::new(),
            preview_table: PreviewTable::new(),
            save_panel: SavePanel::new(),
            step: 0,
            ab_common_keys: vec![],
            bc_common_keys: vec![],
            selected_ab_keys: vec![],
            selected_bc_keys: vec![],
            sort_settings: SortSettings::new(vec![]),
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.step {
                0 => self.render_file_selector(ui),
                1 => self.render_key_selector(ui),
                2 => self.render_join_type_picker(ui),
                3 => self.render_column_selector(ui),
                4 => self.render_preview_table(ui),
                5 => self.render_save_panel(ui),
                _ => {},
            }
        });
    }

    fn render_file_selector(&mut self, ui: &mut Ui) {
        ui.heading("ファイル選択");
        let mut on_next = || {
            self.step = 1;
        };
        let mut on_columns_loaded = |cols: [Vec<String>; 3]| {
            // AとBの共通列
            if !cols[0].is_empty() && !cols[1].is_empty() {
                let set_a: HashSet<_> = cols[0].iter().cloned().collect();
                let set_b: HashSet<_> = cols[1].iter().cloned().collect();
                self.ab_common_keys = set_a.intersection(&set_b).cloned().collect();
            } else {
                self.ab_common_keys = vec![];
            }
            // BとCの共通列
            if !cols[1].is_empty() && !cols[2].is_empty() {
                let set_b: HashSet<_> = cols[1].iter().cloned().collect();
                let set_c: HashSet<_> = cols[2].iter().cloned().collect();
                self.bc_common_keys = set_b.intersection(&set_c).cloned().collect();
            } else {
                self.bc_common_keys = vec![];
            }
            // 出力列候補は全ファイルの和集合
            let mut all_cols: HashSet<String> = HashSet::new();
            for colset in cols.iter() {
                for c in colset {
                    all_cols.insert(c.clone());
                }
            }
            self.column_selector.available_columns = all_cols.into_iter().collect();
            println!("available_columns: {:?}", self.column_selector.available_columns);
        };
        self.file_selector.render(ui, &mut on_next, &mut on_columns_loaded);
    }

    fn render_key_selector(&mut self, ui: &mut Ui) {
        ui.heading("結合キー選択");
        ui.label("AとBの共通列からキーを選択（複数可）");
        for key in &self.ab_common_keys {
            let mut checked = self.selected_ab_keys.contains(key);
            if ui.checkbox(&mut checked, key).changed() {
                if checked {
                    if !self.selected_ab_keys.contains(key) {
                        self.selected_ab_keys.push(key.clone());
                    }
                } else {
                    self.selected_ab_keys.retain(|k| k != key);
                }
            }
        }
        if self.ab_common_keys.is_empty() {
            ui.colored_label(egui::Color32::RED, "AとBに共通する列がありません");
        }
        // Cが選択されている場合のみBとCのキー選択
        if self.file_selector.selected_files[2].is_some() {
            ui.add_space(10.0);
            ui.label("BとCの共通列からキーを選択（複数可）");
            for key in &self.bc_common_keys {
                let mut checked = self.selected_bc_keys.contains(key);
                if ui.checkbox(&mut checked, key).changed() {
                    if checked {
                        if !self.selected_bc_keys.contains(key) {
                            self.selected_bc_keys.push(key.clone());
                        }
                    } else {
                        self.selected_bc_keys.retain(|k| k != key);
                    }
                }
            }
            if self.bc_common_keys.is_empty() {
                ui.colored_label(egui::Color32::RED, "BとCに共通する列がありません");
            }
        }
        ui.add_space(10.0);
        let ab_ok = !self.selected_ab_keys.is_empty();
        let bc_ok = self.file_selector.selected_files[2].is_none() || !self.selected_bc_keys.is_empty();
        let next_enabled = ab_ok && bc_ok;
        if ui.add_enabled(next_enabled, egui::Button::new("次へ")).clicked() {
            if next_enabled {
                self.step = 2;
            }
        }
        if self.step > 0 && ui.button("前へ").clicked() {
            self.step -= 1;
        }
    }

    fn render_join_type_picker(&mut self, ui: &mut Ui) {
        ui.heading("結合形式選択");
        let mut on_next = || {
            self.step = 3;
        };
        self.join_type_picker.render(ui, &mut on_next);
        if self.step > 0 && ui.button("前へ").clicked() {
            self.step -= 1;
        }
    }

    fn render_column_selector(&mut self, ui: &mut Ui) {
        ui.heading("Column Selector");
        let selected_columns = self.column_selector.selected_columns.clone();
        let file_paths: Vec<_> = self.file_selector.selected_files.iter()
            .filter_map(|f| f.as_ref())
            .collect();
        println!("on_next called! file_paths: {:?}, selected_columns: {:?}", file_paths, self.column_selector.selected_columns);

        // ソート設定の候補を選択列に合わせて更新
        self.sort_settings.candidates = selected_columns.clone();
        self.sort_settings.sort_keys.retain(|k| self.sort_settings.candidates.contains(&k.column));

        let mut on_next = || {
            if file_paths.len() == 2 && !self.selected_ab_keys.is_empty() {
                // --- 2ファイルjoin ---
                let mut dfs = vec![];
                for path in &file_paths {
                    if let Ok(mut workbook) = calamine::open_workbook_auto(path) {
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
                            let df = clean_and_infer_columns(&header, &columns);
                            dfs.push(df);
                        }
                    }
                }
                if dfs.len() == 2 {
                    let left = &dfs[0];
                    let right = &dfs[1];
                    let join_keys: Vec<&str> = self.selected_ab_keys.iter().map(|s| s.as_str()).collect();
                    let polars_join_type = to_polars_join_type(self.join_type_picker.selected_join_type.as_ref().unwrap());
                    let jt = self.join_type_picker.selected_join_type.as_ref().unwrap();

                    let (left, right) = match jt {
                        JoinType::Right => (&dfs[1], &dfs[0]),
                        _ => (&dfs[0], &dfs[1]),
                    };

                    let mut joined = left.join(
                        right,
                        &join_keys,
                        &join_keys,
                        polars_join_type.into(),
                    ).unwrap();
                    let out_cols: Vec<&str> = selected_columns.iter().map(|s| s.as_str()).collect();

                    // --- ソート前にソートキーを数値型にキャスト ---
                    if !self.sort_settings.sort_keys.is_empty() {
                        for sort_key in &self.sort_settings.sort_keys {
                            let col = sort_key.column.as_str();
                            if let Ok(series) = joined.column(col) {
                                if series.dtype() == &polars::prelude::DataType::Utf8 {
                                    // まずInt64で試し、だめならFloat64
                                    let parsed_int = series.utf8().unwrap().into_iter().map(|opt| opt.and_then(|s| s.parse::<i64>().ok())).collect::<Int64Chunked>();
                                    if parsed_int.null_count() < parsed_int.len() {
                                        joined.with_column(parsed_int.into_series()).unwrap();
                                    } else {
                                        let parsed_float = series.utf8().unwrap().into_iter().map(|opt| opt.and_then(|s| s.parse::<f64>().ok())).collect::<Float64Chunked>();
                                        joined.with_column(parsed_float.into_series()).unwrap();
                                    }
                                }
                            }
                        }
                    }
                    // --- ソート ---
                    let mut out_df = match joined.select(&out_cols) {
                        Ok(df) => df,
                        Err(e) => {
                            println!("select error: {:?}", e);
                            return;
                        }
                    };
                    if !self.sort_settings.sort_keys.is_empty() {
                        let by: Vec<&str> = self.sort_settings.sort_keys.iter().map(|k| k.column.as_str()).collect();
                        let reverse: Vec<bool> = self.sort_settings.sort_keys.iter().map(|k| k.order == SortOrder::Descending).collect();
                        out_df = out_df.sort(&by, reverse, false).unwrap_or(out_df);
                    }
                    // --- プレビュー・保存用に全カラムをUtf8型にキャスト ---
                    for col in &out_cols {
                        if let Ok(series) = out_df.column(col) {
                            if series.dtype() != &polars::prelude::DataType::Utf8 {
                                let casted = series.cast(&polars::prelude::DataType::Utf8).unwrap();
                                out_df.with_column(casted).unwrap();
                            }
                        }
                    }
                    // プレビュー用データ
                    let mut preview_data = vec![];
                    for row in 0..out_df.height() {
                        let mut row_vec = vec![];
                        for col in &out_cols {
                            let val = out_df.column(col).unwrap().utf8().unwrap().get(row).unwrap_or("").to_string();
                            row_vec.push(val);
                        }
                        preview_data.push(row_vec);
                    }
                    self.preview_table.columns = selected_columns.clone();
                    self.preview_table.preview_data = preview_data;
                    println!("columns: {:?}", self.preview_table.columns);
                    println!("preview_data: {:?}", self.preview_table.preview_data);
                    println!("selected_columns: {:?}", selected_columns);
                }
            } else if file_paths.len() == 3 && !self.selected_ab_keys.is_empty() && !self.selected_bc_keys.is_empty() {
                // --- 3ファイルjoin ---
                let mut dfs = vec![];
                for path in &file_paths {
                    if let Ok(mut workbook) = calamine::open_workbook_auto(path) {
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
                            let df = clean_and_infer_columns(&header, &columns);
                            dfs.push(df);
                        }
                    }
                }
                if dfs.len() == 3 {
                    // 1. A+B join
                    let ab_keys: Vec<&str> = self.selected_ab_keys.iter().map(|s| s.as_str()).collect();
                    let polars_join_type = to_polars_join_type(self.join_type_picker.selected_join_type.as_ref().unwrap());
                    let ab_joined = dfs[0].join(
                        &dfs[1],
                        &ab_keys,
                        &ab_keys,
                        polars_join_type.clone().into(),
                    ).unwrap();

                    // 2. (A+B)+C join
                    let bc_keys: Vec<&str> = self.selected_bc_keys.iter().map(|s| s.as_str()).collect();
                    let mut abc_joined = ab_joined.join(
                        &dfs[2],
                        &bc_keys,
                        &bc_keys,
                        polars_join_type.into(),
                    ).unwrap();

                    let out_cols: Vec<&str> = selected_columns.iter().map(|s| s.as_str()).collect();
                    // --- ソート前にソートキーを数値型にキャスト ---
                    if !self.sort_settings.sort_keys.is_empty() {
                        for sort_key in &self.sort_settings.sort_keys {
                            let col = sort_key.column.as_str();
                            if let Ok(series) = abc_joined.column(col) {
                                if series.dtype() == &polars::prelude::DataType::Utf8 {
                                    let parsed_int = series.utf8().unwrap().into_iter().map(|opt| opt.and_then(|s| s.parse::<i64>().ok())).collect::<Int64Chunked>();
                                    if parsed_int.null_count() < parsed_int.len() {
                                        abc_joined.with_column(parsed_int.into_series()).unwrap();
                                    } else {
                                        let parsed_float = series.utf8().unwrap().into_iter().map(|opt| opt.and_then(|s| s.parse::<f64>().ok())).collect::<Float64Chunked>();
                                        abc_joined.with_column(parsed_float.into_series()).unwrap();
                                    }
                                }
                            }
                        }
                    }
                    // --- ソート ---
                    let mut out_df = match abc_joined.select(&out_cols) {
                        Ok(df) => df,
                        Err(e) => {
                            println!("select error: {:?}", e);
                            return;
                        }
                    };
                    if !self.sort_settings.sort_keys.is_empty() {
                        let by: Vec<&str> = self.sort_settings.sort_keys.iter().map(|k| k.column.as_str()).collect();
                        let reverse: Vec<bool> = self.sort_settings.sort_keys.iter().map(|k| k.order == SortOrder::Descending).collect();
                        out_df = out_df.sort(&by, reverse, false).unwrap_or(out_df);
                    }
                    // --- プレビュー・保存用に全カラムをUtf8型にキャスト ---
                    for col in &out_cols {
                        if let Ok(series) = out_df.column(col) {
                            if series.dtype() != &polars::prelude::DataType::Utf8 {
                                let casted = series.cast(&polars::prelude::DataType::Utf8).unwrap();
                                out_df.with_column(casted).unwrap();
                            }
                        }
                    }
                    // プレビュー用データ
                    let mut preview_data = vec![];
                    for row in 0..out_df.height() {
                        let mut row_vec = vec![];
                        for col in &out_cols {
                            let val = out_df.column(col).unwrap().utf8().unwrap().get(row).unwrap_or("").to_string();
                            row_vec.push(val);
                        }
                        preview_data.push(row_vec);
                    }
                    self.preview_table.columns = selected_columns.clone();
                    self.preview_table.preview_data = preview_data;
                    println!("columns: {:?}", self.preview_table.columns);
                    println!("preview_data: {:?}", self.preview_table.preview_data);
                    println!("selected_columns: {:?}", selected_columns);
                }
            } else {
                println!("条件NG: file_paths.len() = {}, selected_ab_keys = {:?}, selected_bc_keys = {:?}", file_paths.len(), self.selected_ab_keys, self.selected_bc_keys);
            }
            self.step = 4;
        };
        self.column_selector.render(ui, &mut on_next);
        // ソートキーUIを下に表示
        self.sort_settings.render(ui);
        if self.step > 0 && ui.button("前へ").clicked() {
            self.step -= 1;
        }
    }

    fn render_preview_table(&mut self, ui: &mut Ui) {
        ui.heading("プレビュー");
        let mut on_next = || {
            self.step = 5;
        };
        self.preview_table.render(ui, &mut on_next);
        if self.step > 0 && ui.button("前へ").clicked() {
            self.step -= 1;
        }
    }

    pub(crate) fn open_in_explorer(path: &str) {
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer")
                .arg("/select,")
                .arg(path)
                .spawn();
        }
    }

    fn render_save_panel(&mut self, ui: &mut Ui) {
        ui.heading("保存");
        thread_local! {
            static LAST_SAVED: RefCell<Option<String>> = RefCell::new(None);
            static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
        }
        let mut on_save = |file_name: &str| {
            match save_to_xlsx(file_name, &self.preview_table.columns, &self.preview_table.preview_data) {
                Ok(_) => {
                    LAST_SAVED.with(|s| *s.borrow_mut() = Some(file_name.to_string()));
                    LAST_ERROR.with(|e| *e.borrow_mut() = None);
                    Self::open_in_explorer(file_name);
                }
                Err(e) => {
                    LAST_ERROR.with(|err| *err.borrow_mut() = Some(format!("保存に失敗しました: {:?}", e)));
                }
            }
        };
        self.save_panel.render(ui, &mut on_save);
        LAST_SAVED.with(|s| {
            if let Some(saved_file) = &*s.borrow() {
                ui.colored_label(egui::Color32::BLUE, format!("{} に保存しました", saved_file));
            }
        });
        LAST_ERROR.with(|e| {
            if let Some(error) = &*e.borrow() {
                ui.colored_label(egui::Color32::RED, error);
            }
        });
        if self.step > 0 && ui.button("前へ").clicked() {
            self.step -= 1;
        }
        if ui.button("最初に戻る").clicked() {
            self.step = 0;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.update(ctx);
    }
}
