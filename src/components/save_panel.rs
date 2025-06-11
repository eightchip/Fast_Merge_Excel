use std::path::{Path, PathBuf};
use rfd::FileDialog;
use egui::Ui;
use umya_spreadsheet::{Spreadsheet, writer::xlsx, new_file};
use umya_spreadsheet::writer::xlsx::XlsxError;
use crate::components::button::AppButton;
use std::io::{Error as IoError, ErrorKind};
use std::fs;

#[derive(Clone, Debug)]
pub enum SaveError {
    FileInUse(String),      // ファイルが使用中
    PermissionDenied(String), // アクセス権限なし
    PathNotFound(String),   // パスが見つからない
    Other(String),          // その他のエラー
}

impl From<XlsxError> for SaveError {
    fn from(err: XlsxError) -> Self {
        let err_str = format!("{:?}", err);
        let err_str_lower = err_str.to_lowercase();
        
        println!("[SAVE_ERROR] Original error: {:?}", err);
        println!("[SAVE_ERROR] Error string: {}", err_str);
        
        // Windowsでのファイル使用中エラーを包括的に検出
        if err_str_lower.contains("permission denied") || 
           err_str_lower.contains("access is denied") ||
           err_str_lower.contains("being used by another process") ||
           err_str_lower.contains("the process cannot access the file") ||
           err_str_lower.contains("sharing violation") ||
           err_str_lower.contains("file is in use") ||
           err_str_lower.contains("ファイルが使用中") ||
           err_str_lower.contains("アクセスが拒否") ||
           err_str_lower.contains("error code 32") ||
           err_str_lower.contains("error code 5") {
            println!("[SAVE_ERROR] Detected as FileInUse");
            SaveError::FileInUse(err_str)
        } else if err_str_lower.contains("permission") {
            println!("[SAVE_ERROR] Detected as PermissionDenied");
            SaveError::PermissionDenied(err_str)
        } else if err_str_lower.contains("no such file or directory") ||
                  err_str_lower.contains("path not found") ||
                  err_str_lower.contains("cannot find the path") {
            println!("[SAVE_ERROR] Detected as PathNotFound");
            SaveError::PathNotFound(err_str)
        } else {
            println!("[SAVE_ERROR] Detected as Other");
            SaveError::Other(err_str)
        }
    }
}

impl SaveError {
    pub fn user_friendly_message(&self) -> String {
        match self {
            SaveError::FileInUse(details) => {
                format!("❌ ファイル使用中エラー\n\n{}\n\n📋 対処方法:\n• Excelなどでファイルを開いている場合は閉じる\n• 別のファイル名で保存する\n• しばらく待ってから再試行する", details)
            },
            SaveError::PermissionDenied(details) => {
                format!("❌ アクセス権限エラー\n\n{}\n\n📋 対処方法:\n• 管理者権限でアプリを実行する\n• 別の場所（デスクトップなど）に保存する\n• ファイルの読み取り専用属性を解除する", details)
            },
            SaveError::PathNotFound(details) => {
                format!("❌ パスエラー\n\n{}\n\n📋 対処方法:\n• フォルダを作成してから保存する\n• 既存のフォルダを選択する", details)
            },
            SaveError::Other(details) => {
                format!("❌ 保存エラー\n\n{}\n\n📋 対処方法:\n• ファイル名に特殊文字が含まれていないか確認\n• 別の場所に保存してみる\n• アプリを再起動する", details)
            }
        }
    }
    
    pub fn suggested_filename(&self, original: &str) -> Option<String> {
        match self {
            SaveError::FileInUse(_) => {
                // タイムスタンプ付きの代替ファイル名を提案
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                
                if let Some(stem) = Path::new(original).file_stem() {
                    if let Some(ext) = Path::new(original).extension() {
                        return Some(format!("{}_{}.{}", 
                            stem.to_string_lossy(), 
                            now, 
                            ext.to_string_lossy()
                        ));
                    }
                }
                Some(format!("merged_output_{}.xlsx", now))
            },
            _ => None
        }
    }
}

