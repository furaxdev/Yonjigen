//! Pipeline de rendu : projection perspective 4D->3D->2D, et coupe (slice) w=c.

use crate::mesh4::Mesh4;
use crate::vec3::Vec3;

/// Mesh "plat" en 3D, résultat d'une projection ou d'une coupe.
#[derive(Debug, Clone, Default)]
pub struct Mesh3 {
    pub vertices: Vec<Vec3>,
    pub edges: Vec<(usize, usize)>,
    /// Profondeur w d'origine pour chaque sommet (pour la couleur / tri).
    pub depth: Vec<f64>,
}

/// Projette un point 4D vers 3D en perspective, la distance `w_camera`
/// faisant office de focale sur l'axe w (plus w est loin de la caméra,
/// plus le point est "rapetissé").
pub fn project_point_perspective(p: crate::math4::Vec4, w_camera: f64) -> Vec3 {
    let denom = w_camera - p.w;
    let scale = if denom.abs() > 1e-6 {
        w_camera / denom
    } else {
        1.0
    };
    Vec3::new(p.x * scale, p.y * scale, p.z * scale)
}

/// Projette un mesh 4D entier en 3D (mode "Projection").
pub fn project_mesh_perspective(mesh: &Mesh4, w_camera: f64) -> Mesh3 {
    let vertices: Vec<Vec3> = mesh
        .vertices
        .iter()
        .map(|v| project_point_perspective(*v, w_camera))
        .collect();
    let depth: Vec<f64> = mesh.vertices.iter().map(|v| v.w).collect();
    Mesh3 {
        vertices,
        edges: mesh.edges.clone(),
        depth,
    }
}

/// Projette un point 3D vers l'écran 2D (perspective simple, caméra sur -z en regardant +z).
/// Retourne (x, y, depth) où depth sert au tri/à l'assombrissement.
pub fn project_point_screen(p: Vec3, camera_distance: f64) -> (f32, f32, f32) {
    let denom = camera_distance - p.z;
    let scale = if denom.abs() > 1e-6 {
        camera_distance / denom
    } else {
        1.0
    };
    ((p.x * scale) as f32, (p.y * scale) as f32, p.z as f32)
}

/// Coupe (slice) d'un mesh 4D par l'hyperplan w = c.
/// Utilise la liste de faces : chaque face planaire convexe qui traverse le
/// plan contribue un segment (les deux points d'entrée/sortie).
pub fn slice_mesh(mesh: &Mesh4, c: f64) -> Mesh3 {
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut depth: Vec<f64> = Vec::new();

    let find_or_add = |p: Vec3, verts: &mut Vec<Vec3>, depth: &mut Vec<f64>| -> usize {
        for (i, v) in verts.iter().enumerate() {
            let d = (v.x - p.x).powi(2) + (v.y - p.y).powi(2) + (v.z - p.z).powi(2);
            if d < 1e-10 {
                return i;
            }
        }
        verts.push(p);
        depth.push(c);
        verts.len() - 1
    };

    for face in &mesh.faces {
        let n = face.len();
        if n < 3 {
            continue;
        }
        let mut crossings: Vec<Vec3> = Vec::new();
        for i in 0..n {
            let a = mesh.vertices[face[i]];
            let b = mesh.vertices[face[(i + 1) % n]];
            let (wa, wb) = (a.w - c, b.w - c);
            if wa == 0.0 && wb == 0.0 {
                continue;
            }
            if (wa >= 0.0) != (wb >= 0.0) {
                let t = wa / (wa - wb);
                let p = a.lerp(b, t);
                crossings.push(Vec3::new(p.x, p.y, p.z));
            }
        }
        if crossings.len() == 2 {
            let i0 = find_or_add(crossings[0], &mut vertices, &mut depth);
            let i1 = find_or_add(crossings[1], &mut vertices, &mut depth);
            if i0 != i1 {
                edges.push((i0, i1));
            }
        }
    }

    Mesh3 {
        vertices,
        edges,
        depth,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh4::Mesh4;

    #[test]
    fn slicing_tesseract_at_zero_gives_cube() {
        let t = Mesh4::tesseract(1.0);
        let cube = slice_mesh(&t, 0.0);
        assert_eq!(cube.vertices.len(), 8, "un cube a 8 sommets");
        assert_eq!(cube.edges.len(), 12, "un cube a 12 arêtes");
        for v in &cube.vertices {
            assert!((v.x.abs() - 1.0).abs() < 1e-9);
            assert!((v.y.abs() - 1.0).abs() < 1e-9);
            assert!((v.z.abs() - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn slicing_outside_range_gives_empty() {
        let t = Mesh4::tesseract(1.0);
        let empty = slice_mesh(&t, 5.0);
        assert!(empty.vertices.is_empty());
    }

    #[test]
    fn perspective_projection_identity_at_origin() {
        let p = crate::math4::Vec4::new(1.0, 2.0, 3.0, 0.0);
        let proj = project_point_perspective(p, 5.0);
        assert!((proj.x - 1.0).abs() < 1e-9);
        assert!((proj.y - 2.0).abs() < 1e-9);
        assert!((proj.z - 3.0).abs() < 1e-9);
    }
}
