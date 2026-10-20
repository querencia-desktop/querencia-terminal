//! Kopieren und Einfügen wie im GNOME Terminal von GNOME 2.
//!
//! Die Zwischenablage selbst bedient VTE über GDK, unter Wayland also das
//! Datenaustauschprotokoll des Compositors (`wl_data_device`, CLIPBOARD)
//! und die primäre Auswahl (`primary_selection`, PRIMARY):
//!
//! - Kopieren: `Terminal::copy_clipboard_format` legt die markierte Stelle
//!   als Text in CLIPBOARD; Zeilenumbrüche und UTF-8 bleiben erhalten.
//! - Einfügen: `Terminal::paste_clipboard` liest CLIPBOARD asynchron. VTE
//!   bereitet den Text auf (Zeilenende als CR, Steuerzeichen entfernt) und
//!   klammert ihn mit `ESC[200~ … ESC[201~` ein, wenn das Programm im
//!   Terminal das verlangt (Bracketed Paste, DECSET 2004, z. B. bash ab
//!   5.1). Dann führt die Shell eingefügte Zeilen erst mit Enter aus.
//! - Markieren mit der Maus legt den Text wie bisher nur in PRIMARY, die
//!   mittlere Maustaste fügt PRIMARY ein (VTE). CLIPBOARD ändert sich
//!   dabei nicht.
//!
//! Die Kürzel fängt ein Tastatur-Controller in der Capture-Phase ab, vor
//! VTE: Strg+Umschalt+C/V und Umschalt+Einfg gehen nie als Zeichen an die
//! Shell, auch wenn nichts markiert oder die Zwischenablage leer ist.
//! Strg+C, Strg+V und alle anderen Tasten bleiben unverändert bei VTE.
//!
//! Inhalte der Zwischenablage werden nie protokolliert.

use gtk4::prelude::*;
use gtk4::{gdk, glib};
use vte4::prelude::*;

/// Was ein Tastenkürzel auslöst.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shortcut {
    /// Strg+Umschalt+C: Markierung in die Zwischenablage.
    Copy,
    /// Strg+Umschalt+V und Umschalt+Einfg: Zwischenablage einfügen.
    Paste,
}

/// Modifikatoren, die für die Kürzel zählen. Feststell- und Num-Taste
/// (Lock, Mod2) und die Maustasten bleiben außen vor.
fn relevant(state: gdk::ModifierType) -> gdk::ModifierType {
    state
        & (gdk::ModifierType::SHIFT_MASK
            | gdk::ModifierType::CONTROL_MASK
            | gdk::ModifierType::ALT_MASK
            | gdk::ModifierType::SUPER_MASK
            | gdk::ModifierType::HYPER_MASK
            | gdk::ModifierType::META_MASK)
}

/// Ordnet einen Tastendruck einem Kürzel zu. `keyval` ist der Wert, den
/// GDK mit allen Modifikatoren übersetzt hat (mit Umschalt also `C`); groß
/// und klein zählen gleich, damit auch die Feststelltaste nicht stört. Die
/// Tasten C und V liegen im deutschen QWERTZ-Layout wie im US-Layout.
pub fn shortcut(keyval: gdk::Key, state: gdk::ModifierType) -> Option<Shortcut> {
    let ctrl_shift = gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::SHIFT_MASK;
    let mods = relevant(state);
    if mods == ctrl_shift {
        match keyval.to_lower() {
            gdk::Key::c => return Some(Shortcut::Copy),
            gdk::Key::v => return Some(Shortcut::Paste),
            _ => {}
        }
    }
    if mods == gdk::ModifierType::SHIFT_MASK
        && matches!(keyval, gdk::Key::Insert | gdk::Key::KP_Insert)
    {
        return Some(Shortcut::Paste);
    }
    None
}

/// Öffnet diese Taste das Kontextmenü (VTE: Menü-Taste, Umschalt+F10)?
pub fn opens_context_menu(keyval: gdk::Key, state: gdk::ModifierType) -> bool {
    let mods = relevant(state);
    (keyval == gdk::Key::Menu && mods.is_empty())
        || (keyval == gdk::Key::F10 && mods == gdk::ModifierType::SHIFT_MASK)
}

