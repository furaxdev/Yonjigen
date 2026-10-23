//! Mesh 4D : sommets + arêtes, et génération de primitives.

use crate::math4::Vec4;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh4 {
    pub vertices: Vec<Vec4>,
    /// Arêtes comme paires d'indices dans `vertices`.
    pub edges: Vec<(usize, usize)>,
    /// Faces planaires (boucles d'indices), utilisées pour la coupe (slice).
    /// Vide quand non applicable : la primitive reste affichable en projection.
    pub faces: Vec<Vec<usize>>,
}

impl Mesh4 {
    pub fn new(vertices: Vec<Vec4>, edges: Vec<(usize, usize)>) -> Self {
        Mesh4 {
            vertices,
            edges,
            faces: Vec::new(),
        }
    }

    pub fn with_faces(mut self, faces: Vec<Vec<usize>>) -> Self {
        self.faces = faces;
        self
    }

    /// Tesseract (hypercube) de demi-côté `half_extent` centré à l'origine.
    pub fn tesseract(half_extent: f64) -> Self {
        let mut vertices = Vec::with_capacity(16);
        for i in 0..16 {
            let x = if i & 1 != 0 {
                half_extent
            } else {
                -half_extent
            };
            let y = if i & 2 != 0 {
                half_extent
            } else {
                -half_extent
            };
            let z = if i & 4 != 0 {
                half_extent
            } else {
                -half_extent
            };
            let w = if i & 8 != 0 {
                half_extent
            } else {
                -half_extent
            };
            vertices.push(Vec4::new(x, y, z, w));
        }
        let mut edges = Vec::new();
        for i in 0..16 {
            for bit in 0..4 {
                let j = i ^ (1 << bit);
                if j > i {
                    edges.push((i, j));
                }
            }
        }
        let mut faces = Vec::new();
        // Choisit 2 axes "libres" parmi les 4, les 2 autres sont fixés (±).
        for free_a in 0..4 {
            for free_b in (free_a + 1)..4 {
                let fixed_bits: Vec<usize> =
                    (0..4).filter(|b| *b != free_a && *b != free_b).collect();
                for fixed_mask in 0..4 {
                    let base = ((fixed_mask & 1) << fixed_bits[0])
                        | (((fixed_mask >> 1) & 1) << fixed_bits[1]);
                    // Ordre en code Gray pour former une boucle convexe.
                    let loop_bits = [0, 1 << free_a, (1 << free_a) | (1 << free_b), 1 << free_b];
                    let face: Vec<usize> = loop_bits.iter().map(|b| base | b).collect();
                    faces.push(face);
                }
            }
        }
        Mesh4::new(vertices, edges).with_faces(faces)
    }

    /// 5-cell (simplexe 4D régulier) : 5 sommets tous reliés entre eux.
    pub fn five_cell(scale: f64) -> Self {
        // Coordonnées simples et symétriques (pas les plus "régulières" au sens strict
        // mais topologiquement correctes et faciles à vérifier).
        let base = [
            Vec4::new(1.0, 1.0, 1.0, -1.0 / 2.0_f64.sqrt()),
            Vec4::new(1.0, -1.0, -1.0, -1.0 / 2.0_f64.sqrt()),
            Vec4::new(-1.0, 1.0, -1.0, -1.0 / 2.0_f64.sqrt()),
            Vec4::new(-1.0, -1.0, 1.0, -1.0 / 2.0_f64.sqrt()),
            Vec4::new(0.0, 0.0, 0.0, 4.0 / 2.0_f64.sqrt()),
        ];
        let vertices = base.iter().map(|v| *v * scale).collect();
        let mut edges = Vec::new();
        for i in 0..5 {
            for j in (i + 1)..5 {
                edges.push((i, j));
            }
        }
        Mesh4::new(vertices, edges)
    }

    /// 16-cell : généralisation de l'octaèdre, 8 sommets sur les axes.
    pub fn sixteen_cell(scale: f64) -> Self {
        let vertices = vec![
            Vec4::new(scale, 0.0, 0.0, 0.0),
            Vec4::new(-scale, 0.0, 0.0, 0.0),
            Vec4::new(0.0, scale, 0.0, 0.0),
            Vec4::new(0.0, -scale, 0.0, 0.0),
            Vec4::new(0.0, 0.0, scale, 0.0),
            Vec4::new(0.0, 0.0, -scale, 0.0),
            Vec4::new(0.0, 0.0, 0.0, scale),
            Vec4::new(0.0, 0.0, 0.0, -scale),
        ];
        let mut edges = Vec::new();
        for i in 0..8 {
            for j in (i + 1)..8 {
                // Pas d'arête entre les deux sommets opposés du même axe.
                if i / 2 != j / 2 {
                    edges.push((i, j));
                }
            }
        }
        Mesh4::new(vertices, edges)
    }

