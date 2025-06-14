use egui::Ui;
use crate::app::AppState;
use std::sync::{Arc, Mutex};
// use crate::steps::async_step::async_step_transition;
use polars::prelude::DataFrame;
use polars::prelude::DataFrameJoinOps;
use polars::prelude::*;
use polars::prelude::Series;
// use crate::components::preview_table::PreviewTable;
// use std::time::{SystemTime, UNIX_EPOCH};

// データサンプリングによる数値列判定関数
fn is_numeric_column(df: &DataFrame, col_name: &str) -> bool {
    // 列名による除外（識別子系の列は数値データでも文字列扱い）
    let col_lower = col_name.to_lowercase();
    let identifier_keywords = [
        "コード", "code", "番号", "id", "取引先", "部門", "科目", "勘定", 
        "品目", "商品", "得意先", "仕入先", "顧客", "customer", "vendor",
        "account", "item", "product", "supplier", "client"
    ];
    
    // 識別子系の列名を含む場合は数値列として扱わない
    for keyword in &identifier_keywords {
        if col_lower.contains(keyword) {
            return false;
        }
    }
    
    if let Ok(column) = df.column(col_name) {
        let sample_size = df.height().min(10); // 最初の10行をサンプリング
        let mut numeric_count = 0;
        let mut total_count = 0;
        let mut has_mixed_alphanumeric = false; // 英数字混合データの存在チェック
        
        for i in 0..sample_size {
            if let Ok(val) = column.get(i) {
                let val_str = val.to_string()
                    .trim_matches(&['"', '\'', ' ', '\t', '\n', '\r'] as &[_])
                    .replace(",", "");
                
                // null値やN/Aは除外
                if val_str != "null" && !val_str.is_empty() && val_str != "N/A" && val_str != "NaN" {
                    total_count += 1;
                    
                    // 英数字混合パターンをチェック（例：AA0025, XYZ123など）
                    let has_alpha = val_str.chars().any(|c| c.is_alphabetic());
                    let has_digit = val_str.chars().any(|c| c.is_numeric());
                    if has_alpha && has_digit {
                        has_mixed_alphanumeric = true;
                    }
                    
                    // 数値として解析できるかチェック
                    if val_str.parse::<f64>().is_ok() {
                        numeric_count += 1;
                    }
                }
            }
        }
        
        // 英数字混合データが存在する場合は、数値列として扱わない
        if has_mixed_alphanumeric {
            return false;
        }
        
        // 70%以上が数値として解析できる場合は数値列と判断
        let numeric_ratio = if total_count > 0 {
            numeric_count as f64 / total_count as f64
        } else {
            0.0
        };
        
        let is_numeric = numeric_ratio >= 0.7;
        is_numeric
    } else {
        false
    }
}

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
    let column_names: Vec<String> = df.get_column_names().iter().map(|s| s.to_string()).collect();
    
    // 数値列を探す（元の列と_right列のペアを探す）
    let mut numeric_column_pairs = Vec::new();
    
    // データサンプリングによる数値列判定
    let sample_size = df.height().min(10); // 最初の10行をサンプリング
    let mut numeric_columns = Vec::new();
    
    for col_name in &column_names {
        if col_name.ends_with("_right") {
            continue; // _right列はスキップ（後で対応する元列とペアにする）
        }
        
        if let Ok(column) = df.column(col_name) {
            let mut numeric_count = 0;
            let mut total_count = 0;
            
            // サンプルデータを分析
            for i in 0..sample_size {
                if let Ok(val) = column.get(i) {
                    let val_str = val.to_string()
                        .trim_matches(&['"', '\'', ' ', '\t', '\n', '\r'] as &[_])
                        .replace(",", "");
                    
                    // null値やN/Aは除外
                    if val_str != "null" && !val_str.is_empty() && val_str != "N/A" && val_str != "NaN" {
                        total_count += 1;
                        
                        // 数値として解析できるかチェック
                        if val_str.parse::<f64>().is_ok() {
                            numeric_count += 1;
                        }
                    }
                }
            }
            
            // 70%以上が数値として解析できる場合は数値列と判断
            let numeric_ratio = if total_count > 0 {
                numeric_count as f64 / total_count as f64
            } else {
                0.0
            };
            
            if numeric_ratio >= 0.7 {
                numeric_columns.push(col_name.clone());
            }
        }
    }
    
    // 数値列の元列と_right列のペアを作成
    for original_col in &numeric_columns {
        let right_col_name = format!("{}_right", original_col);
        if column_names.contains(&right_col_name) {
            numeric_column_pairs.push((original_col.clone(), right_col_name.clone()));
        }
    }
    
    if numeric_column_pairs.is_empty() {
        return df;
    }
    
    // すべてのペアで差額計算を実行
    let mut result_df = df.clone();
    for (col1_name, col2_name) in &numeric_column_pairs {
        let diff_col_name = format!("差額_{}vs{}", col1_name, col2_name.replace("_right", ""));
        
        // 数値型として列を取得し、差額を計算
        match (result_df.column(col1_name), result_df.column(col2_name)) {
            (Ok(col1), Ok(col2)) => {
                
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
                
                // 差額計算 (col1 - col2) = 当期 - 前期
                let diff_values: Vec<f64> = col1_values.iter().zip(col2_values.iter())
                    .map(|(v1, v2)| v1 - v2)
                    .collect();
                
                // 新しい差額列を作成
                let diff_series = Series::new(&diff_col_name, diff_values);
                
                // 新しい列を追加したDataFrameを作成
                let mut all_series = result_df.get_columns().to_vec();
                all_series.push(diff_series);
                match DataFrame::new(all_series) {
                    Ok(new_df) => {
                        result_df = new_df;
                    }
                    Err(_e) => {
                        // エラーログは削除（パフォーマンス向上）
                    }
                }
            }
            _ => {
                // 列の取得に失敗した場合はスキップ
            }
        }
    }
    
    result_df
}

