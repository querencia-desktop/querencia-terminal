# Querencia Terminal

Terminalemulator für Querencia Desktop, eine Desktopumgebung im Stil von
GNOME 2 und MATE. Vorbild für Aussehen und Bedienung ist MATE Terminal
(Referenz: Fork `pottrauschen/mate-terminal`). Es wird kein C-Code
portiert.

**Stand: M3.0a, Entwicklungsstufe.** Das Repository ist eigenständig
und vom Desktop-Workspace (`entwicklungs-vm`) getrennt. Eine produktive
Integration in Querencia Desktop gibt es noch nicht: keine
`.desktop`-Datei, keine Installation und keine Einrichtung als
Standardterminal.

## Technik

- Rust (Edition 2024, MSRV 1.92)
- GTK 4 über `gtk4` 0.11 (Feature `v4_10`, siehe unten)
- VTE für GTK 4 über `vte4` 0.10.0 (`libvte-2.91-gtk4`), als lokale,
  korrigierte Kopie unter `vendor/vte4/` (siehe „Lokaler vte4-Patch“)
- Kein libadwaita und keine Headerbar

Terminalemulation, PTY, ANSI-Sequenzen, Darstellung, Tastatur und
Scrollback übernimmt VTE. Die Anwendung baut das Fenster, startet die
Shell und reagiert auf das Prozessende.

## Voraussetzungen

Ubuntu 26.04:

```
libgtk-4-dev          (GTK ≥ 4.10; getestet mit 4.22.4)
libvte-2.91-gtk4-dev  (VTE ≥ 0.65; Ubuntu 26.04: 0.84.0-2)
pkg-config, Rust ≥ 1.92
```

`gtk4` steht mit dem Feature `v4_10` in `Cargo.toml`, weil `vte4` 0.10
den Typ `gtk::Accessible` braucht, den `gtk4` 0.11 erst ab diesem Feature
exportiert. Ohne das Feature kompiliert `vte4` nicht.

## Lokaler vte4-Patch

`Cargo.toml` ersetzt `vte4` 0.10.0 von crates.io per `[patch.crates-io]`
durch die Kopie unter `vendor/vte4/`. Die Versionsnummer bleibt 0.10.0.
In `Cargo.lock` steht `vte4` deshalb ohne `source` und `checksum`.

**Herkunft:** Das Archiv `vte4-0.10.0.crate` von crates.io (SHA256
`c61732974d4915c5112a82506cdc86e19a83d786497c2c8bcd80d065825c695d`, wie
zuvor in `Cargo.lock`) wurde unverändert entpackt. Es stammt aus
Upstream-Commit `051bd0277bfb28b627282498d8fc6ad8e9d3a214` von
<https://gitlab.gnome.org/World/Rust/vte4-rs> (siehe
`vendor/vte4/.cargo_vcs_info.json`). Die Lizenz bleibt MIT
(`vendor/vte4/LICENSE`). Geändert sind nur `src/terminal.rs` und
`src/pty.rs`; alle übrigen Dateien entsprechen dem Archiv.

**Fehler:** `spawn_async` und `spawn_with_fds_async` von `Terminal` und
`Pty` legen die Daten für `child_setup` als
`Box<glib::thread_guard::ThreadGuard<Q>>` an. Die zugehörige
Freigabefunktion `child_setup_data_destroy_func` gab sie aber als `Box<Q>`
frei. Die Folgen sind in einer Probe nachgewiesen: Bei einem Closure mit
eingefangenen Daten liest der Drop falsche Daten und gibt mit falscher
Größe frei (16 Byte belegt, 8 Byte freigegeben). Bei einem leeren Closure,
wie es dieses Projekt verwendet, bleiben pro Start 8 Byte liegen. Der
Fehler steckt auch im Upstream-Hauptzweig (Stand 08.03.2026).

**Korrektur:** An allen vier Stellen wird jetzt der tatsächlich belegte
Typ `Box<ThreadGuard<Q>>` freigegeben, und zwar nur im Eigentümerthread
(`ThreadGuard::is_owner()`).

