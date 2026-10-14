//! Querencia Terminal – Terminalemulator für Querencia Desktop.
//!
//! M3.0a: ein Fenster mit einem VTE-Terminal und einer Shell. Die
//! Terminalemulation übernimmt vollständig VTE.

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
    app.connect_activate(window::build);
    app.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_id_is_valid() {
        assert!(gio::Application::id_is_valid(APP_ID));
    }
}
