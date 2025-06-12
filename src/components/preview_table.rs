use egui::{Ui, Color32, RichText};
use crate::components::button::AppButton;
use std::sync::{Arc, Mutex};
use crate::app::AppState;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub enum SortDirection {
    None,
    Ascending,
    Descending,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColumnSort {
    pub direction: SortDirection,
    pub priority: Option<usize>, // 1-5の優先度
}

#[derive(Clone, Debug)]
pub struct PreviewTable {
    pub preview_data: Vec<Vec<String>>, // プレビューデータを保持
    pub columns: Vec<String>, // 列名
    pub page: usize, // 現在のページ番号
    pub initialized: bool, // 初期化フラグ
    pub column_sorts: Vec<ColumnSort>, // 各列のソート設定
    // キャッシュを追加
    cached_column_widths: Option<Vec<f32>>,
    pub last_render_frame: u64,
}

impl Default for PreviewTable {
    fn default() -> Self {
        PreviewTable {
            preview_data: Vec::new(),
            columns: Vec::new(),
            page: 0,
            initialized: false,
            column_sorts: Vec::new(),
            cached_column_widths: None,
            last_render_frame: 0,
        }
    }
}

impl PreviewTable {
    pub fn new() -> Self {
        PreviewTable {
            columns: vec![],
            preview_data: vec![],
            page: 0,
            initialized: false,
            column_sorts: Vec::new(),
            cached_column_widths: None,
            last_render_frame: 0,
        }
    }

    pub fn set_preview_data(&mut self, data: Vec<Vec<String>>) {
        self.preview_data = data; // プレビューデータを設定
        self.page = 0;
        // ソート設定を初期化
        self.initialize_sort_settings();
        // キャッシュをクリア
        self.cached_column_widths = None;
    }
    
    /// 新しいデータを設定し、現在の列順序を保持する
    pub fn set_preview_data_with_columns(&mut self, new_columns: Vec<String>, data: Vec<Vec<String>>) {
        // 既存の列順序がある場合は保持する
        if !self.columns.is_empty() && self.columns.len() == new_columns.len() {
            // 現在の列順序に従ってデータを並び替える
            let reordered_data = self.reorder_data_by_current_columns(&new_columns, data);
            self.preview_data = reordered_data;
        } else {
            // 初回設定または列数が変わった場合は新しい順序を採用
            self.columns = new_columns;
            self.preview_data = data;
            // ソート設定を初期化
            self.initialize_sort_settings();
        }
        
        self.page = 0;
        // キャッシュをクリア
        self.cached_column_widths = None;
    }
    
    /// ソート設定を初期化
    fn initialize_sort_settings(&mut self) {
        self.column_sorts = self.columns.iter().map(|_| ColumnSort {
            direction: SortDirection::None,
            priority: None,
        }).collect();
    }
    
    /// 列のソート設定を変更
    pub fn set_column_sort(&mut self, col_index: usize, direction: SortDirection) {
        if col_index >= self.column_sorts.len() {
            return;
        }
        
        // 既存の優先度を取得
        let current_priority = self.column_sorts[col_index].priority;
        
        // 新しい設定を適用
        self.column_sorts[col_index].direction = direction.clone();
        
        match direction {
            SortDirection::None => {
                // ソートを解除する場合は優先度も削除し、他の優先度を詰める
                if let Some(removed_priority) = current_priority {
                    self.column_sorts[col_index].priority = None;
                    self.compact_priorities(removed_priority);
                }
            }
            _ => {
                // ソートを設定する場合
                if current_priority.is_none() {
                    // 新しい優先度を割り当て
                    self.column_sorts[col_index].priority = self.get_next_priority();
                }
            }
        }
        
        // ソートを適用
        self.apply_sort();
    }
    
    /// 次の利用可能な優先度を取得（1-5）
    fn get_next_priority(&self) -> Option<usize> {
        let used_priorities: HashSet<usize> = self.column_sorts
            .iter()
            .filter_map(|sort| sort.priority)
            .collect();
            
        for priority in 1..=5 {
            if !used_priorities.contains(&priority) {
                return Some(priority);
            }
        }
        None // 5つすべて使用中
    }
    
    /// 優先度を詰める（削除された優先度より大きいものを1つずつ下げる）
    fn compact_priorities(&mut self, removed_priority: usize) {
        for sort in &mut self.column_sorts {
            if let Some(priority) = sort.priority {
                if priority > removed_priority {
                    sort.priority = Some(priority - 1);
                }
            }
        }
    }
    
    /// ソートを適用
    fn apply_sort(&mut self) {
        // 優先度でソートキーを取得
        let mut sort_keys: Vec<(usize, SortDirection)> = Vec::new();
        
        for priority in 1..=5 {
            for (col_index, sort) in self.column_sorts.iter().enumerate() {
                if sort.priority == Some(priority) {
                    sort_keys.push((col_index, sort.direction.clone()));
                    break;
                }
            }
        }
        
        if sort_keys.is_empty() {
            return; // ソートキーがない場合は何もしない
        }
        
        // データをソート
        self.preview_data.sort_by(|a, b| {
            for (col_index, direction) in &sort_keys {
                if *col_index >= a.len() || *col_index >= b.len() {
                    continue;
                }
                
                let val_a = &a[*col_index];
                let val_b = &b[*col_index];
                
                // 数値として解析を試行
                let comparison = match (val_a.parse::<f64>(), val_b.parse::<f64>()) {
                    (Ok(num_a), Ok(num_b)) => num_a.partial_cmp(&num_b).unwrap_or(std::cmp::Ordering::Equal),
                    _ => val_a.cmp(val_b), // 文字列として比較
                };
                
                let final_comparison = match direction {
                    SortDirection::Ascending => comparison,
                    SortDirection::Descending => comparison.reverse(),
                    SortDirection::None => std::cmp::Ordering::Equal,
                };
                
                if final_comparison != std::cmp::Ordering::Equal {
                    return final_comparison;
                }
            }
            std::cmp::Ordering::Equal
        });
    }

    /// 現在の列順序に従って新しいデータを並び替える
    fn reorder_data_by_current_columns(&self, new_columns: &[String], data: Vec<Vec<String>>) -> Vec<Vec<String>> {
        // 現在の列順序に対応する新しい列のインデックスを見つける
        let mut column_mapping = Vec::new();
        
        for current_col in &self.columns {
            if let Some(new_index) = new_columns.iter().position(|c| c == current_col) {
                column_mapping.push(new_index);
            } else {
                return data; // 列が見つからない場合は元のデータを返す
            }
        }
        
        // データを並び替える
        let reordered_data = data.into_iter().map(|row| {
            column_mapping.iter().map(|&idx| {
                if idx < row.len() {
                    row[idx].clone()
                } else {
                    String::new()
                }
            }).collect()
        }).collect();
        
        reordered_data
    }

    /// ヘッダー＋最初の数行の最大文字数からカラム幅を自動計算（キャッシュ付き）
    pub fn auto_column_widths(&mut self) -> &Vec<f32> {
        if self.cached_column_widths.is_none() {
            let mut widths = vec![];
            for (i, col) in self.columns.iter().enumerate() {
                let mut max_len = col.chars().count();
                for row in self.preview_data.iter().take(10) {
                    if let Some(cell) = row.get(i) {
                        max_len = max_len.max(cell.chars().count());
                    }
                }
                // 文字数×ピクセル幅係数で幅を決定（日本語は1文字約14px+余白）
                widths.push((max_len as f32) * 14.0 + 24.0);
            }
            self.cached_column_widths = Some(widths);
        }
        self.cached_column_widths.as_ref().unwrap()
    }

    /// 列を左に移動（インデックスを小さくする）
    pub fn move_column_left(&mut self, col_index: usize) {
        if col_index > 0 && col_index < self.columns.len() {
            // 列名を入れ替え
            self.columns.swap(col_index - 1, col_index);
            
            // ソート設定も入れ替え
            if col_index < self.column_sorts.len() && col_index - 1 < self.column_sorts.len() {
                self.column_sorts.swap(col_index - 1, col_index);
            }
            
            // データも入れ替え
            for row in &mut self.preview_data {
                if col_index < row.len() {
                    row.swap(col_index - 1, col_index);
                }
            }
            
            // キャッシュをクリア
            self.cached_column_widths = None;
        }
    }
    
    /// 列を右に移動（インデックスを大きくする）
    pub fn move_column_right(&mut self, col_index: usize) {
        if col_index < self.columns.len() - 1 {
            // 列名を入れ替え
            self.columns.swap(col_index, col_index + 1);
            
            // ソート設定も入れ替え
            if col_index < self.column_sorts.len() && col_index + 1 < self.column_sorts.len() {
                self.column_sorts.swap(col_index, col_index + 1);
            }
            
            // データも入れ替え
            for row in &mut self.preview_data {
                if col_index + 1 < row.len() {
                    row.swap(col_index, col_index + 1);
                }
            }
            
            // キャッシュをクリア
            self.cached_column_widths = None;
        }
    }

    pub fn render(&mut self, ui: &mut Ui, on_next: &mut dyn FnMut()) {
        // 軽量な再帰防止システム（ちらつき防止）
        static mut IS_RENDERING: bool = false;
        unsafe {
            if IS_RENDERING {
                return;
            }
            IS_RENDERING = true;
        }
        egui::ScrollArea::vertical().id_source("preview_table_scroll_area").show(ui, |ui| {
            self.render_internal(ui, on_next);
        });
        unsafe {
            IS_RENDERING = false;
        }
    }
    
    fn render_internal(&mut self, ui: &mut Ui, on_next: &mut dyn FnMut()) {
        ui.label(format!("プレビュー（全{}行、ページ表示）", self.preview_data.len()));
        
        // パフォーマンス情報を表示
        if self.preview_data.len() > 100 {
            ui.label(format!("⚠️ 大量データ（{}行）- ページ切り替えで快適に閲覧", self.preview_data.len()));
        }
        
        let rows_per_page = 20;
        
        // 列順序変更UI
        ui.horizontal(|ui| {
            ui.label("列順序調整:");
        });
        
        // 各列に対する移動ボタン
        if self.columns.len() > 1 {
            let mut column_move_actions = Vec::new();
            
            ui.horizontal(|ui| {
                for (i, col_name) in self.columns.iter().enumerate() {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(format!("{}:", col_name));
                            ui.horizontal(|ui| {
                                // 左移動ボタン
                                if i > 0 {
                                    if ui.button("◀").clicked() {
                                        column_move_actions.push(("left", i));
                                    }
                                } else {
                                    ui.add_enabled(false, egui::Button::new("◀"));
                                }
                                
                                // 右移動ボタン
                                if i < self.columns.len() - 1 {
                                    if ui.button("▶").clicked() {
                                        column_move_actions.push(("right", i));
                                    }
                                } else {
                                    ui.add_enabled(false, egui::Button::new("▶"));
                                }
                            });
                        });
                    });
                }
            });
            
            // 収集した移動操作を実行
            for (direction, index) in column_move_actions {
                match direction {
                    "left" => self.move_column_left(index),
                    "right" => self.move_column_right(index),
                    _ => {}
                }
            }
            
            ui.separator();
        }
        
        // ソート設定UI
        if !self.columns.is_empty() {
            ui.horizontal(|ui| {
                ui.label("ソート設定:");
            });
            
            let mut sort_change_actions = Vec::new();
            
            ui.horizontal(|ui| {
                for (i, col_name) in self.columns.iter().enumerate() {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(format!("{}:", col_name));
                            
                            // 現在のソート状態を取得
                            let current_sort = if i < self.column_sorts.len() {
                                &self.column_sorts[i]
                            } else {
                                &ColumnSort { direction: SortDirection::None, priority: None }
                            };
                            
                            // 優先度表示
                            if let Some(priority) = current_sort.priority {
                                ui.label(format!("優先度: {}", priority));
                            } else {
                                ui.label("ソートなし");
                            }
                            
                            // ソートボタン
                            ui.horizontal(|ui| {
                                // 昇順ボタン
                                let ascending_active = matches!(current_sort.direction, SortDirection::Ascending);
                                let ascending_button = if ascending_active {
                                    egui::Button::new("↑").fill(egui::Color32::from_rgb(100, 150, 255))
                                } else {
                                    egui::Button::new("↑")
                                };
                                if ui.add(ascending_button).clicked() {
                                    let new_direction = if ascending_active {
                                        SortDirection::None
                                    } else {
                                        SortDirection::Ascending
                                    };
                                    sort_change_actions.push((i, new_direction));
                                }
                                
                                // 降順ボタン
                                let descending_active = matches!(current_sort.direction, SortDirection::Descending);
                                let descending_button = if descending_active {
                                    egui::Button::new("↓").fill(egui::Color32::from_rgb(255, 150, 100))
                                } else {
                                    egui::Button::new("↓")
                                };
                                if ui.add(descending_button).clicked() {
                                    let new_direction = if descending_active {
                                        SortDirection::None
                                    } else {
                                        SortDirection::Descending
                                    };
                                    sort_change_actions.push((i, new_direction));
                                }
                            });
                        });
                    });
                }
            });
            
            // 収集したソート変更操作を実行
            for (col_index, direction) in sort_change_actions {
                self.set_column_sort(col_index, direction);
            }
            
            ui.separator();
        }
        
        let total = self.preview_data.len();
        let total_pages = (total + rows_per_page - 1) / rows_per_page;
        let start = self.page * rows_per_page;
        let end = ((self.page + 1) * rows_per_page).min(total);
        
        // まずcol_widthsを計算してからpreview_rowsを取得（borrowing conflict回避）
        let col_widths = self.auto_column_widths().clone();
        let preview_rows = if start < end {
            &self.preview_data[start..end]
        } else {
            &[]
        };
        
        egui::ScrollArea::horizontal().show(ui, |ui| {
            let mut table = egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .min_scrolled_height(800.0);
            for w in &col_widths {
                table = table.column(egui_extras::Column::initial(*w));
            }
            table
                .header(20.0, |mut header| {
                    for col in &self.columns {
                        header.col(|ui| { ui.label(col); });
                    }
                })
                .body(|mut body| {
                    for row in preview_rows {
                        body.row(18.0, |mut row_ui| {
                            for cell in row {
                                row_ui.col(|ui| {
                                    ui.label(cell);
                                });
                            }
                        });
                    }
                });
        });
        ui.horizontal(|ui| {
            // 最初のページへボタン
            if self.page > 0 {
                if ui.button("≪ 最初").clicked() {
                    self.page = 0;
                }
            } else {
                ui.add_enabled(false, egui::Button::new("≪ 最初"));
            }
            // 前のページへボタン
            if self.page > 0 {
                if ui.button("◀ 前へ").clicked() {
                    self.page -= 1;
                }
            } else {
                ui.add_enabled(false, egui::Button::new("◀ 前へ"));
            }
            // ページ情報を中央に表示
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.label(format!("{}/{}ページ ({}-{} / {}行)", 
                    self.page + 1, 
                    total_pages.max(1),
                    start + 1,
                    end,
                    total
                ));
            });
            // 次のページへボタン
            if self.page + 1 < total_pages {
                if ui.button("次へ ▶").clicked() {
                    self.page += 1;
                }
            } else {
                ui.add_enabled(false, egui::Button::new("次へ ▶"));
            }
            // 最後のページへボタン
            if self.page + 1 < total_pages {
                if ui.button("最後 ≫").clicked() {
                    self.page = total_pages - 1;
                }
            } else {
                ui.add_enabled(false, egui::Button::new("最後 ≫"));
            }
        });
        ui.add_space(10.0);
        if ui.button("次へ").clicked() {
            on_next();
        }
    }

    /// 現在のソート済みデータと列名を取得（保存用）
    pub fn get_sorted_data(&self) -> (Vec<String>, Vec<Vec<String>>) {
        (self.columns.clone(), self.preview_data.clone())
    }
    
    /// ソート済みデータが利用可能かチェック
    pub fn has_sorted_data(&self) -> bool {
        !self.columns.is_empty() && !self.preview_data.is_empty()
    }
    
    /// ソートが適用されているかチェック
    pub fn is_sorted(&self) -> bool {
        self.column_sorts.iter().any(|sort| sort.direction != SortDirection::None)
    }
}


