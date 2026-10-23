use crate::theme;
use crate::viewport::{RenderMode, ViewportState};
use egui::{Color32, RichText};
use yonjigen_core::math4::RotationPlane;
use yonjigen_core::scene::{Scene, ShapeKind};

pub struct YonjigenApp {
    scene: Scene,
    selected: Option<u64>,
    viewport: ViewportState,
    renaming: Option<(u64, String)>,
    current_file: Option<std::path::PathBuf>,
    status: String,
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
        }
    }
}

impl YonjigenApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::setup(&cc.egui_ctx);
        Self::default()
    }

    fn draw_logo(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("四次元")
                    .monospace()
                    .size(20.0)
                    .color(theme::NEON_PURPLE)
                    .strong(),
            );
            ui.label(
                RichText::new("YONJIGEN")
                    .monospace()
                    .size(16.0)
                    .color(theme::NEON_BLUE),
            );
        });
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                self.draw_logo(ui);
                ui.separator();
                ui.menu_button("Fichier", |ui| {
                    if ui.button("Nouveau").clicked() {
                        self.scene = Scene::default();
                        self.selected = None;
                        self.current_file = None;
                        self.status = "Nouvelle scène.".into();
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
                                    self.status = format!("Ouvert : {}", path.display());
                                    self.current_file = Some(path);
                                }
                                None => self.status = "Échec de l'ouverture.".into(),
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
                if ui.button("Exporter le jeu").clicked() {
                    self.status = "Export standalone : à venir (étape 6).".into();
                }
                ui.separator();
                ui.label(RichText::new(&self.status).color(theme::TEXT_MAIN).small());
            });
        });
    }

    fn save_scene(&mut self) {
        let Some(path) = self.current_file.clone() else {
            self.status = "Aucun fichier : utilisez Sauvegarder sous.".into();
            return;
        };
        match self.scene.to_json() {
            Ok(json) => match std::fs::write(&path, json) {
                Ok(()) => self.status = format!("Sauvegardé : {}", path.display()),
                Err(e) => self.status = format!("Erreur d'écriture : {e}"),
            },
            Err(e) => self.status = format!("Erreur de sérialisation : {e}"),
        }
    }

    fn hierarchy_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("hierarchy")
            .resizable(true)
            .default_size(220.0)
            .show(ui, |ui| {
                ui.heading("Hiérarchie");
                ui.menu_button("+ Ajouter", |ui| {
                    for shape in ShapeKind::ALL {
                        if ui.button(shape.label()).clicked() {
                            let id = self.scene.add(shape.label(), shape);
                            self.selected = Some(id);
                            ui.close();
                        }
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let ids: Vec<u64> = self.scene.entities.iter().map(|e| e.id).collect();
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
                        ui.horizontal(|ui| {
                            if let Some((rid, name)) = &mut self.renaming
                                && *rid == id
                            {
                                let resp = ui.text_edit_singleline(name);
                                if resp.lost_focus() {
                                    rename_now = true;
                                }
                                return;
                            }
                            let label = format!("{entity_name}  [{entity_shape_label}]");
                            if ui.selectable_label(is_selected, label).clicked() {
                                self.selected = Some(id);
                            }
                        });
                        if rename_now
                            && let Some((_, name)) = self.renaming.take()
                            && let Some(e) = self.scene.find_mut(id)
                        {
                            e.name = name;
                        }
                        ui.horizontal(|ui| {
                            ui.add_space(12.0);
                            if ui.small_button("Renommer").clicked() {
                                self.renaming = Some((id, entity_name.clone()));
                            }
                            if ui.small_button("Dupliquer").clicked() {
                                self.selected = self.scene.duplicate(id);
                            }
                            if ui.small_button("Suppr").clicked() {
                                self.scene.remove(id);
                                if self.selected == Some(id) {
                                    self.selected = None;
                                }
                            }
                        });
                    }
                });
            });
    }

    fn inspector_panel(&mut self, ui: &mut egui::Ui) {
        egui::Panel::right("inspector")
            .resizable(true)
            .default_size(280.0)
            .show(ui, |ui| {
                ui.heading("Inspecteur");
                ui.separator();
                let Some(id) = self.selected else {
                    ui.label("Aucune entité sélectionnée.");
                    return;
                };
                let Some(entity) = self.scene.find_mut(id) else {
                    ui.label("Aucune entité sélectionnée.");
                    return;
                };
                ui.label(RichText::new(&entity.name).strong());
                ui.checkbox(&mut entity.visible, "Visible");
                ui.separator();

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

                ui.label("Échelle");
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

                ui.separator();
                let mut color = [
                    entity.color[0] as f32 / 255.0,
                    entity.color[1] as f32 / 255.0,
                    entity.color[2] as f32 / 255.0,
                ];
                if ui.color_edit_button_rgb(&mut color).changed() {
                    entity.color = [
                        (color[0] * 255.0) as u8,
                        (color[1] * 255.0) as u8,
                        (color[2] * 255.0) as u8,
                    ];
                }
            });
    }

    fn viewport_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.selectable_value(
                &mut self.viewport.mode,
                RenderMode::Projection,
                "Projection",
            );
            ui.selectable_value(&mut self.viewport.mode, RenderMode::Slice, "Coupe");
            ui.selectable_value(&mut self.viewport.mode, RenderMode::Both, "Les deux");
            ui.separator();
            let label = if self.viewport.playing {
                "⏸ Pause"
            } else {
                "▶ Play"
            };
            if ui.button(label).clicked() {
                self.viewport.playing = !self.viewport.playing;
            }
        });
        ui.horizontal(|ui| {
            ui.label("Coupe w:");
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

        self.top_bar(ui);
        self.hierarchy_panel(ui);
        self.inspector_panel(ui);

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::BLACK))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    self.viewport_controls(ui);
                    ui.separator();
                    let remaining = ui.available_size();
                    let (rect, _) = ui.allocate_exact_size(remaining, egui::Sense::hover());
                    let mut child_ui =
                        ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(*ui.layout()));
                    self.viewport
                        .show(&mut child_ui, &self.scene, self.selected);
                });
            });
    }
}
