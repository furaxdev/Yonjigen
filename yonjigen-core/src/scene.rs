//! Scène et entités : ECS minimal (composants attachés directement à l'entité).

use crate::math4::Transform4;
use crate::mesh4::Mesh4;
use serde::{Deserialize, Serialize};

/// Forme générée procéduralement (pas de mesh importé pour le MVP).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ShapeKind {
    Tesseract,
    FiveCell,
    SixteenCell,
    TwentyFourCell,
    Hypersphere,
    Duoprism,
}

impl ShapeKind {
    pub const ALL: [ShapeKind; 6] = [
        ShapeKind::Tesseract,
        ShapeKind::FiveCell,
        ShapeKind::SixteenCell,
        ShapeKind::TwentyFourCell,
        ShapeKind::Hypersphere,
        ShapeKind::Duoprism,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ShapeKind::Tesseract => "Tesseract",
            ShapeKind::FiveCell => "5-Cell",
            ShapeKind::SixteenCell => "16-Cell",
            ShapeKind::TwentyFourCell => "24-Cell",
            ShapeKind::Hypersphere => "Hypersphère",
            ShapeKind::Duoprism => "Duoprisme",
        }
    }

    pub fn generate_mesh(self) -> Mesh4 {
        match self {
            ShapeKind::Tesseract => Mesh4::tesseract(1.0),
            ShapeKind::FiveCell => Mesh4::five_cell(1.0),
            ShapeKind::SixteenCell => Mesh4::sixteen_cell(1.0),
            ShapeKind::TwentyFourCell => Mesh4::twenty_four_cell(1.0),
            ShapeKind::Hypersphere => Mesh4::hypersphere(1.0, 10),
            ShapeKind::Duoprism => Mesh4::duoprism(6, 6, 0.7, 0.7),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: u64,
    pub name: String,
    pub transform: Transform4,
    pub shape: ShapeKind,
    pub color: [u8; 3],
    pub visible: bool,
    /// Composant Script : code source Lua exécuté chaque frame pendant Play.
    #[serde(default)]
    pub script: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub entities: Vec<Entity>,
    next_id: u64,
}

impl Default for Scene {
    fn default() -> Self {
        Scene {
            name: "Nouvelle scène".to_string(),
            entities: Vec::new(),
            next_id: 1,
        }
    }
}

impl Scene {
    pub fn add(&mut self, name: impl Into<String>, shape: ShapeKind) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.entities.push(Entity {
            id,
            name: name.into(),
            transform: Transform4::default(),
            shape,
            color: [0xd8, 0xd8, 0xd8],
            visible: true,
            script: None,
        });
        id
    }

    pub fn remove(&mut self, id: u64) {
        self.entities.retain(|e| e.id != id);
    }

    pub fn duplicate(&mut self, id: u64) -> Option<u64> {
        let source = self.entities.iter().find(|e| e.id == id)?.clone();
        let new_id = self.next_id;
        self.next_id += 1;
        let mut copy = source;
        copy.id = new_id;
        copy.name = format!("{} (copie)", copy.name);
        self.entities.push(copy);
        Some(new_id)
    }

    pub fn find_mut(&mut self, id: u64) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == id)
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(s: &str) -> serde_json::Result<Scene> {
        serde_json::from_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_remove_duplicate_roundtrip() {
        let mut scene = Scene::default();
        let id = scene.add("Cube4D", ShapeKind::Tesseract);
        assert_eq!(scene.entities.len(), 1);
        let dup_id = scene.duplicate(id).unwrap();
        assert_eq!(scene.entities.len(), 2);
        assert_ne!(id, dup_id);
        scene.remove(id);
        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].id, dup_id);
    }

    #[test]
    fn scene_json_roundtrip() {
        let mut scene = Scene::default();
        scene.add("Forme", ShapeKind::FiveCell);
        let json = scene.to_json().unwrap();
        let back = Scene::from_json(&json).unwrap();
        assert_eq!(back.entities.len(), 1);
        assert_eq!(back.entities[0].shape, ShapeKind::FiveCell);
    }
}
