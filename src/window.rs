//! Fensteraufbau: `ApplicationWindow` mit VTE-Terminal und Scrollleiste.
//!
//! Keine Headerbar: Die Titelleiste stellt der Compositor (labwc, SSD)
//! oder ersatzweise GTK mit seiner Standard-Titelleiste.

use gtk4::prelude::*;
use vte4::prelude::*;

use crate::{clipboard, context_menu, shell};

const TITLE: &str = "Querencia Terminal";

/// Anfangsgröße in Zeichenzellen, wie bei MATE Terminal.
const COLUMNS: i64 = 80;
const ROWS: i64 = 24;

/// Zeilen im Scrollback.
const SCROLLBACK_LINES: i64 = 10_000;

pub fn build(app: &gtk4::Application) {
    let terminal = vte4::Terminal::new();
    // Die natürliche Größe des Widgets folgt aus Spalten und Zeilen; das
    // Fenster übernimmt sie, weil keine Standardgröße gesetzt ist.
    terminal.set_size(COLUMNS, ROWS);
    terminal.set_scrollback_lines(SCROLLBACK_LINES);
    terminal.set_scroll_on_keystroke(true);
    terminal.set_scroll_on_output(false);
    terminal.set_hexpand(true);
    terminal.set_vexpand(true);

    let scrollbar =
        gtk4::Scrollbar::new(gtk4::Orientation::Vertical, terminal.vadjustment().as_ref());

    let content = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    content.append(&terminal);
    content.append(&scrollbar);

    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title(TITLE)
        .child(&content)
        .build();

    // Kopieren/Einfügen: Kürzel und Kontextmenü (siehe clipboard.rs).
    // Schwache Referenzen: Menü und Controller hängen am Terminal selbst.
    let terminal_weak = terminal.downgrade();
    let query_weak = terminal.downgrade();
    let menu = context_menu::ContextMenu::new(
        move || match query_weak.upgrade() {
            Some(terminal) => context_menu::Availability {
                has_selection: terminal.has_selection(),
                clipboard_has_text: clipboard::clipboard_has_text(&terminal),
            },
            None => context_menu::Availability {
                has_selection: false,
                clipboard_has_text: false,
            },
        },
        move |command| {
            let Some(terminal) = terminal_weak.upgrade() else {
                return;
            };
            match command {
                context_menu::Command::Copy => {
                    clipboard::copy(&terminal);
                }
                context_menu::Command::Paste => clipboard::paste(&terminal),
                context_menu::Command::SelectAll => clipboard::select_all(&terminal),
            }
        },
    );
    terminal.set_context_menu(Some(menu.popover()));
    let keyboard_menu = menu.clone();
    clipboard::install_shortcuts(&terminal, move || keyboard_menu.opening_by_keyboard());

    // Shell-Ende schließt das Fenster; mit dem letzten Fenster endet die
    // Anwendung. Schwache Referenz, damit Terminal und Fenster keinen
    // Referenzzyklus bilden.
    let window_weak = window.downgrade();
    terminal.connect_child_exited(move |_, status| {
        eprintln!(
            "querencia-terminal: Shell beendet ({})",
            shell::ChildExit::from_wait_status(status)
        );
        if let Some(window) = window_weak.upgrade() {
            window.close();
        }
    });

    // GtkWindow und GtkRoot bieten beide set_focus mit gleicher Wirkung.
    GtkWindowExt::set_focus(&window, Some(&terminal));
    window.present();

    shell::start(&terminal);
}
