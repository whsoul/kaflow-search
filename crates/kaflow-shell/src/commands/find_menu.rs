//! 화면 내 찾기 — 우클릭 네이티브 컨텍스트 메뉴.
//!
//! FE(FindBar)가 우클릭 시 클릭 지점의 scope 정보(has_scope/to_multi)와 **표시 라벨 전체**를
//! 넘겨 호출한다 (i18n — 라벨 문자열은 FE Lingui 카탈로그가 단일 출처, 2026-07-15 계약 변경).
//! 메뉴는 Tauri 네이티브 팝업(시스템 메뉴 모양)으로 뜨고, 항목 선택은 Builder 의
//! `on_menu_event`(lib.rs)에서 `find_menu` 이벤트로 FE 에 전달된다. **항목 id 는 불변 계약**:
//!   - "scope" / "all"        : 찾기 범위(영역/전체)
//!   - "to_multi" / "to_multi_drill" : 상세검색으로 전환(검색조건 / 검색+drill조건)
//! (디버그 빌드에서는 devtools 항목도 포함.)

use serde::Deserialize;
use tauri::menu::{ContextMenu, Menu, MenuItem, PredefinedMenuItem};

/// FE 가 현재 로케일로 번역해 전달하는 메뉴 라벨 묶음.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindMenuLabels {
    /// "이 영역에서 찾기 (영역명)" — scope 라벨 포함 완성 문자열.
    pub scope: String,
    /// "화면에서 찾기"
    pub all: String,
    /// "상세검색으로 전환 (검색 조건)"
    pub to_multi: String,
    /// "상세검색으로 전환 (검색+drill 조건)"
    pub to_multi_drill: String,
    /// "검사 (개발자 도구)" — debug 빌드에서만 사용.
    pub devtools: String,
}

#[tauri::command]
pub fn show_find_context_menu(
    app: tauri::AppHandle,
    window: tauri::Window,
    has_scope: bool,
    // "" = 없음 / "results" = 결과영역 검색조건 / "drill" = drill 검색+drill조건
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

    // 상세검색으로 전환 — 결과 영역(검색 후)에서만.
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
