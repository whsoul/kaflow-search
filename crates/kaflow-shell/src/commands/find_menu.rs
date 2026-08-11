//! The right-click menu for find-on-page.
//!
//! ⚠️ **The labels are passed in, not written here.** Translated text has one home, and it
//! is not this side — a label built here would be the one string in the menu that never
//! changed language.
//!
//! **The item ids are a fixed contract** (`scope`, `all`, `to_multi`, `to_multi_drill`):
//! a selection is reported back by id, so renaming one silently breaks the response.

use serde::Deserialize;
use tauri::menu::{ContextMenu, Menu, MenuItem, PredefinedMenuItem};

/// The menu labels, already translated by the caller.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindMenuLabels {
    /// Search within the clicked area — a complete phrase, area name included.
    pub scope: String,
    /// Search the whole page.
    pub all: String,
    /// Move to advanced search, carrying the query.
    pub to_multi: String,
    /// Move to advanced search, carrying the query and the drill.
    pub to_multi_drill: String,
    /// Open developer tools. Debug builds only.
    pub devtools: String,
}

#[tauri::command]
pub fn show_find_context_menu(
    app: tauri::AppHandle,
    window: tauri::Window,
    has_scope: bool,
    // "" = none, "results" = the results area, "drill" = a drilled-into range
    to_multi: String,
    labels: FindMenuLabels,
) -> Result<(), String> {
    let find_scope = MenuItem::with_id(&app, "find_scope", &labels.scope, has_scope, None::<&str>)
        .map_err(|e| e.to_string())?;
    let find_all = MenuItem::with_id(&app, "find_all", &labels.all, true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let menu = Menu::new(&app).map_err(|e| e.to_string())?;
    menu.append(&find_scope).map_err(|e| e.to_string())?;
    menu.append(&find_all).map_err(|e| e.to_string())?;

    // Offered only where there are results to move across with.
    if to_multi == "results" || to_multi == "drill" {
        let (id, text) = if to_multi == "drill" {
            ("to_multi_drill", &labels.to_multi_drill)
        } else {
            ("to_multi", &labels.to_multi)
        };
        let sep = PredefinedMenuItem::separator(&app).map_err(|e| e.to_string())?;
        let to_multi_item =
            MenuItem::with_id(&app, id, text, true, None::<&str>).map_err(|e| e.to_string())?;
        menu.append(&sep).map_err(|e| e.to_string())?;
        menu.append(&to_multi_item).map_err(|e| e.to_string())?;
    }

    #[cfg(debug_assertions)]
    {
        let sep = PredefinedMenuItem::separator(&app).map_err(|e| e.to_string())?;
        let devtools =
            MenuItem::with_id(&app, "find_devtools", &labels.devtools, true, None::<&str>)
                .map_err(|e| e.to_string())?;
        menu.append(&sep).map_err(|e| e.to_string())?;
        menu.append(&devtools).map_err(|e| e.to_string())?;
    }

    menu.popup(window).map_err(|e| e.to_string())?;
    Ok(())
}