// 同期的なプレビューデータ生成関数
fn generate_preview_data_sync(state: &mut AppState) {
                    // 1. ファイルパス取得
                    let files = &state.file_selector.selected_files;
                    let paths: Vec<_> = files.iter().filter_map(|f| f.as_ref()).collect();
                    if paths.len() < 2 {
                        state.preview_result = Some((vec![], vec![]));
                        return;
                    }
    
    // 2. DataFrame化（最初の1000行のみ読み込み）
                    use crate::components::file_selector::read_xlsx_to_df;
    let mut dfs: Vec<DataFrame> = Vec::new();
    
    for (i, path) in paths.iter().enumerate() {
        let df = read_xlsx_to_df(path);
        
        // 全データを表示（行数制限を削除）
        dfs.push(df);
    }
    
                    if dfs.iter().any(|df| df.width() == 0) {
                        state.preview_result = Some((vec![], vec![]));
                        return;
                    }
    
                    // 3. 結合キー・種別取得
    let is_multi_stage = matches!(state.mode, crate::app::MergeMode::MultiStageJoin);
    let is_zennen_taihi = matches!(state.mode, crate::app::MergeMode::ZennenTaihi);
    let (keys, join_type) = if is_multi_stage {
        // MultiStageJoin は左結合固定
        (&state.selected_ab_keys, Some(crate::components::join_type_picker::JoinType::Left))
    } else {
        (&state.key_selector.selected_keys, state.join_type_picker.selected_join_type.clone())
    };
    let selected_columns = state.column_selector.selected_columns.clone();
    
    if keys.is_empty() || join_type.is_none() || selected_columns.is_empty() {
        state.preview_result = Some((vec![], vec![]));
        return;
    }
    
    let join_type = join_type.unwrap();
    use crate::components::join_type_picker::to_polars_join_type;
    let polars_join_type = if is_multi_stage {
        polars::prelude::JoinType::Outer
    } else if is_zennen_taihi {
        // 前年対比モードは必ず完全外部結合
        polars::prelude::JoinType::Outer
    } else {
        to_polars_join_type(&join_type)
    };

    // 4. join実行（A,B,C対応: MultiStageJoinのみ2段階結合）
    let mut df = dfs[0].clone();
    if is_multi_stage && dfs.len() >= 3 {
        // 2段階左結合: (A left join B) left join C
        let ab_keys = &state.selected_ab_keys;
        let bc_keys = &state.selected_bc_keys;
        let left_keys_ab: Vec<&str> = ab_keys.iter().map(|s| s.as_str()).collect();
        let right_keys_ab = left_keys_ab.clone();
        let left_keys_bc: Vec<&str> = bc_keys.iter().map(|s| s.as_str()).collect();
        let right_keys_bc = left_keys_bc.clone();
        // 1. AとBを左結合
        if let Ok(ab_joined) = dfs[0].join(
            &dfs[1],
            left_keys_ab.as_slice(),
            right_keys_ab.as_slice(),
            polars::prelude::JoinType::Left.into(),
        ) {
            // 2. ABとCを左結合
            if let Ok(abc_joined) = ab_joined.join(
                &dfs[2],
                left_keys_bc.as_slice(),
                right_keys_bc.as_slice(),
                polars::prelude::JoinType::Left.into(),
            ) {
                df = abc_joined;
            } else {
                state.preview_result = Some((vec![], vec![]));
                return;
            }
        } else {
            state.preview_result = Some((vec![], vec![]));
            return;
        }
    } else if dfs.len() >= 2 {
        let right = &dfs[1];
        let left_keys: Vec<&str> = keys.iter().map(|s| s.as_str()).collect();
        let right_keys = left_keys.clone();
        if let Ok(joined) = df.join(
            right,
            left_keys.as_slice(),
            right_keys.as_slice(),
            polars_join_type.into(),
        ) {
            df = joined;
        } else {
            state.preview_result = Some((vec![], vec![]));
            return;
        }
    }
    
    // 5. 前年対比の場合は差額列を追加（列選択前に実行）
    let df_with_diff = if is_zennen_taihi {
        // 前年対比処理：結合後のDataFrameから直接差額計算
        add_difference_columns_from_joined_df(df.clone())
    } else {
        df.clone()
    };
    
    // 6. 出力列のみ抽出（差額列も自動的に含める）
    let mut out_df = df_with_diff.clone();
    if !selected_columns.is_empty() {
        // 差額列が作成されているかチェック
        let all_columns: Vec<String> = df_with_diff.get_column_names().iter().map(|s| s.to_string()).collect();
        let mut final_columns = Vec::new();
        
        // 選択された列のみを選択された順序で処理（全モード共通）
        for selected_col in &selected_columns {
            let mut found_col = None;
            
            // 1. 直接マッチを試行
            if all_columns.contains(selected_col) {
                found_col = Some(selected_col.clone());
            } 
            // 2. MultiStageJoinモードの場合のみ部分マッチを試行
            else if is_multi_stage {
                // 部分マッチを試行（結合により列名が変更された可能性）
                for available_col in &all_columns {
                    // 列名の一部が一致する場合（例：selected="金額" -> available="金額_right"）
                    if available_col.contains(selected_col) || selected_col.contains(available_col) {
                        found_col = Some(available_col.clone());
                        break;
                    }
                }
            }
            
            // 見つかった列を追加（重複チェック）
            if let Some(col) = found_col {
                if !final_columns.contains(&col) {
                    final_columns.push(col);
                }
            }
        }
        
        // 前年対比モードでは、ユーザーが選択した列に対して3つの列セットを表示
        if is_zennen_taihi {
            let mut zennen_taihi_columns = Vec::new();
            
            for selected_col in &selected_columns {
                // 1. 当期データ列（ユーザーが選択した列）
                if all_columns.contains(selected_col) && !zennen_taihi_columns.contains(selected_col) {
                    zennen_taihi_columns.push(selected_col.clone());
                }
                
                // 2. 前期データ列（_right列）
                let right_col_name = format!("{}_right", selected_col);
                if all_columns.contains(&right_col_name) && !zennen_taihi_columns.contains(&right_col_name) {
                    zennen_taihi_columns.push(right_col_name.clone());
                }
                
                // 3. 差額列
                let diff_col_name = format!("差額_{}vs{}", selected_col, selected_col);
                if all_columns.contains(&diff_col_name) && !zennen_taihi_columns.contains(&diff_col_name) {
                    zennen_taihi_columns.push(diff_col_name.clone());
                }
            }
            
            final_columns = zennen_taihi_columns;
        }
        
        // 空の場合は全列を使用
        if final_columns.is_empty() {
            final_columns = all_columns.clone();
        }
        
        let col_refs: Vec<&str> = final_columns.iter().map(|s| s.as_str()).collect();
        
                        if let Ok(filtered) = out_df.select(col_refs.as_slice()) {
                            out_df = filtered;
                        }
                    }
    
    let final_df = out_df;
    
    // 数値列を事前判定
    let column_names: Vec<String> = final_df.get_column_names().iter().map(|s| s.to_string()).collect();
    let mut numeric_columns = std::collections::HashSet::new();
    for col_name in column_names {
        if is_numeric_column(&final_df, col_name.as_str()) || col_name.starts_with("差額_") {
            numeric_columns.insert(col_name.to_string());
        }
    }
    
    // 7. プレビュー用データ変換
    let column_names: Vec<String> = final_df.get_column_names().iter().map(|s| s.to_string()).collect();
    let all_preview_rows: Vec<Vec<String>> = (0..final_df.height()).map(|i| {
        final_df.get_row(i).unwrap().0.iter().enumerate().map(|(col_idx, v)| {
            let original_value = v.to_string();
            let mut cleaned = original_value.trim_matches(&['"', '\'', ' ', '\t', '\n', '\r'] as &[_]).to_string();
            if cleaned.starts_with('"') && cleaned.ends_with('"') && cleaned.len() >= 2 {
                cleaned = cleaned[1..cleaned.len()-1].to_string();
            }
            let col_name = if col_idx < final_df.get_column_names().len() {
                final_df.get_column_names()[col_idx]
            } else { "" };
            let is_truly_null = cleaned == "null" || cleaned == "N/A" || cleaned == "NaN" || cleaned == "None" || cleaned.is_empty();
            if is_zennen_taihi {
                if is_numeric_column(&final_df, col_name) || col_name.starts_with("差額_") {
                    if is_truly_null { "0".to_string() } else { cleaned }
                } else {
                    if is_truly_null { "".to_string() } else { cleaned }
                }
            } else {
                if col_name.contains("取引先") || col_name.contains("コード") || col_name.contains("番号") {
                    if cleaned.trim().is_empty() { "".to_string() } else { cleaned }
                } else {
                    if !is_truly_null { cleaned } else {
                        if numeric_columns.contains(col_name) { "0".to_string() } else { "".to_string() }
                    }
                }
            }
        }).collect()
    }).collect();
    state.preview_result = Some((column_names.clone(), all_preview_rows.clone()));
    state.complete_result_data = Some((column_names, all_preview_rows));
}

