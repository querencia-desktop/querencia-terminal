//! Reiter wie im GNOME Terminal 2.x: Befehle, Tastenkürzel, Reiterleiste,
//! Reitertitel, Arbeitsverzeichnis neuer Reiter und die Frage, ob in einem
//! Reiter noch ein Programm läuft.
//!
//! Hier steht nur, was sich ohne Fenster prüfen lässt; `window.rs` hängt es
//! an GtkNotebook und VTE.
//!
//! Kürzel (GNOME Terminal 2.x):
//!
//! - Strg+Umschalt+T neuer Reiter, Strg+Umschalt+W Reiter schließen,
//!   Strg+Umschalt+Q Fenster schließen;
//! - Strg+Bild auf/ab vorheriger/nächster Reiter, Strg+Umschalt+Bild auf/ab
//!   Reiter nach links/rechts verschieben, Alt+1 … Alt+9 Reiter 1 … 9.
//!
//! Wie bei den gesperrten Aktionen von GNOME Terminal 2.x greift ein Kürzel
//! nur, wenn sein Befehl gerade ausführbar ist; sonst geht die Taste an die
//! Shell. Am ersten und letzten Reiter geht es nicht im Kreis weiter, und
//! mit nur einem Reiter bekommt die Shell Strg+Bild auf/ab und Alt+Ziffer.

use std::path::{Path, PathBuf};

use gtk4::{gdk, glib};

use crate::i18n::gettext;

/// Was ein Kürzel oder ein Menüeintrag mit den Reitern tut.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    NewTab,
    CloseTab,
    CloseWindow,
    PreviousTab,
    NextTab,
    MoveTabLeft,
    MoveTabRight,
    /// Reiter mit dieser Stelle (ab 0) zeigen, Alt+1 = 0.
    GotoTab(usize),
}

/// Höchste Alt-Ziffer (Alt+9).
pub const MAX_DIRECT: usize = 9;

/// Modifikatoren, die für die Kürzel zählen (ohne Feststell- und Num-Taste).
fn relevant(state: gdk::ModifierType) -> gdk::ModifierType {
    state
        & (gdk::ModifierType::SHIFT_MASK
            | gdk::ModifierType::CONTROL_MASK
            | gdk::ModifierType::ALT_MASK
            | gdk::ModifierType::SUPER_MASK
            | gdk::ModifierType::HYPER_MASK
            | gdk::ModifierType::META_MASK)
}

/// Ordnet einen Tastendruck einem Reiterbefehl zu. `keyval` ist wie bei
/// `clipboard::shortcut` der von GDK mit allen Modifikatoren übersetzte
/// Wert; groß und klein zählen gleich. Bild auf/ab gibt es auch auf dem
/// Ziffernblock (ohne Num-Taste).
pub fn shortcut(keyval: gdk::Key, state: gdk::ModifierType) -> Option<Command> {
    let ctrl = gdk::ModifierType::CONTROL_MASK;
    let ctrl_shift = ctrl | gdk::ModifierType::SHIFT_MASK;
    let mods = relevant(state);
    let page_up = matches!(keyval, gdk::Key::Page_Up | gdk::Key::KP_Page_Up);
    let page_down = matches!(keyval, gdk::Key::Page_Down | gdk::Key::KP_Page_Down);
    if mods == ctrl_shift {
        if page_up {
            return Some(Command::MoveTabLeft);
        }
        if page_down {
            return Some(Command::MoveTabRight);
        }
        return match keyval.to_lower() {
            gdk::Key::t => Some(Command::NewTab),
            gdk::Key::w => Some(Command::CloseTab),
            gdk::Key::q => Some(Command::CloseWindow),
            _ => None,
        };
    }
    if mods == ctrl {
        if page_up {
            return Some(Command::PreviousTab);
        }
        if page_down {
            return Some(Command::NextTab);
        }
        return None;
    }
    if mods == gdk::ModifierType::ALT_MASK {
        let digit = keyval.to_unicode()?.to_digit(10)? as usize;
        if (1..=MAX_DIRECT).contains(&digit) {
            return Some(Command::GotoTab(digit - 1));
        }
    }
    None
}

/// Ist der Befehl bei `count` Reitern und dem Reiter `current` ausführbar?
pub fn enabled(command: Command, count: usize, current: usize) -> bool {
    let several = count > 1;
    match command {
        Command::NewTab | Command::CloseTab | Command::CloseWindow => true,
        Command::PreviousTab | Command::MoveTabLeft => several && current > 0,
        Command::NextTab | Command::MoveTabRight => several && current + 1 < count,
        Command::GotoTab(index) => several && index < count,
    }
}

