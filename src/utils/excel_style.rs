// src/utils/excel_style.rs
use umya_spreadsheet::{
    Worksheet,
    Style,
    PatternFill,
    PatternValues,
};

/// Excelシートに共通スタイル（タイトル行ハイライト）を適用する
///
/// # 引数
/// - `sheet`      : 書式を適用するシート
/// - `header_row` : ヘッダー行番号（1始まり）
/// - `last_col`   : 最終列番号（1始まり）
/// - `last_row`   : 最終行番号（1始まり）
pub fn apply_common_style(
    sheet: &mut Worksheet,
    header_row: u32,
    last_col: u32,
    last_row: u32,
) {
    // ─── ヘッダー行用スタイルを作成 ───
    let mut header_style = Style::default();

    //   (A) 背景色（水色）
    let mut fill = PatternFill::default();
    fill.set_pattern_type(PatternValues::Solid);
    fill.get_foreground_color_mut().set_argb("FFBFEFFF"); // 水色
    fill.get_background_color_mut().set_argb("FFFFFFFF"); // 白
    header_style.get_fill_mut().set_pattern_fill(fill);

    //   (B) 太字
    header_style.get_font_mut().set_bold(true);

    // ヘッダー行に適用
    for col in 1..=last_col {
        sheet
            .get_cell_mut((col, header_row))
            .set_style(header_style.clone());
    }
}