**Grund für die Thread-Prüfung:** VTE gibt die Daten frei, wenn GLib den
`GTask` des Spawn-Vorgangs finalisiert. Das macht der Thread, der die
letzte Referenz abgibt, und das kann auch der Worker-Thread von GLib sein.
In 9000 Messläufen geschah das nie, ausgeschlossen ist es aber nicht. Dort
würde `ThreadGuard::drop()` abbrechen, innerhalb der `extern "C"`-Funktion
wird daraus ein Prozessabbruch (SIGABRT).

**Bewusst akzeptierter Speicherverlust:** Läuft die Freigabe in einem
fremden Thread, wird die Allocation mit `std::mem::forget` absichtlich
nicht freigegeben, und das Closure wird nicht gedroppt. Bei diesem
Projekt sind das höchstens 8 Byte pro Shell-Start, und nur in diesem
seltenen Fall.

**Rückkehr:** Sobald eine veröffentlichte `vte4`-Version den Fehler
nachweislich behebt, ohne den Thread-Abbruch einzuführen, entfallen
`vendor/vte4/` und `[patch.crates-io]`, und es gilt wieder die offizielle
Abhängigkeit. Eine Meldung an Upstream steht noch aus.

Die Kopie ist kein Mitglied des Projekt-Workspace. `cargo fmt --all`
prüft sie deshalb nicht, und Clippy wendet `-D warnings` nicht auf sie
an. Beim Bauen erscheint eine Warnung aus dem unveränderten generierten
Upstream-Code (`vendor/vte4/src/auto/terminal.rs:15`, unbenutzter Import
`object::ObjectType`). Sie blockiert keine Prüfung und bleibt bewusst
unverändert.

## Bauen und Starten

```
cargo build
cargo run
```

Das Programm braucht eine laufende Wayland- (oder X11-)Sitzung. Meldungen
zum Shell-Start und Prozessende gehen mit dem Präfix `querencia-terminal:`
nach stderr.

## M3.0a – Umfang

Umgesetzt (gebaut und geprüft siehe Abschnitt „Prüfstand“):

- GTK4-`Application` mit der App-ID `org.example.Querencia.Terminal`
  (vorläufig), `NON_UNIQUE`: Jeder Start ist ein eigener Prozess.
- `ApplicationWindow` mit dem Titel „Querencia Terminal“, ohne
  Headerbar. Darin ein VTE-Terminal und eine Scrollleiste rechts, wie bei
  MATE.
- Anfangsgröße 80×24 Zeichen über `Terminal::set_size`; das Fenster
  übernimmt die natürliche Größe des Terminals.
- 10 000 Zeilen Scrollback, Sprung ans Ende bei Tastendruck.
- Schrift und Farben: Vorgaben von VTE (keine Profile).
- Shell-Start über `Terminal::spawn_async` (asynchron, PTY über VTE):
  - Shell-Auswahl wie bei MATE Terminal (`egg_shell`): `$SHELL`, dann
    die Login-Shell aus der Benutzerdatenbank (`vte_get_user_shell`, also
    über NSS), zuletzt `/bin/sh`. Ein Kandidat zählt nur mit absolutem
    Pfad, als reguläre Datei und mit gesetztem Ausführungsbit.
  - argv ist eine Liste mit nur dem Shell-Pfad; es wird keine
    Befehlszeile aus Zeichenketten zusammengesetzt und keine weitere
    Shell aufgerufen.
  - Das Arbeitsverzeichnis ist das Home-Verzeichnis (`g_get_home_dir`).
  - Die Umgebung des Prozesses wird vererbt; VTE setzt `TERM`,
    `COLORTERM`, `VTE_VERSION` und `PWD`.
- Startfehler werden auf stderr und rot im Terminal gemeldet; das
  Fenster bleibt dann offen.
- Shell-Ende (`exit`, Ctrl+D, Signal) über `child-exited`: Der Status wird
  auf stderr protokolliert, danach schließt sich das Fenster. Mit dem
  letzten Fenster endet die Anwendung.
- Die Tastatur verarbeitet ausschließlich VTE (keine eigenen Kürzel).

Noch nicht umgesetzt:

- M3.0b: Menüleiste (Datei, Bearbeiten, Ansicht, Terminal, Reiter,
  Hilfe), Reiter, Tastenkürzel, Suche, Kopieren und Einfügen über Menüs,
  Kontextmenü.
