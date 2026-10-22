//! Fensteraufbau: Menüleiste, Reiter (GtkNotebook) mit je einem
//! VTE-Terminal und Scrollleiste.
//!
//! Keine Headerbar: Die Titelleiste stellt der Compositor (labwc, SSD)
//! oder ersatzweise GTK mit seiner Standard-Titelleiste. Der Fenstertitel
//! bleibt „Querencia Terminal“; die Reiter tragen den Titel, den die Shell
//! setzt.
//!
//! Reiter wie im GNOME Terminal 2.x (Befehle und Kürzel in `tabs.rs`):
//! die Leiste erscheint ab dem zweiten Reiter oder immer (Ansicht →
//! „Reiterleiste immer anzeigen“, `terminal.ini`); Reiter lassen sich mit
//! der Maus verschieben, ein Mittelklick schließt einen Reiter, ein
//! Doppelklick auf die freie Leiste öffnet einen neuen. Ein neuer Reiter
//! beginnt im Arbeitsverzeichnis des aktuellen. Läuft in einem Reiter noch
//! ein Programm, fragt das Schließen von Reiter oder Fenster nach.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::{Rc, Weak};

use gtk4::prelude::*;
use gtk4::{gio, glib};
use vte4::prelude::*;

use crate::dialogs::{self, Closing};
use crate::tabs::{self, TabBar};
use crate::{clipboard, context_menu, menus, shell};

const TITLE: &str = "Querencia Terminal";

/// Anfangsgröße in Zeichenzellen, wie bei MATE Terminal.
const COLUMNS: i64 = 80;
const ROWS: i64 = 24;

/// Zeilen im Scrollback.
const SCROLLBACK_LINES: i64 = 10_000;

/// Breite der Reiterbeschriftung in Zeichen (die Leiste bleibt rechts frei
/// für den Doppelklick).
const TAB_LABEL_CHARS: i32 = 20;

