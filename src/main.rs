//! Querencia Terminal – Terminalemulator für Querencia Desktop.
//!
//! M3.0a: ein Fenster mit einem VTE-Terminal und einer Shell. Die
//! Terminalemulation übernimmt vollständig VTE.
//!
//! Kopieren und Einfügen (Strg+Umschalt+C/V, Umschalt+Einfg, Kontextmenü)
//! wie im GNOME Terminal von GNOME 2: `clipboard.rs`, `context_menu.rs`.

mod clipboard;
mod context_menu;
mod shell;
mod window;

use gtk4::prelude::*;
use gtk4::{gio, glib};

/// Vorläufige Anwendungs-ID bis zur Festlegung des Namensraums.
const APP_ID: &str = "org.example.Querencia.Terminal";

fn main() -> glib::ExitCode {
    // NON_UNIQUE: Jeder Start ist ein eigener Prozess mit eigener Umgebung.
    // Mehrere Fenster in einer Instanz kommen erst mit M3.0b.
    let app = gtk4::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_startup(|_| install_style());
    app.connect_activate(window::build);
    app.run()
}

/// Stil der klassischen Querencia-Menüs für das Kontextmenü; die Regeln
/// gelten nur für `popover.querencia-menu`.
fn install_style() {
    let Some(display) = gtk4::gdk::Display::default() else {
        return;
    };
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(context_menu::MENU_CSS);
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
}
