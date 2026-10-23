use crate::theme;
use crate::viewport::{RenderMode, ViewportState};
use egui::RichText;
use yonjigen_core::math4::RotationPlane;
use yonjigen_core::scene::{Scene, ShapeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CentralTab {
    Scene,
    Game,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BottomTab {
    Project,
    Console,
}

pub struct YonjigenApp {
    scene: Scene,
    selected: Option<u64>,
    viewport: ViewportState,
    renaming: Option<(u64, String)>,
    current_file: Option<std::path::PathBuf>,
    status: String,
    log: Vec<String>,
    central_tab: CentralTab,
    bottom_tab: BottomTab,
    hierarchy_filter: String,
}

impl Default for YonjigenApp {
    fn default() -> Self {
        let mut scene = Scene::default();
        let id = scene.add("Tesseract", ShapeKind::Tesseract);
        YonjigenApp {
            scene,
            selected: Some(id),
            viewport: ViewportState::default(),
            renaming: None,
            current_file: None,
            status: "Prêt.".to_string(),
            log: vec!["Prêt.".to_string()],
            central_tab: CentralTab::Scene,
            bottom_tab: BottomTab::Console,
            hierarchy_filter: String::new(),
        }
    }
}

impl YonjigenApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::setup(&cc.egui_ctx);
        Self::default()
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        self.log.push(msg.clone());
        self.status = msg;
    }

    /// Barre de menu façon Unity : fine, plate, texte seul.
    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu_bar")
            .frame(
                egui::Frame::new()
                    .fill(theme::BG_HEADER)
                    .inner_margin(egui::Margin::symmetric(12, 8)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("四次元").size(12.0).color(theme::TEXT_MAIN));
                    ui.add_space(6.0);
                    ui.menu_button("Fichier", |ui| {
                        if ui.button("Nouveau").clicked() {
                            self.scene = Scene::default();
                            self.selected = None;
                            self.current_file = None;
                            self.set_status("Nouvelle scène.");
                            ui.close();
                        }
                        if ui.button("Ouvrir...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Yonjigen", &["yonjigen"])
                                .pick_file()
                            {
                                match std::fs::read_to_string(&path)
                                    .ok()
                                    .and_then(|s| Scene::from_json(&s).ok())
                                {
                                    Some(scene) => {
                                        self.scene = scene;
                                        self.selected = self.scene.entities.first().map(|e| e.id);
                                        self.set_status(format!("Ouvert : {}", path.display()));
                                        self.current_file = Some(path);
                                    }
                                    None => self.set_status("Échec de l'ouverture."),
                                }
                            }
                            ui.close();
                        }
                        if ui.button("Sauvegarder").clicked() {
                            self.save_scene();
                            ui.close();
                        }
                        if ui.button("Sauvegarder sous...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Yonjigen", &["yonjigen"])
                                .set_file_name("scene.yonjigen")
                                .save_file()
                            {
                                self.current_file = Some(path);
                                self.save_scene();
                            }
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Exporter le jeu...").clicked() {
                            self.set_status("Export standalone : à venir (étape 6).");
                            ui.close();
                        }
                    });
                    ui.menu_button("Édition", |ui| {
                        let _ = ui.button("Annuler (bientôt)");
                        let _ = ui.button("Rétablir (bientôt)");
                    });
                    ui.menu_button("Jeu", |ui| {
                        let label = if self.viewport.playing {
                            "Pause"
                        } else {
                            "Play"
                        };
                        if ui.button(label).clicked() {
                            self.viewport.playing = !self.viewport.playing;
                            ui.close();
                        }
                        if ui.button("Stop").clicked() {
                            self.viewport.playing = false;
                            ui.close();
                        }
                    });
                    ui.menu_button("Fenêtre", |ui| {
                        let _ = ui.button("Disposition par défaut (bientôt)");
                    });
                    ui.menu_button("Aide", |ui| {
                        ui.label("Yonjigen (四次元) — moteur et éditeur 4D.");
                    });
                });
            });
    }

    /// Barre d'outils principale : centrée sur play/pause/stop, comme Unity.
    fn toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("toolbar")
            .frame(
                egui::Frame::new()
                    .fill(theme::BG_PANEL)
                    .inner_margin(egui::Margin::symmetric(14, 10)),
            )
            .show(ui, |ui| {
                ui.columns(3, |cols| {
                    cols[0].horizontal(|ui| {
                        ui.label(RichText::new("🖐 ✛ ⟳ ⇲").color(theme::TEXT_MAIN).small());
                    });
                    cols[1].with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.horizontal(|ui| {
                                let play_label = if self.viewport.playing { "⏸" } else { "▶" };
                                if ui
                                    .button(play_label)
                                    .on_hover_text("Play / Pause")
                                    .clicked()
                                {
                                    self.viewport.playing = !self.viewport.playing;
                                }
                                if ui.button("⏹").on_hover_text("Stop").clicked() {
                                    self.viewport.playing = false;
                                }
                            });
                        },
                    );
                    cols[2].with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(&self.status).color(theme::TEXT_MAIN).small());
                    });
                });
            });
    }

    fn save_scene(&mut self) {
        let Some(path) = self.current_file.clone() else {
            self.set_status("Aucun fichier : utilisez Sauvegarder sous.");
            return;
        };
        match self.scene.to_json() {
            Ok(json) => match std::fs::write(&path, json) {
                Ok(()) => self.set_status(format!("Sauvegardé : {}", path.display())),
                Err(e) => self.set_status(format!("Erreur d'écriture : {e}")),
            },
            Err(e) => self.set_status(format!("Erreur de sérialisation : {e}")),
        }
    }

    fn hierarchy_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("hierarchy")
            .resizable(true)
            .default_size(240.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::BG_PANEL)
                    .inner_margin(egui::Margin::symmetric(12, 12)),
            )
            .show(ui, |ui| {
                // Partie basse : Project / Console, comme le bas de l'éditeur Unity.
                egui::Panel::bottom("bottom_tabs")
                    .resizable(true)
                    .default_size(180.0)
                    .frame(
                        egui::Frame::new()
                            .fill(theme::BG_PANEL)
                            .inner_margin(egui::Margin::symmetric(4, 10)),
                    )
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.selectable_value(
                                &mut self.bottom_tab,
                                BottomTab::Project,
                                "Project",
                            );
                            ui.selectable_value(
                                &mut self.bottom_tab,
                                BottomTab::Console,
                                "Console",
                            );
                        });
                        ui.separator();
                        egui::ScrollArea::vertical()
                            .id_salt("bottom_scroll")
                            .show(ui, |ui| match self.bottom_tab {
                                BottomTab::Project => {
                                    ui.label(
                                        RichText::new("Aucun asset pour l'instant (étape 6).")
                                            .color(theme::TEXT_MAIN)
                                            .small(),
                                    );
                                }
                                BottomTab::Console => {
                                    for line in self.log.iter().rev() {
                                        ui.label(
                                            RichText::new(line).color(theme::TEXT_MAIN).small(),
                                        );
                                    }
                                }
                            });
                    });

                // Partie haute : la hiérarchie de scène elle-même.
                ui.horizontal(|ui| {
                    ui.heading("Hierarchy");
                    ui.menu_button("+ Create", |ui| {
                        for shape in ShapeKind::ALL {
                            if ui.button(shape.label()).clicked() {
                                let id = self.scene.add(shape.label(), shape);
                                self.selected = Some(id);
                                ui.close();
                            }
                        }
                    });
                });
                ui.add(
                    egui::TextEdit::singleline(&mut self.hierarchy_filter)
                        .hint_text("Rechercher...")
                        .desired_width(f32::INFINITY),
                );
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("hierarchy_scroll")
                    .show(ui, |ui| {
                        let filter = self.hierarchy_filter.to_lowercase();
                        let ids: Vec<u64> = self
                            .scene
                            .entities
                            .iter()
                            .filter(|e| {
                                filter.is_empty() || e.name.to_lowercase().contains(&filter)
                            })
                            .map(|e| e.id)
                            .collect();
                        for id in ids {
                            let Some((entity_name, entity_shape_label)) = self
                                .scene
                                .entities
                                .iter()
                                .find(|e| e.id == id)
                                .map(|e| (e.name.clone(), e.shape.label()))
                            else {
                                continue;
                            };
                            let is_selected = self.selected == Some(id);
                            let mut rename_now = false;
                            let mut start_rename = false;
                            let mut duplicate_now = false;
                            let mut delete_now = false;

                            if let Some((rid, name)) = &mut self.renaming
                                && *rid == id
                            {
                                let resp = ui.text_edit_singleline(name);
                                resp.request_focus();
                                if resp.lost_focus() {
                                    rename_now = true;
                                }
                            } else {
                                let label = format!("🧊 {entity_name}  [{entity_shape_label}]");
                                let resp = ui.selectable_label(is_selected, label);
                                if resp.clicked() {
                                    self.selected = Some(id);
                                }
                                if resp.double_clicked() {
                                    start_rename = true;
                                }
                                resp.context_menu(|ui| {
                                    if ui.button("Renommer").clicked() {
                                        start_rename = true;
                                        ui.close();
                                    }
                                    if ui.button("Dupliquer").clicked() {
                                        duplicate_now = true;
                                        ui.close();
                                    }
                                    if ui.button("Supprimer").clicked() {
                                        delete_now = true;
                                        ui.close();
                                    }
                                });
                            }

                            if rename_now
                                && let Some((_, name)) = self.renaming.take()
                                && let Some(e) = self.scene.find_mut(id)
                            {
                                e.name = name;
                            }
                            if start_rename {
                                self.renaming = Some((id, entity_name.clone()));
                            }
                            if duplicate_now {
                                self.selected = self.scene.duplicate(id);
                            }
                            if delete_now {
                                self.scene.remove(id);
                                if self.selected == Some(id) {
                                    self.selected = None;
                                }
                            }
                        }
                    });
            });
    }

    fn inspector_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::right("inspector")
            .resizable(true)
            .default_size(300.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::BG_PANEL)
                    .inner_margin(egui::Margin::symmetric(14, 12)),
            )
            .show(ui, |ui| {
                ui.heading("Inspector");
                ui.separator();
                let Some(id) = self.selected else {
                    ui.label(RichText::new("Aucune entité sélectionnée.").color(theme::TEXT_MAIN));
                    return;
                };
                let Some(entity) = self.scene.find_mut(id) else {
                    ui.label(RichText::new("Aucune entité sélectionnée.").color(theme::TEXT_MAIN));
                    return;
                };

                ui.horizontal(|ui| {
                    ui.checkbox(&mut entity.visible, "");
                    ui.add(
                        egui::TextEdit::singleline(&mut entity.name).desired_width(f32::INFINITY),
                    );
                });
                ui.add_space(4.0);

                egui::CollapsingHeader::new("Transform 4D")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label("Position");
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::new(&mut entity.transform.position.x)
                                    .prefix("x:")
                                    .speed(0.05),
                            );
                            ui.add(
                                egui::DragValue::new(&mut entity.transform.position.y)
                                    .prefix("y:")
                                    .speed(0.05),
                            );
                            ui.add(
                                egui::DragValue::new(&mut entity.transform.position.z)
                                    .prefix("z:")
                                    .speed(0.05),
                            );
                            ui.add(
                                egui::DragValue::new(&mut entity.transform.position.w)
                                    .prefix("w:")
                                    .speed(0.05),
                            );
                        });

                        ui.label("Scale");
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::new(&mut entity.transform.scale.x)
                                    .prefix("x:")
                                    .speed(0.02),
                            );
                            ui.add(
                                egui::DragValue::new(&mut entity.transform.scale.y)
                                    .prefix("y:")
                                    .speed(0.02),
                            );
                            ui.add(
                                egui::DragValue::new(&mut entity.transform.scale.z)
                                    .prefix("z:")
                                    .speed(0.02),
                            );
                            ui.add(
                                egui::DragValue::new(&mut entity.transform.scale.w)
                                    .prefix("w:")
                                    .speed(0.02),
                            );
                        });

                        ui.label("Rotation (degrés, 6 plans)");
                        egui::Grid::new("rotation_grid")
                            .num_columns(2)
                            .show(ui, |ui| {
                                for (i, plane) in RotationPlane::ALL.iter().enumerate() {
                                    let mut deg = entity.transform.rotation[i].to_degrees();
                                    ui.label(plane.label());
                                    if ui
                                        .add(egui::DragValue::new(&mut deg).suffix("°").speed(0.5))
                                        .changed()
                                    {
                                        entity.transform.rotation[i] = deg.to_radians();
                                    }
                                    ui.end_row();
                                }
                            });
                    });

                egui::CollapsingHeader::new("Forme 4D")
                    .default_open(true)
                    .show(ui, |ui| {
                        egui::ComboBox::from_label("Primitive")
                            .selected_text(entity.shape.label())
                            .show_ui(ui, |ui| {
                                for shape in ShapeKind::ALL {
                                    ui.selectable_value(&mut entity.shape, shape, shape.label());
                                }
                            });
                        let mut color = [
                            entity.color[0] as f32 / 255.0,
                            entity.color[1] as f32 / 255.0,
                            entity.color[2] as f32 / 255.0,
                        ];
                        ui.horizontal(|ui| {
                            ui.label("Couleur fil de fer");
                            if ui.color_edit_button_rgb(&mut color).changed() {
                                entity.color = [
                                    (color[0] * 255.0) as u8,
                                    (color[1] * 255.0) as u8,
                                    (color[2] * 255.0) as u8,
                                ];
                            }
                        });
                    });
            });
    }

    /// Barre d'outils du viewport lui-même (équivalent de la Scene view toolbar de Unity).
    fn viewport_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut self.viewport.mode,
                RenderMode::Projection,
                "Projection",
            );
            ui.selectable_value(&mut self.viewport.mode, RenderMode::Slice, "Coupe");
            ui.selectable_value(&mut self.viewport.mode, RenderMode::Both, "Les deux");
            ui.separator();
            ui.label("w:");
            ui.add(egui::Slider::new(&mut self.viewport.w_slice, -1.5..=1.5));
        });
        ui.horizontal(|ui| {
            ui.label("Vitesse rotation:");
            for (i, plane) in RotationPlane::ALL.iter().enumerate() {
                ui.add(
                    egui::DragValue::new(&mut self.viewport.rotation_speeds[i])
                        .prefix(format!("{}:", plane.label()))
                        .speed(0.02),
                );
            }
        });
    }
}

impl eframe::App for YonjigenApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let dt = ctx.input(|i| i.stable_dt);
        self.viewport.step(&mut self.scene, self.selected, dt);
        if self.viewport.playing {
            ctx.request_repaint();
        }

        self.menu_bar(ui);
        self.toolbar(ui);
        self.hierarchy_panel(ui);
        self.inspector_panel(ui);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::BG_PANEL)
                    .inner_margin(egui::Margin::same(12)),
            )
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.central_tab, CentralTab::Scene, "Scene");
                        ui.selectable_value(&mut self.central_tab, CentralTab::Game, "Jeu");
                    });
                    ui.separator();
                    if self.central_tab == CentralTab::Scene {
                        self.viewport_controls(ui);
                        ui.separator();
                    }
                    let remaining = ui.available_size();
                    let (rect, _) = ui.allocate_exact_size(remaining, egui::Sense::hover());
                    let mut child_ui =
                        ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(*ui.layout()));
                    let show_gizmos = self.central_tab == CentralTab::Scene;
                    self.viewport
                        .show(&mut child_ui, &self.scene, self.selected, show_gizmos);
                });
            });
    }
}