/// Stil von Menüleiste, Menüs und Reiterleiste: Ambiance-Farbwerte wie im
/// Querencia Editor, nur für das Terminalfenster und seine Dialoge.
///
/// Flächen, Schrift und Ränder als benannte Farben des Themas
/// (`@theme_bg_color`, `@theme_fg_color`, `@theme_base_color`,
/// `@menu_bg_color`, `@borders`): unter Querencia-Klassisch hell genau die
/// bisherigen Werte (#f2f1f0, #3c3c3c, #ffffff, #f5f4f2, #c7c2be), im dunklen
/// Thema dessen Werte, ohne eigenen `@media`-Block und ohne Bindung an das
/// Farbschema (QD-109). Nur die orange Hervorhebung ist fest; sie gilt hell
/// wie dunkel.
pub const WINDOW_CSS: &str = "\
window.qt-window menubar { background-color: @theme_bg_color; border-bottom: 1px solid @borders; padding: 1px 2px; }
window.qt-window menubar > item { padding: 3px 8px; border-radius: 0; color: @theme_fg_color; }
window.qt-window menubar > item:hover, window.qt-window menubar > item:selected { background-image: linear-gradient(to bottom, #f6a179, #f07746); color: #1a1a1a; }
window.qt-window popover.menu contents { border-radius: 0; padding: 2px 0; background-color: @menu_bg_color; border: 1px solid @borders; }
window.qt-window popover.menu modelbutton { border-radius: 0; padding: 3px 16px; }
window.qt-window popover.menu modelbutton:hover, window.qt-window popover.menu modelbutton:selected { background-image: linear-gradient(to bottom, #f6a179, #f07746); color: #1a1a1a; }
window.qt-window notebook > header { background-color: @theme_bg_color; border-bottom: 1px solid @borders; }
window.qt-window notebook > header tab { padding: 2px 8px; border-radius: 0; }
window.qt-window notebook > header tab:checked { background-color: @theme_base_color; box-shadow: inset 0 -2px #f07746; }
window.qt-window button.qt-tab-close { padding: 0; min-width: 16px; min-height: 16px; }
window.qt-dialog button { border-radius: 2px; min-width: 80px; }
window.qt-dialog label.qt-keys { font-weight: bold; }
";

/// Ein Reiter: Terminal mit Scrollleiste, Beschriftung, Shell-PID.
struct Tab {
    terminal: vte4::Terminal,
    /// Seite im Notebook (Terminal und Scrollleiste).
    page: gtk4::Box,
    label: gtk4::Label,
    shell_pid: Cell<Option<i32>>,
}

impl Tab {
    /// Läuft außer der Shell ein Programm im Vordergrund?
    fn running(&self) -> bool {
        self.shell_pid
            .get()
            .is_some_and(tabs::shell_has_foreground_process)
    }

    /// Arbeitsverzeichnis für einen neuen Reiter: OSC 7 der Shell, sonst
    /// `/proc/<shell>/cwd`.
    fn directory(&self) -> Option<PathBuf> {
        let uri = self.terminal.current_directory_uri();
        uri.and_then(|uri| tabs::directory_from_uri(&uri, &glib::host_name()))
            .or_else(|| self.shell_pid.get().and_then(tabs::shell_directory))
    }

    fn update_title(&self) {
        let title = tabs::tab_title(self.terminal.window_title().as_deref());
        // Nur die Länge, nie der Titel selbst (er enthält oft Pfade).
        clipboard::debug(&format!("Reitertitel ({} Zeichen)", title.chars().count()));
        self.label.set_text(&title);
        self.label.set_tooltip_text(Some(&title));
    }
}

struct Inner {
    window: gtk4::ApplicationWindow,
    menubar: gtk4::PopoverMenuBar,
    notebook: gtk4::Notebook,
    tabs: RefCell<Vec<Rc<Tab>>>,
    tab_bar: Cell<TabBar>,
    settings: PathBuf,
    /// Das Schließen ist bestätigt oder ohne Frage erlaubt; keine weitere
    /// Nachfrage.
    closing: Cell<bool>,
}

#[derive(Clone)]
struct TerminalWindow(Rc<Inner>);

/// Rückruf einer einfachen Fensteraktion.
type WindowAction = fn(&TerminalWindow);

/// Ziel eines Mausklicks im Notebook.
enum Spot {
    Tab(Rc<Tab>),
    /// Freie Fläche der Reiterleiste.
    Bar,
    /// Terminal, Scrollleiste, Pfeile.
    Elsewhere,
}

thread_local! {
    /// Die offenen Fenster. Rückrufe halten nur schwache Referenzen; stark
    /// gehalten wird ein Fenster hier, bis GTK es zerstört.
    static WINDOWS: RefCell<Vec<TerminalWindow>> = const { RefCell::new(Vec::new()) };
}

pub fn build(app: &gtk4::Application) {
    let settings = tabs::settings_path();
    let tab_bar = tabs::load_tab_bar(&settings);
    let win = TerminalWindow::new(app, tab_bar, settings);
    WINDOWS.with(|w| w.borrow_mut().push(win.clone()));
    win.add_tab(None);
    win.0.window.present();
}

impl TerminalWindow {
    fn new(app: &gtk4::Application, tab_bar: TabBar, settings: PathBuf) -> Self {
        let notebook = gtk4::Notebook::builder()
            .scrollable(true)
            .show_border(false)
            .show_tabs(tab_bar.visible(1))
            .hexpand(true)
            .vexpand(true)
            .build();
        // Die Reiter selbst nehmen keinen Fokus; Tasten gehen ans Terminal.
        notebook.set_focusable(false);

        let menubar = gtk4::PopoverMenuBar::from_model(Some(&menus::model()));
        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        content.append(&menubar);
        // Die Menüleiste nimmt nur während eines offenen Menüs den Fokus.
        menubar.set_focusable(false);
        content.append(&notebook);

        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title(TITLE)
            .child(&content)
            .build();
        window.add_css_class("qt-window");
        // F10 bleibt beim Programm im Terminal (Midnight Commander, htop).
        window.set_handle_menubar_accel(false);

        let win = TerminalWindow(Rc::new(Inner {
            window,
            notebook,
            tabs: RefCell::new(Vec::new()),
            tab_bar: Cell::new(tab_bar),
            settings,
            closing: Cell::new(false),
            menubar,
        }));
        win.install_actions();
        win.install_shortcuts();
        win.install_tab_bar_mouse();
        win.connect_signals();
        win
    }

    fn weak(&self) -> Weak<Inner> {
        Rc::downgrade(&self.0)
    }

    fn upgrade(weak: &Weak<Inner>) -> Option<Self> {
        weak.upgrade().map(TerminalWindow)
    }

    /// Das Fenster schließt: nach dem Abbau durch GTK nicht mehr festhalten.
    fn forget_later(&self) {
        let weak = self.weak();
        glib::idle_add_local_once(move || {
            let _ = WINDOWS.try_with(|w| {
                w.borrow_mut()
                    .retain(|win| !std::ptr::eq(Rc::as_ptr(&win.0), weak.as_ptr()))
            });
        });
    }

    /// Fensteraktionen der Menüleiste.
    fn install_actions(&self) {
        let window = &self.0.window;
        let simple: [(&str, WindowAction); 14] = [
            ("new-tab", |w| w.run(tabs::Command::NewTab)),
            ("close-tab", |w| w.run(tabs::Command::CloseTab)),
            ("close-window", |w| w.run(tabs::Command::CloseWindow)),
            ("previous-tab", |w| w.run(tabs::Command::PreviousTab)),
            ("next-tab", |w| w.run(tabs::Command::NextTab)),
            ("move-tab-left", |w| w.run(tabs::Command::MoveTabLeft)),
            ("move-tab-right", |w| w.run(tabs::Command::MoveTabRight)),
            ("copy", |w| {
                if let Some(tab) = w.current() {
                    clipboard::copy(&tab.terminal);
                }
            }),
            ("paste", |w| {
                if let Some(tab) = w.current() {
                    clipboard::paste(&tab.terminal);
                }
            }),
            ("select-all", |w| {
                if let Some(tab) = w.current() {
                    clipboard::select_all(&tab.terminal);
                }
            }),
            ("shortcuts", |w| dialogs::shortcuts(&w.0.window)),
            ("about", |w| dialogs::about(&w.0.window)),
            ("online-help", |w| {
                dialogs::open_uri(&w.0.window, &dialogs::online_help_url())
            }),
            ("report-problem", |w| {
                dialogs::open_uri(&w.0.window, dialogs::REPORT_URL)
            }),
        ];
        for (name, run) in simple {
            let action = gio::SimpleAction::new(name, None);
            let weak = self.weak();
            action.connect_activate(move |_, _| {
                clipboard::debug(&format!("Aktion {name}"));
                if let Some(w) = Self::upgrade(&weak) {
                    run(&w);
                }
            });
            window.add_action(&action);
        }

        let always = gio::SimpleAction::new_stateful(
            "tab-bar-always",
            None,
            &(self.0.tab_bar.get() == TabBar::Always).to_variant(),
        );
        let weak = self.weak();
        always.connect_change_state(move |action, value| {
            let Some(w) = Self::upgrade(&weak) else {
                return;
            };
            let Some(on) = value.and_then(|v| v.get::<bool>()) else {
                return;
            };
            action.set_state(&on.to_variant());
            let bar = if on {
                TabBar::Always
            } else {
                TabBar::Automatic
            };
            clipboard::debug(&format!("Reiterleiste: {}", bar.key()));
            w.0.tab_bar.set(bar);
            if let Err(err) = tabs::save_tab_bar(&w.0.settings, bar) {
                eprintln!(
                    "querencia-terminal: Einstellung nicht gespeichert ({}): {err}",
                    w.0.settings.display()
                );
            }
            w.update();
        });
        window.add_action(&always);
    }

    /// Reiterkürzel vor VTE (Capture-Phase am Fenster). Gesperrte Befehle
    /// geben die Taste an die Shell weiter.
    fn install_shortcuts(&self) {
        let keys = gtk4::EventControllerKey::new();
        keys.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let weak = self.weak();
        keys.connect_key_pressed(move |_, keyval, _, state| {
            let Some(w) = Self::upgrade(&weak) else {
                return glib::Propagation::Proceed;
            };
            let Some(command) = tabs::shortcut(keyval, state) else {
                return glib::Propagation::Proceed;
            };
            let (count, current) = w.position();
            if !tabs::enabled(command, count, current) {
                return glib::Propagation::Proceed;
            }
            clipboard::debug(&format!("Reiterkürzel {command:?}"));
            w.run(command);
            glib::Propagation::Stop
        });
        self.0.window.add_controller(keys);
    }

    /// Mittelklick auf einen Reiter schließt ihn, Doppelklick auf die freie
    /// Reiterleiste öffnet einen neuen.
    fn install_tab_bar_mouse(&self) {
        let click = gtk4::GestureClick::new();
        click.set_button(0);
        click.set_propagation_phase(gtk4::PropagationPhase::Capture);
        let weak = self.weak();
        click.connect_pressed(move |gesture, n_press, x, y| {
            let Some(w) = Self::upgrade(&weak) else {
                return;
            };
            let button = gesture.current_button();
            match w.spot(x, y) {
                Spot::Tab(tab) if button == gtk4::gdk::BUTTON_MIDDLE => {
                    gesture.set_state(gtk4::EventSequenceState::Claimed);
                    clipboard::debug("Mittelklick auf Reiter");
                    w.close_tab(&tab);
                }
                Spot::Bar if button == gtk4::gdk::BUTTON_PRIMARY && n_press == 2 => {
                    gesture.set_state(gtk4::EventSequenceState::Claimed);
                    clipboard::debug("Doppelklick auf die Reiterleiste");
                    w.new_tab();
                }
                Spot::Tab(_) | Spot::Bar => {}
                // Klicks ins Terminal gehören VTE.
                Spot::Elsewhere => {
                    gesture.set_state(gtk4::EventSequenceState::Denied);
                }
            }
        });
        self.0.notebook.add_controller(click);
    }

    fn connect_signals(&self) {
        let weak = self.weak();
        self.0.window.connect_close_request(move |_| {
            let Some(w) = Self::upgrade(&weak) else {
                return glib::Propagation::Proceed;
            };
            let tabs = w.0.tabs.borrow().clone();
            if w.0.closing.get() || !tabs.iter().any(|t| t.running()) {
                w.forget_later();
                return glib::Propagation::Proceed;
            }
            let what = if tabs.len() > 1 {
                Closing::Window
            } else {
                Closing::Terminal
            };
            let weak = w.weak();
            dialogs::confirm_close(&w.0.window, what, move || {
                if let Some(w) = Self::upgrade(&weak) {
                    w.0.closing.set(true);
                    w.0.window.close();
                }
            });
            glib::Propagation::Stop
        });

        // Nach einem Menüeintrag oder Dialog bliebe der Fokus sonst auf dem
        // geschlossenen Menü; Tippen löste den Eintrag dann erneut aus.
        let weak = self.weak();
        self.0.window.connect_focus_widget_notify(move |_| {
            let weak = weak.clone();
            glib::idle_add_local_once(move || {
                if let Some(w) = Self::upgrade(&weak) {
                    w.keep_focus_in_terminal();
                }
            });
        });
        let weak = self.weak();
        self.0.window.connect_is_active_notify(move |window| {
            if window.is_active()
                && let Some(w) = Self::upgrade(&weak)
            {
                w.keep_focus_in_terminal();
            }
        });

        // Nach jedem Wechsel und Verschieben: Fokus ins Terminal, Leiste und
        // Aktionen neu.
        let weak = self.weak();
        self.0.notebook.connect_switch_page(move |_, page, _| {
            let page = page.clone();
            let weak = weak.clone();
            glib::idle_add_local_once(move || {
                if let Some(w) = Self::upgrade(&weak) {
                    if let Some(tab) = w.tab_for_page(&page) {
                        tab.terminal.grab_focus();
                    }
                    w.update();
                }
            });
        });
        let weak = self.weak();
        self.0.notebook.connect_page_reordered(move |_, _, _| {
            if let Some(w) = Self::upgrade(&weak) {
                w.update();
            }
        });
    }

    /// Fokus zurück ins aktuelle Terminal, wenn er auf keinem Widget oder
    /// auf der Menüleiste ohne offenes Menü liegt.
    fn keep_focus_in_terminal(&self) {
        let stray = match GtkWindowExt::focus(&self.0.window) {
            None => true,
            Some(focus) => {
                let in_open_menu = focus
                    .ancestor(gtk4::Popover::static_type())
                    .is_some_and(|p| p.is_visible());
                focus.is_ancestor(&self.0.menubar) && !in_open_menu
            }
        };
        if stray && let Some(tab) = self.current() {
            clipboard::debug("Fokus zurück ins Terminal");
            tab.terminal.grab_focus();
        }
    }

    /// Anzahl der Reiter und Stelle des aktuellen.
    fn position(&self) -> (usize, usize) {
        let count = self.0.notebook.n_pages() as usize;
        let current = self.0.notebook.current_page().unwrap_or(0) as usize;
        (count, current)
    }

    fn current(&self) -> Option<Rc<Tab>> {
        let page = self.0.notebook.nth_page(self.0.notebook.current_page())?;
        self.tab_for_page(&page)
    }

    fn tab_for_page(&self, page: &gtk4::Widget) -> Option<Rc<Tab>> {
        self.0
            .tabs
            .borrow()
            .iter()
            .find(|t| t.page.upcast_ref::<gtk4::Widget>() == page)
            .cloned()
    }

    fn tab_for_terminal(&self, terminal: &vte4::Terminal) -> Option<Rc<Tab>> {
        self.0
            .tabs
            .borrow()
            .iter()
            .find(|t| &t.terminal == terminal)
            .cloned()
    }

    /// Führt einen Reiterbefehl aus (Kürzel, Menü, Kontextmenü).
    fn run(&self, command: tabs::Command) {
        let notebook = &self.0.notebook;
        let (count, current) = self.position();
        match command {
            tabs::Command::NewTab => self.new_tab(),
            tabs::Command::CloseTab => {
                if let Some(tab) = self.current() {
                    self.close_tab(&tab);
                }
            }
            tabs::Command::CloseWindow => self.0.window.close(),
            tabs::Command::PreviousTab | tabs::Command::NextTab | tabs::Command::GotoTab(_) => {
                if let Some(index) = tabs::target(command, count, current) {
                    notebook.set_current_page(Some(index as u32));
                }
            }
            tabs::Command::MoveTabLeft | tabs::Command::MoveTabRight => {
                if let (Some(index), Some(tab)) =
                    (tabs::target(command, count, current), self.current())
                {
                    notebook.reorder_child(&tab.page, Some(index as u32));
                    self.update();
                }
            }
        }
    }

    /// Neuer Reiter im Arbeitsverzeichnis des aktuellen.
    fn new_tab(&self) {
        let directory = self.current().and_then(|tab| tab.directory());
        self.add_tab(directory.as_deref());
    }

    fn add_tab(&self, directory: Option<&Path>) {
        let terminal = vte4::Terminal::new();
        // Die natürliche Größe des Widgets folgt aus Spalten und Zeilen; das
        // Fenster übernimmt sie beim ersten Reiter, weil keine Standardgröße
        // gesetzt ist.
        terminal.set_size(COLUMNS, ROWS);
        terminal.set_scrollback_lines(SCROLLBACK_LINES);
        terminal.set_scroll_on_keystroke(true);
        terminal.set_scroll_on_output(false);
        terminal.set_hexpand(true);
        terminal.set_vexpand(true);

        let scrollbar =
            gtk4::Scrollbar::new(gtk4::Orientation::Vertical, terminal.vadjustment().as_ref());
        let page = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        page.append(&terminal);
        page.append(&scrollbar);

        let label = gtk4::Label::builder()
            .label(tabs::tab_title(None))
            .width_chars(TAB_LABEL_CHARS)
            .max_width_chars(TAB_LABEL_CHARS)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .xalign(0.0)
            .build();
        let close = gtk4::Button::from_icon_name("window-close");
        close.add_css_class("flat");
        close.add_css_class("qt-tab-close");
        close.set_tooltip_text(Some(&crate::i18n::gettext("Close tab")));
        close.set_focus_on_click(false);
        close.set_focusable(false);
        let tab_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        tab_box.append(&label);
        tab_box.append(&close);

        let tab = Rc::new(Tab {
            terminal: terminal.clone(),
            page: page.clone(),
            label,
            shell_pid: Cell::new(None),
        });
        tab.update_title();
        self.0.tabs.borrow_mut().push(tab.clone());

        self.install_context_menu(&terminal);

        let weak_tab = Rc::downgrade(&tab);
        terminal.connect_window_title_changed(move |_| {
            if let Some(tab) = weak_tab.upgrade() {
                tab.update_title();
            }
        });

        // Shell-Ende schließt den Reiter ohne Nachfrage; mit dem letzten
        // Reiter schließt das Fenster, mit dem letzten Fenster endet die
        // Anwendung. Entfernt wird im Leerlauf, nicht aus dem Signal des
        // Terminals heraus.
        let weak = self.weak();
        let weak_tab = Rc::downgrade(&tab);
        terminal.connect_child_exited(move |_, status| {
            eprintln!(
                "querencia-terminal: Shell beendet ({})",
                shell::ChildExit::from_wait_status(status)
            );
            let weak = weak.clone();
            let weak_tab = weak_tab.clone();
            glib::idle_add_local_once(move || {
                if let (Some(w), Some(tab)) = (Self::upgrade(&weak), weak_tab.upgrade()) {
                    w.remove_tab(&tab);
                }
            });
        });

        let weak = self.weak();
        let weak_tab = Rc::downgrade(&tab);
        close.connect_clicked(move |_| {
            if let (Some(w), Some(tab)) = (Self::upgrade(&weak), weak_tab.upgrade()) {
                w.close_tab(&tab);
            }
        });

        let notebook = &self.0.notebook;
        let index = notebook.append_page(&page, Some(&tab_box));
        notebook.set_tab_reorderable(&page, true);
        notebook.set_current_page(Some(index));
        self.update();
        GtkWindowExt::set_focus(&self.0.window, Some(&terminal));

        let weak_tab = Rc::downgrade(&tab);
        shell::start(&terminal, directory, move |pid| {
            if let Some(tab) = weak_tab.upgrade() {
                tab.shell_pid.set(Some(pid));
            }
        });
    }

    /// Kontextmenü und Kopierkürzel eines Terminals (siehe clipboard.rs).
    /// Schwache Referenzen: Menü und Controller hängen am Terminal selbst.
    fn install_context_menu(&self, terminal: &vte4::Terminal) {
        let query_weak = terminal.downgrade();
        let terminal_weak = terminal.downgrade();
        let weak = self.weak();
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
                    context_menu::Command::NewTab => {
                        if let Some(w) = Self::upgrade(&weak) {
                            w.new_tab();
                        }
                    }
                    context_menu::Command::CloseTab => {
                        if let Some(w) = Self::upgrade(&weak)
                            && let Some(tab) = w.tab_for_terminal(&terminal)
                        {
                            w.close_tab(&tab);
                        }
                    }
                }
            },
        );
        terminal.set_context_menu(Some(menu.popover()));
        let keyboard_menu = menu.clone();
        clipboard::install_shortcuts(terminal, move || keyboard_menu.opening_by_keyboard());
    }

    /// Schließt einen Reiter, mit Nachfrage, wenn darin noch ein Programm
    /// läuft. Der letzte Reiter schließt das Fenster.
    fn close_tab(&self, tab: &Rc<Tab>) {
        if self.0.closing.get() || !tab.running() {
            self.remove_tab(tab);
            return;
        }
        let weak = self.weak();
        let weak_tab = Rc::downgrade(tab);
        dialogs::confirm_close(&self.0.window, Closing::Terminal, move || {
            if let (Some(w), Some(tab)) = (Self::upgrade(&weak), weak_tab.upgrade()) {
                w.remove_tab(&tab);
            }
        });
    }

    /// Entfernt den Reiter ohne Frage. Mit dem Terminal endet die Shell
    /// (VTE schließt das PTY, die Shell bekommt SIGHUP).
    fn remove_tab(&self, tab: &Rc<Tab>) {
        let notebook = &self.0.notebook;
        if let Some(index) = notebook.page_num(&tab.page) {
            notebook.remove_page(Some(index));
        }
        self.0.tabs.borrow_mut().retain(|t| !Rc::ptr_eq(t, tab));
        let left = self.0.tabs.borrow().len();
        clipboard::debug(&format!("Reiter geschlossen, {left} übrig"));
        if left == 0 {
            self.0.closing.set(true);
            self.0.window.close();
            return;
        }
        self.update();
        if let Some(tab) = self.current() {
            tab.terminal.grab_focus();
        }
    }

    /// Reiterleiste zeigen oder verbergen, Aktionen sperren oder freigeben.
    fn update(&self) {
        let (count, current) = self.position();
        self.0
            .notebook
            .set_show_tabs(self.0.tab_bar.get().visible(count));
        for (name, command) in [
            ("previous-tab", tabs::Command::PreviousTab),
            ("next-tab", tabs::Command::NextTab),
            ("move-tab-left", tabs::Command::MoveTabLeft),
            ("move-tab-right", tabs::Command::MoveTabRight),
        ] {
            if let Some(action) = self
                .0
                .window
                .lookup_action(name)
                .and_then(|a| a.downcast::<gio::SimpleAction>().ok())
            {
                action.set_enabled(tabs::enabled(command, count, current));
            }
        }
    }

    /// Wo im Notebook liegt der Punkt (Koordinaten des Notebooks)?
    fn spot(&self, x: f64, y: f64) -> Spot {
        let notebook = self.0.notebook.upcast_ref::<gtk4::Widget>();
        let mut widget = notebook.pick(x, y, gtk4::PickFlags::DEFAULT);
        while let Some(w) = widget {
            if &w == notebook {
                break;
            }
            match w.css_name().as_str() {
                "tab" => {
                    let tabs = self.0.tabs.borrow();
                    let tab = tabs.iter().find(|t| {
                        self.0
                            .notebook
                            .tab_label(&t.page)
                            .and_then(|l| l.parent())
                            .is_some_and(|p| p == w)
                    });
                    return tab.map_or(Spot::Elsewhere, |t| Spot::Tab(t.clone()));
                }
                // Pfeile der zu vollen Leiste blättern nur.
                "arrow" => return Spot::Elsewhere,
                "header" | "tabs" => return Spot::Bar,
                _ => {}
            }
            widget = w.parent();
        }
        Spot::Elsewhere
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flächen und Ränder kommen aus dem Thema (hell wie bisher, dunkel
    /// wie das Thema); feste Werte nur noch für die orange Hervorhebung.
    #[test]
    fn window_css_takes_surfaces_from_the_theme() {
        for fixed in ["#f2f1f0", "#c7c2be", "#3c3c3c", "#f5f4f2", "#ffffff"] {
            assert!(!WINDOW_CSS.contains(fixed), "fester Hellwert {fixed}");
        }
        assert!(!WINDOW_CSS.contains("@media"));
        assert!(WINDOW_CSS.contains("menubar { background-color: @theme_bg_color;"));
        assert!(WINDOW_CSS.contains("header { background-color: @theme_bg_color;"));
        assert!(WINDOW_CSS.contains("tab:checked { background-color: @theme_base_color;"));
    }

    #[test]
    fn window_css_is_scoped_to_the_terminal() {
        for rule in WINDOW_CSS.lines().filter(|l| !l.is_empty()) {
            assert!(
                rule.starts_with("window.qt-window ") || rule.starts_with("window.qt-dialog "),
                "{rule}"
            );
        }
    }
}
