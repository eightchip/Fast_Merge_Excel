use egui::{Context, Ui};
use crate::app::AppState;
use std::sync::{Arc, Mutex};
use crate::components::split_file_selector::SplitFileSelector;
use crate::components::key_selector::KeySelector;
use crate::components::preview_table::PreviewTable;
use crate::components::save_panel::SavePanel;
use polars::prelude::*;
use calamine::{open_workbook_auto, Reader};
use crate::components::file_selector::get_columns_from_xlsx;
use umya_spreadsheet::{Spreadsheet, Worksheet, writer::xlsx};
use std::path::Path;
use polars::prelude::PolarsError;

pub fn render_split_save_wizard(app_state: Arc<Mutex<AppState>>, ui: &mut Ui, ctx: &Context) {
    let mut state = app_state.lock().unwrap();
    
    match state.step {
        0 => render_file_select(&mut state, ui),
        1 => render_key_select(&mut state, ui),
        2 => render_preview(&mut state, ui, ctx),
        3 => render_save_columns(&mut state, ui),
        4 => render_save_complete(&mut state, ui),
        _ => {}
    }
}

fn render_file_select(state: &mut AppState, ui: &mut Ui) {
    ui.heading("分割するExcelファイルを選択してください");
    ui.add_space(20.0);
    let mut on_file_selected = |path: Option<std::path::PathBuf>| {
        state.split_file_path = path.clone();
        if let Some(ref p) = path {
            let columns = get_columns_from_xlsx(p);
            state.key_selector.available_keys = columns;
        } else {
            state.key_selector.available_keys.clear();
        }
    };
    state.split_file_selector.render(ui, &mut on_file_selected);
    if state.split_file_path.is_some() {
        let all_columns = get_columns_from_xlsx(state.split_file_path.as_ref().unwrap());
        state.split_all_columns = all_columns;
        ui.add_space(20.0);
        if ui.button("次へ").clicked() {
            state.step = 1;
        }
    }
}

fn render_key_select(state: &mut AppState, ui: &mut Ui) {
    ui.heading("分割の基準となる主キーを選択してください");
    ui.add_space(20.0);
    let mut on_next = || {
        state.step = 2;
    };
    state.key_selector.render(ui, &mut on_next);
    if !state.key_selector.selected_keys.is_empty() {
        ui.add_space(20.0);
        if ui.button("次へ").clicked() {
            state.step = 2;
        }
    }
}

fn render_preview(state: &mut AppState, ui: &mut Ui, _ctx: &Context) {
    ui.heading("分割プレビュー");
    ui.add_space(20.0);
    let need_recalc =
        state.split_preview_table.is_none()
        || state.split_preview_last_file.as_ref() != state.split_file_path.as_ref()
        || state.split_preview_last_keys.as_ref() != Some(&state.key_selector.selected_keys);
    if let (Some(path), keys) = (&state.split_file_path, &state.key_selector.selected_keys) {
        if !keys.is_empty() {
            if need_recalc {
                // ファイルをDataFrameで読み込み
                let df = crate::components::file_selector::read_xlsx_to_df(path);
                // 主キーでグループ化し件数を集計
                let groupby = df.group_by(keys).unwrap();
                let groups = groupby.get_groups();
                let counts = groupby.count().unwrap();
                // 件数カラム名を自動検出
                let col_names = counts.get_column_names();
                let count_col_name = col_names
                    .iter()
                    .find(|name| name.ends_with("_count"))
                    .expect("件数カラムが見つかりません");
                let count_col = counts.column(count_col_name).unwrap();
                // 合計ファイル数（主キーのユニーク数）
                let total_files = counts.height();
                // プレビューは最大20件まで
                let preview_limit = 20.min(counts.height());
                let key_cols: Vec<_> = keys.iter().map(|k| counts.column(k).unwrap()).collect();
                let mut table: Vec<Vec<String>> = vec![];
                for i in 0..preview_limit {
                    let mut row: Vec<String> = vec![];
                    for col in &key_cols {
                        let mut val = col.get(i).unwrap().to_string();
                        if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                            val = val[1..val.len()-1].to_string();
                        }
                        row.push(val);
                    }
                    let mut count_val = count_col.get(i).unwrap().to_string();
                    if count_val.starts_with('"') && count_val.ends_with('"') && count_val.len() >= 2 {
                        count_val = count_val[1..count_val.len()-1].to_string();
                    }
                    row.push(count_val);
                    table.push(row);
                }
                // ヘッダー
                let mut headers = keys.clone();
                headers.push("件数".to_string());
                // プレビュー用tableを主キー1列目で昇順ソート
                table.sort_by(|a, b| a[0].cmp(&b[0]));
                // キャッシュ
                state.split_preview_table = Some((headers, table));
                state.split_preview_total_files = Some(total_files);
                state.split_preview_last_file = state.split_file_path.clone();
                state.split_preview_last_keys = Some(keys.clone());
            }
            // キャッシュから表示
            let (headers, table) = state.split_preview_table.as_ref().unwrap();
            let total_files = state.split_preview_total_files.unwrap_or(0);
            ui.label(format!("分割されるファイル数: {}", total_files));
            ui.add_space(10.0);
            egui::Grid::new("split_preview_table").striped(true).show(ui, |ui| {
                for h in headers {
                    ui.heading(h);
                }
                ui.end_row();
                for row in table {
                    for cell in row {
                        ui.label(cell);
                    }
                    ui.end_row();
                }
            });
            if total_files > 20 {
                ui.label(format!("...（{}件中、上位20件のみ表示）", total_files));
            }
        } else {
            ui.colored_label(egui::Color32::RED, "主キーが選択されていません");
            state.split_preview_table = None;
            state.split_preview_total_files = None;
            state.split_preview_last_file = None;
            state.split_preview_last_keys = None;
        }
    } else {
        ui.colored_label(egui::Color32::RED, "ファイルまたは主キーが未選択です");
        state.split_preview_table = None;
        state.split_preview_total_files = None;
        state.split_preview_last_file = None;
        state.split_preview_last_keys = None;
    }
    // 画面遷移ボタン
    ui.add_space(20.0);
    ui.horizontal(|ui| {
        if ui.button("前へ").clicked() {
            if state.step > 0 {
                state.step -= 1;
            }
        }
        if ui.button("次へ").clicked() {
            state.step += 1;
        }
    });
}

