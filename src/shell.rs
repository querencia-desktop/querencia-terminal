//! Shell-Auswahl, Start über VTE und Auswertung des Prozessendes.

use std::fmt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use gtk4::{gio, glib};
use vte4::prelude::*;

use crate::i18n::{gettext, gettext_f};

/// Letzter Ausweg, wenn weder `$SHELL` noch der passwd-Eintrag taugen.
pub const FALLBACK_SHELL: &str = "/bin/sh";

/// Wählt die Shell in derselben Reihenfolge wie MATE Terminal (`egg_shell`):
/// `$SHELL`, dann der passwd-Eintrag des Benutzers, dann `/bin/sh`.
/// Ein Kandidat zählt nur mit absolutem Pfad und wenn `usable` zustimmt.
pub fn choose_shell(
    env_shell: Option<&str>,
    passwd_shell: Option<&str>,
    usable: impl Fn(&Path) -> bool,
) -> Option<String> {
    [env_shell, passwd_shell, Some(FALLBACK_SHELL)]
        .into_iter()
        .flatten()
        .find(|candidate| {
            let path = Path::new(candidate);
            path.is_absolute() && usable(path)
        })
        .map(str::to_owned)
}

/// Shell für den aktuellen Benutzer.
fn user_shell() -> Option<String> {
    let env_shell = std::env::var_os("SHELL").and_then(|s| s.into_string().ok());
    choose_shell(
        env_shell.as_deref(),
        passwd_shell().as_deref(),
        is_executable_file,
    )
}

/// Login-Shell aus der Benutzerdatenbank (über NSS, also auch LDAP/SSSD).
fn passwd_shell() -> Option<String> {
    // SAFETY: vte_get_user_shell() hat keine Vorbedingungen und liefert
    // einen neu belegten Dateinamen oder NULL (transfer full).
    let shell: Option<PathBuf> =
        unsafe { glib::translate::from_glib_full(vte4::ffi::vte_get_user_shell()) };
    shell.and_then(|path| path.into_os_string().into_string().ok())
}

