use polars::prelude::*;
use std::collections::HashMap;

/// header: 列名、columns: 各列の値（列ごとにVec<String>）
pub fn clean_and_infer_columns(header: &[String], columns: &[Vec<String>]) -> DataFrame {
    let mut columns_map: HashMap<String, Vec<DataType>> = HashMap::new();
    for (i, col_name) in header.iter().enumerate() {
        let mut types = vec![];
        for val in &columns[i] {
            if val.trim().is_empty() || val == "N/A" {
                continue;
            }
            if val.parse::<i64>().is_ok() {
                types.push(DataType::Int64);
            } else if val.parse::<f64>().is_ok() {
                types.push(DataType::Float64);
            } else {
                types.push(DataType::Utf8);
            }
        }
        columns_map.insert(col_name.clone(), types);
    }
    let mut df = DataFrame::default();
    for (i, col_name) in header.iter().enumerate() {
        let col_data = columns[i].iter().map(|v| {
            let v = v.trim_matches(&['\"', '\'', ' '] as &[_]).to_string();
            if v.is_empty() || v == "N/A" { "".to_string() } else { v }
        }).collect::<Vec<_>>();
        let col_lower = col_name.to_lowercase();
        let is_identifier = ["コード", "code", "番号", "id", "取引先"].iter().any(|k| col_lower.contains(k));
        if is_identifier {
            let s = Series::new(col_name, col_data);
            df.with_column(s).unwrap();
            continue;
        }
        let dtype = columns_map.get(col_name).and_then(|types| types.iter().max_by_key(|t| match t {
            DataType::Int64 => 3,
            DataType::Float64 => 2,
            DataType::Utf8 => 1,
            _ => 0,
        })).cloned().unwrap_or(DataType::Utf8);
        match dtype {
            DataType::Int64 => {
                let vals: Vec<Option<i64>> = col_data.iter().map(|v| v.parse::<i64>().ok()).collect();
                let s = Series::new(col_name, vals);
                df.with_column(s).unwrap();
            },
            DataType::Float64 => {
                let vals: Vec<Option<f64>> = col_data.iter().map(|v| v.parse::<f64>().ok()).collect();
                let s = Series::new(col_name, vals);
                df.with_column(s).unwrap();
            },
            _ => {
                let s = Series::new(col_name, col_data);
                df.with_column(s).unwrap();
            }
        }
    }
    df
} 