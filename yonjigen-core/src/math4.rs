//! Maths 4D : vecteurs, matrices homogènes 5x5, rotations sur les 6 plans.

use serde::{Deserialize, Serialize};
use std::ops::{Add, Div, Index, IndexMut, Mul, Sub};

/// Vecteur à 4 composantes spatiales (x, y, z, w).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl Vec4 {
    pub const ZERO: Vec4 = Vec4 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };

    pub fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Vec4 { x, y, z, w }
    }

    pub fn splat(v: f64) -> Self {
        Vec4::new(v, v, v, v)
    }

    pub fn dot(self, other: Vec4) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }

    pub fn length_sq(self) -> f64 {
        self.dot(self)
    }

    pub fn length(self) -> f64 {
        self.length_sq().sqrt()
    }

    pub fn normalized(self) -> Vec4 {
        let len = self.length();
        if len <= f64::EPSILON {
            self
        } else {
            self / len
        }
    }

    pub fn lerp(self, other: Vec4, t: f64) -> Vec4 {
        self + (other - self) * t
    }

    /// Homogénéise en (x, y, z, w, 1.0) pour la multiplication par Mat5.
    pub fn to_homogeneous(self) -> [f64; 5] {
        [self.x, self.y, self.z, self.w, 1.0]
    }

    pub fn from_homogeneous(h: [f64; 5]) -> Vec4 {
        Vec4::new(h[0], h[1], h[2], h[3])
    }
}

impl Add for Vec4 {
    type Output = Vec4;
    fn add(self, rhs: Vec4) -> Vec4 {
        Vec4::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

impl Sub for Vec4 {
    type Output = Vec4;
    fn sub(self, rhs: Vec4) -> Vec4 {
        Vec4::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
            self.w - rhs.w,
        )
    }
}

impl Mul<f64> for Vec4 {
    type Output = Vec4;
    fn mul(self, rhs: f64) -> Vec4 {
        Vec4::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

impl Div<f64> for Vec4 {
    type Output = Vec4;
    fn div(self, rhs: f64) -> Vec4 {
        Vec4::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
    }
}

/// Les 6 plans de rotation possibles en 4D (groupe SO(4)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RotationPlane {
    Xy,
    Xz,
    Xw,
    Yz,
    Yw,
    Zw,
}

impl RotationPlane {
    pub const ALL: [RotationPlane; 6] = [
        RotationPlane::Xy,
        RotationPlane::Xz,
        RotationPlane::Xw,
        RotationPlane::Yz,
        RotationPlane::Yw,
        RotationPlane::Zw,
    ];

    /// Indices des deux axes affectés par la rotation dans ce plan.
    fn axes(self) -> (usize, usize) {
        match self {
            RotationPlane::Xy => (0, 1),
            RotationPlane::Xz => (0, 2),
            RotationPlane::Xw => (0, 3),
            RotationPlane::Yz => (1, 2),
            RotationPlane::Yw => (1, 3),
            RotationPlane::Zw => (2, 3),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RotationPlane::Xy => "XY",
            RotationPlane::Xz => "XZ",
            RotationPlane::Xw => "XW",
            RotationPlane::Yz => "YZ",
            RotationPlane::Yw => "YW",
            RotationPlane::Zw => "ZW",
        }
    }
}

/// Matrice 5x5 (transformation homogène en 4D : rotation/échelle 4x4 + translation).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mat5 {
    pub m: [[f64; 5]; 5],
}

impl Mat5 {
    #[allow(clippy::needless_range_loop)]
    pub fn identity() -> Self {
        let mut m = [[0.0; 5]; 5];
        for i in 0..5 {
            m[i][i] = 1.0;
        }
        Mat5 { m }
    }

    pub fn translation(t: Vec4) -> Self {
        let mut mat = Mat5::identity();
        mat.m[0][4] = t.x;
        mat.m[1][4] = t.y;
        mat.m[2][4] = t.z;
        mat.m[3][4] = t.w;
        mat
    }

    pub fn scale(s: Vec4) -> Self {
        let mut mat = Mat5::identity();
        mat.m[0][0] = s.x;
        mat.m[1][1] = s.y;
        mat.m[2][2] = s.z;
        mat.m[3][3] = s.w;
        mat
    }

    /// Rotation d'angle `radians` dans le plan donné.
    pub fn rotation(plane: RotationPlane, radians: f64) -> Self {
        let mut mat = Mat5::identity();
        let (a, b) = plane.axes();
        let (c, s) = (radians.cos(), radians.sin());
        mat.m[a][a] = c;
        mat.m[a][b] = -s;
        mat.m[b][a] = s;
        mat.m[b][b] = c;
        mat
    }

