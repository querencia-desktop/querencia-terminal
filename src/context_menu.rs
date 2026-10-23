//! Kontextmenü des Terminals im Stil von GNOME 2 (Ubuntu 10.04).
//!
//! VTE öffnet das Menü selbst (`Terminal::set_context_menu`) und verhält
//! sich dabei wie GNOME Terminal 2.x: Rechtsklick öffnet es, außer ein
//! Programm im Terminal wertet die Maus aus (dann bekommt das Programm den
//! Klick und Umschalt+Rechtsklick öffnet das Menü); dazu Menü-Taste und
//! Umschalt+F10. Die obere linke Ecke liegt am Mauszeiger.
//!
//! Das Menü selbst ist kein `PopoverMenu` (modern, ohne GNOME-2-Verhalten),
//! sondern eine einfache klassische Liste mit denselben CSS-Klassen und
//! Regeln wie die Menüs von Panel und Desktop (`querencia-menu`,
//! `qmenu-item`, `qmenu-active`; siehe [`MENU_CSS`]):
//!
//! - Kopieren, Einfügen, Trennlinie, Alles auswählen, Trennlinie, Neuer
//!   Reiter, Reiter schließen (die Reiterzeilen kamen mit 0.3.0 unten
//!   dazu, damit die bisherigen Zeilen an ihrem Platz bleiben);
//! - Kopieren nur mit Markierung, Einfügen nur, wenn die Zwischenablage
//!   Text anbietet, sonst ausgegraut (wie GNOME Terminal);
//! - Zeiger: die Zeile unter dem Zeiger ist orange hervorgehoben, ein
//!   Klick löst sie aus; Tastatur: Auf/Ab, Pos1/Ende, Enter/Leertaste,
//!   Escape, und die Zugriffstasten wie in GTK-2-Menüs (deutsch K, E, A,
//!   R, S; englisch C, P, A, T, L). Über die Tastatur geöffnet ist die erste wählbare
//!   Zeile hervorgehoben.
//!
//! Die Beschriftungen kommen aus gettext mit `_` vor der Zugriffstaste
//! („_Kopieren“); angezeigt wird der Text ohne Unterstrich wie bisher.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, glib};

use crate::i18n::gettext;

