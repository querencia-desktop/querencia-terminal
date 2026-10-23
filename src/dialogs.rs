//! Dialoge als gewöhnliche, modale Fenster ohne eigene Titelleiste (wie im
//! Querencia Editor): labwc dekoriert sie wie jedes andere Fenster. Escape
//! und der Schließen-Knopf brechen ab; die Antwort kommt genau einmal.
//!
//! Jeder Dialog wird beim Öffnen neu gebaut und beim Schließen zerstört,
//! nie versteckt und wieder gezeigt (labwc 0.9.8 mit GTK 4.22 verliert dann
//! die Titelleiste, QD-076).

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};

use crate::i18n::{gettext, gettext_f};
use crate::tabs;

const APP_NAME: &str = "Querencia Terminal";

/// Webseite von Querencia Desktop (Info-Dialog).
pub const WEBSITE: &str = "https://querencialinux.org";

/// Meldestelle (Hilfe → Fehler melden): Auswahl der Vorlagen für neue Issues.
pub const REPORT_URL: &str = "https://github.com/querencia-desktop/querencia/issues/new/choose";

/// Online-Hilfe in der Sprache der Oberfläche. Die Adresse steht im
/// Katalog, damit jede Übersetzung ihre Sprachseite öffnet.
pub fn online_help_url() -> String {
    // TRANSLATORS: Adresse der Online-Hilfe (Hilfe → Online-Hilfe); nur den
    // Sprachteil ersetzen, deutsch https://querencialinux.org/de/docs/.
    gettext("https://querencialinux.org/en/docs/")
}

/// Öffnet `uri` im Standardprogramm (über das Portal, sonst über GIO). Ein
/// Fehler erscheint nur auf stderr; das Terminal bleibt bedienbar.
pub fn open_uri(parent: &impl IsA<gtk4::Window>, uri: &str) {
    crate::clipboard::debug(&format!("Adresse öffnen: {uri}"));
    let shown = uri.to_owned();
    gtk4::UriLauncher::new(uri).launch(
        Some(parent.as_ref()),
        gio::Cancellable::NONE,
        move |result| {
            if let Err(err) = result {
                eprintln!("querencia-terminal: {shown} nicht geöffnet: {err}");
            }
        },
    );
}

/// Was geschlossen werden soll, wenn noch ein Programm läuft.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Closing {
    /// Ein Reiter (oder das Fenster mit nur diesem Reiter).
    Terminal,
    /// Das Fenster mit mehreren Reitern.
    Window,
}

/// Schließt `window` im nächsten Leerlauf, nicht aus dem laufenden Klick-
/// oder Tasten-Handler heraus.
fn close_later(window: &gtk4::Window) {
    let window = window.clone();
    glib::idle_add_local_once(move || window.close());
}

fn modal_window(parent: &impl IsA<gtk4::Window>, title: &str) -> gtk4::Window {
    let window = gtk4::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .resizable(false)
        .destroy_with_parent(true)
        .build();
    window.add_css_class("qt-dialog");
    crate::clipboard::debug(&format!("Dialog geöffnet: {title}"));
    // Als Fenster der Anwendung tragen Dialoge die App-ID
    // org.querencia.Terminal; labwc zeigt das Symbol über die Desktop-Datei.
    if let Some(app) = parent.as_ref().application() {
        window.set_application(Some(&app));
    }
    window
}

fn close_on_escape(window: &gtk4::Window) {
    let keys = gtk4::EventControllerKey::new();
    keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
    let w = window.downgrade();
    keys.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            if let Some(w) = w.upgrade() {
                close_later(&w);
            }
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    window.add_controller(keys);
}

fn button_row(buttons: &[&gtk4::Button]) -> gtk4::Box {
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    row.set_halign(gtk4::Align::End);
    row.set_margin_top(12);
    for b in buttons {
        row.append(*b);
    }
    row
}

fn content(child: &impl IsA<gtk4::Widget>, buttons: &gtk4::Box) -> gtk4::Box {
    let outer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    outer.set_margin_top(12);
    outer.set_margin_bottom(12);
    outer.set_margin_start(12);
    outer.set_margin_end(12);
    outer.append(child);
    outer.append(buttons);
    outer
}