// 完全に簡素化されたプレビュー表示関数
pub fn render_preview(app_state: Arc<Mutex<AppState>>, ui: &mut Ui, ctx: &egui::Context) {
    // 軽量なプレビュー生成制御（ちらつき防止）
    static mut GENERATION_IN_PROGRESS: bool = false;
    
    let mut state = app_state.lock().unwrap();
    
    // プレビューデータがない かつ 生成中でない場合のみ生成
    unsafe {
        if state.preview_result.is_none() && !GENERATION_IN_PROGRESS {
            GENERATION_IN_PROGRESS = true;
            
            // 同期的にプレビュー生成（フリーズ防止）
            generate_preview_data_sync(&mut state);
            
            GENERATION_IN_PROGRESS = false;
        }
    }
    
    // プレビューデータの表示
    let (columns, preview_data) = if let Some((columns, data)) = &state.preview_result {
        (columns.clone(), data.clone())
    } else {
        unsafe {
            if GENERATION_IN_PROGRESS {
                ui.label("プレビューを生成中...");
                ui.spinner();
                return;
            }
        }
        (vec![], vec![])
    };
    
    if !columns.is_empty() && !preview_data.is_empty() {
        // データ更新チェック（シンプルな比較でちらつき防止）
        let needs_update = state.preview_table.columns != columns || 
                          state.preview_table.preview_data.len() != preview_data.len();
        
        if needs_update {
            state.preview_table.set_preview_data_with_columns(columns.clone(), preview_data.clone());
        }
        
        let next_step = state.step + 1;
        let mut step_to_update = None;
        
        // プレビューテーブルのレンダリング
        state.preview_table.render(ui, &mut || {
            step_to_update = Some(next_step);
        });
        
        // 「前へ」ボタンを追加
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button("← 前へ（列選択）").clicked() {
                state.step = 3; // 列選択画面に戻る
            }
        });
        
        // ステップ更新（一度だけ）
        if let Some(new_step) = step_to_update {
            state.step = new_step;
        }
    } else {
        ui.label("プレビューデータがありません");
    }
}