fn render_save_columns(state: &mut AppState, ui: &mut Ui) {
    // 初期化: split_columns, split_columns_selected
    if state.split_columns.is_empty() {
        let all_columns = if !state.split_all_columns.is_empty() {
            &state.split_all_columns
        } else if let Some((headers, _)) = &state.split_preview_table {
            // 件数列は除外
            &headers.iter().filter(|c| *c != "件数").cloned().collect::<Vec<_>>()
        } else {
            &vec![]
        };
        state.split_columns = all_columns.clone();
        state.split_columns_selected = all_columns.iter().map(|_| true).collect();
    }
    ui.heading("分割保存する列を選択・並べ替え");
    ui.add_space(10.0);
    // 全選択・全解除ボタン
    ui.horizontal(|ui| {
        if ui.button("全選択").clicked() {
            for (i, col) in state.split_columns.iter().enumerate() {
                if !state.key_selector.selected_keys.contains(col) {
                    state.split_columns_selected[i] = true;
                }
            }
        }
        if ui.button("全解除").clicked() {
            for (i, col) in state.split_columns.iter().enumerate() {
                if !state.key_selector.selected_keys.contains(col) {
                    state.split_columns_selected[i] = false;
                }
            }
        }
    });
    ui.add_space(10.0);
    // 列リスト
    egui::Grid::new("split_columns_grid").striped(true).show(ui, |ui| {
        for i in 0..state.split_columns.len() {
            let col = state.split_columns[i].clone();
            let is_key = state.key_selector.selected_keys.contains(&col);
            // チェックボックス
            if is_key {
                ui.add_enabled(false, egui::Checkbox::new(&mut true, "")); // 主キーは固定
            } else {
                ui.checkbox(&mut state.split_columns_selected[i], "");
            }
            // ↑↓ボタン
            if ui.button("↑").clicked() && i > 0 {
                state.split_columns.swap(i, i-1);
                state.split_columns_selected.swap(i, i-1);
            }
            if ui.button("↓").clicked() && i+1 < state.split_columns.len() {
                state.split_columns.swap(i, i+1);
                state.split_columns_selected.swap(i, i+1);
            }
            // 列名（クリックでソート）
            let mut sort_icon = "";
            if let Some((sort_idx, asc)) = state.split_columns_sort {
                if sort_idx == i { sort_icon = if asc { "↑" } else { "↓" }; }
            }
            if ui.button(format!("{}{}", col, sort_icon)).clicked() {
                if let Some((sort_idx, asc)) = state.split_columns_sort {
                    if sort_idx == i {
                        state.split_columns_sort = Some((i, !asc)); // 昇降反転
                    } else {
                        state.split_columns_sort = Some((i, true));
                    }
                } else {
                    state.split_columns_sort = Some((i, true));
                }
                // ソート実行
                let asc = state.split_columns_sort.unwrap().1;
                let mut zipped: Vec<_> = state.split_columns.iter().cloned().zip(state.split_columns_selected.iter().cloned()).collect();
                zipped.sort_by(|a, b| if asc { a.0.cmp(&b.0) } else { b.0.cmp(&a.0) });
                let (cols, sels): (Vec<_>, Vec<_>) = zipped.into_iter().unzip();
                state.split_columns = cols;
                state.split_columns_selected = sels;
            }
            ui.end_row();
        }
    });
    ui.add_space(20.0);
    ui.horizontal(|ui| {
        if ui.button("前へ").clicked() {
            if state.step > 0 {
                state.step -= 1;
            }
        }
        if ui.button("次へ").clicked() {
            // 保存処理
            if let (Some(path), keys) = (&state.split_file_path, &state.key_selector.selected_keys) {
                use chrono::Local;
                use std::fs;
                use std::path::PathBuf;
                let now = Local::now().format("%y%m%d%H%M%S").to_string();
                let out_dir = PathBuf::from(format!("./{}", now));
                let _ = fs::create_dir_all(&out_dir);
                // Polarsで元データを読み込み
                let df = crate::components::file_selector::read_xlsx_to_df(path);
                // 選択された列だけ抽出
                let selected_cols: Vec<String> = state.split_columns.iter().enumerate()
                    .filter(|(i, _)| state.split_columns_selected[*i])
                    .map(|(_, c)| c.clone()).collect();
                let df = df.select(&selected_cols).unwrap();
                // 主キーごとにグループ化
                let groupby = df.group_by(keys).unwrap();
                groupby.apply(|sub_df| {
                    if sub_df.height() == 0 {
                        return Ok(sub_df.clone());
                    }
                    // sub_df: このグループのDataFrame
                    // 主キー値はsub_dfの最初の行から取得
                    let key_vals: Vec<String> = keys.iter()
                        .map(|k| sub_df.column(k).unwrap().get(0).unwrap().to_string())
                        .collect();
                    let key_str = key_vals.iter()
                        .map(|v| sanitize_filename(v))
                        .collect::<Vec<_>>()
                        .join("_");
                    // ファイル名生成・保存処理
                    let file_path = out_dir.join(format!("{}.xlsx", key_str));
                    println!("Saving: {:?}", file_path);
                    let mut book = Spreadsheet::default();
                    let sheet = book.new_sheet("Sheet1").map_err(|e| PolarsError::ComputeError(format!("{:?}", e).into()))?;
                    // ヘッダー
                    for (col_idx, name) in sub_df.get_column_names().iter().enumerate() {
                        sheet.get_cell_mut(((col_idx + 1) as u32, 1u32)).set_value(name.to_string());
                    }
                    // データ
                    for row_idx in 0..sub_df.height() {
                        let row = sub_df.get_row(row_idx)?;
                        for (col_idx, val) in row.0.iter().enumerate() {
                            let mut s = val.to_string();
                            if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                                s = s[1..s.len()-1].to_string();
                            }
                            sheet.get_cell_mut(((col_idx + 1) as u32, (row_idx + 2) as u32)).set_value(s);
                        }
                    }
                    if let Err(e) = xlsx::write(&book, &file_path) {
                        state.save_error_message = Some(format!("保存失敗: {}", e));
                        return Err(PolarsError::ComputeError(format!("保存失敗: {}", e).into()));
                    }
                    Ok(sub_df.clone())
                }).unwrap();
            }
            state.step += 1;
        }
    });
}