/// Die Regeln der klassischen Menüs, Zeile für Zeile wie in
/// `querencia-panel/src/main.rs` (HMP-09), damit das Terminalmenü genauso
/// aussieht wie Panel- und Desktopmenüs.
///
/// Fläche, Schrift und Rand als benannte Farben des Themas (`@menu_bg_color`,
/// `@menu_fg_color`, `@borders`; hell #f5f4f2, #3c3c3c, #c7c2be wie bisher),
/// damit das Menü dem Dunkelmodus folgt (QD-109). Pfeil und Trennlinie haben
/// keine eigene Themenfarbe; sie sind aus diesen gemischt (hell #686868 statt
/// #686463 und #dbd8d4 statt #dcd8d3, dunkel nahe den Werten des Panels).
pub const MENU_CSS: &str = "\
popover.querencia-menu { padding: 0; }
popover.querencia-menu > contents { padding: 2px 0; border: 1px solid @borders; border-radius: 0; box-shadow: none; background-color: @menu_bg_color; color: @menu_fg_color; }
popover.querencia-menu .qmenu { min-width: 150px; }
popover.querencia-menu .qmenu-item { min-height: 20px; padding: 1px 10px 1px 4px; border: 1px solid transparent; outline: none; }
popover.querencia-menu .qmenu-item .qmenu-arrow { color: mix(@menu_fg_color, @menu_bg_color, 0.24); }
popover.querencia-menu .qmenu-item.qmenu-active { background-color: #f07746; background-image: linear-gradient(to bottom, #f8875d, #ea703d); border-color: #b36c4e; box-shadow: inset 0 1px #fd9975; color: #ffffff; }
popover.querencia-menu .qmenu-item.qmenu-active .qmenu-arrow { color: #ffffff; }
popover.querencia-menu .qmenu-item:disabled { color: @insensitive_fg_color; }
popover.querencia-menu separator { margin: 3px 1px; min-height: 1px; background-color: mix(@menu_bg_color, @borders, 0.57); }
";

const MENU_CLASS: &str = "querencia-menu";
const ITEM_CLASS: &str = "qmenu-item";
const ACTIVE_CLASS: &str = "qmenu-active";
/// Breite der Symbolspalte der klassischen Menüs (ohne Symbol leer).
const ICON_COLUMN: i32 = 16;

/// Was eine Menüzeile auslöst.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Copy,
    Paste,
    SelectAll,
    NewTab,
    CloseTab,
}

impl Command {
    /// Beschriftung mit `_` vor der Zugriffstaste, übersetzt (GNOME 2:
    /// „_Kopieren“, „_Einfügen“, „Alles _auswählen“).
    pub fn label(self) -> String {
        match self {
            // TRANSLATORS: Kontextmenü des Terminals. Die Zugriffstasten (_)
            // der drei Zeilen müssen verschieden sein.
            Command::Copy => gettext("_Copy"),
            // TRANSLATORS: Kontextmenü des Terminals.
            Command::Paste => gettext("_Paste"),
            // TRANSLATORS: Kontextmenü des Terminals.
            Command::SelectAll => gettext("Select _All"),
            // TRANSLATORS: Kontextmenü und Menü „Datei“; die Zugriffstaste
            // darf im Kontextmenü nicht mit Kopieren/Einfügen/Alles
            // auswählen zusammenfallen.
            Command::NewTab => gettext("New _Tab"),
            // TRANSLATORS: Kontextmenü und Menü „Datei“ (wie „New _Tab“).
            Command::CloseTab => gettext("C_lose Tab"),
        }
    }
}

/// Eine Zeile des Menüs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Entry { command: Command },
    Separator,
}

/// Die Zeilen in der Reihenfolge von GNOME Terminal 2.x, die Reiterzeilen
/// unten.
pub const ITEMS: [Item; 7] = [
    Item::Entry {
        command: Command::Copy,
    },
    Item::Entry {
        command: Command::Paste,
    },
    Item::Separator,
    Item::Entry {
        command: Command::SelectAll,
    },
    Item::Separator,
    Item::Entry {
        command: Command::NewTab,
    },
    Item::Entry {
        command: Command::CloseTab,
    },
];

/// Teilt eine Beschriftung in den angezeigten Text und die Zugriffstaste
/// (klein): „Alles _auswählen“ → („Alles auswählen“, `a`). `__` steht für
/// einen Unterstrich.
pub fn split_mnemonic(label: &str) -> (String, Option<char>) {
    let mut text = String::with_capacity(label.len());
    let mut key = None;
    let mut chars = label.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '_' {
            text.push(c);
            continue;
        }
        match chars.next() {
            Some('_') => text.push('_'),
            Some(next) => {
                if key.is_none() {
                    key = next.to_lowercase().next();
                }
                text.push(next);
            }
            None => {}
        }
    }
    (text, key)
}

/// Zustand, aus dem sich die wählbaren Zeilen ergeben.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Availability {
    pub has_selection: bool,
    pub clipboard_has_text: bool,
}

/// Ist der Befehl jetzt ausführbar?
pub fn enabled(command: Command, available: Availability) -> bool {
    match command {
        Command::Copy => available.has_selection,
        Command::Paste => available.clipboard_has_text,
        Command::SelectAll | Command::NewTab | Command::CloseTab => true,
    }
}

/// Wählbarkeit jeder Zeile von [`ITEMS`].
pub fn selectable(available: Availability) -> Vec<bool> {
    ITEMS
        .iter()
        .map(|item| match item {
            Item::Entry { command, .. } => enabled(*command, available),
            Item::Separator => false,
        })
        .collect()
}