#[derive(Clone, Debug)]
pub struct SavePanel {
    pub save_path: Option<PathBuf>, // 保存先のパスを保持
    pub file_name: String, // 入力中のファイル名
    pub last_error: Option<SaveError>, // 最後のエラー
    pub retry_count: u32,   // リトライ回数
    pub abs_save_path: PathBuf, // 絶対パス（file_name/save_pathに応じて動的に更新）
}

impl SavePanel {
    pub fn new() -> Self {
        let file_name = "merged_output.xlsx".to_string();
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        let abs_save_path = exe_dir.join(&file_name);
        SavePanel {
            save_path: Some(exe_dir),
            file_name,
            last_error: None,
            retry_count: 0,
            abs_save_path,
        }
    }

    pub fn set_save_path(&mut self, path: PathBuf) {
        // パスとファイル名を分離して反映
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            self.file_name = name.to_string();
        }
        if let Some(parent) = path.parent() {
            self.save_path = Some(parent.to_path_buf());
        }
        self.update_abs_save_path();
    }

    pub fn update_abs_save_path(&mut self) {
        let dir = self.save_path.clone().unwrap_or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
        });
        self.abs_save_path = dir.join(&self.file_name);
    }

    pub fn render(&mut self, ui: &mut Ui, on_save: &mut dyn FnMut(&str)) {
        ui.label("保存先ファイル名（.xlsx）");
        ui.horizontal(|ui| {
            if AppButton::new("参照...").show(ui).clicked() {
                if let Some(path) = FileDialog::new().add_filter("XLSX", &["xlsx"]).set_directory(self.save_path.clone().unwrap_or_else(|| {
                    std::env::current_exe()
                        .ok()
                        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                        .unwrap_or_else(|| PathBuf::from("."))
                })).save_file() {
                    self.set_save_path(path);
                    self.last_error = None; // エラーリセット
                    self.retry_count = 0;
                }
            }
            let resp = ui.text_edit_singleline(&mut self.file_name);
            if resp.changed() {
                self.update_abs_save_path();
            }
        });
        self.update_abs_save_path();
        ui.label(format!("保存先: {}", self.abs_save_path.display()));
        ui.colored_label(egui::Color32::DARK_GRAY, "※デフォルトはアプリの実行ファイルがある場所ですが、どちらも変更可能です");
        // ... 以降のエラー表示・保存ボタンUIはそのまま ...
        if let Some(error) = &self.last_error {
            ui.add_space(10.0);
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("⚠️");
                ui.colored_label(egui::Color32::RED, "保存エラー");
            });
            ui.colored_label(egui::Color32::RED, error.user_friendly_message());
            if let Some(suggested) = error.suggested_filename(&self.file_name) {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label("💡 提案:");
                    if ui.button(&format!("「{}」で保存", suggested)).clicked() {
                        self.file_name = suggested;
                        self.last_error = None;
                        self.retry_count = 0;
                        self.update_abs_save_path();
                    }
                });
            }
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("🔄 再試行").clicked() {
                    self.retry_count += 1;
                    on_save(&self.file_name);
                }
                if ui.button("✕ エラーを閉じる").clicked() {
                    self.last_error = None;
                    self.retry_count = 0;
                }
            });
            ui.separator();
            ui.add_space(10.0);
        }
        let valid = self.file_name.ends_with(".xlsx") && !self.file_name.trim().is_empty();
        if !self.file_name.ends_with(".xlsx") {
            ui.colored_label(egui::Color32::RED, ".xlsx形式のみ許可されます");
        }
        ui.add_space(10.0);
        if valid {
            if AppButton::new("保存").show(ui).clicked() {
                self.retry_count = 0; // 新しい保存試行時はリセット
                on_save(&self.file_name);
            }
        } else {
            AppButton::new("保存")
                .with_fill(egui::Color32::from_gray(180))
                .with_text_color(egui::Color32::from_gray(80))
                .show(ui);
        }
    }
    
    pub fn set_error(&mut self, error: SaveError) {
        self.last_error = Some(error);
    }
    
    pub fn clear_error(&mut self) {
        self.last_error = None;
        self.retry_count = 0;
    }
}

