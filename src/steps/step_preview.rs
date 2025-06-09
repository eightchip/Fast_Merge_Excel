use egui::Ui;
use crate::app::AppState;
use std::sync::{Arc, Mutex};
use crate::steps::async_step::async_step_transition;
use polars::prelude::DataFrame;
use polars::prelude::DataFrameJoinOps;
use polars::prelude::*;
use polars::prelude::Series;
use crate::components::preview_table::PreviewTable;

// 前年対比用の差額列を追加する関数
fn add_difference_columns(df: DataFrame, selected_columns: &[String]) -> DataFrame {
    println!("[DIFF] Adding difference columns for Zennen Taihi");
    
    // 金額と思われる列を探す
    let amount_keywords = ["金額", "amount", "価格", "売上"];
    let mut amount_columns = Vec::new();
    
    for col_name in selected_columns {
        let col_lower = col_name.to_lowercase();
        if amount_keywords.iter().any(|keyword| col_lower.contains(keyword)) {
            amount_columns.push(col_name.clone());
            println!("[DIFF] Found amount column: {}", col_name);
        }
    }
    
    // 前年対比の差額計算を実装
    if amount_columns.len() >= 2 {
        println!("[DIFF] Calculating difference between {} and {}", amount_columns[0], amount_columns[1]);
        
        // 安全な差額計算を実装
        let col1_name = &amount_columns[0];
        let col2_name = &amount_columns[1];
        let diff_col_name = format!("差額_{}vs{}", col1_name, col2_name);
        
        // 数値型として列を取得し、差額を計算
        match (df.column(col1_name), df.column(col2_name)) {
            (Ok(col1), Ok(col2)) => {
                println!("[DIFF] Successfully got columns for difference calculation");
                
                // より安全な数値変換：文字列データを直接解析
                let col1_values: Vec<f64> = (0..col1.len()).map(|i| {
                    match col1.get(i) {
                        Ok(val) => {
                            let val_str = val.to_string()
                                .trim_matches(&['"', '\'', ' ', '\t', '\n', '\r'] as &[_])
                                .replace(",", ""); // カンマ区切りの数値にも対応
                            
                            // null値を0で置き換え
                            if val_str == "null" || val_str.is_empty() || val_str == "N/A" || val_str == "NaN" {
                                0.0
                            } else {
                                val_str.parse::<f64>().unwrap_or(0.0)
                            }
                        }
                        Err(_) => 0.0,
                    }
                }).collect();
                
                let col2_values: Vec<f64> = (0..col2.len()).map(|i| {
                    match col2.get(i) {
                        Ok(val) => {
                            let val_str = val.to_string()
                                .trim_matches(&['"', '\'', ' ', '\t', '\n', '\r'] as &[_])
                                .replace(",", ""); // カンマ区切りの数値にも対応
                            
                            // null値を0で置き換え
                            if val_str == "null" || val_str.is_empty() || val_str == "N/A" || val_str == "NaN" {
                                0.0
                            } else {
                                val_str.parse::<f64>().unwrap_or(0.0)
                            }
                        }
                        Err(_) => 0.0,
                    }
                }).collect();
                
                println!("[DIFF] Parsed {} values from {}, {} values from {}", 
                    col1_values.len(), col1_name, col2_values.len(), col2_name);
                println!("[DIFF] Sample values - {}: {:?}, {}: {:?}", 
                    col1_name, &col1_values[0..col1_values.len().min(3)],
                    col2_name, &col2_values[0..col2_values.len().min(3)]);
                
                // 差額計算 (col1 - col2)
                let diff_values: Vec<f64> = col1_values.iter().zip(col2_values.iter())
                    .map(|(v1, v2)| v1 - v2)
                    .collect();
                
                println!("[DIFF] Calculated {} difference values, sample: {:?}", 
                    diff_values.len(), &diff_values[0..diff_values.len().min(3)]);
                
                // 新しい差額列を作成
                let diff_series = Series::new(&diff_col_name, diff_values);
                
                // 新しい列を追加したDataFrameを作成
                let mut all_series = df.get_columns().to_vec();
                all_series.push(diff_series);
                match DataFrame::new(all_series) {
                    Ok(new_df) => {
                        println!("[DIFF] Successfully added difference column: {} (total columns: {})", 
                            diff_col_name, new_df.width());
                        return new_df;
                    }
                    Err(e) => {
                        println!("[DIFF] Error creating DataFrame with difference column: {:?}", e);
                    }
                }
            }
            _ => {
                println!("[DIFF] Failed to get columns {} or {} for difference calculation", col1_name, col2_name);
            }
        }
    } else {
        println!("[DIFF] Not enough amount columns found ({}), skipping difference calculation", amount_columns.len());
        // 金額列が見つからない場合でも、比較情報列を追加
        if !amount_columns.is_empty() {
            println!("[DIFF] Adding comparison info for single amount column: {}", amount_columns[0]);
        }
    }
    
    println!("[DIFF] Returning original DataFrame without modifications");
    df
}