/// Neue Stelle des aktuellen Reiters (Verschieben) bzw. Stelle des Reiters,
/// der gezeigt wird (Wechseln); `None`, wenn der Befehl nichts zu zeigen
/// oder zu verschieben hat oder gesperrt ist.
pub fn target(command: Command, count: usize, current: usize) -> Option<usize> {
    if !enabled(command, count, current) {
        return None;
    }
    match command {
        Command::PreviousTab | Command::MoveTabLeft => Some(current - 1),
        Command::NextTab | Command::MoveTabRight => Some(current + 1),
        Command::GotoTab(index) => Some(index),
        Command::NewTab | Command::CloseTab | Command::CloseWindow => None,
    }
}

/// Wann die Reiterleiste zu sehen ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabBar {
    /// Ab dem zweiten Reiter (GNOME Terminal 2.x).
    #[default]
    Automatic,
    /// Auch mit nur einem Reiter.
    Always,
}

impl TabBar {
    /// Wert in `terminal.ini` (`[Reiter] Leiste=`).
    pub fn key(self) -> &'static str {
        match self {
            TabBar::Automatic => "automatisch",
            TabBar::Always => "immer",
        }
    }

    /// Unbekannte oder fehlende Werte gelten als „automatisch“.
    pub fn from_key(value: Option<&str>) -> Self {
        match value.map(str::trim) {
            Some("immer") => TabBar::Always,
            _ => TabBar::Automatic,
        }
    }

    pub fn visible(self, count: usize) -> bool {
        match self {
            TabBar::Automatic => count > 1,
            TabBar::Always => true,
        }
    }
}

/// Einstellungsdatei: `$XDG_CONFIG_HOME/querencia/terminal.ini`.
pub fn settings_path() -> PathBuf {
    glib::user_config_dir().join("querencia/terminal.ini")
}

const SETTINGS_GROUP: &str = "Reiter";
const SETTINGS_KEY: &str = "Leiste";

/// Liest die Einstellung der Reiterleiste; ohne Datei „automatisch“.
pub fn load_tab_bar(path: &Path) -> TabBar {
    let file = glib::KeyFile::new();
    if file
        .load_from_file(path, glib::KeyFileFlags::KEEP_COMMENTS)
        .is_err()
    {
        return TabBar::Automatic;
    }
    TabBar::from_key(file.string(SETTINGS_GROUP, SETTINGS_KEY).ok().as_deref())
}

/// Schreibt die Einstellung; andere Gruppen und Kommentare bleiben stehen.
pub fn save_tab_bar(path: &Path, bar: TabBar) -> Result<(), String> {
    let file = glib::KeyFile::new();
    if file
        .load_from_file(path, glib::KeyFileFlags::KEEP_COMMENTS)
        .is_err()
    {
        let _ = file.set_comment(
            None,
            None,
            " Querencia Terminal – wird beim Ändern der Einstellung geschrieben",
        );
    }
    file.set_string(SETTINGS_GROUP, SETTINGS_KEY, bar.key());
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    file.save_to_file(path).map_err(|e| e.to_string())
}

/// Reitertitel aus dem Fenstertitel, den die Shell setzt (OSC 0/2); ohne
/// Titel „Terminal“ wie im GNOME Terminal 2.x.
pub fn tab_title(window_title: Option<&str>) -> String {
    match window_title.map(str::trim) {
        Some(title) if !title.is_empty() => title.to_owned(),
        // TRANSLATORS: Reitertitel, solange die Shell keinen Titel setzt.
        _ => gettext("Terminal"),
    }
}

/// Ordner aus der `current-directory-uri` von VTE (OSC 7, `file://host/pfad`).
/// Nur Ordner dieses Rechners zählen: ohne Host, `localhost` oder der eigene
/// Rechnername (`local_host`); ein Pfad über SSH auf einem anderen Rechner
/// nicht.
pub fn directory_from_uri(uri: &str, local_host: &str) -> Option<PathBuf> {
    let (path, host) = glib::filename_from_uri(uri).ok()?;
    let local = match host.as_deref() {
        None | Some("") | Some("localhost") => true,
        Some(host) => host.eq_ignore_ascii_case(local_host),
    };
    local.then_some(path)
}

