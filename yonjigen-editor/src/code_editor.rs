//! Éditeur de code Lua intégré : coloration syntaxique, numéros de ligne,
//! autocomplétion basique de l'API moteur, console de logs/erreurs.

use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, RichText};
use std::sync::Arc;
use yonjigen_core::script::API_SIGNATURES;

const KEYWORDS: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in",
    "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

const KEYWORD_COLOR: Color32 = Color32::from_rgb(0x56, 0x9c, 0xd6);
const STRING_COLOR: Color32 = Color32::from_rgb(0xce, 0x91, 0x78);
const COMMENT_COLOR: Color32 = Color32::from_rgb(0x6a, 0x99, 0x55);
const NUMBER_COLOR: Color32 = Color32::from_rgb(0xb5, 0xce, 0xa8);
const DEFAULT_COLOR: Color32 = Color32::from_rgb(0xd4, 0xd4, 0xd4);

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Tokenise très simplement une ligne de Lua pour la coloration syntaxique.
/// Pas un vrai lexer (pas de strings multi-lignes `[[...]]`), suffisant pour
/// un éditeur de script de scène.
fn highlight_line(job: &mut LayoutJob, line: &str, font: FontId) {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let fmt = |color: Color32, font: FontId| TextFormat {
        font_id: font,
        color,
        ..Default::default()
    };

    while i < chars.len() {
        let c = chars[i];

        if c == '-' && chars.get(i + 1) == Some(&'-') {
            let rest: String = chars[i..].iter().collect();
            job.append(&rest, 0.0, fmt(COMMENT_COLOR, font.clone()));
            break;
        }

        if c == '"' || c == '\'' {
            let quote = c;
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != quote {
                i += 1;
            }
            i = (i + 1).min(chars.len());
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, fmt(STRING_COLOR, font.clone()));
            continue;
        }

        if c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            job.append(&s, 0.0, fmt(NUMBER_COLOR, font.clone()));
            continue;
        }

        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && is_ident_char(chars[i]) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let color = if KEYWORDS.contains(&word.as_str()) {
                KEYWORD_COLOR
            } else {
                DEFAULT_COLOR
            };
            job.append(&word, 0.0, fmt(color, font.clone()));
            continue;
        }

        job.append(&c.to_string(), 0.0, fmt(DEFAULT_COLOR, font.clone()));
        i += 1;
    }
}

fn layouter(ui: &egui::Ui, text: &dyn egui::TextBuffer, wrap_width: f32) -> Arc<egui::Galley> {
    let font = FontId::monospace(13.0);
    let mut job = LayoutJob::default();
    for (n, line) in text.as_str().split('\n').enumerate() {
        if n > 0 {
            job.append(
                "\n",
                0.0,
                TextFormat {
                    font_id: font.clone(),
                    ..Default::default()
                },
            );
        }
        highlight_line(&mut job, line, font.clone());
    }
    job.wrap.max_width = wrap_width;
    ui.painter().layout_job(job)
}

/// Dessine l'éditeur pour une entité : source Lua éditable + barre d'actions.
/// Retourne `true` si le bouton "Exécuter" a été cliqué.
pub fn show(ui: &mut egui::Ui, source: &mut String) -> bool {
    let mut run_clicked = false;
    ui.horizontal(|ui| {
        run_clicked = ui.button("▶ Exécuter").clicked();
        ui.menu_button("API ▾", |ui| {
            for (sig, desc) in API_SIGNATURES {
                if ui.button(*sig).on_hover_text(*desc).clicked() {
                    if !source.ends_with('\n') && !source.is_empty() {
                        source.push('\n');
                    }
                    source.push_str(sig);
                    ui.close();
                }
            }
        });
    });
    ui.separator();

    let line_count = source.split('\n').count().max(1);
    egui::ScrollArea::vertical()
        .id_salt("code_scroll")
        .max_height(320.0)
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                let gutter: String = (1..=line_count)
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                ui.add(
                    egui::Label::new(
                        RichText::new(gutter)
                            .monospace()
                            .color(Color32::from_gray(0x80)),
                    )
                    .selectable(false),
                );
                ui.add(
                    egui::TextEdit::multiline(source)
                        .font(FontId::monospace(13.0))
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                );
            });
        });
    run_clicked
}

/// Affiche la console de logs/erreurs du script (derniers messages en bas).
pub fn show_console(ui: &mut egui::Ui, log: &[String]) {
    egui::ScrollArea::vertical()
        .id_salt("code_console")
        .max_height(140.0)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            for line in log {
                let color = if line.contains("Erreur") {
                    Color32::from_rgb(0xe0, 0x6c, 0x55)
                } else {
                    crate::theme::TEXT_MAIN
                };
                ui.label(RichText::new(line).monospace().small().color(color));
            }
        });
}
