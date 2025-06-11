// src/components/mod.rs

pub mod column_selector;
pub mod file_selector;
pub mod join_type_picker;
pub mod key_selector;
pub mod preview_table;
pub mod save_panel;
pub mod sort_settings;
pub mod cleaner;
pub mod compare_file_selector;
pub mod compare_key_selector;
pub mod multi_stage_file_selector;
pub mod multi_stage_key_selector;
pub mod preview_spinner;
pub mod preview_async;
pub mod button;
pub mod split_file_selector;
// pub mod join_keys;
// pub mod file_source; // もし存在する場合は追加

// 必要に応じて、型を再エクスポート
// pub use file_list::FileList;
// pub use join_step::{JoinStep, JoinType};
// pub use preview::Preview;
// pub use save::Saver;
// pub use wizard::Wizard;
// pub use column_select::ColumnSelect;
// pub use sort_settings::SortSettings;

pub use crate::app::MergeMode;