// 前年対比用の差額列を追加する関数（結合後のDataFrame用）
fn add_difference_columns_from_joined_df(df: DataFrame) -> DataFrame {
    println!("[DIFF] Adding difference columns from joined DataFrame");
    
    let column_names: Vec<String> = df.get_column_names().iter().map(|s| s.to_string()).collect();
    println!("[DIFF] Available columns: {:?}", column_names);
    
    // 金額と思われる列を探す（元の列と_right列のペアを探す）
    let amount_keywords = ["金額", "amount", "価格", "売上"];
    let mut amount_column_pairs = Vec::new();
    
    for keyword in &amount_keywords {
        // 元の列を探す
        if let Some(original_col) = column_names.iter().find(|col| {
            let col_lower = col.to_lowercase();
            col_lower.contains(keyword) && !col.ends_with("_right")
        }) {
            // 対応する_right列を探す
            let right_col_name = format!("{}_right", original_col);
            if column_names.contains(&right_col_name) {
                amount_column_pairs.push((original_col.clone(), right_col_name.clone()));
                println!("[DIFF] Found amount column pair: {} <-> {}", original_col, right_col_name);
            }
        }
    }
    
    if amount_column_pairs.is_empty() {
        println!("[DIFF] No amount column pairs found, returning original DataFrame");
        return df;
    }
    
    // 最初のペアで差額計算を実行
    let (col1_name, col2_name) = &amount_column_pairs[0];
    let diff_col_name = format!("差額_{}vs{}", col1_name, col2_name.replace("_right", ""));
    
    println!("[DIFF] Calculating difference: {} - {} = {}", col1_name, col2_name, diff_col_name);
    
    // 数値型として列を取得し、差額を計算
    match (df.column(col1_name), df.column(col2_name)) {
        (Ok(col1), Ok(col2)) => {
            println!("[DIFF] Successfully got columns for difference calculation");
            
            // より安全な数値変換：文字列データを直接解析
            let col1_values: Vec<f64> = (0..col1.len()).map(|i| {
                match col1.get(i) {
                    Ok(val) => {
                        let val_str = val.to_string()
                            .trim_matches(&['"', '\'', ' ', '\t', '\n', '\r'] as &[_])
                            .replace(",", ""); // カンマ区切りの数値にも対応
                        
                        // null値を0で置き換え
                        if val_str == "null" || val_str.is_empty() || val_str == "N/A" || val_str == "NaN" {
                            0.0
                        } else {
                            val_str.parse::<f64>().unwrap_or(0.0)
                        }
                    }
                    Err(_) => 0.0,
                }
            }).collect();
            
            let col2_values: Vec<f64> = (0..col2.len()).map(|i| {
                match col2.get(i) {
                    Ok(val) => {
                        let val_str = val.to_string()
                            .trim_matches(&['"', '\'', ' ', '\t', '\n', '\r'] as &[_])
                            .replace(",", ""); // カンマ区切りの数値にも対応
                        
                        // null値を0で置き換え
                        if val_str == "null" || val_str.is_empty() || val_str == "N/A" || val_str == "NaN" {
                            0.0
                        } else {
                            val_str.parse::<f64>().unwrap_or(0.0)
                        }
                    }
                    Err(_) => 0.0,
                }
            }).collect();
            
            println!("[DIFF] Parsed {} values from {}, {} values from {}", 
                col1_values.len(), col1_name, col2_values.len(), col2_name);
            println!("[DIFF] Sample values - {}: {:?}, {}: {:?}", 
                col1_name, &col1_values[0..col1_values.len().min(3)],
                col2_name, &col2_values[0..col2_values.len().min(3)]);
            
            // 差額計算 (col1 - col2) = 当期 - 前期
            let diff_values: Vec<f64> = col1_values.iter().zip(col2_values.iter())
                .map(|(v1, v2)| v1 - v2)
                .collect();
            
            println!("[DIFF] Calculated {} difference values, sample: {:?}", 
                diff_values.len(), &diff_values[0..diff_values.len().min(3)]);
            
            // 新しい差額列を作成
            let diff_series = Series::new(&diff_col_name, diff_values);
            
            // 新しい列を追加したDataFrameを作成
            let mut all_series = df.get_columns().to_vec();
            all_series.push(diff_series);
            match DataFrame::new(all_series) {
                Ok(new_df) => {
                    println!("[DIFF] Successfully added difference column: {} (total columns: {})", 
                        diff_col_name, new_df.width());
                    return new_df;
                }
                Err(e) => {
                    println!("[DIFF] Error creating DataFrame with difference column: {:?}", e);
                }
            }
        }
        _ => {
            println!("[DIFF] Failed to get columns {} or {} for difference calculation", col1_name, col2_name);
        }
    }
    
    println!("[DIFF] Returning original DataFrame without modifications");
    df
}