fn render_save_complete(state: &mut AppState, ui: &mut Ui) {
    if let Some(msg) = &state.save_error_message {
        ui.colored_label(egui::Color32::RED, format!("保存エラー: {}", msg));
    } else {
        ui.heading("分割保存が完了しました！");
        ui.label("出力フォルダを開くなどの案内をここに表示できます。");
    }
    if ui.button("最初に戻る").clicked() {
        *state = AppState::new();
    }
}

fn sanitize_filename(s: &str) -> String {
    // Windowsで使えない文字を全て_に置換
    let forbidden = ['<', '>', ':', '\"', '/', '\\', '|', '?', '*'];
    s.chars()
        .map(|c| if forbidden.contains(&c) { '_' } else { c })
        .collect()
}

fn save_df_to_excel(sub_df: &DataFrame, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut book = Spreadsheet::default();
    let sheet = book.new_sheet("Sheet1")?;
    // ヘッダー
    for (col_idx, name) in sub_df.get_column_names().iter().enumerate() {
        sheet.get_cell_mut(((col_idx + 1) as u32, 1u32)).set_value(name.to_string());
    }
    // データ
    for row_idx in 0..sub_df.height() {
        let row = sub_df.get_row(row_idx)?;
        for (col_idx, val) in row.0.iter().enumerate() {
            let mut s = val.to_string();
            if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                s = s[1..s.len()-1].to_string();
            }
            sheet.get_cell_mut(((col_idx + 1) as u32, (row_idx + 2) as u32)).set_value(s);
        }
    }
    xlsx::write(&book, file_path)?;
    Ok(())
} 