//! Querencia Terminal – Terminalemulator für Querencia Desktop.
//!
//! Ein Fenster mit Menüleiste und Reitern, je Reiter ein VTE-Terminal mit
//! einer Shell (`window.rs`, Reiter und Kürzel in `tabs.rs`, Menüleiste in
//! `menus.rs`). Die Terminalemulation übernimmt vollständig VTE.
//!
//! Kopieren und Einfügen (Strg+Umschalt+C/V, Umschalt+Einfg, Kontextmenü)
//! wie im GNOME Terminal von GNOME 2: `clipboard.rs`, `context_menu.rs`.
//!
//! Texte über GNU gettext, msgid englisch, Deutsch aus `po/de.po`
//! (`i18n.rs`).

mod clipboard;
mod context_menu;
mod dialogs;
mod i18n;
mod menus;
mod shell;
mod tabs;
mod window;

use gtk4::prelude::*;
use gtk4::{gio, glib};

/// Anwendungs-ID; zugleich Desktop-ID (`data/org.querencia.Terminal.desktop`)
/// und `StartupWMClass`, damit Fensterliste und Titelleiste den Starter finden.
const APP_ID: &str = "org.querencia.Terminal";

fn main() -> glib::ExitCode {
    i18n::init();
    // NON_UNIQUE: Jeder Start ist ein eigener Prozess mit eigener Umgebung.
    // Mehrere Fenster in einer Instanz kommen erst mit M3.0b; mehrere
    // Shells in einem Fenster gibt es als Reiter.
    let app = gtk4::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_startup(|_| install_style());
    app.connect_activate(window::build);
    app.run()
}

/// Stil der klassischen Querencia-Menüs für das Kontextmenü (Regeln nur
/// für `popover.querencia-menu`) und für Menüleiste, Reiterleiste und
/// Dialoge (nur `window.qt-window`, `window.qt-dialog`).
fn install_style() {
    let Some(display) = gtk4::gdk::Display::default() else {
        return;
    };
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(&format!("{}{}", context_menu::MENU_CSS, window::WINDOW_CSS));
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_id_is_valid() {
        assert!(gio::Application::id_is_valid(APP_ID));
    }

    /// Desktop-ID, Fensterklasse und App-ID müssen gleich sein; sonst findet
    /// die Fensterliste den Starter (Name, Symbol) nicht.
    #[test]
    fn desktop_file_matches_app_id() {
        const DESKTOP: &str = include_str!("../data/org.querencia.Terminal.desktop");
        assert_eq!(APP_ID, "org.querencia.Terminal");
        let wm_class = DESKTOP
            .lines()
            .find_map(|l| l.strip_prefix("StartupWMClass="));
        assert_eq!(wm_class, Some(APP_ID));
        assert!(DESKTOP.lines().any(|l| l == "Name=Querencia Terminal"));
        assert!(DESKTOP.lines().any(|l| l == "Exec=querencia-terminal"));
        assert!(!DESKTOP.contains("org.example"));
    }
}
