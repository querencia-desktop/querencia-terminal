//! Menüleiste wie GNOME Terminal 2.x, auf das Nötige gekürzt:
//!
//!   Datei | Bearbeiten | Ansicht | Reiter | Hilfe
//!
//! Die Einträge lösen Fensteraktionen (`win.*`, `window.rs`) aus. Die Kürzel
//! stehen nur als Anzeige im Menü (Attribut `accel`); ausgewertet werden sie
//! vor VTE von `tabs::shortcut` und `clipboard::shortcut`, damit eine
//! gesperrte Aktion die Taste an die Shell weitergibt.
//!
//! F10 und Alt+Buchstabe öffnen die Menüleiste nicht, sie bleiben bei der
//! Shell (Midnight Commander, readline): F10 schaltet `window.rs` ab, und
//! die Menüköpfe stehen ohne Zugriffstaste in der Leiste (GTK würde Alt+B
//! sonst vor VTE „Bearbeiten“ öffnen lassen). Die Übersetzungen behalten
//! den Unterstrich, damit eine spätere Einstellung „Zugriffstasten der
//! Menüs“ (GNOME Terminal 2.x) nur das Abschneiden abschalten muss. In den
//! geöffneten Menüs gelten die Zugriffstasten wie gewohnt.

use gtk4::gio;
use gtk4::prelude::*;

use crate::context_menu;
use crate::i18n::gettext;

/// Ein Menüeintrag: Beschriftung mit Zugriffstaste, Aktion, angezeigtes
/// Kürzel (GTK-Schreibweise).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub label: String,
    pub action: &'static str,
    pub accel: Option<&'static str>,
}

fn entry(label: String, action: &'static str, accel: Option<&'static str>) -> Entry {
    Entry {
        label,
        action,
        accel,
    }
}

/// Ein Menü der Leiste: Überschrift und Abschnitte (durch Trennlinien
/// getrennt).
#[derive(Debug, Clone)]
pub struct Menu {
    pub title: String,
    pub sections: Vec<Vec<Entry>>,
}

/// Die Menüs in ihrer Reihenfolge.
pub fn menus() -> Vec<Menu> {
    vec![
        Menu {
            // TRANSLATORS: Menüleiste. Die Zugriffstasten der fünf Menüs
            // müssen verschieden sein.
            title: gettext("_File"),
            sections: vec![
                vec![entry(
                    context_menu::Command::NewTab.label(),
                    "win.new-tab",
                    Some("<Control><Shift>t"),
                )],
                vec![
                    entry(
                        context_menu::Command::CloseTab.label(),
                        "win.close-tab",
                        Some("<Control><Shift>w"),
                    ),
                    // TRANSLATORS: Menü „Datei“.
                    entry(
                        gettext("Close _Window"),
                        "win.close-window",
                        Some("<Control><Shift>q"),
                    ),
                ],
            ],
        },
        Menu {
            // TRANSLATORS: Menüleiste.
            title: gettext("_Edit"),
            sections: vec![
                vec![
                    entry(
                        context_menu::Command::Copy.label(),
                        "win.copy",
                        Some("<Control><Shift>c"),
                    ),
                    entry(
                        context_menu::Command::Paste.label(),
                        "win.paste",
                        Some("<Control><Shift>v"),
                    ),
                ],
                vec![entry(
                    context_menu::Command::SelectAll.label(),
                    "win.select-all",
                    None,
                )],
            ],
        },
        Menu {
            // TRANSLATORS: Menüleiste.
            title: gettext("_View"),
            sections: vec![vec![entry(
                // TRANSLATORS: Menü „Ansicht“, Häkchen: Reiterleiste auch mit
                // nur einem Reiter zeigen.
                gettext("Always Show _Tab Bar"),
                "win.tab-bar-always",
                None,
            )]],
        },
        Menu {
            // TRANSLATORS: Menüleiste.
            title: gettext("_Tabs"),
            sections: vec![
                vec![
                    // TRANSLATORS: Menü „Reiter“.
                    entry(
                        gettext("_Previous Tab"),
                        "win.previous-tab",
                        Some("<Control>Page_Up"),
                    ),
                    // TRANSLATORS: Menü „Reiter“.
                    entry(
                        gettext("_Next Tab"),
                        "win.next-tab",
                        Some("<Control>Page_Down"),
                    ),
                ],
                vec![
                    // TRANSLATORS: Menü „Reiter“.
                    entry(
                        gettext("Move Tab _Left"),
                        "win.move-tab-left",
                        Some("<Control><Shift>Page_Up"),
                    ),
                    // TRANSLATORS: Menü „Reiter“.
                    entry(
                        gettext("Move Tab _Right"),
                        "win.move-tab-right",
                        Some("<Control><Shift>Page_Down"),
                    ),
                ],
            ],
        },
        Menu {
            // TRANSLATORS: Menüleiste.
            title: gettext("_Help"),
            sections: vec![vec![
                // TRANSLATORS: Menü „Hilfe“, öffnet die Kürzelübersicht.
                entry(gettext("_Keyboard Shortcuts"), "win.shortcuts", None),
                // TRANSLATORS: Menü „Hilfe“.
                entry(gettext("_About"), "win.about", None),
            ]],
        },
    ]
}