// 前年対比モード用の列順序を最適化する関数
fn optimize_column_order_for_zennen_taihi(columns: Vec<String>, all_columns: &[String]) -> Vec<String> {
    let mut key_columns = Vec::new();       // 結合キー列
    let mut current_columns = Vec::new();   // 当期データ列
    let mut previous_columns = Vec::new();  // 前期データ列(_right)
    let mut diff_columns = Vec::new();      // 差額列
    
    // 金額キーワード
    let amount_keywords = ["金額", "amount", "価格", "売上"];
    
    for col in columns {
        if col.starts_with("差額_") {
            // 差額列
            diff_columns.push(col);
        } else if col.ends_with("_right") {
            // 前期データ列
            previous_columns.push(col);
        } else {
            // 金額列かどうかチェック
            let is_amount_col = amount_keywords.iter().any(|keyword| {
                col.to_lowercase().contains(keyword)
            });
            
            if is_amount_col {
                // 当期データ列
                current_columns.push(col);
            } else {
                // 結合キー列
                key_columns.push(col);
            }
        }
    }
    
    // 最適化された順序で結合: キー → 当期 → 前期 → 差額
    let mut optimized = Vec::new();
    optimized.extend(key_columns);
    optimized.extend(current_columns);
    optimized.extend(previous_columns);
    optimized.extend(diff_columns);
    
    println!("[COLUMN_ORDER] Optimized order: Keys={:?}, Current={:?}, Previous={:?}, Diff={:?}", 
        optimized.iter().take(1).collect::<Vec<_>>(),
        optimized.iter().skip(1).take(1).collect::<Vec<_>>(),
        optimized.iter().filter(|c| c.ends_with("_right")).collect::<Vec<_>>(),
        optimized.iter().filter(|c| c.starts_with("差額_")).collect::<Vec<_>>()
    );
    
    optimized
}