/// Tastaturschritt: nächste (`forward`) oder vorige wählbare Zeile, am Ende
/// wieder von vorn wie in GTK-2-Menüs. Ohne Hervorhebung beginnt Ab oben,
/// Auf unten.
pub fn step(selectable: &[bool], current: Option<usize>, forward: bool) -> Option<usize> {
    let n = selectable.len();
    if n == 0 || !selectable.iter().any(|s| *s) {
        return None;
    }
    let mut i = match (current, forward) {
        (Some(c), true) => (c + 1) % n,
        (Some(c), false) => (c + n - 1) % n,
        (None, true) => 0,
        (None, false) => n - 1,
    };
    for _ in 0..n {
        if selectable[i] {
            return Some(i);
        }
        i = if forward {
            (i + 1) % n
        } else {
            (i + n - 1) % n
        };
    }
    None
}

/// Erste bzw. letzte wählbare Zeile (Pos1/Ende, Öffnen per Tastatur).
pub fn first(selectable: &[bool]) -> Option<usize> {
    step(selectable, None, true)
}

pub fn last(selectable: &[bool]) -> Option<usize> {
    step(selectable, None, false)
}

/// Zeile zur Zugriffstaste `ch`, wenn sie wählbar ist; `mnemonics` hat je
/// Zeile von [`ITEMS`] die Taste aus [`split_mnemonic`].
pub fn mnemonic_row(mnemonics: &[Option<char>], selectable: &[bool], ch: char) -> Option<usize> {
    let ch = ch.to_lowercase().next()?;
    mnemonics
        .iter()
        .zip(selectable)
        .position(|(key, sel)| *key == Some(ch) && *sel)
}

type Run = Rc<dyn Fn(Command)>;
type Query = Rc<dyn Fn() -> Availability>;

struct Inner {
    popover: gtk4::Popover,
    rows: Vec<Option<gtk4::Box>>,
    mnemonics: Vec<Option<char>>,
    selectable: RefCell<Vec<bool>>,
    active: Cell<Option<usize>>,
    by_keyboard: Cell<bool>,
    run: Run,
}

/// Das Kontextmenü; `popover()` gehört an `Terminal::set_context_menu`.
#[derive(Clone)]
pub struct ContextMenu(Rc<Inner>);

impl ContextMenu {
    /// `query` liefert beim Öffnen den aktuellen Zustand, `run` führt einen
    /// gewählten Befehl aus (nach dem Schließen des Menüs).
    pub fn new(
        query: impl Fn() -> Availability + 'static,
        run: impl Fn(Command) + 'static,
    ) -> Self {
        let popover = gtk4::Popover::builder().has_arrow(false).build();
        // add_css_class, nie css_classes: GTKs eigene Klasse `background`
        // (Textfarbe) muss bleiben.
        popover.add_css_class(MENU_CLASS);

        let list = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        list.add_css_class("qmenu");
        let mut rows = Vec::new();
        let mut mnemonics = Vec::new();
        for item in ITEMS {
            match item {
                Item::Entry { command } => {
                    let (label, key) = split_mnemonic(&command.label());
                    mnemonics.push(key);
                    let row = gtk4::Box::builder()
                        .orientation(gtk4::Orientation::Horizontal)
                        .spacing(6)
                        .accessible_role(gtk4::AccessibleRole::MenuItem)
                        .build();
                    row.add_css_class(ITEM_CLASS);
                    // Leere Symbolspalte wie bei den übrigen Querencia-Menüs.
                    let icon = gtk4::Image::new();
                    icon.set_pixel_size(ICON_COLUMN);
                    row.append(&icon);
                    row.append(
                        &gtk4::Label::builder()
                            .label(label.as_str())
                            .xalign(0.0)
                            .hexpand(true)
                            .build(),
                    );
                    list.append(&row);
                    rows.push(Some(row));
                }
                Item::Separator => {
                    list.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
                    rows.push(None);
                    mnemonics.push(None);
                }
            }
        }
        popover.set_child(Some(&list));

        let menu = ContextMenu(Rc::new(Inner {
            popover,
            rows,
            mnemonics,
            selectable: RefCell::new(vec![false; ITEMS.len()]),
            active: Cell::new(None),
            by_keyboard: Cell::new(false),
            run: Rc::new(run),
        }));
        menu.connect(Rc::new(query));
        menu
    }

