use egui::Ui;

#[derive(Debug, PartialEq, Clone)]
pub enum JoinType {
    Left,
    Right,
    Inner,
    FullOuter,
    Concat,
}

pub struct JoinTypePicker {
    pub selected_join_type: Option<JoinType>, // 選択された結合形式を保持
}

impl JoinTypePicker {
    pub fn new() -> Self {
        JoinTypePicker {
            selected_join_type: None, // 初期化時は選択なし
        }
    }

    pub fn render(&mut self, ui: &mut Ui, on_next: &mut dyn FnMut()) {
        ui.label("結合形式を選択してください");
        let join_types = [
            (JoinType::Left, "左結合 (Left Join)"),
            (JoinType::Right, "右結合 (Right Join)"),
            (JoinType::Inner, "内部結合 (Inner Join)"),
            (JoinType::FullOuter, "外部結合 (Full Outer Join)"),
            (JoinType::Concat, "縦結合 (Concat)"),
        ];
        for (ty, label) in &join_types {
            let mut checked = self.selected_join_type.as_ref() == Some(ty);
            if ui.radio(checked, *label).clicked() {
                self.selected_join_type = Some(ty.clone());
            }
        }
        ui.add_space(10.0);
        let next_enabled = self.selected_join_type.is_some();
        if ui.add_enabled(next_enabled, egui::Button::new("次へ")).clicked() {
            if next_enabled {
                on_next();
            }
        }
    }
}

pub fn to_polars_join_type(jt: &JoinType) -> polars::prelude::JoinType {
    use polars::prelude::JoinType as PJT;
    match jt {
        JoinType::Left => PJT::Left,
        JoinType::Right => PJT::Left, // polarsにRightがないのでLeftで代用
        JoinType::Inner => PJT::Inner,
        JoinType::FullOuter => PJT::Outer,
        JoinType::Concat => PJT::Left, // Concatはjoinではなく縦結合なので仮
    }
}