fn is_executable_file(path: &Path) -> bool {
    path.metadata()
        .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

/// Startet die Shell asynchron im PTY des Terminals. VTE überwacht den
/// Kindprozess selbst und meldet sein Ende über `child-exited`.
pub fn start(terminal: &vte4::Terminal) {
    let Some(shell) = user_shell() else {
        // TRANSLATORS: steht hinter „Error:“, daher klein und ohne Punkt.
        report_error(terminal, &gettext("no executable shell found"));
        return;
    };
    let home = glib::home_dir();
    // Ohne UTF-8-Pfad übernimmt VTE das Arbeitsverzeichnis des Prozesses.
    let cwd = home.to_str();
    eprintln!(
        "querencia-terminal: starte {shell} in {}",
        cwd.unwrap_or("(Arbeitsverzeichnis des Prozesses)")
    );

    // argv als Liste, keine Zeichenkettenverkettung. Leeres envv: VTE
    // übernimmt die Umgebung des Prozesses und ergänzt TERM, COLORTERM,
    // VTE_VERSION und PWD.
    let argv = [shell.as_str()];
    let shell_name = shell.clone();
    let terminal_weak = terminal.downgrade();
    terminal.spawn_async(
        vte4::PtyFlags::DEFAULT,
        cwd,
        &argv,
        &[],
        glib::SpawnFlags::DEFAULT,
        || {},
        -1,
        gio::Cancellable::NONE,
        move |result| match result {
            Ok(pid) => eprintln!("querencia-terminal: {shell_name} läuft als PID {}", pid.0),
            Err(err) => {
                if let Some(terminal) = terminal_weak.upgrade() {
                    let error = err.to_string();
                    report_error(
                        &terminal,
                        &gettext_f(
                            // TRANSLATORS: {shell} ist der Pfad der Shell, {error} die
                            // Meldung von GLib; steht hinter „Error:“.
                            "{shell} could not be started: {error}",
                            &[("shell", &shell_name), ("error", &error)],
                        ),
                    );
                }
            }
        },
    );
}

/// Meldet einen Startfehler auf stderr und im Terminal selbst. Das Fenster
/// bleibt offen, damit die Meldung lesbar ist.
fn report_error(terminal: &vte4::Terminal, message: &str) {
    eprintln!("querencia-terminal: Fehler: {message}");
    // TRANSLATORS: rot vor der Meldung im Terminal, z. B. „Error: no
    // executable shell found“.
    let label = gettext("Error:");
    let text = format!("\x1b[1;31m{label}\x1b[0m {message}\r\n");
    terminal.feed(text.as_bytes());
}

/// Rohstatus aus `waitpid`, wie ihn VTE mit `child-exited` liefert.
#[derive(Debug, PartialEq, Eq)]
pub enum ChildExit {
    Exited(i32),
    Signaled(i32),
    Unknown(i32),
}

impl ChildExit {
    /// Zerlegt den Linux-Wartestatus (`WIFEXITED`/`WIFSIGNALED`).
    pub fn from_wait_status(status: i32) -> Self {
        match status & 0x7f {
            0 => Self::Exited((status >> 8) & 0xff),
            0x7f => Self::Unknown(status),
            signal => Self::Signaled(signal),
        }
    }
}

impl fmt::Display for ChildExit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exited(code) => write!(f, "Exit-Code {code}"),
            Self::Signaled(signal) => write!(f, "Signal {signal}"),
            Self::Unknown(status) => write!(f, "unbekannter Status {status}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn only(allowed: &'static [&'static str]) -> impl Fn(&Path) -> bool {
        move |path| allowed.iter().any(|a| Path::new(a) == path)
    }

    #[test]
    fn env_shell_comes_first() {
        let shell = choose_shell(Some("/bin/zsh"), Some("/bin/bash"), |_| true);
        assert_eq!(shell.as_deref(), Some("/bin/zsh"));
    }

    #[test]
    fn unusable_env_shell_falls_back_to_passwd() {
        let shell = choose_shell(
            Some("/nicht/vorhanden"),
            Some("/bin/bash"),
            only(&["/bin/bash", "/bin/sh"]),
        );
        assert_eq!(shell.as_deref(), Some("/bin/bash"));
    }

    #[test]
    fn relative_env_shell_is_ignored() {
        let shell = choose_shell(Some("bash"), None, |_| true);
        assert_eq!(shell.as_deref(), Some(FALLBACK_SHELL));
    }

    #[test]
    fn empty_env_shell_is_ignored() {
        let shell = choose_shell(Some(""), Some("/bin/bash"), |_| true);
        assert_eq!(shell.as_deref(), Some("/bin/bash"));
    }

    #[test]
    fn missing_candidates_fall_back_to_bin_sh() {
        let shell = choose_shell(None, None, only(&["/bin/sh"]));
        assert_eq!(shell.as_deref(), Some(FALLBACK_SHELL));
    }

    #[test]
    fn no_usable_shell_yields_none() {
        assert_eq!(choose_shell(Some("/bin/zsh"), None, |_| false), None);
    }

    #[test]
    fn real_bin_sh_is_executable() {
        assert!(is_executable_file(Path::new(FALLBACK_SHELL)));
        assert!(!is_executable_file(Path::new("/etc/passwd")));
        assert!(!is_executable_file(Path::new("/bin")));
    }

    #[test]
    fn wait_status_exit_code() {
        assert_eq!(ChildExit::from_wait_status(0), ChildExit::Exited(0));
        assert_eq!(ChildExit::from_wait_status(1 << 8), ChildExit::Exited(1));
        assert_eq!(
            ChildExit::from_wait_status(127 << 8),
            ChildExit::Exited(127)
        );
    }

    #[test]
    fn wait_status_signal() {
        // SIGHUP = 1, SIGKILL = 9; Bit 0x80 markiert einen Core-Dump.
        assert_eq!(ChildExit::from_wait_status(1), ChildExit::Signaled(1));
        assert_eq!(ChildExit::from_wait_status(9), ChildExit::Signaled(9));
        assert_eq!(
            ChildExit::from_wait_status(0x80 | 11),
            ChildExit::Signaled(11)
        );
    }

    #[test]
    fn wait_status_unknown() {
        // -1 ist kein gültiger Wartestatus und darf nicht als Exit-Code gelten.
        assert_eq!(ChildExit::from_wait_status(-1), ChildExit::Unknown(-1));
    }
}
