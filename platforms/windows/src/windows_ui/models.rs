//! Adapters from persisted Rust data to Slint models.

use slint::{ModelRc, SharedString, VecModel};

use crate::settings::{ExcludedApp, Hotkey, Shortcut};
use crate::{AppEntry, ShortcutEntry};

pub(super) fn caps(hotkey: Hotkey) -> ModelRc<SharedString> {
    let rows: Vec<SharedString> = hotkey.caps().iter().map(|c| (*c).into()).collect();
    ModelRc::new(VecModel::from(rows))
}

pub(super) fn apps(apps: &[ExcludedApp]) -> ModelRc<AppEntry> {
    let rows = apps
        .iter()
        .map(|app| AppEntry {
            id: app.id.clone().into(),
            name: app.name.clone().into(),
        })
        .collect::<Vec<_>>();
    ModelRc::new(VecModel::from(rows))
}

pub(super) fn shortcuts(shortcuts: &[Shortcut]) -> ModelRc<ShortcutEntry> {
    let rows = shortcuts
        .iter()
        .map(|shortcut| ShortcutEntry {
            trigger: shortcut.trigger.clone().into(),
            expansion: shortcut.expansion.clone().into(),
        })
        .collect::<Vec<_>>();
    ModelRc::new(VecModel::from(rows))
}
