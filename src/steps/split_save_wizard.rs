use egui::{Context, Ui};
use crate::app::AppState;
use std::sync::{Arc, Mutex};
// use crate::components::split_file_selector::SplitFileSelector;
// use crate::components::key_selector::KeySelector;
// use crate::components::preview_table::PreviewTable;
// use crate::components::save_panel::SavePanel;
use polars::prelude::*;
use calamine::Reader;
// use crate::components::file_selector::get_columns_from_xlsx;
use umya_spreadsheet::{Spreadsheet, writer::xlsx};
use std::path::Path;
// use polars::prelude::PolarsError;
// use crate::components::button::AppButton;
// use std::process::Command;
use std::path::PathBuf;
use magic_merge_excel_2::utils::excel;

pub fn render_split_save_wizard(app_state: Arc<Mutex<AppState>>, ui: &mut Ui, ctx: &Context) {
    let mut state = app_state.lock().unwrap();
    
    match state.step {
        1 => render_file_select(&mut state, ui),
        2 => render_key_select(&mut state, ui),
        3 => render_preview(&mut state, ui, ctx),
        4 => render_save_columns(&mut state, ui),
        5 => render_save_complete(&mut state, ui),
        _ => {}
    }
}

fn render_file_select(state: &mut AppState, ui: &mut Ui) {
    ui.heading("分割するExcelファイルを選択してください");
    ui.add_space(20.0);
    let mut on_file_selected = |path: Option<std::path::PathBuf>| {
        state.split_file_path = path.clone();
        if let Some(ref p) = path {
            let columns = excel::get_available_columns(p).unwrap_or_default();
            state.key_selector.available_keys = columns;
        } else {
            state.key_selector.available_keys.clear();
        }
    };
    state.split_file_selector.render(ui, &mut on_file_selected);

    ui.add_space(20.0);
    ui.horizontal(|ui| {
        if ui.button("前へ").clicked() {
            state.step = 0;
        }
        let next_enabled = state.split_file_path.is_some();
        if next_enabled {
            if ui.button("次へ").clicked() {
                state.step = 2;
            }
        } else {
            ui.add_enabled(false, egui::Button::new("次へ"));
        }
    });
}

fn render_key_select(state: &mut AppState, ui: &mut Ui) {
    ui.heading("分割の基準となる主キーを選択してください");
    ui.add_space(20.0);
    let mut on_next = || {
        state.step = 3;
    };
    state.key_selector.render(ui, &mut on_next);
    ui.add_space(20.0);
    if ui.button("前へ").clicked() {
        if state.step > 0 {
            state.step -= 1;
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
                let headers = excel::get_available_columns(path).unwrap_or_default();
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
        // split_all_columnsが空ならファイルから再取得
        if state.split_all_columns.is_empty() {
            if let Some(path) = &state.split_file_path {
                state.split_all_columns = excel::get_available_columns(path).unwrap_or_default();
            }
        }
        // 件数列は除外
        let all_columns: Vec<String> = state.split_all_columns.iter().filter(|c| *c != "件数").cloned().collect();
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
        let next_enabled = state.split_columns_selected.iter().any(|&b| b);
        if next_enabled {
            if ui.button("次へ").clicked() {
                // 保存処理
                if let (Some(path), keys) = (&state.split_file_path, &state.key_selector.selected_keys) {
                    use chrono::Local;
                    use std::path::PathBuf;
                    let now = Local::now().format("%y%m%d%H%M%S").to_string();
                    let out_dir = PathBuf::from(format!("./{}", now));
                    // 選択された列だけ抽出
                    let selected_cols: Vec<String> = state.split_columns.iter().enumerate()
                        .filter(|(i, _)| state.split_columns_selected[*i])
                        .map(|(_, c)| c.clone()).collect();
                    // 分割実行
                    let result = excel::split_excel_by_key(
                        path,
                        &out_dir,
                        keys,
                        1000000 // 1ファイルあたりの最大行数（必要に応じて調整）
                    );
                    match result {
                        Ok(_files) => {
                            let abs_dir = out_dir.canonicalize().unwrap_or(out_dir.clone());
                            let dir_str = abs_dir.to_string_lossy().to_string();
                            let _ = std::process::Command::new("explorer").arg(&dir_str).spawn();
                            state.split_folder_opened = true;
                            state.save_error_message = None;
                        },
                        Err(e) => {
                            state.save_error_message = Some(format!("保存失敗: {}", e));
                        }
                    }
                }
                state.step += 1;
            }
        } else {
            ui.add_enabled(false, egui::Button::new("次へ"));
        }
    });
}

fn render_save_complete(state: &mut AppState, ui: &mut Ui) {
    if let Some(msg) = &state.save_error_message {
        ui.colored_label(egui::Color32::RED, format!("保存エラー: {}", msg));
    } else {
        ui.heading("分割保存が完了しました！");
        ui.label("出力フォルダが自動で開きます");
        if !state.split_folder_opened {
            if let Some(path) = &state.split_file_path {
                use chrono::Local;
                let now = Local::now().format("%y%m%d%H%M%S").to_string();
                let out_dir = PathBuf::from(format!("./{}", now));
                let abs_dir = out_dir.canonicalize().unwrap_or(out_dir.clone());
                let dir_str = abs_dir.to_string_lossy().to_string();
                let _ = std::process::Command::new("explorer").arg(&dir_str).spawn();
                state.split_folder_opened = true;
            }
        }
    }
    if ui.button("最初に戻る").clicked() {
        *state = AppState::new();
    }
}

fn sanitize_filename(s: &str) -> String {
    // Windowsで使えない文字を全て_に置換し、連続アンダースコアや先頭・末尾の_も除去
    let forbidden = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut sanitized: String = s.chars()
        .map(|c| if forbidden.contains(&c) { '_' } else { c })
        .collect();
    // 連続アンダースコアを1つにまとめる
    while sanitized.contains("__") {
        sanitized = sanitized.replace("__", "_");
    }
    // 先頭・末尾のアンダースコアを除去
    sanitized = sanitized.trim_matches('_').to_string();
    sanitized
}

fn save_df_to_excel(sub_df: &DataFrame, file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut book = Spreadsheet::default();
    let sheet = book.new_sheet("Sheet1")?;
    // ヘッダー
    for (col_idx, name) in sub_df.get_column_names().iter().enumerate() {
        sheet.get_cell_mut((1u32, (col_idx + 1) as u32)).set_value_string(name.to_string());
    }
    // データ
    for row_idx in 0..sub_df.height() {
        let row = sub_df.get_row(row_idx)?;
        for (col_idx, val) in row.0.iter().enumerate() {
            let mut s = val.to_string();
            if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                s = s[1..s.len()-1].to_string();
            }
            let cell = sheet.get_cell_mut((((row_idx + 2) as u32), (col_idx + 1) as u32));
            cell.set_value(s);
        }
    }
    xlsx::write(&book, file_path)?;
    Ok(())
} 