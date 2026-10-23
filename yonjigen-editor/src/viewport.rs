//! Panneau viewport 4D : orbite, zoom, slider w, modes Projection/Coupe/Les deux.

use egui::{Color32, Pos2, Sense, Stroke, Vec2};
use yonjigen_core::math4::RotationPlane;
use yonjigen_core::render::{Mesh3, project_mesh_perspective, project_point_screen, slice_mesh};
use yonjigen_core::scene::Scene;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Projection,
    Slice,
    Both,
}

pub struct ViewportState {
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub w_slice: f64,
    pub mode: RenderMode,
    pub camera_w: f64,
    pub playing: bool,
    /// Vitesse de rotation (rad/s) pour chacun des 6 plans.
    pub rotation_speeds: [f64; 6],
}

impl Default for ViewportState {
    fn default() -> Self {
        ViewportState {
            yaw: 0.6,
            pitch: 0.4,
            zoom: 160.0,
            w_slice: 0.0,
            mode: RenderMode::Both,
            camera_w: 3.0,
            playing: true,
            rotation_speeds: [0.0, 0.0, 0.3, 0.0, 0.5, 0.2],
        }
    }
}

impl ViewportState {
    pub fn step(&mut self, scene: &mut Scene, selected: Option<u64>, dt: f32) {
        if !self.playing {
            return;
        }
        let Some(id) = selected else { return };
        if let Some(entity) = scene.find_mut(id) {
            for (i, plane) in RotationPlane::ALL.iter().enumerate() {
                let _ = plane;
                entity.transform.rotation[i] += self.rotation_speeds[i] * dt as f64;
            }
        }
    }

    fn rotate_orbit(&self, p: yonjigen_core::vec3::Vec3) -> yonjigen_core::vec3::Vec3 {
        // Rotation yaw (autour de Y) puis pitch (autour de X), pour l'orbite caméra.
        let (sy, cy) = (self.yaw.sin() as f64, self.yaw.cos() as f64);
        let x1 = p.x * cy + p.z * sy;
        let z1 = -p.x * sy + p.z * cy;
        let (sp, cp) = (self.pitch.sin() as f64, self.pitch.cos() as f64);
        let y2 = p.y * cp - z1 * sp;
        let z2 = p.y * sp + z1 * cp;
        yonjigen_core::vec3::Vec3::new(x1, y2, z2)
    }

    fn paint_mesh3(
        &self,
        painter: &egui::Painter,
        center: Pos2,
        mesh: &Mesh3,
        base_color: Color32,
        w_range: (f64, f64),
    ) {
        let screen_points: Vec<Pos2> = mesh
            .vertices
            .iter()
            .map(|v| {
                let rotated = self.rotate_orbit(*v);
                let (x, y, _depth) = project_point_screen(rotated, 6.0);
                Pos2::new(center.x + x * self.zoom, center.y - y * self.zoom)
            })
            .collect();

        let (w_min, w_max) = w_range;
        let w_span = (w_max - w_min).max(1e-6);

        for &(a, b) in &mesh.edges {
            if a >= screen_points.len() || b >= screen_points.len() {
                continue;
            }
            let t = ((mesh.depth[a] + mesh.depth[b]) / 2.0 - w_min) / w_span;
            let t = t.clamp(0.0, 1.0) as f32;
            let color = lerp_color(base_color, Color32::WHITE, t * 0.4);
            painter.line_segment(
                [screen_points[a], screen_points[b]],
                Stroke::new(2.0, color),
            );
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, scene: &Scene, selected: Option<u64>) {
        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, crate::theme::BG_DARK);

        if response.dragged() {
            let delta: Vec2 = response.drag_delta();
            self.yaw += delta.x * 0.01;
            self.pitch += delta.y * 0.01;
        }
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            self.zoom = (self.zoom + scroll).clamp(20.0, 800.0);
        }

        let center = rect.center();
        // Axes de repère (x=rouge, y=vert, z=bleu néon) pour l'orientation.
        let axis_len = 60.0;
        for (dir, color) in [
            (
                yonjigen_core::vec3::Vec3::new(1.0, 0.0, 0.0),
                Color32::from_rgb(0xff, 0x50, 0x50),
            ),
            (
                yonjigen_core::vec3::Vec3::new(0.0, 1.0, 0.0),
                Color32::from_rgb(0x50, 0xff, 0x80),
            ),
            (
                yonjigen_core::vec3::Vec3::new(0.0, 0.0, 1.0),
                crate::theme::NEON_BLUE,
            ),
        ] {
            let p = self.rotate_orbit(dir);
            let (x, y, _) = project_point_screen(p, 6.0);
            painter.line_segment(
                [
                    center,
                    Pos2::new(center.x + x * axis_len, center.y - y * axis_len),
                ],
                Stroke::new(1.5, color),
            );
        }

        for entity in &scene.entities {
            if !entity.visible {
                continue;
            }
            let is_selected = selected == Some(entity.id);
            let base_color = Color32::from_rgb(entity.color[0], entity.color[1], entity.color[2]);
            let base_color = if is_selected {
                lerp_color(base_color, Color32::WHITE, 0.25)
            } else {
                base_color
            };

            let mesh4 = entity.shape.generate_mesh();
            let world = entity.transform.matrix();
            let transformed: Vec<_> = mesh4
                .vertices
                .iter()
                .map(|v| world.transform_point(*v))
                .collect();
            let mut world_mesh = mesh4.clone();
            world_mesh.vertices = transformed;

            let w_values: Vec<f64> = world_mesh.vertices.iter().map(|v| v.w).collect();
            let w_min = w_values.iter().cloned().fold(f64::INFINITY, f64::min);
            let w_max = w_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            if matches!(self.mode, RenderMode::Projection | RenderMode::Both) {
                let projected = project_mesh_perspective(&world_mesh, self.camera_w);
                self.paint_mesh3(&painter, center, &projected, base_color, (w_min, w_max));
            }
            if matches!(self.mode, RenderMode::Slice | RenderMode::Both) {
                let sliced = slice_mesh(&world_mesh, self.w_slice);
                let slice_color = crate::theme::NEON_BLUE;
                let slice_color = if is_selected {
                    lerp_color(slice_color, Color32::WHITE, 0.25)
                } else {
                    slice_color
                };
                self.paint_mesh3(&painter, center, &sliced, slice_color, (w_min, w_max));
            }
        }

        painter.text(
            rect.left_top() + Vec2::new(8.0, 6.0),
            egui::Align2::LEFT_TOP,
            format!("w = {:.2}  |  mode: {:?}", self.w_slice, self.mode),
            egui::FontId::monospace(12.0),
            crate::theme::TEXT_MAIN,
        );
    }
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}
