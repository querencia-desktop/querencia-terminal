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
//! - Kopieren, Einfügen, Trennlinie, Alles auswählen;
//! - Kopieren nur mit Markierung, Einfügen nur, wenn die Zwischenablage
//!   Text anbietet, sonst ausgegraut (wie GNOME Terminal);
//! - Zeiger: die Zeile unter dem Zeiger ist orange hervorgehoben, ein
//!   Klick löst sie aus; Tastatur: Auf/Ab, Pos1/Ende, Enter/Leertaste,
//!   Escape, und die Zugriffstasten K, E, A wie in GTK-2-Menüs. Über die
//!   Tastatur geöffnet ist die erste wählbare Zeile hervorgehoben.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{gdk, glib};

/// Die Regeln der klassischen Menüs, Zeile für Zeile wie in
/// `querencia-panel/src/main.rs` (HMP-09), damit das Terminalmenü genauso
/// aussieht wie Panel- und Desktopmenüs.
pub const MENU_CSS: &str = "\
popover.querencia-menu { padding: 0; }
popover.querencia-menu > contents { padding: 2px 0; border: 1px solid #c7c2be; border-radius: 0; box-shadow: none; background-color: #f5f4f2; color: #3c3c3c; }
popover.querencia-menu .qmenu { min-width: 150px; }
popover.querencia-menu .qmenu-item { min-height: 20px; padding: 1px 10px 1px 4px; border: 1px solid transparent; outline: none; }
popover.querencia-menu .qmenu-item .qmenu-arrow { color: #686463; }
popover.querencia-menu .qmenu-item.qmenu-active { background-color: #f07746; background-image: linear-gradient(to bottom, #f8875d, #ea703d); border-color: #b36c4e; box-shadow: inset 0 1px #fd9975; color: #ffffff; }
popover.querencia-menu .qmenu-item.qmenu-active .qmenu-arrow { color: #ffffff; }
popover.querencia-menu .qmenu-item:disabled { color: @insensitive_fg_color; }
popover.querencia-menu separator { margin: 3px 1px; min-height: 1px; background-color: #dcd8d3; }
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
}

/// Eine Zeile des Menüs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Entry {
        label: &'static str,
        /// Zugriffstaste (GNOME 2: „_Kopieren“, „_Einfügen“, „Alles _auswählen“).
        mnemonic: char,
        command: Command,
    },
    Separator,
}

/// Die Zeilen in der Reihenfolge von GNOME Terminal 2.x.
pub const ITEMS: [Item; 4] = [
    Item::Entry {
        label: "Kopieren",
        mnemonic: 'k',
        command: Command::Copy,
    },
    Item::Entry {
        label: "Einfügen",
        mnemonic: 'e',
        command: Command::Paste,
    },
    Item::Separator,
    Item::Entry {
        label: "Alles auswählen",
        mnemonic: 'a',
        command: Command::SelectAll,
    },
];

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
        Command::SelectAll => true,
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

/// Zeile zur Zugriffstaste `ch`, wenn sie wählbar ist.
pub fn mnemonic_row(selectable: &[bool], ch: char) -> Option<usize> {
    let ch = ch.to_lowercase().next()?;
    ITEMS.iter().enumerate().find_map(|(i, item)| match item {
        Item::Entry { mnemonic, .. } if *mnemonic == ch && selectable[i] => Some(i),
        _ => None,
    })
}

type Run = Rc<dyn Fn(Command)>;
type Query = Rc<dyn Fn() -> Availability>;

struct Inner {
    popover: gtk4::Popover,
    rows: Vec<Option<gtk4::Box>>,
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
        for item in ITEMS {
            match item {
                Item::Entry { label, .. } => {
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
                            .label(label)
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
                }
            }
        }
        popover.set_child(Some(&list));

        let menu = ContextMenu(Rc::new(Inner {
            popover,
            rows,
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
                    .and_then(|ch| mnemonic_row(&selectable, ch))
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
        let Item::Entry { command, .. } = ITEMS[index] else {
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

    fn labels() -> Vec<&'static str> {
        ITEMS
            .iter()
            .map(|item| match item {
                Item::Entry { label, .. } => *label,
                Item::Separator => "---",
            })
            .collect()
    }

    #[test]
    fn items_in_gnome_terminal_order() {
        assert_eq!(labels(), ["Kopieren", "Einfügen", "---", "Alles auswählen"]);
    }

    #[test]
    fn copy_needs_selection_and_paste_needs_text() {
        assert_eq!(selectable(ALL), [true, true, false, true]);
        assert_eq!(selectable(NOTHING), [false, false, false, true]);
        let only_selection = Availability {
            has_selection: true,
            clipboard_has_text: false,
        };
        assert_eq!(selectable(only_selection), [true, false, false, true]);
        let only_text = Availability {
            has_selection: false,
            clipboard_has_text: true,
        };
        assert_eq!(selectable(only_text), [false, true, false, true]);
    }

    #[test]
    fn keyboard_skips_separator_and_disabled_rows_and_wraps() {
        let all = selectable(ALL);
        assert_eq!(step(&all, None, true), Some(0));
        assert_eq!(step(&all, Some(0), true), Some(1));
        assert_eq!(step(&all, Some(1), true), Some(3));
        assert_eq!(step(&all, Some(3), true), Some(0));
        assert_eq!(step(&all, Some(0), false), Some(3));
        assert_eq!(step(&all, None, false), Some(3));
        let nothing = selectable(NOTHING);
        assert_eq!(first(&nothing), Some(3));
        assert_eq!(step(&nothing, Some(3), true), Some(3));
        assert_eq!(last(&nothing), Some(3));
        assert_eq!(step(&[false, false], None, true), None);
        assert_eq!(step(&[], None, true), None);
    }

    #[test]
    fn mnemonics_only_for_enabled_rows() {
        let all = selectable(ALL);
        assert_eq!(mnemonic_row(&all, 'k'), Some(0));
        assert_eq!(mnemonic_row(&all, 'E'), Some(1));
        assert_eq!(mnemonic_row(&all, 'a'), Some(3));
        assert_eq!(mnemonic_row(&all, 'x'), None);
        let nothing = selectable(NOTHING);
        assert_eq!(mnemonic_row(&nothing, 'k'), None);
        assert_eq!(mnemonic_row(&nothing, 'e'), None);
        assert_eq!(mnemonic_row(&nothing, 'a'), Some(3));
    }

    #[test]
    fn menu_css_is_scoped_to_classic_menus() {
        for rule in MENU_CSS.lines().filter(|l| !l.is_empty()) {
            assert!(rule.starts_with("popover.querencia-menu"), "{rule}");
        }
        // Hervorhebung wie Ambiance (Ubuntu 10.04): orange, weiße Schrift.
        assert!(MENU_CSS.contains(".qmenu-item.qmenu-active { background-color: #f07746;"));
        assert!(MENU_CSS.contains("color: #ffffff; }"));
    }
}