    pub fn popover(&self) -> &gtk4::Popover {
        &self.0.popover
    }

    /// Das nächste Öffnen kommt von der Tastatur (Menü-Taste, Umschalt+F10).
    pub fn opening_by_keyboard(&self) {
        self.0.by_keyboard.set(true);
    }

    /// Wählbarkeit der Zeilen (nach dem letzten Öffnen).
    pub fn selectable(&self) -> Vec<bool> {
        self.0.selectable.borrow().clone()
    }

    /// Hervorgehobene Zeile.
    pub fn active(&self) -> Option<usize> {
        self.0.active.get()
    }

    fn connect(&self, query: Query) {
        let inner = &self.0;
        // Bei jedem Öffnen den Zustand neu bestimmen (VTE ruft popup()).
        inner.popover.connect_show(glib::clone!(
            #[weak(rename_to = inner)]
            self.0,
            move |_| {
                let available = query();
                let selectable = selectable(available);
                for (row, sensitive) in inner.rows.iter().zip(&selectable) {
                    if let Some(row) = row {
                        row.set_sensitive(*sensitive);
                    }
                }
                let keyboard = inner.by_keyboard.replace(false);
                let start = if keyboard { first(&selectable) } else { None };
                inner.selectable.replace(selectable);
                ContextMenu(inner.clone()).highlight(start);
                if crate::clipboard::debugging() {
                    eprintln!(
                        "querencia-terminal: Kontextmenü offen ({}): Kopieren={}, Einfügen={}",
                        if keyboard { "Tastatur" } else { "Maus" },
                        on_off(available.has_selection),
                        on_off(available.clipboard_has_text)
                    );
                }
            }
        ));
        inner.popover.connect_closed(glib::clone!(
            #[weak(rename_to = inner)]
            self.0,
            move |_| ContextMenu(inner).highlight(None)
        ));

        for (index, row) in inner.rows.iter().enumerate() {
            let Some(row) = row else { continue };
            let motion = gtk4::EventControllerMotion::new();
            motion.connect_enter(glib::clone!(
                #[weak(rename_to = inner)]
                self.0,
                move |_, _, _| {
                    let menu = ContextMenu(inner);
                    if menu.selectable().get(index).copied().unwrap_or(false) {
                        menu.highlight(Some(index));
                    }
                }
            ));
            motion.connect_leave(glib::clone!(
                #[weak(rename_to = inner)]
                self.0,
                move |_| {
                    if inner.active.get() == Some(index) {
                        ContextMenu(inner).highlight(None);
                    }
                }
            ));
            row.add_controller(motion);

            // Jede Maustaste löst aus, beim Loslassen über der Zeile (GTK 2).
            let click = gtk4::GestureClick::new();
            click.set_button(0);
            click.connect_released(glib::clone!(
                #[weak(rename_to = inner)]
                self.0,
                #[weak]
                row,
                move |gesture, _, x, y| {
                    gesture.set_state(gtk4::EventSequenceState::Claimed);
                    if row.contains(x, y) {
                        ContextMenu(inner).activate(index);
                    }
                }
            ));
            row.add_controller(click);
        }

        let keys = gtk4::EventControllerKey::new();
        keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
        keys.connect_key_pressed(glib::clone!(
            #[weak(rename_to = inner)]
            self.0,
            #[upgrade_or]
            glib::Propagation::Proceed,
            move |_, keyval, _, _| ContextMenu(inner).on_key(keyval)
        ));
        inner.popover.add_controller(keys);
    }

