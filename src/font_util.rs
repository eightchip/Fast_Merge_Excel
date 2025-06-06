use eframe::egui::{self, Context};

pub fn set_japanese_font(ctx: &Context) {
    let mut fonts = egui::FontDefinitions::default();
    let font_data = std::fs::read("assets/NotoSansJP-Regular.ttf")
        .expect("フォントファイルが見つかりません");
    fonts.font_data.insert(
        "noto_jp".to_owned(),
        egui::FontData::from_owned(font_data),
    );
    fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "noto_jp".to_owned());
    fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "noto_jp".to_owned());
    ctx.set_fonts(fonts);
}
