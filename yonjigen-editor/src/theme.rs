//! Thème sombre façon Unity : gris anthracite, accent bleu discret, UI dense.

use egui::{Color32, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle, Vec2};

/// Fond du viewport (équivalent de la Scene view de Unity).
pub const BG_VIEWPORT: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);
/// Fond des panneaux (Hiérarchie, Inspecteur...), gris Unity standard.
pub const BG_PANEL: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);
/// Barre de menu / onglets, légèrement plus sombre que les panneaux.
pub const BG_HEADER: Color32 = Color32::from_rgb(0x2d, 0x2d, 0x2d);
/// Texte principal : gris clair, pas blanc pur (comme Unity).
pub const TEXT_MAIN: Color32 = Color32::from_rgb(0xc8, 0xc8, 0xc8);
/// Bleu d'accentuation (sélection, focus), discret comme dans Unity.
pub const ACCENT: Color32 = Color32::from_rgb(0x3a, 0x72, 0xc4);
/// Bleu plus clair pour le survol / les axes.
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(0x5a, 0x9b, 0xe0);
/// Orange du contour de sélection, comme le highlight d'objet sélectionné
/// dans la Scene view de Unity.
pub const SELECTED_OUTLINE: Color32 = Color32::from_rgb(0xff, 0x8c, 0x1a);

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

/// Police principale de l'UI : Inter (SIL OFL 1.1), embarquée dans le binaire
/// pour un rendu identique sur Linux/Windows/macOS sans dépendre des polices
/// système. Voir `assets/fonts/Inter-OFL-LICENSE.txt`.
const INTER_REGULAR: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.otf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/Inter-SemiBold.otf");

pub fn setup(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "inter".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(INTER_REGULAR)),
    );
    fonts.font_data.insert(
        "inter-semibold".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(INTER_SEMIBOLD)),
    );

    // Inter en priorité, la police CJK système en repli pour "四次元"
    // (Inter ne couvre pas les idéogrammes).
    let proportional = fonts.families.entry(FontFamily::Proportional).or_default();
    proportional.insert(0, "inter".to_owned());
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
    }
    fonts
        .families
        .entry(FontFamily::Name("heading".into()))
        .or_default()
        .insert(0, "inter-semibold".to_owned());

    ctx.set_fonts(fonts);

    ctx.all_styles_mut(|style| {
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = BG_PANEL;
        style.visuals.window_fill = BG_PANEL;
        style.visuals.extreme_bg_color = BG_VIEWPORT;
        style.visuals.faint_bg_color = BG_HEADER;
        style.visuals.override_text_color = Some(TEXT_MAIN);

        style.visuals.widgets.noninteractive.bg_fill = BG_PANEL;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_MAIN);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x45, 0x45, 0x45);
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_MAIN);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x50, 0x50, 0x50);
        style.visuals.widgets.hovered.bg_stroke =
            Stroke::new(1.0, Color32::from_rgb(0x6a, 0x6a, 0x6a));
        style.visuals.widgets.active.bg_fill = ACCENT;
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_LIGHT);
        style.visuals.selection.bg_fill = ACCENT.gamma_multiply(0.6);
        style.visuals.selection.stroke = Stroke::new(1.0, ACCENT_LIGHT);
        style.visuals.hyperlink_color = ACCENT_LIGHT;

        // Coins bien arrondis partout, plus doux que le style Unity d'origine.
        let rounded = CornerRadius::same(8);
        style.visuals.widgets.noninteractive.corner_radius = rounded;
        style.visuals.widgets.inactive.corner_radius = rounded;
        style.visuals.widgets.hovered.corner_radius = rounded;
        style.visuals.widgets.active.corner_radius = rounded;
        style.visuals.widgets.open.corner_radius = rounded;
        style.visuals.window_corner_radius = rounded;
        style.visuals.menu_corner_radius = CornerRadius::same(10);

        // Espacement généreux : UI plus aérée, moins dense.
        style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        style.spacing.button_padding = Vec2::new(10.0, 6.0);
        style.spacing.menu_margin = Margin::same(8);
        style.spacing.window_margin = Margin::same(12);
        style.spacing.indent = 20.0;
        style.spacing.interact_size.y = 26.0;
        style.spacing.scroll.bar_width = 10.0;

        // Police système proportionnelle (sans-serif), comme l'UI Unity -
        // le monospace reste réservé à l'éditeur de code (étape 3).
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(15.0, FontFamily::Name("heading".into())),
        );
        style
            .text_styles
            .insert(TextStyle::Body, FontId::new(12.5, FontFamily::Proportional));
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(12.5, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(10.5, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(12.5, FontFamily::Monospace),
        );
    });
}