/// Bietet die Zwischenablage Text an? Wie GNOME Terminal prüft das nur die
/// angebotenen Formate, nicht den Inhalt: GDK liefert einen Text (`String`)
/// aus `text/plain` mit oder ohne Zeichensatz und aus den X11-Namen, die
/// XWayland-Programme anbieten.
pub fn offers_text(formats: &gdk::ContentFormats) -> bool {
    formats.contains_type(glib::Type::STRING)
        || formats.mime_types().iter().any(|mime| is_text_mime(mime))
}

fn is_text_mime(mime: &str) -> bool {
    let mime = mime.to_ascii_lowercase();
    mime == "text/plain"
        || mime.starts_with("text/plain;")
        || matches!(
            mime.as_str(),
            "utf8_string" | "string" | "text" | "compound_text"
        )
}

/// Text in der Zwischenablage des Terminals vorhanden?
pub fn clipboard_has_text(terminal: &vte4::Terminal) -> bool {
    offers_text(&terminal.clipboard().formats())
}

/// Kopiert die Markierung nach CLIPBOARD. Ohne Markierung passiert nichts:
/// VTE würde sonst eine leere Zeichenkette anbieten und die Zwischenablage
/// leeren.
pub fn copy(terminal: &vte4::Terminal) -> bool {
    if !terminal.has_selection() {
        debug("Kopieren: nichts markiert, Zwischenablage unverändert");
        return false;
    }
    terminal.copy_clipboard_format(vte4::Format::Text);
    debug("Kopieren: Markierung in die Zwischenablage gelegt");
    true
}

/// Fügt CLIPBOARD ein (asynchron, über VTE mit Bracketed Paste).
pub fn paste(terminal: &vte4::Terminal) {
    debug("Einfügen: Zwischenablage angefordert");
    terminal.paste_clipboard();
}

/// Markiert den ganzen Inhalt (mit Verlauf), wie „Alles auswählen“.
pub fn select_all(terminal: &vte4::Terminal) {
    terminal.select_all();
    debug("Alles auswählen");
}

/// Hängt die Kürzel an das Terminal. `on_menu_key` erfährt, dass das
/// Kontextmenü gleich über die Tastatur geöffnet wird (VTE öffnet es
/// selbst; die Taste geht weiter an VTE).
pub fn install_shortcuts(terminal: &vte4::Terminal, on_menu_key: impl Fn() + 'static) {
    let keys = gtk4::EventControllerKey::new();
    keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
    keys.connect_key_pressed(glib::clone!(
        #[weak]
        terminal,
        #[upgrade_or]
        glib::Propagation::Proceed,
        move |_, keyval, _, state| {
            if opens_context_menu(keyval, state) {
                on_menu_key();
                return glib::Propagation::Proceed;
            }
            match shortcut(keyval, state) {
                Some(Shortcut::Copy) => {
                    debug("Strg+Umschalt+C");
                    copy(&terminal);
                    glib::Propagation::Stop
                }
                Some(Shortcut::Paste) => {
                    debug("Einfügen per Tastatur");
                    paste(&terminal);
                    glib::Propagation::Stop
                }
                None => glib::Propagation::Proceed,
            }
        }
    ));
    terminal.add_controller(keys);
}

/// `QUERENCIA_TERMINAL_DEBUG=1`: Ereignisse mit Uhrzeit auf stderr (für die
/// isolierten Prüfläufe), nie Inhalte.
pub fn debugging() -> bool {
    std::env::var_os("QUERENCIA_TERMINAL_DEBUG").is_some_and(|v| v == "1")
}