pub fn save_to_xlsx(file_name: &str, columns: &[String], data: &[Vec<String>]) -> Result<(), SaveError> {
    println!("[SAVE] Starting save process for: {}", file_name);
    println!("[SAVE] Columns: {} rows: {}", columns.len(), data.len());
    
    let path = Path::new(file_name);
    // 親ディレクトリの存在チェック
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => {
            if !parent.exists() {
                println!("[SAVE] Parent directory does not exist: {:?}", parent);
                // ディレクトリがなければ自動作成
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return Err(SaveError::PathNotFound(format!("フォルダ「{}」の作成に失敗しました: {}", parent.display(), e)));
                }
            }
        },
        _ => {
            // parentがNoneまたは空文字列→カレントディレクトリ直下なので何もしない
        }
    }
    
    // 事前チェック
    if path.exists() {
        println!("[SAVE] File already exists, checking if it's accessible: {}", file_name);
        
        // ファイルが使用中かチェック（Windows固有の方法）
        match std::fs::OpenOptions::new()
            .write(true)
            .truncate(false)
            .open(path) {
            Ok(_) => {
                println!("[SAVE] File is accessible for writing");
            }
            Err(e) => {
                println!("[SAVE] File access check failed: {:?}", e);
                let error_msg = format!("{:?}", e);
                let error_lower = error_msg.to_lowercase();
                
                if error_lower.contains("permission denied") || 
                   error_lower.contains("access is denied") ||
                   error_lower.contains("being used by another process") ||
                   error_lower.contains("the process cannot access the file") ||
                   error_lower.contains("sharing violation") ||
                   error_lower.contains("file is in use") ||
                   error_lower.contains("os error 32") ||
                   error_lower.contains("os error 5") {
                    return Err(SaveError::FileInUse(format!("ファイル「{}」が他のアプリケーションで使用中のため、保存できません。\n\nExcelなどでファイルを開いている場合は閉じてから再試行してください。", file_name)));
                } else {
                    return Err(SaveError::PermissionDenied(format!("ファイル「{}」への書き込み権限がありません。", file_name)));
                }
            }
        }
    }

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
    
    // 保存実行
    println!("[SAVE] Writing to file...");
    match xlsx::write(&book, path) {
        Ok(_) => {
            println!("[SAVE] xlsx::write completed, verifying file...");
            
            // 保存成功の検証
            if path.exists() {
                match std::fs::metadata(path) {
                    Ok(metadata) => {
                        let file_size = metadata.len();
                        println!("[SAVE] File verification successful - size: {} bytes", file_size);
                        
                        if file_size > 0 {
                            println!("[SAVE] Successfully saved file: {}", file_name);
    Ok(())
                        } else {
                            println!("[SAVE] File was created but is empty (0 bytes)");
                            Err(SaveError::Other(format!("ファイル「{}」は作成されましたが、データが書き込まれませんでした。", file_name)))
                        }
                    }
                    Err(e) => {
                        println!("[SAVE] Failed to get file metadata: {:?}", e);
                        Err(SaveError::Other(format!("ファイル「{}」の保存状態を確認できませんでした: {:?}", file_name, e)))
                    }
                }
            } else {
                println!("[SAVE] File does not exist after write operation");
                Err(SaveError::Other(format!("ファイル「{}」の保存に失敗しました（ファイルが作成されませんでした）。", file_name)))
            }
        },
        Err(e) => {
            println!("[SAVE] xlsx::write failed: {:?}", e);
            Err(SaveError::from(e))
        }
    }
}