pub fn render_preview(app_state: Arc<Mutex<AppState>>, ui: &mut Ui, ctx: &egui::Context) {
    let (current_step, next_step) = {
        let state = app_state.lock().unwrap();
        (state.step, state.step + 1)
    };
    let app_state_clone = app_state.clone();

    // 現在の状態を取得
    let (is_processing, preview_data_empty) = {
        let state = app_state.lock().unwrap();
        (state.is_processing, state.preview_table.preview_data.is_empty())
    };

    // プレビューデータが空で、かつ処理中でない場合のみ生成を開始
    if preview_data_empty && !is_processing {
        println!("[DEBUG] Starting preview generation");
        let files = {
            let state = app_state.lock().unwrap();
            state.file_selector.selected_files.clone()
        };
        let paths: Vec<_> = files.iter().filter_map(|f| f.as_ref()).collect();
        if paths.len() >= 2 {
            let ctx = ctx.clone();
            async_step_transition(app_state_clone.clone(), current_step, move || {
                move |state: &mut AppState| {
                    // 1. ファイルパス取得
                    let files = &state.file_selector.selected_files;
                    println!("[PREVIEW] files: {:?}", files);
                    let paths: Vec<_> = files.iter().filter_map(|f| f.as_ref()).collect();
                    println!("[PREVIEW] paths: {:?}", paths);
                    if paths.len() < 2 {
                        println!("[PREVIEW] paths < 2, returning");
                        state.preview_result = Some((vec![], vec![]));
                        return;
                    }
                    // 2. DataFrame化
                    use crate::components::file_selector::read_xlsx_to_df;
                    let mut dfs: Vec<DataFrame> = paths.iter().map(|p| read_xlsx_to_df(p)).collect();
                    println!("[PREVIEW] DataFrame shapes: {:?}", dfs.iter().map(|df| (df.width(), df.height())).collect::<Vec<_>>());
                    if dfs.iter().any(|df| df.width() == 0) {
                        println!("[PREVIEW] some DataFrame width == 0, returning");
                        state.preview_result = Some((vec![], vec![]));
                        return;
                    }
                    
                    // データの内容をサンプル表示
                    for (i, df) in dfs.iter().enumerate() {
                        println!("[PREVIEW] File {} sample data (first 2 rows):", i);
                        if df.height() > 0 {
                            for row_idx in 0..df.height().min(2) {
                                if let Ok(row) = df.get_row(row_idx) {
                                    println!("[PREVIEW]   Row {}: {:?}", row_idx, row.0.iter().take(3).collect::<Vec<_>>());
                                }
                            }
                        }
                    }
                    // 3. 結合キー・種別取得
                    let keys = &state.key_selector.selected_keys;
                    let join_type = state.join_type_picker.selected_join_type.clone();
                    let selected_columns = state.column_selector.selected_columns.clone();
                    println!("[PREVIEW] keys: {:?}", keys);
                    println!("[PREVIEW] join_type: {:?}", join_type);
                    println!("[PREVIEW] selected_columns: {:?}", selected_columns);
                    if keys.is_empty() || join_type.is_none() || selected_columns.is_empty() {
                        println!("[PREVIEW] keys/join_type/selected_columns not set, returning");
                        state.preview_result = Some((vec![], vec![]));
                        return;
                    }
                    let join_type = join_type.unwrap();
                    use crate::components::join_type_picker::to_polars_join_type;
                    let polars_join_type = to_polars_join_type(&join_type);

                    // 4. join実行（A,Bのみ対応。Cは未対応）
                    let mut df = dfs[0].clone();
                    println!("[PREVIEW] before join - Left DataFrame shape: {}x{}", df.width(), df.height());
                    println!("[PREVIEW] before join - Left DataFrame first 3 rows: {:?}", 
                        df.head(Some(3)).to_string());
                    
                    if dfs.len() >= 2 {
                        let right = &dfs[1];
                        println!("[PREVIEW] before join - Right DataFrame shape: {}x{}", right.width(), right.height());
                        println!("[PREVIEW] before join - Right DataFrame first 3 rows: {:?}", 
                            right.head(Some(3)).to_string());
                        
                        let left_keys: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
                        let right_keys = left_keys.clone();
                        println!("[PREVIEW] Join keys: {:?}", left_keys);
                        println!("[PREVIEW] Join type: {:?}", polars_join_type);
                        
                        if let Ok(joined) = df.join(
                            right,
                            left_keys.as_slice(),
                            right_keys.as_slice(),
                            polars_join_type.into(),
                        ) {
                            df = joined;
                            println!("[PREVIEW] after join shape: {}x{}", df.width(), df.height());
                            println!("[PREVIEW] after join - first 5 rows: {:?}", 
                                df.head(Some(5)).to_string());
                            
                            // さらに詳細な結合後データの確認
                            if df.height() > 0 {
                                println!("[PREVIEW] Join result - sample data:");
                                for row_idx in 0..df.height().min(3) {
                                    if let Ok(row) = df.get_row(row_idx) {
                                        println!("[PREVIEW]   Joined Row {}: {:?}", row_idx, row.0);
                                    }
                                }
                            } else {
                                println!("[PREVIEW] WARNING: Join resulted in 0 rows!");
                            }
                        } else {
                            println!("[PREVIEW] join failed, returning");
                            state.preview_result = Some((vec![], vec![]));
                            return;
                        }
                    }
                    // 5. 前年対比の場合は差額列を追加（列選択前に実行）
                    let is_zennen_taihi = matches!(state.mode, crate::app::MergeMode::ZennenTaihi);
                    println!("[PREVIEW] Current mode: {:?}, is_zennen_taihi: {}", state.mode, is_zennen_taihi);
                    println!("[PREVIEW] DataFrame before difference calculation - columns: {:?}", df.get_column_names());
                    println!("[PREVIEW] DataFrame before difference calculation - shape: {}x{}", df.width(), df.height());
                    
                    let df_with_diff = if is_zennen_taihi {
                        // 前年対比処理：結合後のDataFrameから直接差額計算
                        println!("[PREVIEW] *** STARTING ZENNEN TAIHI DIFFERENCE CALCULATION ***");
                        let result = add_difference_columns_from_joined_df(df.clone());
                        println!("[PREVIEW] *** FINISHED DIFFERENCE CALCULATION - new shape: {}x{} ***", result.width(), result.height());
                        println!("[PREVIEW] *** New columns after difference calculation: {:?} ***", result.get_column_names());
                        result
                    } else {
                        println!("[PREVIEW] Not Zennen Taihi mode, using original joined data");
                        df.clone()
                    };
                    
                    // 6. 出力列のみ抽出（差額列も自動的に含める）
                    let mut out_df = df_with_diff.clone();
                    if !selected_columns.is_empty() {
                        // 差額列が作成されているかチェック
                        let all_columns: Vec<String> = df_with_diff.get_column_names().iter().map(|s| s.to_string()).collect();
                        let mut final_columns = selected_columns.clone();
                        
                        // 差額列を自動的に追加
                        for col in &all_columns {
                            if col.starts_with("差額_") && !final_columns.contains(col) {
                                final_columns.push(col.clone());
                                println!("[PREVIEW] Automatically including difference column: {}", col);
                            }
                        }
                        
                        // 前年対比モードの場合、_right列も自動的に追加
                        if is_zennen_taihi {
                            for col in &all_columns {
                                if col.ends_with("_right") && !final_columns.contains(col) {
                                    final_columns.push(col.clone());
                                    println!("[PREVIEW] Automatically including _right column for Zennen Taihi: {}", col);
                                }
                            }
                            
                            // 前年対比モードでは列順序を最適化
                            final_columns = optimize_column_order_for_zennen_taihi(final_columns, &all_columns);
                            println!("[PREVIEW] Optimized column order for Zennen Taihi: {:?}", final_columns);
                        }
                        
                        let col_refs: Vec<&str> = final_columns.iter().map(|s| s.as_str()).collect();
                        println!("[PREVIEW] Column selection - original: {:?}, final: {:?}", selected_columns, final_columns);
                        
                        if let Ok(filtered) = out_df.select(col_refs.as_slice()) {
                            out_df = filtered;
                            println!("[PREVIEW] After column selection: {}x{}", out_df.width(), out_df.height());
                        } else {
                            println!("[PREVIEW] Column selection failed, using all columns");
                        }
                    }
                    println!("[PREVIEW] out_df shape: {}x{}", out_df.width(), out_df.height());
                    println!("[PREVIEW] out_df columns: {:?}", out_df.get_column_names());
                    
                    // サンプルデータを表示
                    if out_df.height() > 0 {
                        println!("[PREVIEW] Sample data after column selection:");
                        for row_idx in 0..out_df.height().min(3) {
                            if let Ok(row) = out_df.get_row(row_idx) {
                                println!("[PREVIEW]   Row {}: {:?}", row_idx, row.0);
                            }
                        }
                    } else {
                        println!("[PREVIEW] WARNING: No data rows after column selection!");
                    }
                    
                    let final_df = out_df;
                    
                    // 7. プレビュー用データ変換（全行表示）
                    let height = final_df.height();
                    println!("[PREVIEW] Final DataFrame height: {}, generating preview data for all {} rows...", height, height);
                    let mut preview_rows = vec![];
                    
                    // 全行をプレビューに含める（最大制限を撤廃）
                    println!("[PREVIEW] Will generate {} preview rows (showing all data)", height);
                    for i in 0..height {
                        let row = final_df.get_row(i).unwrap().0.iter().map(|v| {
                            // データの清理：quotes、余分な空白を削除
                            let mut cleaned = v.to_string()
                                .trim_matches(&['"', '\'', ' ', '\t', '\n', '\r'] as &[_])
                                .to_string();
                            
                            // さらに二重quotes等も処理
                            if cleaned.starts_with('"') && cleaned.ends_with('"') && cleaned.len() >= 2 {
                                cleaned = cleaned[1..cleaned.len()-1].to_string();
                            }
                            
                            // null値を0で置き換える処理
                            if cleaned == "null" || cleaned.is_empty() || cleaned == "N/A" || cleaned == "NaN" {
                                "0".to_string()
                            } else {
                                cleaned
                            }
                        }).collect();
                        preview_rows.push(row);
                    }
                    println!("[PREVIEW] Generated {} preview rows", preview_rows.len());
                    println!("[PREVIEW] Preview row samples: {:?}", preview_rows.get(0..preview_rows.len().min(2)));
                    
                    // 8. 完全なデータも保存（保存用）
                    let mut complete_rows = vec![];
                    println!("[PREVIEW] Generating complete data for {} rows...", height);
                    for i in 0..height {
                        let row = final_df.get_row(i).unwrap().0.iter().map(|v| {
                            // データの清理：quotes、余分な空白を削除（保存用も同様に処理）
                            let mut cleaned = v.to_string()
                                .trim_matches(&['"', '\'', ' ', '\t', '\n', '\r'] as &[_])
                                .to_string();
                            
                            // さらに二重quotes等も処理
                            if cleaned.starts_with('"') && cleaned.ends_with('"') && cleaned.len() >= 2 {
                                cleaned = cleaned[1..cleaned.len()-1].to_string();
                            }
                            
                            // null値を0で置き換える処理（保存用も同様に処理）
                            if cleaned == "null" || cleaned.is_empty() || cleaned == "N/A" || cleaned == "NaN" {
                                "0".to_string()
                            } else {
                                cleaned
                            }
                        }).collect();
                        complete_rows.push(row);
                    }
                    println!("[PREVIEW] Generated {} complete rows for saving", complete_rows.len());
                    
                    // 9. PreviewTableにセット（UIスレッドでのみ行うため、ここではpreview_resultに格納）
                    let columns: Vec<String> = final_df.get_column_names().iter().map(|s| s.to_string()).collect();
                    println!("[PREVIEW] Final columns for output: {:?}", columns);
                    println!("[PREVIEW] Preview data sample: {:?}", preview_rows.get(0..preview_rows.len().min(5)));
                    println!("[PREVIEW] Complete data sample: {:?}", complete_rows.get(0..complete_rows.len().min(5)));
                    
                    state.preview_result = Some((columns.clone(), preview_rows));
                    state.complete_result_data = Some((columns, complete_rows));
                    println!("[DEBUG] Preview data set in preview_result");
                    println!("[ASYNC] state ptr: {:p}", state);
                }
            });
        }
    }

    // プレビューデータの処理（一度だけ）
    {
        let mut state = app_state.lock().unwrap();
        if state.preview_result.is_some() {
            let (columns, preview_rows) = state.preview_result.take().unwrap();
            println!("[DEBUG] Setting preview data - columns: {} rows: {}", columns.len(), preview_rows.len());
            println!("[DEBUG] First few preview rows: {:?}", preview_rows.get(0..3.min(preview_rows.len())));
            
            // 新しいメソッドを使用して列順序を保持
            state.preview_table.set_preview_data_with_columns(columns, preview_rows);
            println!("[DEBUG] Preview table data set - total rows: {}", state.preview_table.preview_data.len());
        }
    }
    
    // UIの表示（Mutexロック分離）
    let (mut preview_table_copy, first_row_data, debug_info) = {
        let state = app_state.lock().unwrap();
        let first_row = if !state.preview_table.preview_data.is_empty() {
            Some(state.preview_table.preview_data[0].clone())
        } else {
            None
        };
        let debug_info = format!("Preview table: {} columns, {} rows, page: {}", 
            state.preview_table.columns.len(), 
            state.preview_table.preview_data.len(),
            state.preview_table.page);
        (state.preview_table.clone(), first_row, debug_info)
    };
    
    println!("[DEBUG] {}", debug_info);
    
    // Mutexロック外でUIを描画
    preview_table_copy.render(
        ui,
        &mut move || {
            // 保存ページへの移行は重い処理ではないので、直接ステップを進める
            let mut state = app_state_clone.lock().unwrap();
            state.step = next_step;
        },
    );
    
    // ページ変更をステートに反映
    {
        let mut state = app_state.lock().unwrap();
        state.preview_table.page = preview_table_copy.page;
        
        let mut data_changed = false;
        
        // 列移動の結果も反映
        if state.preview_table.columns != preview_table_copy.columns {
            println!("[DEBUG] Column order changed, updating state");
            println!("[DEBUG] Old order: {:?}", state.preview_table.columns);
            println!("[DEBUG] New order: {:?}", preview_table_copy.columns);
            state.preview_table.columns = preview_table_copy.columns.clone();
            data_changed = true;
        }
        
        // ソート設定の変更も反映
        if state.preview_table.column_sorts != preview_table_copy.column_sorts {
            println!("[DEBUG] Sort settings changed, updating state");
            println!("[DEBUG] Old sorts: {:?}", state.preview_table.column_sorts);
            println!("[DEBUG] New sorts: {:?}", preview_table_copy.column_sorts);
            state.preview_table.column_sorts = preview_table_copy.column_sorts;
            data_changed = true;
        }
        
        // データが変更された場合のみ更新
        if data_changed {
            state.preview_table.preview_data = preview_table_copy.preview_data;
        }
    }
    
    if let Some(first_row) = first_row_data {
        ui.label(format!("1行目: {:?}", first_row));
    }
}