    /// Tastatur im offenen Menü. Escape bleibt bei GTK (schließt).
    pub fn on_key(&self, keyval: gdk::Key) -> glib::Propagation {
        let selectable = self.selectable();
        let current = self.active();
        let target = match keyval {
            gdk::Key::Down | gdk::Key::KP_Down | gdk::Key::Tab => {
                Some(step(&selectable, current, true))
            }
            gdk::Key::Up | gdk::Key::KP_Up | gdk::Key::ISO_Left_Tab => {
                Some(step(&selectable, current, false))
            }
            gdk::Key::Home | gdk::Key::KP_Home => Some(first(&selectable)),
            gdk::Key::End | gdk::Key::KP_End => Some(last(&selectable)),
            gdk::Key::Return | gdk::Key::KP_Enter | gdk::Key::space | gdk::Key::KP_Space => {
                if let Some(index) = current {
                    self.activate(index);
                }
                return glib::Propagation::Stop;
            }
            _ => {
                if let Some(index) = keyval
                    .to_unicode()
                    .and_then(|ch| mnemonic_row(&self.0.mnemonics, &selectable, ch))
                {
                    self.activate(index);
                    return glib::Propagation::Stop;
                }
                None
            }
        };
        match target {
            Some(index) => {
                self.highlight(index);
                glib::Propagation::Stop
            }
            None => glib::Propagation::Proceed,
        }
    }

    fn highlight(&self, index: Option<usize>) {
        self.0.active.set(index);
        for (i, row) in self.0.rows.iter().enumerate() {
            if let Some(row) = row {
                if Some(i) == index {
                    row.add_css_class(ACTIVE_CLASS);
                } else {
                    row.remove_css_class(ACTIVE_CLASS);
                }
            }
        }
    }

    /// Schließt das Menü und führt die Zeile aus, wenn sie wählbar ist.
    pub fn activate(&self, index: usize) {
        if !self.selectable().get(index).copied().unwrap_or(false) {
            return;
        }
        let Item::Entry { command } = ITEMS[index] else {
            return;
        };
        self.0.popover.popdown();
        (self.0.run)(command);
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "an" } else { "aus" }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: Availability = Availability {
        has_selection: true,
        clipboard_has_text: true,
    };
    const NOTHING: Availability = Availability {
        has_selection: false,
        clipboard_has_text: false,
    };

    /// Beschriftungen ohne Katalog, also die englischen msgids.
    fn labels() -> Vec<String> {
        ITEMS
            .iter()
            .map(|item| match item {
                Item::Entry { command } => command.label(),
                Item::Separator => "---".to_string(),
            })
            .collect()
    }

    fn mnemonics(labels: &[&str]) -> Vec<Option<char>> {
        labels.iter().map(|l| split_mnemonic(l).1).collect()
    }

    #[test]
    fn items_in_gnome_terminal_order() {
        assert_eq!(
            labels(),
            [
                "_Copy",
                "_Paste",
                "---",
                "Select _All",
                "---",
                "New _Tab",
                "C_lose Tab"
            ]
        );
    }

    #[test]
    fn mnemonic_is_split_from_the_label() {
        assert_eq!(split_mnemonic("_Kopieren"), ("Kopieren".into(), Some('k')));
        assert_eq!(
            split_mnemonic("Alles _auswählen"),
            ("Alles auswählen".into(), Some('a'))
        );
        assert_eq!(
            split_mnemonic("Select _All"),
            ("Select All".into(), Some('a'))
        );
        assert_eq!(split_mnemonic("_Über"), ("Über".into(), Some('ü')));
        assert_eq!(split_mnemonic("a__b"), ("a_b".into(), None));
        assert_eq!(split_mnemonic("ohne"), ("ohne".into(), None));
        assert_eq!(split_mnemonic("Ende_"), ("Ende".into(), None));
    }

