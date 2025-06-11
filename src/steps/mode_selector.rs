use egui::Ui;
use crate::app::{AppState, MergeMode};
use std::sync::{Arc, Mutex};

pub fn render_mode_selector(app_state: Arc<Mutex<AppState>>, ui: &mut Ui) {
    ui.heading("利用目的を選択してください");
    let modes = [
        (MergeMode::ZennenTaihi, "前年対比（外部結合）"),
        (MergeMode::MultiStageJoin, "入金照合（2段階左結合）"),
        (MergeMode::SplitSave, "主キー毎の分割保存"),
        // (MergeMode::KanzenIcchi, "完全一致検証（内部結合）"),
        // (MergeMode::Hikaku, "比較用結合（左/右結合）"),
        // (MergeMode::TateRenketsu, "縦連結（concat）"),
    ];
    let mut state = app_state.lock().unwrap();
    for (mode, label) in &modes {
        let checked = &state.mode == mode;
        if ui.radio(checked, *label).clicked() {
            let mut new_state = AppState::new();
            new_state.mode = mode.clone();
            *state = new_state;
        }
    }
    ui.add_space(10.0);
    let next_enabled = state.mode != MergeMode::None;
    if next_enabled {
        if ui.button("次へ").clicked() {
            state.step = 1;
        }
    } else {
        ui.add_enabled(false, egui::Button::new("次へ"));
    }
} 