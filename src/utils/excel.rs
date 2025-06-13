use std::path::Path;
use umya_spreadsheet::*;
use polars::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use crate::utils::excel;
use calamine::{open_workbook_auto, Reader};

// シート名をサニタイズする関数
fn sanitize_sheet_name(name: &str) -> String {
    let mut sanitized = name
        .replace([':', '\\', '/', '?', '*', '[', ']'], "_")
        .replace('"', "");
    // シート名の最大長は31「文字」
    let mut chars = sanitized.chars();
    let mut result = String::new();
    for _ in 0..31 {
        if let Some(c) = chars.next() {
            result.push(c);
        } else {
            break;
        }
    }
    result
}

// セル値の型に応じた適切な設定
fn set_cell_value(cell: &mut Cell, value: &str) {
    // 数値として解析できる場合も文字列として書き込む
    cell.set_value_string(value);
}

pub fn get_available_columns(input_path: &std::path::Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut workbook = open_workbook_auto(input_path)?;
    let sheets = workbook.worksheets();
    let sheet_names: Vec<_> = sheets.iter().map(|(name, _)| name).collect();
    println!("シート一覧: {:?}", sheet_names);
    if let Some((_name, sheet)) = sheets.get(0) {
        for (i, row) in sheet.rows().take(3).enumerate() {
            println!("row {}: {:?}", i, row.iter().map(|cell| cell.to_string()).collect::<Vec<_>>());
        }
        if let Some(row) = sheet.rows().next() {
            let headers: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();
            println!("取得したヘッダー: {:?}", headers);
            return Ok(headers);
        }
    }
    Ok(vec![])
}

pub fn split_excel_by_key(
    input_path: &Path,
    output_dir: &Path,
    key_columns: &[String],
    max_rows_per_file: usize,
) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
    // 出力ディレクトリが存在しない場合は作成
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // calamineでヘッダーとデータを取得
    let mut workbook = open_workbook_auto(input_path)?;
    let sheets = workbook.worksheets();
    let sheet = &sheets[0].1;
    let mut rows = sheet.rows();
    let headers: Vec<String> = if let Some(row) = rows.next() {
        row.iter().map(|cell| cell.to_string()).collect()
    } else {
        return Err("No header row found".into());
    };

    // キー列のインデックスを取得
    let key_indices: Vec<usize> = key_columns
        .iter()
        .map(|key| {
            headers
                .iter()
                .position(|h| h == key)
                .ok_or_else(|| format!("Key column not found: {}. Available columns: {:?}", key, headers))
        })
        .collect::<Result<Vec<usize>, String>>()?;

    // データを読み込み
    let mut data: Vec<Vec<String>> = Vec::new();
    for row in rows {
        let row_data: Vec<String> = row.iter().map(|cell| cell.to_string()).collect();
        data.push(row_data);
    }

    // キーでグループ化
    let mut groups: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    for row in &data {
        let key: String = key_indices
            .iter()
            .map(|&idx| row.get(idx).cloned().unwrap_or_default())
            .collect::<Vec<String>>()
            .join("_");
        groups.entry(key).or_default().push(row.clone());
    }

    // 各グループをファイルに分割
    let mut output_files = Vec::new();
    for (key, rows) in groups {
        let num_files = (rows.len() + max_rows_per_file - 1) / max_rows_per_file;
        for i in 0..num_files {
            let start = i * max_rows_per_file;
            let end = (start + max_rows_per_file).min(rows.len());
            let chunk = &rows[start..end];

            // 新しいワークブックを作成
            let mut new_workbook = new_file();
            let new_sheet = if let Some(s) = new_workbook.get_sheet_by_name_mut("Sheet1") {
                s
            } else {
                new_workbook.get_sheet_mut(&0).ok_or_else(|| "Failed to get sheet from new workbook")?
            };

            // ヘッダーを書き込み
            for (col, header) in headers.iter().enumerate() {
                new_sheet.get_cell_mut(((col as u32) + 1, 1u32)).set_value_string(header);
            }

            // データを書き込み
            for (row_idx, row) in chunk.iter().enumerate() {
                for (col_idx, header) in headers.iter().enumerate() {
                    let value = row.get(col_idx).cloned().unwrap_or_default();
                    let cell = new_sheet.get_cell_mut(((col_idx as u32) + 1, (row_idx as u32) + 2));
                    set_cell_value(cell, &value);
                }
            }

            // ファイル名を生成（サニタイズ済みのキーを使用）
            let sanitized_key = sanitize_sheet_name(&key);
            let file_name = format!("{}_{}.xlsx", sanitized_key, i + 1);
            let output_path = output_dir.join(file_name);

            // ファイルに書き込み
            umya_spreadsheet::writer::xlsx::write(&new_workbook, &output_path)?;

            output_files.push(output_path);
        }
    }

    Ok(output_files)
} 