/// Läuft im Terminal der Shell `shell_pid` gerade ein anderes Programm im
/// Vordergrund? `stat` ist der Inhalt von `/proc/<shell_pid>/stat`; Feld 8
/// (`tpgid`) ist die Vordergrund-Prozessgruppe des steuernden Terminals.
/// Wie GNOME Terminal 2.x: Vordergrundgruppe ≠ Shell. Hintergrundjobs (`&`)
/// zählen nicht. Unlesbares gilt als „nichts läuft“.
pub fn foreground_process(stat: &str, shell_pid: i32) -> bool {
    // Der Programmname in Klammern darf Leerzeichen und Klammern enthalten.
    let Some((_, rest)) = stat.rsplit_once(')') else {
        return false;
    };
    // Felder nach dem Namen: state ppid pgrp session tty_nr tpgid …
    match rest.split_whitespace().nth(5).map(str::parse::<i32>) {
        Some(Ok(tpgid)) => tpgid > 0 && tpgid != shell_pid,
        _ => false,
    }
}

/// Liest `/proc/<pid>/stat` für [`foreground_process`].
pub fn shell_has_foreground_process(shell_pid: i32) -> bool {
    std::fs::read_to_string(format!("/proc/{shell_pid}/stat"))
        .is_ok_and(|stat| foreground_process(&stat, shell_pid))
}

/// Arbeitsverzeichnis der Shell über `/proc` (wenn die Shell kein OSC 7
/// sendet; GNOME Terminal 2.x las es ebenso aus `/proc/<pid>/cwd`).
pub fn shell_directory(shell_pid: i32) -> Option<PathBuf> {
    std::fs::read_link(format!("/proc/{shell_pid}/cwd")).ok()
}