pub fn debug(message: &str) {
    if debugging() {
        let t = glib::DateTime::now_local()
            .ok()
            .and_then(|d| d.format("%T.%f").ok())
            .unwrap_or_default();
        eprintln!("querencia-terminal: {t} {message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdk::Key;
    use gdk::ModifierType as M;

    const CS: M = M::CONTROL_MASK.union(M::SHIFT_MASK);

    #[test]
    fn ctrl_shift_c_copies_and_ctrl_shift_v_pastes() {
        // Mit Umschalt liefert GDK den Großbuchstaben.
        assert_eq!(shortcut(Key::C, CS), Some(Shortcut::Copy));
        assert_eq!(shortcut(Key::V, CS), Some(Shortcut::Paste));
        // Feststelltaste an: Kleinbuchstabe, Lock-Bit gesetzt.
        assert_eq!(shortcut(Key::c, CS | M::LOCK_MASK), Some(Shortcut::Copy));
        assert_eq!(shortcut(Key::v, CS | M::LOCK_MASK), Some(Shortcut::Paste));
        // Num-Taste (Mod2) stört nicht.
        assert_eq!(
            shortcut(Key::C, CS | M::from_bits_truncate(1 << 4)),
            Some(Shortcut::Copy)
        );
    }

    #[test]
    fn shift_insert_pastes() {
        assert_eq!(shortcut(Key::Insert, M::SHIFT_MASK), Some(Shortcut::Paste));
        assert_eq!(
            shortcut(Key::KP_Insert, M::SHIFT_MASK),
            Some(Shortcut::Paste)
        );
    }

    #[test]
    fn terminal_control_keys_stay_with_vte() {
        // Strg+C (SIGINT), Strg+V (literal next) und Co. gehen an die Shell.
        for key in [Key::c, Key::v, Key::d, Key::z, Key::backslash] {
            assert_eq!(shortcut(key, M::CONTROL_MASK), None, "{key:?}");
        }
        // Ohne Modifikatoren oder nur mit Umschalt: normale Zeichen.
        assert_eq!(shortcut(Key::c, M::empty()), None);
        assert_eq!(shortcut(Key::C, M::SHIFT_MASK), None);
        assert_eq!(shortcut(Key::V, M::SHIFT_MASK), None);
        // Einfg ohne Umschalt, Strg+Einfg und Strg+Umschalt+Einfg behält
        // VTE (Kopieren bzw. Einfügen wie im klassischen VTE).
        assert_eq!(shortcut(Key::Insert, M::empty()), None);
        assert_eq!(shortcut(Key::Insert, M::CONTROL_MASK), None);
        assert_eq!(shortcut(Key::Insert, CS), None);
        // Weitere Modifikatoren machen das Kürzel ungültig.
        assert_eq!(shortcut(Key::C, CS | M::ALT_MASK), None);
        assert_eq!(shortcut(Key::V, CS | M::SUPER_MASK), None);
        assert_eq!(shortcut(Key::Insert, M::SHIFT_MASK | M::ALT_MASK), None);
        // Andere Buchstaben mit Strg+Umschalt bleiben bei VTE.
        assert_eq!(shortcut(Key::X, CS), None);
        assert_eq!(shortcut(Key::A, CS), None);
    }

    #[test]
    fn context_menu_keys() {
        assert!(opens_context_menu(Key::Menu, M::empty()));
        assert!(opens_context_menu(Key::F10, M::SHIFT_MASK));
        assert!(!opens_context_menu(Key::F10, M::empty()));
        assert!(!opens_context_menu(Key::Menu, M::CONTROL_MASK));
    }

    #[test]
    fn text_formats_count_as_text() {
        for mime in [
            "text/plain;charset=utf-8",
            "text/plain;charset=UTF-8",
            "text/plain",
            "UTF8_STRING",
            "TEXT",
        ] {
            let formats = gdk::ContentFormats::new(&[mime]);
            assert!(offers_text(&formats), "{mime}");
        }
        assert!(offers_text(&gdk::ContentFormats::for_type(
            glib::Type::STRING
        )));
    }

    #[test]
    fn empty_or_image_clipboard_is_no_text() {
        assert!(!offers_text(&gdk::ContentFormats::new(&[])));
        assert!(!offers_text(&gdk::ContentFormats::new(&["image/png"])));
        assert!(!offers_text(&gdk::ContentFormats::new(&[
            "text/html",
            "text/uri-list"
        ])));
        assert!(!offers_text(&gdk::ContentFormats::new(&["text/plainx"])));
    }
}