    #[allow(clippy::needless_range_loop)]
    pub fn mul_mat(&self, rhs: &Mat5) -> Mat5 {
        let mut out = [[0.0; 5]; 5];
        for i in 0..5 {
            for j in 0..5 {
                let mut sum = 0.0;
                for k in 0..5 {
                    sum += self.m[i][k] * rhs.m[k][j];
                }
                out[i][j] = sum;
            }
        }
        Mat5 { m: out }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn transform_point(&self, p: Vec4) -> Vec4 {
        let h = p.to_homogeneous();
        let mut out = [0.0; 5];
        for i in 0..5 {
            let mut sum = 0.0;
            for k in 0..5 {
                sum += self.m[i][k] * h[k];
            }
            out[i] = sum;
        }
        Vec4::from_homogeneous(out)
    }

    /// Transforme une direction (ignore la translation).
    #[allow(clippy::needless_range_loop)]
    pub fn transform_dir(&self, d: Vec4) -> Vec4 {
        let h = [d.x, d.y, d.z, d.w, 0.0];
        let mut out = [0.0; 4];
        for (i, slot) in out.iter_mut().enumerate() {
            let mut sum = 0.0;
            for k in 0..5 {
                sum += self.m[i][k] * h[k];
            }
            *slot = sum;
        }
        Vec4::new(out[0], out[1], out[2], out[3])
    }
}

impl Index<(usize, usize)> for Mat5 {
    type Output = f64;
    fn index(&self, (r, c): (usize, usize)) -> &f64 {
        &self.m[r][c]
    }
}

impl IndexMut<(usize, usize)> for Mat5 {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut f64 {
        &mut self.m[r][c]
    }
}

/// Transform 4D complet : position, 6 angles de rotation, échelle.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform4 {
    pub position: Vec4,
    /// Angles en radians pour chaque plan, indexés comme RotationPlane::ALL.
    pub rotation: [f64; 6],
    pub scale: Vec4,
}

impl Default for Transform4 {
    fn default() -> Self {
        Transform4 {
            position: Vec4::ZERO,
            rotation: [0.0; 6],
            scale: Vec4::splat(1.0),
        }
    }
}

impl Transform4 {
    pub fn matrix(&self) -> Mat5 {
        let mut rot = Mat5::identity();
        for (i, plane) in RotationPlane::ALL.iter().enumerate() {
            if self.rotation[i] != 0.0 {
                rot = rot.mul_mat(&Mat5::rotation(*plane, self.rotation[i]));
            }
        }
        let scale = Mat5::scale(self.scale);
        let translation = Mat5::translation(self.position);
        translation.mul_mat(&rot).mul_mat(&scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn rotation_preserves_length() {
        let v = Vec4::new(1.0, 2.0, 3.0, 4.0);
        for plane in RotationPlane::ALL {
            let r = Mat5::rotation(plane, 0.7321);
            let rotated = r.transform_point(v);
            assert!(
                (rotated.length() - v.length()).abs() < 1e-9,
                "plane {:?}",
                plane
            );
        }
    }

    #[test]
    fn rotation_full_turn_is_identity() {
        let v = Vec4::new(1.0, -2.0, 0.5, 3.0);
        let r = Mat5::rotation(RotationPlane::Xw, 2.0 * PI);
        let rotated = r.transform_point(v);
        assert!((rotated.x - v.x).abs() < 1e-9);
        assert!((rotated.w - v.w).abs() < 1e-9);
    }

    #[test]
    fn translation_moves_point() {
        let t = Mat5::translation(Vec4::new(1.0, 2.0, 3.0, 4.0));
        let p = t.transform_point(Vec4::ZERO);
        assert_eq!(p, Vec4::new(1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn scale_scales_point() {
        let s = Mat5::scale(Vec4::new(2.0, 2.0, 2.0, 2.0));
        let p = s.transform_point(Vec4::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(p, Vec4::new(2.0, 2.0, 2.0, 2.0));
    }

    #[test]
    fn transform_matrix_combines_trs() {
        let t = Transform4 {
            position: Vec4::new(1.0, 0.0, 0.0, 0.0),
            rotation: [0.0; 6],
            scale: Vec4::splat(2.0),
        };
        let p = t.matrix().transform_point(Vec4::new(1.0, 0.0, 0.0, 0.0));
        assert_eq!(p, Vec4::new(3.0, 0.0, 0.0, 0.0));
    }
}