- M3.0c: Profile, Schrift, Farbschemata, gespeicherte Einstellungen,
  `.desktop`-Datei, Desktop-Integration, Standardterminal.

## Prüfstand

Stand 14.10.2026, gegen VTE 0.84.0 und GTK 4.22.4 der VM:

- `cargo fmt --all --check`, `cargo check --all-targets`, `cargo build`,
  `cargo test` (11 Tests) und `cargo clippy --all-targets -- -D warnings`
  laufen ohne Fehler, auch mit der lokalen `vte4`-Kopie.
- Ohne Display endet das Programm mit `Failed to open display` und
  Exit-Code 1.
- Die Freigabe der `child_setup`-Daten ist mit isolierten Proben geprüft:
  richtige Größe, genau ein Drop, kein Abbruch in einem fremden Thread.
  Zur Laufzeit lief das über den `Pty`-Pfad, die vier korrigierten
  Funktionen sind wörtlich gleich. `Terminal::spawn_async` selbst ist erst
  mit der GUI-Abnahme zur Laufzeit geprüft.

Die Funktionen an der grafischen Oberfläche sind noch nicht abgenommen.
Die Anleitung dazu steht in `docs/m3-0a-gui-abnahme.md`.

## Bekannte Einschränkungen

- **Titelleiste:** GTK 4 nutzt unter Wayland Serverdekoration, wenn der
  Compositor sie bevorzugt (labwc: KDE-Protokoll `server-decoration`).
  Sonst zeichnet GTK seine eigene Standard-Titelleiste. Die Anwendung
  setzt keine eigene.
- **Fenster schließen bei laufender Shell:** keine Rückfrage. Beim
  Schließen wird das PTY geschlossen, und der Kernel schickt der Shell
  SIGHUP. Ob VTE zusätzlich selbst SIGHUP sendet, ist nicht geprüft.
  Prozesse, die SIGHUP ignorieren oder abgetrennt sind (`nohup`,
  `disown`, `setsid`), laufen weiter, wie bei anderen Terminals auch.
- **systemd-Scope:** VTE verschiebt die Shell in einen eigenen
  systemd-User-Scope (`vte-spawn-*.scope`). Wenn das Terminal endet, gibt
  es deshalb keinen Abbruch über die cgroup; das Beenden hängt allein am
  SIGHUP.
- **Fenster sehr früh geschlossen:** Wird das Fenster geschlossen, bevor
  der Shell-Start fertig ist, beendet VTE den neuen Prozess laut
  Dokumentation selbst. Die Logzeile „läuft als PID“ kann dann trotzdem
  erscheinen.
- **Shell-Ende:** Das Fenster schließt sich immer, auch bei einem
  Exit-Code ungleich 0. Die Option „offen halten“ (MATE `exit-action`)
  folgt mit den Profilen.
- **Startfehler:** nur als Textzeile im Terminal, ohne Infoleiste und ohne
  „Neu starten“.
- **Shell-Prüfung:** Die Ausführbarkeit wird über das Ausführungsbit
  geprüft, nicht über `access(X_OK)` für den aktuellen Benutzer.
- **Pfade:** Shell- und Home-Pfade müssen UTF-8 sein. Ist das
  Home-Verzeichnis kein UTF-8, startet die Shell im aktuellen
  Arbeitsverzeichnis des Prozesses.
- **Home-Verzeichnis:** Fehlt das Arbeitsverzeichnis, weicht VTE selbst
  auf `$HOME` aus. Zeigt `$HOME` auf kein vorhandenes Verzeichnis, scheitert
  der Start mit „Failed to change to directory“, das Fenster bleibt mit
  der Fehlermeldung offen. Belegt mit einer Probe über `vte4::Pty`, nicht
  im Fenster.
- **Ohne Display:** GTK bricht mit einer Meldung ab.

## Struktur

```
src/main.rs    Application, App-ID
src/window.rs  Fenster, Terminal, Scrollleiste, Reaktion auf Prozessende
src/shell.rs   Shell-Auswahl, Start über VTE, Fehlermeldung, Wartestatus
vendor/vte4/   lokale Kopie von vte4 0.10.0 mit Patch (Lizenz MIT)
```

## Lizenz

GPL-3.0-or-later. `vendor/vte4/` behält seine eigene MIT-Lizenz.
