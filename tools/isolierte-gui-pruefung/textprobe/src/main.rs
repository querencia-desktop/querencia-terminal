//! textprobe <ausgabe> [<vorgabe>]
//!
//! Ein Fenster mit einem GTK-4-Textfeld. Strg+V/Strg+C sind die normalen
//! GTK-Kürzel (Zwischenablage über GDK/Wayland). Jede Änderung des Textes
//! wird nach <ausgabe> geschrieben (über eine temporäre Datei), damit der
//! Test den eingefügten Text byte-genau vergleichen kann. Mit <vorgabe>
//! startet das Feld mit diesem Text, ganz markiert (zum Kopieren).
use gtk4::prelude::*;
use gtk4::{gio, glib};

fn main() -> glib::ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let out = args.get(1).cloned().expect("Ausgabedatei");
    let preset = args.get(2).map(|p| std::fs::read_to_string(p).expect("Vorgabe"));
    let app = gtk4::Application::builder()
        .application_id("org.example.Querencia.Textprobe")
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();
    app.connect_activate(move |app| {
        let view = gtk4::TextView::builder().monospace(true).build();
        let buffer = view.buffer();
        if let Some(text) = &preset {
            buffer.set_text(text);
            let (start, end) = buffer.bounds();
            buffer.select_range(&start, &end);
        }
        let out = out.clone();
        buffer.connect_changed(move |buffer| {
            let (start, end) = buffer.bounds();
            let text = buffer.text(&start, &end, true);
            let tmp = format!("{out}.tmp");
            std::fs::write(&tmp, text.as_bytes()).expect("schreiben");
            std::fs::rename(&tmp, &out).expect("umbenennen");
        });
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("Textprobe")
            .default_width(560)
            .default_height(260)
            .child(&view)
            .build();
        window.present();
        view.grab_focus();
    });
    app.run_with_args::<&str>(&[])
}
