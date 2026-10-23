//! Thème sombre néon violet/bleu, esthétique pixel art, coins carrés.

use egui::{Color32, CornerRadius, FontFamily, FontId, Stroke, TextStyle};

pub const BG_DARK: Color32 = Color32::from_rgb(0x0b, 0x0b, 0x16);
pub const BG_PANEL: Color32 = Color32::from_rgb(0x12, 0x12, 0x22);
pub const NEON_PURPLE: Color32 = Color32::from_rgb(0xb0, 0x40, 0xff);
pub const NEON_BLUE: Color32 = Color32::from_rgb(0x30, 0xd0, 0xff);
pub const TEXT_MAIN: Color32 = Color32::from_rgb(0xe8, 0xe8, 0xff);

/// Cherche une police CJK du système (Noto CJK, Hiragino, Yu Gothic...).
fn find_system_cjk_font() -> Option<Vec<u8>> {
    const CANDIDATES: &[&str] = &[
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJKjp-Regular.otf",
        "/usr/share/fonts/opentype/source-han-sans/SourceHanSans-Regular.otf",
        "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "C:\\Windows\\Fonts\\YuGothM.ttc",
        "C:\\Windows\\Fonts\\msgothic.ttc",
    ];
    for path in CANDIDATES {
        if let Ok(bytes) = std::fs::read(path) {
            return Some(bytes);
        }
    }
    None
}

pub fn setup(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Some(bytes) = find_system_cjk_font() {
        fonts.font_data.insert(
            "cjk".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(bytes)),
        );
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .push("cjk".to_owned());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("cjk".to_owned());
    }
    ctx.set_fonts(fonts);

    ctx.all_styles_mut(|style| {
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = BG_PANEL;
        style.visuals.window_fill = BG_PANEL;
        style.visuals.extreme_bg_color = BG_DARK;
        style.visuals.override_text_color = Some(TEXT_MAIN);
        style.visuals.widgets.noninteractive.bg_fill = BG_PANEL;
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x1a, 0x1a, 0x30);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x2a, 0x1a, 0x40);
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, NEON_BLUE);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(0x3a, 0x1a, 0x50);
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.5, NEON_PURPLE);
        style.visuals.selection.bg_fill = NEON_PURPLE.gamma_multiply(0.5);
        style.visuals.hyperlink_color = NEON_BLUE;

        // Coins carrés partout : zéro arrondi, esthétique 8-bit.
        let zero = CornerRadius::ZERO;
        style.visuals.widgets.noninteractive.corner_radius = zero;
        style.visuals.widgets.inactive.corner_radius = zero;
        style.visuals.widgets.hovered.corner_radius = zero;
        style.visuals.widgets.active.corner_radius = zero;
        style.visuals.widgets.open.corner_radius = zero;
        style.visuals.window_corner_radius = zero;
        style.visuals.menu_corner_radius = zero;

        style
            .text_styles
            .insert(TextStyle::Heading, FontId::new(18.0, FontFamily::Monospace));
        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Monospace));
        style
            .text_styles
            .insert(TextStyle::Button, FontId::new(14.0, FontFamily::Monospace));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::new(11.0, FontFamily::Monospace));
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(14.0, FontFamily::Monospace),
        );
    });
}
