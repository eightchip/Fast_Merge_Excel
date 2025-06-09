use eframe::egui::{self, Context};

pub fn set_japanese_font(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();

    // フォントデータをバイナリに埋め込むことで、実行ファイル単体で動作させる
    const FONT_BYTES: &[u8] = include_bytes!("../assets/NotoSansJP-Regular.ttf");

    fonts.font_data.insert(
        "noto_jp".to_owned(),
        egui::FontData::from_static(FONT_BYTES),
    );
    fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "noto_jp".to_owned());
    fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "noto_jp".to_owned());
    ctx.set_fonts(fonts);
}