    /// 24-cell : 24 sommets, permutations de (±1, ±1, 0, 0).
    pub fn twenty_four_cell(scale: f64) -> Self {
        let mut vertices = Vec::with_capacity(24);
        let coords = [1.0, -1.0];
        for &a in &coords {
            for &b in &coords {
                vertices.push(Vec4::new(a, b, 0.0, 0.0) * scale);
                vertices.push(Vec4::new(a, 0.0, b, 0.0) * scale);
                vertices.push(Vec4::new(a, 0.0, 0.0, b) * scale);
                vertices.push(Vec4::new(0.0, a, b, 0.0) * scale);
                vertices.push(Vec4::new(0.0, a, 0.0, b) * scale);
                vertices.push(Vec4::new(0.0, 0.0, a, b) * scale);
            }
        }
        // Deux sommets sont reliés si leur distance au carré vaut 2*scale^2
        // (distance la plus courte entre sommets distincts de ce polytope).
        let threshold = 2.0 * scale * scale;
        let mut edges = Vec::new();
        for i in 0..vertices.len() {
            for j in (i + 1)..vertices.len() {
                let d = (vertices[i] - vertices[j]).length_sq();
                if (d - threshold).abs() < 1e-6 {
                    edges.push((i, j));
                }
            }
        }
        Mesh4::new(vertices, edges)
    }

    /// Approximation d'une hypersphère (3-sphère) par un ensemble de points
    /// échantillonnés + arêtes vers les voisins les plus proches.
    pub fn hypersphere(radius: f64, segments: usize) -> Self {
        let mut vertices = Vec::new();
        // Échantillonnage par coordonnées hypersphériques (theta, phi, psi).
        let n = segments.max(3);
        for i in 0..n {
            let theta = std::f64::consts::PI * (i as f64) / (n as f64 - 1.0).max(1.0);
            for j in 0..n {
                let phi = std::f64::consts::PI * (j as f64) / (n as f64 - 1.0).max(1.0);
                let psi = 2.0 * std::f64::consts::PI * (j as f64) / (n as f64);
                let x = radius * theta.sin() * phi.sin() * psi.cos();
                let y = radius * theta.sin() * phi.sin() * psi.sin();
                let z = radius * theta.sin() * phi.cos();
                let w = radius * theta.cos();
                vertices.push(Vec4::new(x, y, z, w));
            }
        }
        // Relie chaque point à son suivant dans la grille (approximation visuelle).
        let mut edges = Vec::new();
        for i in 0..n {
            for j in 0..n {
                let idx = i * n + j;
                if j + 1 < n {
                    edges.push((idx, idx + 1));
                }
                if i + 1 < n {
                    edges.push((idx, idx + n));
                }
            }
        }
        Mesh4::new(vertices, edges)
    }

    /// Duoprisme {p}x{q} : produit de deux polygones réguliers.
    pub fn duoprism(p: usize, q: usize, radius_a: f64, radius_b: f64) -> Self {
        let p = p.max(3);
        let q = q.max(3);
        let mut vertices = Vec::with_capacity(p * q);
        for i in 0..p {
            let a = 2.0 * std::f64::consts::PI * (i as f64) / (p as f64);
            for j in 0..q {
                let b = 2.0 * std::f64::consts::PI * (j as f64) / (q as f64);
                vertices.push(Vec4::new(
                    radius_a * a.cos(),
                    radius_a * a.sin(),
                    radius_b * b.cos(),
                    radius_b * b.sin(),
                ));
            }
        }
        let mut edges = Vec::new();
        for i in 0..p {
            for j in 0..q {
                let idx = i * q + j;
                let next_j = i * q + (j + 1) % q;
                edges.push((idx, next_j));
                let next_i = ((i + 1) % p) * q + j;
                edges.push((idx, next_i));
            }
        }
        Mesh4::new(vertices, edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tesseract_has_16_vertices_and_32_edges() {
        let t = Mesh4::tesseract(1.0);
        assert_eq!(t.vertices.len(), 16);
        assert_eq!(t.edges.len(), 32);
        assert_eq!(t.faces.len(), 24);
    }

    #[test]
    fn five_cell_has_5_vertices_and_10_edges() {
        let c = Mesh4::five_cell(1.0);
        assert_eq!(c.vertices.len(), 5);
        assert_eq!(c.edges.len(), 10);
    }

    #[test]
    fn sixteen_cell_has_8_vertices_and_24_edges() {
        let c = Mesh4::sixteen_cell(1.0);
        assert_eq!(c.vertices.len(), 8);
        assert_eq!(c.edges.len(), 24);
    }

    #[test]
    fn twenty_four_cell_has_24_vertices() {
        let c = Mesh4::twenty_four_cell(1.0);
        assert_eq!(c.vertices.len(), 24);
        assert!(!c.edges.is_empty());
    }
}