/// Ein Eintrag der Kürzelübersicht (Hilfe → Tastenkürzel): Tasten und
/// Wirkung, beide übersetzt.
pub fn shortcut_overview() -> Vec<(String, String)> {
    vec![
        // TRANSLATORS: Hilfe → Tastenkürzel, Tastennamen wie in der
        // Kürzelübersicht von Querencia (Strg, Umschalt, Bild auf …).
        (gettext("Ctrl+Shift+T"), gettext("New tab")),
        (gettext("Ctrl+Shift+W"), gettext("Close tab")),
        (gettext("Ctrl+Shift+Q"), gettext("Close window")),
        (gettext("Ctrl+Page Up"), gettext("Previous tab")),
        (gettext("Ctrl+Page Down"), gettext("Next tab")),
        (
            gettext("Ctrl+Shift+Page Up"),
            gettext("Move tab to the left"),
        ),
        (
            gettext("Ctrl+Shift+Page Down"),
            gettext("Move tab to the right"),
        ),
        (gettext("Alt+1 … Alt+9"), gettext("Switch to tab 1 … 9")),
        (gettext("Middle click on a tab"), gettext("Close tab")),
        (gettext("Double click on the tab bar"), gettext("New tab")),
        (gettext("Ctrl+Shift+C"), gettext("Copy")),
        (gettext("Ctrl+Shift+V / Shift+Insert"), gettext("Paste")),
        (gettext("Menu / Shift+F10"), gettext("Context menu")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use gdk::Key;
    use gdk::ModifierType as M;

    const CS: M = M::CONTROL_MASK.union(M::SHIFT_MASK);

    #[test]
    fn ctrl_shift_letters() {
        assert_eq!(shortcut(Key::T, CS), Some(Command::NewTab));
        assert_eq!(shortcut(Key::W, CS), Some(Command::CloseTab));
        assert_eq!(shortcut(Key::Q, CS), Some(Command::CloseWindow));
        // Feststelltaste und Num-Taste stören nicht.
        assert_eq!(shortcut(Key::t, CS | M::LOCK_MASK), Some(Command::NewTab));
        assert_eq!(
            shortcut(Key::W, CS | M::from_bits_truncate(1 << 4)),
            Some(Command::CloseTab)
        );
    }

    #[test]
    fn page_keys_switch_and_move() {
        assert_eq!(
            shortcut(Key::Page_Up, M::CONTROL_MASK),
            Some(Command::PreviousTab)
        );
        assert_eq!(
            shortcut(Key::Page_Down, M::CONTROL_MASK),
            Some(Command::NextTab)
        );
        assert_eq!(
            shortcut(Key::KP_Page_Down, M::CONTROL_MASK),
            Some(Command::NextTab)
        );
        assert_eq!(shortcut(Key::Page_Up, CS), Some(Command::MoveTabLeft));
        assert_eq!(shortcut(Key::Page_Down, CS), Some(Command::MoveTabRight));
    }

    #[test]
    fn alt_digits_go_to_tabs_one_to_nine() {
        assert_eq!(shortcut(Key::_1, M::ALT_MASK), Some(Command::GotoTab(0)));
        assert_eq!(shortcut(Key::_9, M::ALT_MASK), Some(Command::GotoTab(8)));
        assert_eq!(shortcut(Key::_0, M::ALT_MASK), None);
        // Alt+Umschalt+1 (QWERTZ: „!“) und Strg+1 bleiben bei der Shell.
        assert_eq!(shortcut(Key::exclam, M::ALT_MASK | M::SHIFT_MASK), None);
        assert_eq!(shortcut(Key::_1, M::CONTROL_MASK), None);
    }

    #[test]
    fn other_keys_stay_with_the_shell() {
        // Strg+T/W/Q (Shell-Steuerzeichen) und Bild auf/ab allein.
        for key in [Key::t, Key::w, Key::q] {
            assert_eq!(shortcut(key, M::CONTROL_MASK), None, "{key:?}");
        }
        assert_eq!(shortcut(Key::Page_Up, M::empty()), None);
        assert_eq!(shortcut(Key::Page_Down, M::SHIFT_MASK), None);
        // Kopieren/Einfügen gehört clipboard.rs.
        assert_eq!(shortcut(Key::C, CS), None);
        assert_eq!(shortcut(Key::V, CS), None);
        // Alt+Buchstabe (readline) und weitere Modifikatoren.
        assert_eq!(shortcut(Key::b, M::ALT_MASK), None);
        assert_eq!(shortcut(Key::T, CS | M::ALT_MASK), None);
        assert_eq!(shortcut(Key::Page_Up, M::CONTROL_MASK | M::ALT_MASK), None);
        assert_eq!(shortcut(Key::F10, M::empty()), None);
    }

    #[test]
    fn single_tab_leaves_switch_keys_to_the_shell() {
        for command in [
            Command::PreviousTab,
            Command::NextTab,
            Command::MoveTabLeft,
            Command::MoveTabRight,
            Command::GotoTab(0),
        ] {
            assert!(!enabled(command, 1, 0), "{command:?}");
            assert_eq!(target(command, 1, 0), None);
        }
        for command in [Command::NewTab, Command::CloseTab, Command::CloseWindow] {
            assert!(enabled(command, 1, 0), "{command:?}");
        }
    }

    #[test]
    fn no_wrap_at_the_ends() {
        // drei Reiter
        assert_eq!(target(Command::PreviousTab, 3, 0), None);
        assert_eq!(target(Command::NextTab, 3, 0), Some(1));
        assert_eq!(target(Command::NextTab, 3, 2), None);
        assert_eq!(target(Command::PreviousTab, 3, 2), Some(1));
        assert_eq!(target(Command::MoveTabLeft, 3, 0), None);
        assert_eq!(target(Command::MoveTabRight, 3, 1), Some(2));
        assert_eq!(target(Command::MoveTabRight, 3, 2), None);
        assert_eq!(target(Command::MoveTabLeft, 3, 2), Some(1));
    }

    #[test]
    fn goto_only_existing_tabs() {
        assert_eq!(target(Command::GotoTab(2), 3, 0), Some(2));
        assert_eq!(target(Command::GotoTab(0), 3, 2), Some(0));
        assert_eq!(target(Command::GotoTab(3), 3, 0), None);
        assert!(enabled(Command::GotoTab(1), 2, 1));
        assert_eq!(target(Command::NewTab, 3, 0), None);
    }

    #[test]
    fn tab_bar_from_the_second_tab_or_always() {
        assert!(!TabBar::Automatic.visible(1));
        assert!(TabBar::Automatic.visible(2));
        assert!(TabBar::Always.visible(1));
        assert_eq!(TabBar::default(), TabBar::Automatic);
        assert_eq!(TabBar::from_key(Some("immer")), TabBar::Always);
        assert_eq!(TabBar::from_key(Some(" immer ")), TabBar::Always);
        assert_eq!(TabBar::from_key(Some("automatisch")), TabBar::Automatic);
        assert_eq!(TabBar::from_key(Some("always")), TabBar::Automatic);
        assert_eq!(TabBar::from_key(None), TabBar::Automatic);
        for bar in [TabBar::Automatic, TabBar::Always] {
            assert_eq!(TabBar::from_key(Some(bar.key())), bar);
        }
    }

    #[test]
    fn tab_bar_setting_round_trip_keeps_other_entries() {
        let dir = std::env::temp_dir().join(format!("qt-tabs-{}", std::process::id()));
        let path = dir.join("querencia/terminal.ini");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(load_tab_bar(&path), TabBar::Automatic);
        save_tab_bar(&path, TabBar::Always).unwrap();
        assert_eq!(load_tab_bar(&path), TabBar::Always);
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.starts_with("# Querencia Terminal"), "{text}");
        assert!(text.contains("[Reiter]\nLeiste=immer\n"), "{text}");
        std::fs::write(&path, format!("{text}\n[Anderes]\nWert=1\n")).unwrap();
        save_tab_bar(&path, TabBar::Automatic).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("Leiste=automatisch"), "{text}");
        assert!(text.contains("[Anderes]\nWert=1"), "{text}");
        assert_eq!(load_tab_bar(&path), TabBar::Automatic);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn tab_title_follows_the_shell() {
        assert_eq!(tab_title(Some("anna@testhost: ~")), "anna@testhost: ~");
        assert_eq!(tab_title(Some("  vim  ")), "vim");
        // ohne Katalog die msgid
        assert_eq!(tab_title(Some("")), "Terminal");
        assert_eq!(tab_title(None), "Terminal");
    }

    #[test]
    fn directory_only_from_this_host() {
        assert_eq!(
            directory_from_uri("file:///home/x/Neuer%20Ordner", "dev"),
            Some(PathBuf::from("/home/x/Neuer Ordner"))
        );
        assert_eq!(
            directory_from_uri("file://dev/tmp", "dev"),
            Some(PathBuf::from("/tmp"))
        );
        assert_eq!(
            directory_from_uri("file://DEV/tmp", "dev"),
            Some(PathBuf::from("/tmp"))
        );
        assert_eq!(
            directory_from_uri("file://localhost/tmp", "dev"),
            Some(PathBuf::from("/tmp"))
        );
        assert_eq!(directory_from_uri("file://server/tmp", "dev"), None);
        assert_eq!(directory_from_uri("sftp://dev/tmp", "dev"), None);
        assert_eq!(directory_from_uri("kein uri", "dev"), None);
    }

    #[test]
    fn foreground_group_other_than_shell_means_running() {
        // pid (comm) state ppid pgrp session tty_nr tpgid …
        let idle = "4711 (bash) S 4700 4711 4711 34816 4711 4194304 0 0";
        assert!(!foreground_process(idle, 4711));
        let vim = "4711 (bash) S 4700 4711 4711 34816 4800 4194304 0 0";
        assert!(foreground_process(vim, 4711));
        // Name mit Leerzeichen und Klammern
        let odd = "4711 (a) b (c) S 4700 4711 4711 34816 4800 0";
        assert!(foreground_process(odd, 4711));
        // kein steuerndes Terminal (tpgid -1) und Unlesbares
        let none = "4711 (bash) S 4700 4711 4711 0 -1 0";
        assert!(!foreground_process(none, 4711));
        assert!(!foreground_process("", 4711));
        assert!(!foreground_process("4711 (bash) S 1 2", 4711));
    }

    #[test]
    fn own_process_is_readable() {
        // Der Testprozess hat (unter cargo) meist kein steuerndes Terminal;
        // die Funktion darf dann nicht „läuft“ melden, wenn tpgid fehlt.
        let pid = std::process::id() as i32;
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
        let tpgid: i32 = stat
            .rsplit_once(')')
            .unwrap()
            .1
            .split_whitespace()
            .nth(5)
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(shell_has_foreground_process(pid), tpgid > 0 && tpgid != pid);
        assert!(shell_directory(pid).is_some());
        assert!(!shell_has_foreground_process(i32::MAX));
    }

    #[test]
    fn overview_lists_every_tab_shortcut() {
        let overview = shortcut_overview();
        let keys: Vec<&str> = overview.iter().map(|(k, _)| k.as_str()).collect();
        for key in [
            "Ctrl+Shift+T",
            "Ctrl+Shift+W",
            "Ctrl+Page Up",
            "Ctrl+Page Down",
            "Ctrl+Shift+Page Up",
            "Ctrl+Shift+Page Down",
            "Alt+1 … Alt+9",
            "Ctrl+Shift+C",
        ] {
            assert!(keys.contains(&key), "{key}");
        }
    }
}
