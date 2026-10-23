//! Scripting Lua : exécute le script d'une entité contre son `Transform4`.
//!
//! Décision simple pour le MVP : la machine Lua est recréée à chaque appel
//! (donc à chaque frame pendant Play). C'est un peu plus coûteux qu'une VM
//! persistante, mais ça évite toute la complexité de garder un état Lua en
//! vie entre deux emprunts mutables de la scène, et ça permet l'édition à
//! chaud du script sans redémarrer quoi que ce soit. Les variables locales
//! Lua ne survivent donc pas d'une frame à l'autre : seul l'état du
//! `Transform4` (exposé via l'API) est persistant.

use crate::math4::{Transform4, Vec4};
use mlua::Lua;
use std::cell::RefCell;

/// Nom des fonctions et constantes exposées aux scripts, pour l'autocomplétion
/// de l'éditeur de code.
pub const API_SIGNATURES: &[(&str, &str)] = &[
    ("dt", "dt -- durée de la frame en secondes (nombre)"),
    ("log(msg)", "log(msg) -- affiche un message dans la console"),
    ("get_position()", "get_position() -> x, y, z, w"),
    ("set_position(x, y, z, w)", "set_position(x, y, z, w)"),
    ("get_rotation(plane)", "get_rotation(plane) -> degrés"),
    (
        "set_rotation(plane, degrees)",
        "set_rotation(plane, degrees)",
    ),
    (
        "rotate(plane, delta_degrees)",
        "rotate(plane, delta_degrees) -- ajoute delta_degrees",
    ),
    ("PLANE_XY", "PLANE_XY = 0"),
    ("PLANE_XZ", "PLANE_XZ = 1"),
    ("PLANE_XW", "PLANE_XW = 2"),
    ("PLANE_YZ", "PLANE_YZ = 3"),
    ("PLANE_YW", "PLANE_YW = 4"),
    ("PLANE_ZW", "PLANE_ZW = 5"),
];

/// Exécute `source` une fois, avec `transform` comme état mutable exposé à
/// l'API Lua. Les messages de `log(...)` sont ajoutés à `log_out`.
/// Retourne une erreur lisible (syntaxe ou exécution) en cas d'échec.
pub fn run_frame(
    transform: &mut Transform4,
    dt: f64,
    source: &str,
    log_out: &mut Vec<String>,
) -> Result<(), String> {
    let lua = Lua::new();
    let transform_cell = RefCell::new(*transform);
    let log_cell = RefCell::new(Vec::<String>::new());

    let result: mlua::Result<()> = lua.scope(|scope| {
        let globals = lua.globals();
        globals.set("dt", dt)?;
        globals.set("PLANE_XY", 0)?;
        globals.set("PLANE_XZ", 1)?;
        globals.set("PLANE_XW", 2)?;
        globals.set("PLANE_YZ", 3)?;
        globals.set("PLANE_YW", 4)?;
        globals.set("PLANE_ZW", 5)?;

        globals.set(
            "log",
            scope.create_function(|_, msg: String| {
                log_cell.borrow_mut().push(msg);
                Ok(())
            })?,
        )?;

        globals.set(
            "get_position",
            scope.create_function(|_, ()| {
                let t = transform_cell.borrow();
                Ok((t.position.x, t.position.y, t.position.z, t.position.w))
            })?,
        )?;

        globals.set(
            "set_position",
            scope.create_function(|_, (x, y, z, w): (f64, f64, f64, f64)| {
                transform_cell.borrow_mut().position = Vec4::new(x, y, z, w);
                Ok(())
            })?,
        )?;

        globals.set(
            "get_rotation",
            scope.create_function(|_, plane: usize| {
                let t = transform_cell.borrow();
                Ok(t.rotation.get(plane).copied().unwrap_or(0.0).to_degrees())
            })?,
        )?;

        globals.set(
            "set_rotation",
            scope.create_function(|_, (plane, degrees): (usize, f64)| {
                let mut t = transform_cell.borrow_mut();
                if let Some(slot) = t.rotation.get_mut(plane) {
                    *slot = degrees.to_radians();
                }
                Ok(())
            })?,
        )?;

        globals.set(
            "rotate",
            scope.create_function(|_, (plane, delta_degrees): (usize, f64)| {
                let mut t = transform_cell.borrow_mut();
                if let Some(slot) = t.rotation.get_mut(plane) {
                    *slot += delta_degrees.to_radians();
                }
                Ok(())
            })?,
        )?;

        lua.load(source).exec()
    });

    log_out.extend(log_cell.into_inner());
    *transform = transform_cell.into_inner();
    result.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_can_read_and_write_position() {
        let mut t = Transform4::default();
        let mut log = Vec::new();
        run_frame(&mut t, 0.1, "set_position(1, 2, 3, 4)", &mut log).unwrap();
        assert_eq!(t.position, Vec4::new(1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn script_can_rotate_using_dt() {
        let mut t = Transform4::default();
        let mut log = Vec::new();
        run_frame(&mut t, 2.0, "rotate(PLANE_ZW, 10 * dt)", &mut log).unwrap();
        assert!((t.rotation[5].to_degrees() - 20.0).abs() < 1e-9);
    }

    #[test]
    fn script_log_is_collected() {
        let mut t = Transform4::default();
        let mut log = Vec::new();
        run_frame(&mut t, 0.0, "log('hello')", &mut log).unwrap();
        assert_eq!(log, vec!["hello".to_string()]);
    }

    #[test]
    fn script_syntax_error_is_reported() {
        let mut t = Transform4::default();
        let mut log = Vec::new();
        let err = run_frame(&mut t, 0.0, "this is not lua (((", &mut log);
        assert!(err.is_err());
    }
}
