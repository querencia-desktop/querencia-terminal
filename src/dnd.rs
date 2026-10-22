//! Dateien und Text ins Terminal ziehen, wie im GNOME Terminal (QD-125).
//!
//! Ein `GtkDropTarget` auf jedem VTE-Widget nimmt eine Dateiliste
//! (`GdkFileList`, unter Wayland aus `text/uri-list`) oder Text an:
//!
//! - Dateien: lokale Dateien (`file://`) als Pfad, andere Adressen
//!   unverändert als URI; jedes Stück in einfachen Anführungszeichen
//!   (`'` darin als `'\''`) und mit einem Leerzeichen dahinter, damit man
//!   gleich weiterschreiben kann.
//! - Text: so wie er ist.
//!
//! Eingefügt wird über `Terminal::paste_text`, also an der Schreibmarke wie
//! beim Einfügen aus der Zwischenablage: VTE wandelt Zeilenenden und klammert
//! den Text mit Bracketed Paste ein, wenn die Shell das verlangt. Ein
//! gezogener Text mit Zeilenumbruch wird dann nicht sofort ausgeführt.
//!
//! Nur die Aktion „Kopieren“: eine Quelle, die verschieben will (Files mit
//! Umschalt), löscht dadurch nie etwas. Inhalte werden nie protokolliert,
//! nur die Zahl der Stücke.

use gtk4::prelude::*;
use gtk4::{gdk, gio, glib};
use vte4::prelude::*;

/// Ein Stück für die Shell in einfachen Anführungszeichen.
pub fn quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

/// Text für eine gezogene Dateiliste: lokale Pfade, sonst die URI, jeweils
/// gequotet und mit Leerzeichen dahinter.
pub fn files_text(files: &[gio::File]) -> String {
    files
        .iter()
        .map(|file| {
            let piece = match file.path() {
                Some(path) => path.to_string_lossy().into_owned(),
                None => file.uri().to_string(),
            };
            format!("{} ", quote(&piece))
        })
        .collect()
}

/// Hängt das Ziel an `terminal`.
pub fn install(terminal: &vte4::Terminal) {
    let target = gtk4::DropTarget::new(glib::Type::INVALID, gdk::DragAction::COPY);
    target.set_types(&[gdk::FileList::static_type(), String::static_type()]);
    let weak = terminal.downgrade();
    target.connect_drop(move |_, value, _, _| {
        let Some(terminal) = weak.upgrade() else {
            return false;
        };
        let text = if let Ok(list) = value.get::<gdk::FileList>() {
            let files = list.files();
            crate::clipboard::debug(&format!("Ablegen: {} Datei(en)", files.len()));
            files_text(&files)
        } else if let Ok(text) = value.get::<String>() {
            crate::clipboard::debug(&format!("Ablegen: Text ({} Zeichen)", text.chars().count()));
            text
        } else {
            return false;
        };
        if text.is_empty() {
            return false;
        }
        terminal.paste_text(&text);
        terminal.grab_focus();
        true
    });
    terminal.add_controller(target);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_like_a_posix_shell() {
        assert_eq!(quote("/home/a/b.txt"), "'/home/a/b.txt'");
        assert_eq!(quote("/tmp/mit Leer zeichen"), "'/tmp/mit Leer zeichen'");
        assert_eq!(quote("it's"), r"'it'\''s'");
        assert_eq!(quote("$HOME;`x`"), "'$HOME;`x`'");
    }

    #[test]
    fn files_become_quoted_paths_with_a_space_after_each() {
        let files = [
            gio::File::for_path("/home/nutzer/Bilder/a b.png"),
            gio::File::for_path("/home/nutzer/Grüße.txt"),
        ];
        assert_eq!(
            files_text(&files),
            "'/home/nutzer/Bilder/a b.png' '/home/nutzer/Grüße.txt' "
        );
    }

    #[test]
    fn other_uris_stay_uris() {
        let files = [gio::File::for_uri("sftp://server/pfad/datei.txt")];
        assert_eq!(files_text(&files), "'sftp://server/pfad/datei.txt' ");
    }
}