/// Frage im Stil von GNOME 2: Warnsymbol, fette Hauptzeile, Erklärung.
fn message_box(icon: &str, primary: &str, secondary: &str) -> gtk4::Box {
    let image = gtk4::Image::from_icon_name(icon);
    image.set_pixel_size(48);
    image.set_valign(gtk4::Align::Start);
    let text = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    let p = gtk4::Label::new(None);
    p.set_markup(&format!(
        "<b><big>{}</big></b>",
        glib::markup_escape_text(primary)
    ));
    p.set_wrap(true);
    p.set_max_width_chars(48);
    p.set_xalign(0.0);
    let s = gtk4::Label::new(Some(secondary));
    s.set_wrap(true);
    s.set_max_width_chars(48);
    s.set_xalign(0.0);
    text.append(&p);
    text.append(&s);
    let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    row.append(&image);
    row.append(&text);
    row
}

/// Texte der Nachfrage: Hauptzeile, Erklärung, Knopf (GNOME Terminal 2.x).
pub fn closing_texts(what: Closing) -> (String, String, String) {
    match what {
        Closing::Terminal => (
            // TRANSLATORS: Nachfrage beim Schließen eines Reiters oder des
            // Fensters mit nur einem Reiter.
            gettext("Close this terminal?"),
            gettext(
                "There is still a process running in this terminal. \
                 Closing the terminal will kill it.",
            ),
            // TRANSLATORS: Knopf der Nachfrage „Close this terminal?“.
            gettext("C_lose Terminal"),
        ),
        Closing::Window => (
            // TRANSLATORS: Nachfrage beim Schließen eines Fensters mit
            // mehreren Reitern.
            gettext("Close this window?"),
            gettext(
                "There are still processes running in some terminals in this \
                 window. Closing the window will kill all of them.",
            ),
            // TRANSLATORS: Knopf der Nachfrage „Close this window?“.
            gettext("C_lose Window"),
        ),
    }
}

/// Fragt nach, bevor ein Terminal mit laufendem Programm geschlossen wird.
/// `confirmed` läuft nur nach „Schließen“, im Leerlauf nach dem Abbau des
/// Dialogs. Vorgewählt ist „Abbrechen“ wie im GNOME Terminal 2.x.
pub fn confirm_close(
    parent: &impl IsA<gtk4::Window>,
    what: Closing,
    confirmed: impl FnOnce() + 'static,
) {
    crate::clipboard::debug(&format!("Nachfrage beim Schließen ({what:?})"));
    let window = modal_window(parent, APP_NAME);
    let (primary, secondary, button) = closing_texts(what);
    let cancel = gtk4::Button::with_mnemonic(&gettext("_Cancel"));
    let close = gtk4::Button::with_mnemonic(&button);
    close.add_css_class("destructive-action");
    let body = message_box("dialog-warning", &primary, &secondary);
    window.set_child(Some(&content(&body, &button_row(&[&cancel, &close]))));
    window.set_default_widget(Some(&cancel));
    close_on_escape(&window);

    type Pending = Rc<RefCell<Option<Box<dyn FnOnce()>>>>;
    let pending: Pending = Rc::new(RefCell::new(Some(Box::new(confirmed))));
    let w = window.downgrade();
    let p = pending.clone();
    close.connect_clicked(move |_| {
        let Some(f) = p.borrow_mut().take() else {
            return;
        };
        if let Some(w) = w.upgrade() {
            close_later(&w);
        }
        glib::idle_add_local_once(f);
    });
    let w = window.downgrade();
    cancel.connect_clicked(move |_| {
        if let Some(w) = w.upgrade() {
            close_later(&w);
        }
    });
    // Schließen über labwc oder Escape = Abbrechen.
    window.connect_close_request(move |_| {
        pending.borrow_mut().take();
        glib::Propagation::Proceed
    });
    window.present();
    cancel.grab_focus();
}

/// Hilfe → Tastenkürzel: alle Kürzel des Terminals in zwei Spalten.
pub fn shortcuts(parent: &impl IsA<gtk4::Window>) {
    let window = modal_window(parent, &gettext("Keyboard Shortcuts"));
    let grid = gtk4::Grid::builder()
        .row_spacing(4)
        .column_spacing(24)
        .build();
    for (row, (keys, what)) in tabs::shortcut_overview().iter().enumerate() {
        let keys = gtk4::Label::builder().label(keys).xalign(0.0).build();
        keys.add_css_class("qt-keys");
        let what = gtk4::Label::builder().label(what).xalign(0.0).build();
        grid.attach(&keys, 0, row as i32, 1, 1);
        grid.attach(&what, 1, row as i32, 1, 1);
    }
    let close = gtk4::Button::with_mnemonic(&gettext("_Close"));
    window.set_child(Some(&content(&grid, &button_row(&[&close]))));
    window.set_default_widget(Some(&close));
    close_on_escape(&window);
    let w = window.downgrade();
    close.connect_clicked(move |_| {
        if let Some(w) = w.upgrade() {
            close_later(&w);
        }
    });
    window.present();
    close.grab_focus();
}

