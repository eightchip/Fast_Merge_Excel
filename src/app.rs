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
use crate::components::compare_file_selector::CompareFileSelector;
use crate::components::compare_key_selector::CompareKeySelector;
use std::borrow::Cow;
use crate::components::multi_stage_file_selector::MultiStageFileSelector;
use crate::components::multi_stage_key_selector::MultiStageKeySelector;
use std::sync::{Arc, Mutex};
use crate::components::preview_spinner::PreviewAsyncPanel;
use crate::components::preview_async::PreviewAsyncComponent;
use crate::components::button::AppButton;
use polars::prelude::{JoinArgs, JoinType as PolarsJoinType};

use crate::steps::{step_file_select, step_key_select, step_column_select, step_preview, mode_selector, zennen_taihi_wizard, multi_stage_wizard, kanzen_icchi_wizard, hikaku_wizard, concat_wizard, split_save_wizard};

#[derive(Clone, PartialEq, Debug)]
pub enum MergeMode {
    None,
    ZennenTaihi, // 前年対比
    // KanzenIcchi, // 完全一致検証
    // Hikaku, // 比較用結合
    // TateRenketsu, // 縦連結
    MultiStageJoin, // 2段階左結合
    SplitSave, // 主キー毎の分割保存
}

#[derive(Clone)]
struct PreviewCache {
    df: Option<DataFrame>,
    selected_columns: Vec<String>,
    sort_keys: Vec<SortKey>,
    is_valid: bool,
}

impl PreviewCache {
    fn new() -> Self {
        Self {
            df: None,
            selected_columns: vec![],
            sort_keys: vec![],
            is_valid: false,
        }
    }

    fn invalidate(&mut self) {
        self.is_valid = false;
    }

    fn is_cache_valid(&self, selected_columns: &[String], sort_keys: &[SortKey]) -> bool {
        self.is_valid && 
        self.selected_columns == selected_columns && 
        self.sort_keys == sort_keys
    }
}

pub struct AppState {
    pub step: u8,
    pub is_processing: bool,
    pub mode: MergeMode,
    pub file_selector: FileSelector,
    pub key_selector: KeySelector,
    pub join_type_picker: JoinTypePicker,
    pub column_selector: ColumnSelector,
    pub preview_table: PreviewTable,
    pub save_panel: SavePanel,
    pub ab_common_keys: Vec<String>,
    pub bc_common_keys: Vec<String>,
    pub selected_ab_keys: Vec<String>,
    pub selected_bc_keys: Vec<String>,
    pub sort_settings: SortSettings,
    pub compare_file_selector: CompareFileSelector,
    pub compare_key_selector: CompareKeySelector,
    pub compare_columns: Vec<String>,
    pub multi_stage_file_selector: MultiStageFileSelector,
    pub multi_stage_key_selector: MultiStageKeySelector,
    pub multi_stage_columns: (Vec<String>, Vec<String>), // (A∩B, B∩C)
    pub preview_cache: PreviewCache, // プレビュー用キャッシュ
    pub compare_target_columns: Vec<String>, // 比較対象列（差額計算用）
    pub processing_progress: f32, // 進捗率（0.0〜1.0）
    pub processing_error: Option<String>, // エラー内容
    pub preview_thread_handle: Option<std::thread::JoinHandle<()>>, // バックグラウンドスレッド
    pub preview_result: Option<(Vec<String>, Vec<Vec<String>>)>, // スレッドからの結果
    pub complete_result_data: Option<(Vec<String>, Vec<Vec<String>>)>, // 完全なデータ（保存用）
    pub preview_result_shared: Arc<Mutex<Option<(Vec<String>, Vec<Vec<String>>)>>>,
    pub progress_shared: Arc<Mutex<f32>>,
    pub error_shared: Arc<Mutex<Option<String>>>,
    pub multi_stage_async_preview: PreviewAsyncComponent,
    pub compare_output_columns: Vec<String>,
    pub save_result: Option<bool>, // 保存の成功・失敗（None=未実行, Some(true)=成功, Some(false)=失敗）
    pub save_error_message: Option<String>, // 保存エラーメッセージ
    // 分割保存用
    pub split_file_path: Option<std::path::PathBuf>,
    pub split_file_selector: crate::components::split_file_selector::SplitFileSelector,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            step: 0,
            is_processing: false,
            mode: MergeMode::None,
            file_selector: FileSelector::new(),
            key_selector: KeySelector::new(),
            join_type_picker: JoinTypePicker::new(),
            column_selector: ColumnSelector::new(),
            preview_table: PreviewTable::new(),
            save_panel: SavePanel::new(),
            ab_common_keys: vec![],
            bc_common_keys: vec![],
            selected_ab_keys: vec![],
            selected_bc_keys: vec![],
            sort_settings: SortSettings::new(vec![]),
            compare_file_selector: CompareFileSelector::new(),
            compare_key_selector: CompareKeySelector::new(),
            compare_columns: vec![],
            multi_stage_file_selector: MultiStageFileSelector::new(),
            multi_stage_key_selector: MultiStageKeySelector::new(),
            multi_stage_columns: (vec![], vec![]),
            preview_cache: PreviewCache::new(),
            compare_target_columns: vec![],
            processing_progress: 0.0,
            processing_error: None,
            preview_thread_handle: None,
            preview_result: None,
            complete_result_data: None,
            preview_result_shared: Arc::new(Mutex::new(None)),
            progress_shared: Arc::new(Mutex::new(0.0)),
            error_shared: Arc::new(Mutex::new(None)),
            multi_stage_async_preview: PreviewAsyncComponent::new(),
            compare_output_columns: vec![],
            save_result: None,
            save_error_message: None,
            split_file_path: None,
            split_file_selector: crate::components::split_file_selector::SplitFileSelector::new(),
        }
    }
}

pub struct App {
    pub state: Arc<Mutex<AppState>>,
}

impl App {
    pub fn new() -> Self {
        App {
            state: Arc::new(Mutex::new(AppState::new())),
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        let state = self.state.clone();
        let state_lock = state.lock().unwrap();
        if state_lock.is_processing {
            // 共通スピナー
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add(egui::Spinner::new().size(40.0));
                    ui.label("処理中です。しばらくお待ちください...");
                    println!("[DEBUG] Processing spinner shown");
                });
            });
            // 処理中の場合のみ再描画要求
            ctx.request_repaint();
            return;
        }
        drop(state_lock); // Mutexロックを早めに解放
        egui::CentralPanel::default().show(ctx, |ui| {
            let state = self.state.clone();
            let step = state.lock().unwrap().step;
            let mode = state.lock().unwrap().mode.clone();
            if step == 0 && mode != MergeMode::SplitSave {
                mode_selector::render_mode_selector(state.clone(), ui);
            } else {
                match mode {
                    MergeMode::ZennenTaihi => zennen_taihi_wizard::render_zennen_taihi_wizard(state.clone(), ui, ctx),
                    MergeMode::MultiStageJoin => multi_stage_wizard::render_multi_stage_wizard(state.clone(), ui, ctx),
                    // MergeMode::KanzenIcchi => kanzen_icchi_wizard::render_kanzen_icchi_wizard(state.clone(), ui, ctx),
                    // MergeMode::Hikaku => hikaku_wizard::render_hikaku_wizard(state.clone(), ui, ctx),
                    // MergeMode::TateRenketsu => concat_wizard::render_concat_wizard(state.clone(), ui, ctx),
                    MergeMode::SplitSave => split_save_wizard::render_split_save_wizard(state.clone(), ui, ctx),
                    MergeMode::None => {},
                }
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        self.update(ctx);
    }
}
