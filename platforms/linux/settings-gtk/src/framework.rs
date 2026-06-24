//! Detect which input-method framework is active, so the Settings UI can hide
//! features a given engine doesn't support.
//!
//! Per-app auto-switch (the "Ứng dụng bỏ qua" page) only exists in the Fcitx5 shell
//! (`platforms/linux/fcitx5`). The IBus v1 engine intentionally omits it — IBus
//! can't reliably report the focused app, especially on Wayland (see
//! `platforms/linux/ibus/src/engine.cpp`). So the page is shown only when Fcitx5 is
//! the active IME; on IBus — or when we can't tell — it is hidden rather than shown
//! as a no-op that silently does nothing.

use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib;
use gtk::glib::prelude::*; // ToVariant / FromVariant

/// True when per-app auto-switch is available, i.e. Fcitx5 is the active IME.
pub fn per_app_supported() -> bool {
    is_fcitx5_active()
}

fn is_fcitx5_active() -> bool {
    // Authoritative: the Fcitx5 daemon owns this well-known name on the session bus
    // while it is running.
    if session_name_has_owner("org.fcitx.Fcitx5") {
        return true;
    }
    // Fallback for sessions where the bus query is unavailable: the IM-module env
    // vars a Fcitx5 session exports (set on X11; often unset on Wayland, which is why
    // the bus check comes first).
    ["GTK_IM_MODULE", "QT_IM_MODULE", "XMODIFIERS"]
        .iter()
        .any(|var| std::env::var(var).map(|v| v.contains("fcitx")).unwrap_or(false))
}

/// Ask the session bus daemon whether `name` currently has an owner.
fn session_name_has_owner(name: &str) -> bool {
    let Ok(conn) = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE) else {
        return false;
    };
    let params = glib::Variant::tuple_from_iter([name.to_variant()]);
    conn.call_sync(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "NameHasOwner",
        Some(&params),
        Some(glib::VariantTy::new("(b)").unwrap()),
        gio::DBusCallFlags::NONE,
        -1,
        gio::Cancellable::NONE,
    )
    .ok()
    .and_then(|reply| reply.child_value(0).get::<bool>())
    .unwrap_or(false)
}