/// Hilfe → Info: Name, Version, Stand (Alpha), Vorbild, Webseite, Lizenz
/// und Urheber, wie in den anderen Querencia-Programmen.
pub fn about(parent: &impl IsA<gtk4::Window>) {
    let window = modal_window(
        parent,
        // TRANSLATORS: Titel des Info-Fensters, {name} = „Querencia Terminal“.
        &gettext_f("About {name}", &[("name", APP_NAME)]),
    );
    let body = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    body.set_halign(gtk4::Align::Center);
    let icon = gio::ThemedIcon::from_names(&["org.querencia.Terminal", "utilities-terminal"]);
    let image = gtk4::Image::from_gicon(&icon);
    image.set_pixel_size(64);
    body.append(&image);
    let name = gtk4::Label::new(None);
    name.set_markup(&format!(
        "<b><big>{APP_NAME} {}</big></b>",
        env!("CARGO_PKG_VERSION")
    ));
    // TRANSLATORS: Info-Fenster, Entwicklungsstand unter Name und Version.
    let stage = gtk4::Label::new(Some(&gettext("Alpha – not released for everyday use")));
    let text = gtk4::Label::new(Some(&gettext(
        "A terminal emulator for Querencia Desktop,\n\
         modelled on GNOME Terminal 2.x.",
    )));
    text.set_justify(gtk4::Justification::Center);
    let website = gtk4::Label::new(None);
    website.set_markup(&format!(
        "<a href=\"{WEBSITE}\">{}</a>",
        glib::markup_escape_text(WEBSITE)
    ));
    let w = window.downgrade();
    website.connect_activate_link(move |_, uri| {
        if let Some(w) = w.upgrade() {
            open_uri(&w, uri);
        }
        glib::Propagation::Stop
    });
    let license = gtk4::Label::new(Some(&gettext("License: GNU GPL, version 3 or later")));
    license.add_css_class("dim-label");
    // TRANSLATORS: Info-Fenster, Urheberzeile; „contributors“ bleibt
    // englisch wie im Lizenzkopf der Quellen.
    let copyright = gtk4::Label::new(Some(&gettext("© 2026 Querencia Desktop contributors")));
    copyright.add_css_class("dim-label");
    body.append(&name);
    body.append(&stage);
    body.append(&text);
    body.append(&website);
    body.append(&license);
    body.append(&copyright);
    let close = gtk4::Button::with_mnemonic(&gettext("_Close"));
    window.set_child(Some(&content(&body, &button_row(&[&close]))));
    window.set_default_widget(Some(&close));
    close_on_escape(&window);
    let w = window.downgrade();
    close.connect_clicked(move |_| {
        if let Some(w) = w.upgrade() {
            close_later(&w);
        }
    });
    window.present();
    close.grab_focus();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_texts_like_gnome_terminal() {
        let (p, s, b) = closing_texts(Closing::Terminal);
        assert_eq!(p, "Close this terminal?");
        assert!(s.starts_with("There is still a process running"), "{s}");
        assert_eq!(b, "C_lose Terminal");
        let (p, s, b) = closing_texts(Closing::Window);
        assert_eq!(p, "Close this window?");
        assert!(s.contains("kill all of them"), "{s}");
        assert_eq!(b, "C_lose Window");
    }

    #[test]
    fn addresses_of_querencia() {
        assert_eq!(WEBSITE, "https://querencialinux.org");
        assert_eq!(
            REPORT_URL,
            "https://github.com/querencia-desktop/querencia/issues/new/choose"
        );
        // Ohne Katalog (Tests) gilt die englische Seite.
        assert_eq!(online_help_url(), "https://querencialinux.org/en/docs/");
        let de = include_str!("../po/de.po");
        assert!(de.contains(
            "msgid \"https://querencialinux.org/en/docs/\"\nmsgstr \"https://querencialinux.org/de/docs/\""
        ));
        assert!(de.contains(
            "msgid \"Alpha – not released for everyday use\"\nmsgstr \"Alpha – nicht für den Alltag freigegeben\""
        ));
    }
}