    #[test]
    fn copy_needs_selection_and_paste_needs_text() {
        assert_eq!(
            selectable(ALL),
            [true, true, false, true, false, true, true]
        );
        assert_eq!(
            selectable(NOTHING),
            [false, false, false, true, false, true, true]
        );
        let only_selection = Availability {
            has_selection: true,
            clipboard_has_text: false,
        };
        assert_eq!(
            selectable(only_selection),
            [true, false, false, true, false, true, true]
        );
        let only_text = Availability {
            has_selection: false,
            clipboard_has_text: true,
        };
        assert_eq!(
            selectable(only_text),
            [false, true, false, true, false, true, true]
        );
    }

    #[test]
    fn keyboard_skips_separator_and_disabled_rows_and_wraps() {
        let all = selectable(ALL);
        assert_eq!(step(&all, None, true), Some(0));
        assert_eq!(step(&all, Some(0), true), Some(1));
        assert_eq!(step(&all, Some(1), true), Some(3));
        assert_eq!(step(&all, Some(3), true), Some(5));
        assert_eq!(step(&all, Some(6), true), Some(0));
        assert_eq!(step(&all, Some(0), false), Some(6));
        assert_eq!(step(&all, None, false), Some(6));
        let nothing = selectable(NOTHING);
        assert_eq!(first(&nothing), Some(3));
        assert_eq!(step(&nothing, Some(3), true), Some(5));
        assert_eq!(step(&nothing, Some(6), true), Some(3));
        assert_eq!(last(&nothing), Some(6));
        assert_eq!(step(&[false, false], None, true), None);
        assert_eq!(step(&[], None, true), None);
    }

    #[test]
    fn mnemonics_only_for_enabled_rows() {
        // deutsch wie bis 0.1.0
        let de = mnemonics(&[
            "_Kopieren",
            "_Einfügen",
            "---",
            "Alles _auswählen",
            "---",
            "Neuer _Reiter",
            "Reiter _schließen",
        ]);
        let all = selectable(ALL);
        assert_eq!(mnemonic_row(&de, &all, 'k'), Some(0));
        assert_eq!(mnemonic_row(&de, &all, 'E'), Some(1));
        assert_eq!(mnemonic_row(&de, &all, 'a'), Some(3));
        assert_eq!(mnemonic_row(&de, &all, 'r'), Some(5));
        assert_eq!(mnemonic_row(&de, &all, 's'), Some(6));
        assert_eq!(mnemonic_row(&de, &all, 'x'), None);
        let nothing = selectable(NOTHING);
        assert_eq!(mnemonic_row(&de, &nothing, 'k'), None);
        assert_eq!(mnemonic_row(&de, &nothing, 'e'), None);
        assert_eq!(mnemonic_row(&de, &nothing, 'a'), Some(3));
        // englisch (msgid)
        let en: Vec<String> = labels();
        let en = mnemonics(&en.iter().map(String::as_str).collect::<Vec<_>>());
        assert_eq!(
            en,
            [
                Some('c'),
                Some('p'),
                None,
                Some('a'),
                None,
                Some('t'),
                Some('l')
            ]
        );
        assert_eq!(mnemonic_row(&en, &all, 'P'), Some(1));
        assert_eq!(mnemonic_row(&en, &all, 'e'), None);
    }

    #[test]
    fn menu_css_is_scoped_to_classic_menus() {
        for rule in MENU_CSS.lines().filter(|l| !l.is_empty()) {
            assert!(rule.starts_with("popover.querencia-menu"), "{rule}");
        }
        // Hervorhebung wie Ambiance (Ubuntu 10.04): orange, weiße Schrift.
        assert!(MENU_CSS.contains(".qmenu-item.qmenu-active { background-color: #f07746;"));
        assert!(MENU_CSS.contains("color: #ffffff; }"));
        // Fläche und Schrift folgen dem Thema (QD-109).
        for fixed in ["#f5f4f2", "#3c3c3c", "#c7c2be", "#686463", "#dcd8d3"] {
            assert!(!MENU_CSS.contains(fixed), "fester Hellwert {fixed}");
        }
        assert!(MENU_CSS.contains("background-color: @menu_bg_color; color: @menu_fg_color;"));
    }
}