/// Menükopf ohne Zugriffstaste („_Datei“ → „Datei“); ein doppelter
/// Unterstrich bleibt ein Unterstrich.
pub fn bar_title(title: &str) -> String {
    context_menu::split_mnemonic(title).0.replace('_', "__")
}

/// Menümodell für `PopoverMenuBar`.
pub fn model() -> gio::Menu {
    let bar = gio::Menu::new();
    for menu in menus() {
        let submenu = gio::Menu::new();
        for section in &menu.sections {
            let part = gio::Menu::new();
            for e in section {
                let item = gio::MenuItem::new(Some(&e.label), Some(e.action));
                if let Some(accel) = e.accel {
                    item.set_attribute_value("accel", Some(&accel.to_variant()));
                }
                part.append_item(&item);
            }
            submenu.append_section(None, &part);
        }
        bar.append_submenu(Some(&bar_title(&menu.title)), &submenu);
    }
    bar
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_menu::split_mnemonic;
    use crate::tabs;

    /// msgstr zu `msgid` aus `po/de.po` (nur einzeilige Einträge, wie sie
    /// die Menüs haben).
    fn german(msgid: &str) -> String {
        let de = include_str!("../po/de.po");
        let key = format!("msgid \"{msgid}\"\nmsgstr \"");
        let start = de
            .find(&key)
            .unwrap_or_else(|| panic!("fehlt in de.po: {msgid}"))
            + key.len();
        let end = de[start..].find("\"\n").unwrap() + start;
        de[start..end].to_owned()
    }

    fn assert_unique(labels: &[String], what: &str) {
        let mut keys: Vec<char> = Vec::new();
        for label in labels {
            let key = split_mnemonic(label)
                .1
                .unwrap_or_else(|| panic!("{what}: ohne Zugriffstaste: {label}"));
            assert!(!keys.contains(&key), "{what}: {key} doppelt in {labels:?}");
            keys.push(key);
        }
    }

    #[test]
    fn bar_titles_have_no_mnemonic() {
        assert_eq!(bar_title("_File"), "File");
        assert_eq!(bar_title("_Bearbeiten"), "Bearbeiten");
        assert_eq!(bar_title("a__b"), "a__b");
        for menu in menus() {
            assert!(!bar_title(&menu.title).contains('_'), "{}", menu.title);
        }
    }

    #[test]
    fn menus_in_gnome_terminal_order() {
        let titles: Vec<String> = menus().into_iter().map(|m| m.title).collect();
        assert_eq!(titles, ["_File", "_Edit", "_View", "_Tabs", "_Help"]);
    }

    /// Je Menü und Sprache eindeutige Zugriffstasten (englisch = msgid,
    /// deutsch aus de.po), auch für die Menüleiste und das Kontextmenü.
    #[test]
    fn mnemonics_are_unique_per_menu_and_language() {
        let menus = menus();
        let titles: Vec<String> = menus.iter().map(|m| m.title.clone()).collect();
        assert_unique(&titles, "Menüleiste en");
        assert_unique(
            &titles.iter().map(|t| german(t)).collect::<Vec<_>>(),
            "Menüleiste de",
        );
        for menu in &menus {
            let labels: Vec<String> = menu
                .sections
                .iter()
                .flatten()
                .map(|e| e.label.clone())
                .collect();
            assert_unique(&labels, &menu.title);
            assert_unique(
                &labels.iter().map(|l| german(l)).collect::<Vec<_>>(),
                &german(&menu.title),
            );
        }
        let context: Vec<String> = context_menu::ITEMS
            .iter()
            .filter_map(|item| match item {
                context_menu::Item::Entry { command } => Some(command.label()),
                context_menu::Item::Separator => None,
            })
            .collect();
        assert_unique(&context, "Kontextmenü en");
        assert_unique(
            &context.iter().map(|l| german(l)).collect::<Vec<_>>(),
            "Kontextmenü de",
        );
    }

    /// Zerlegt „<Control><Shift>Page_Up“ ohne GTK (`accelerator_parse`
    /// verlangt ein initialisiertes GTK).
    fn parse(accel: &str) -> (gtk4::gdk::Key, gtk4::gdk::ModifierType) {
        use gtk4::gdk::ModifierType as M;
        let mut mods = M::empty();
        let mut rest = accel;
        while let Some(after) = rest.strip_prefix('<') {
            let (name, tail) = after.split_once('>').unwrap();
            mods |= match name {
                "Control" => M::CONTROL_MASK,
                "Shift" => M::SHIFT_MASK,
                "Alt" => M::ALT_MASK,
                other => panic!("unbekannt: {other}"),
            };
            rest = tail;
        }
        let key = gtk4::gdk::Key::from_name(rest).unwrap();
        (key, mods)
    }

    /// Das angezeigte Kürzel ist genau das, das `tabs::shortcut` auswertet.
    #[test]
    fn shown_accels_match_the_handled_shortcuts() {
        use tabs::Command as C;
        let expected = [
            ("win.new-tab", C::NewTab),
            ("win.close-tab", C::CloseTab),
            ("win.close-window", C::CloseWindow),
            ("win.previous-tab", C::PreviousTab),
            ("win.next-tab", C::NextTab),
            ("win.move-tab-left", C::MoveTabLeft),
            ("win.move-tab-right", C::MoveTabRight),
        ];
        let entries: Vec<Entry> = menus()
            .into_iter()
            .flat_map(|m| m.sections.into_iter().flatten())
            .collect();
        for (action, command) in expected {
            let e = entries.iter().find(|e| e.action == action).unwrap();
            let (key, mods) = parse(e.accel.unwrap());
            // GDK liefert mit Umschalt den Großbuchstaben.
            let key = if mods.contains(gtk4::gdk::ModifierType::SHIFT_MASK) {
                key.to_upper()
            } else {
                key
            };
            assert_eq!(tabs::shortcut(key, mods), Some(command), "{action}");
        }
        for (action, shortcut) in [
            ("win.copy", crate::clipboard::Shortcut::Copy),
            ("win.paste", crate::clipboard::Shortcut::Paste),
        ] {
            let e = entries.iter().find(|e| e.action == action).unwrap();
            let (key, mods) = parse(e.accel.unwrap());
            assert_eq!(
                crate::clipboard::shortcut(key.to_upper(), mods),
                Some(shortcut),
                "{action}"
            );
        }
    }
}
