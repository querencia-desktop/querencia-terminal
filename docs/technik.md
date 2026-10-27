# Querencia Terminal – Technik, Umfang, Prüfstand

Ausführliche Technik- und Prüfangaben, aus dem README ausgelagert
(20.10.2026). Einstieg und Stand: [README](../README.md).

## Technik

- Rust (Edition 2024, MSRV 1.92)
- GTK 4 über `gtk4` 0.11 (Feature `v4_10`, siehe unten)
- VTE für GTK 4 über `vte4` 0.10.0 (`libvte-2.91-gtk4`), als lokale,
  korrigierte Kopie unter `vendor/vte4/` (siehe „Lokaler vte4-Patch“)
- Kein libadwaita und keine Headerbar
- Übersetzungen über GNU gettext (`gettext-rs` 0.8 mit `gettext-system`,
  siehe „Sprachen“)

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
exportiert. Ohne das Feature kompiliert `vte4` nicht. `vte4` steht mit
dem Feature `v0_76` darin (`Terminal::set_context_menu`, VTE ≥ 0.76).

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

Das Programm braucht eine laufende Wayland- (oder X11-)Sitzung. Auf der
Entwicklungsumgebung nie in der laufenden Desktopsitzung starten, sondern nur in
einer isolierten Sitzung (`tools/isolierte-gui-pruefung/`, siehe
„Prüfstand“). Meldungen
zum Shell-Start und Prozessende gehen mit dem Präfix `querencia-terminal:`
nach stderr.

## Sprachen

Seit 0.2.0 sind die Texte der Oberfläche englisch im Code (msgid) und
deutsch in `po/de.po`, seit 0.3.3 auch italienisch, spanisch und französisch
(`po/it.po`, `po/es.po`, `po/fr.po`), wie bei GNOME und Debian (Muster: Querencia Editor
0.2.0, `docs/i18n-muster.md` im Editor-Repo). Übersetzt sind das
Kontextmenü („_Kopieren“, „_Einfügen“, „Alles _auswählen“ bzw. „_Copy“,
„_Paste“, „Select _All“, Zugriffstasten je Sprache), die Startfehler im
Terminal und Comment/Keywords der Desktop-Datei. Nicht übersetzt sind der
Name „Querencia Terminal“, der Fenstertitel und die Meldungen auf stderr.

- Sprachwahl wie bei gettext üblich über `LANGUAGE`, `LC_ALL`,
  `LC_MESSAGES`, `LANG`; ohne passenden Katalog erscheint Englisch.
  `LANGUAGE` wirkt nicht, solange die Locale `C`/`POSIX` ist.
- Die Kataloge `<lang>/LC_MESSAGES/querencia-terminal.mo` sucht das
  Programm in `<Ordner der Binary>/share/locale`, dann in
  `<Ordner der Binary>/../share/locale`, zuletzt in `/usr/share/locale`;
  `QUERENCIA_TERMINAL_LOCALEDIR` setzt den Ordner ausdrücklich.
- po-Pflege ohne weitere Abhängigkeiten mit `werkzeug/i18n.py`
  (`pot`, `update`, `desktop`, `mo <ziel>`, `pruefen`). Die `.mo` ist
  inhaltsgleich mit GNU `msgfmt`, die Desktop-Datei byte-gleich mit
  `msgfmt --desktop` (gettext 0.23.2). `xgettext` liest Rust erst ab 0.24.

```
python3 werkzeug/i18n.py pot && python3 werkzeug/i18n.py update
python3 werkzeug/i18n.py desktop data/org.querencia.Terminal.desktop.in data/org.querencia.Terminal.desktop
python3 werkzeug/i18n.py pruefen
python3 werkzeug/i18n.py mo target/locale
LANGUAGE=en QUERENCIA_TERMINAL_LOCALEDIR=target/locale cargo run
```

Der Test `german_catalog_is_complete` hält `cargo test` auch ohne Python
ehrlich: jede msgid der `.pot` muss in `de.po` übersetzt sein.

## M3.0a – Umfang

Umgesetzt (gebaut und geprüft siehe Abschnitt „Prüfstand“):

- GTK4-`Application` mit der App-ID `org.querencia.Terminal`,
  `NON_UNIQUE`: Jeder Start ist ein eigener Prozess. App-ID, Desktop-ID
  (`data/org.querencia.Terminal.desktop`) und `StartupWMClass` sind
  gleich, damit Fensterliste und Titelleiste Name und Symbol finden
  (Test `desktop_file_matches_app_id`). Bis Oktober 2026 hieß die ID
  `org.example.Querencia.Terminal`.
- `ApplicationWindow` mit dem Titel „Querencia Terminal“, ohne
  Headerbar. Darin (seit 0.3.0) Menüleiste und Reiter, je Reiter ein
  VTE-Terminal und eine Scrollleiste rechts, wie bei MATE (siehe
  „Reiter“).
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
  - Das Arbeitsverzeichnis ist das Home-Verzeichnis (`g_get_home_dir`),
    bei einem neuen Reiter das des aktuellen Reiters.
  - Die Umgebung des Prozesses wird vererbt; VTE setzt `TERM`,
    `COLORTERM`, `VTE_VERSION` und `PWD`.
- Startfehler werden auf stderr und rot im Terminal gemeldet; das
  Fenster bleibt dann offen.
- Shell-Ende (`exit`, Ctrl+D, Signal) über `child-exited`: Der Status wird
  auf stderr protokolliert, danach schließt sich der Reiter, mit dem
  letzten Reiter das Fenster. Mit dem letzten Fenster endet die Anwendung.
- Die Tastatur verarbeitet VTE; eigene Kürzel gibt es nur für Kopieren,
  Einfügen und die Reiter (siehe unten).

## Kopieren und Einfügen (wie GNOME Terminal 2.x)

- **Strg+Umschalt+C** kopiert die Markierung in die Zwischenablage
  (CLIPBOARD). Ohne Markierung bleibt die Zwischenablage unverändert.
- **Strg+Umschalt+V** und **Umschalt+Einfg** fügen die Zwischenablage ein.
  VTE klammert eingefügten Text mit Bracketed Paste ein, wenn das Programm
  im Terminal das verlangt (bash ≥ 5.1): mehrzeiliger Text läuft erst mit
  Enter. Umschalt+Einfg liest bewusst CLIPBOARD (VTE/GNOME Terminal 2.x:
  PRIMARY).
- Die Kürzel gehen nie als Zeichen an die Shell. **Strg+C**, **Strg+V**,
  Strg+D usw. bleiben unverändert Steuerzeichen.
- Weiter von VTE: Markieren legt den Text in PRIMARY, die mittlere
  Maustaste fügt PRIMARY ein, Strg+Einfg kopiert, Strg+Umschalt+Einfg fügt
  ein.
- **Kontextmenü** (Rechtsklick, Menü-Taste, Umschalt+F10; wertet ein
  Programm die Maus aus: Umschalt+Rechtsklick): Kopieren, Einfügen,
  Alles auswählen, im Stil der klassischen Querencia-Menüs. Kopieren ist
  ohne Markierung, Einfügen ohne Text in der Zwischenablage ausgegraut.
- Zwischenablage über GTK/GDK (Wayland `wl_data_device` bzw.
  `primary_selection`), keine X11-Wege. Inhalte werden nie protokolliert;
  `QUERENCIA_TERMINAL_DEBUG=1` meldet nur Ereignisse.
- Isolierte GUI-Prüfung: `tools/isolierte-gui-pruefung/pruefen.sh`.
  Pfade kommen aus `$HOME`, `XDG_*` oder der Umgebung: `QT_GUI_TEST`
  (Testordner, Standard `$HOME/projects/querencia-terminal-gui-test`),
  `QT_LABWC` (Standard `$HOME/opt/labwc-0.9.8/bin/labwc`),
  `QT_RUNTIME_BASIS` (Ort der Laufzeitordner `qterm-runtime-*`, Standard
  `$TMPDIR` bzw. `/tmp`), `CARGO_HOME` (Wayland-Protokolle für
  `bauen.sh`). Die GTK-4-Textprobe meldet sich als `org.querencia.Textprobe`.

## Dateien ins Terminal ziehen (0.3.2, wie GNOME Terminal)

`src/dnd.rs`: ein `GtkDropTarget` je Reiter nimmt eine Dateiliste
(`GdkFileList`, unter Wayland aus `text/uri-list`) oder Text an, nur mit der
Aktion „Kopieren“ (eine Quelle, die verschieben will, löscht so nie etwas).

- Dateien: lokale als Pfad, andere Adressen (sftp://, smb://) als URI;
  jedes Stück in einfachen Anführungszeichen (`'` als `'\''`) und mit einem
  Leerzeichen dahinter. Mehrere Dateien stehen nacheinander.
- Text: unverändert.
- Eingefügt wird mit `Terminal::paste_text` an der Schreibmarke, wie aus der
  Zwischenablage: VTE klammert mit Bracketed Paste, wenn die Shell es
  verlangt (bash ab 5.1); ein gezogener Text mit Zeilenumbruch wird dann
  nicht sofort ausgeführt. Danach bekommt das Terminal den Fokus.
- Mit `QUERENCIA_TERMINAL_DEBUG=1` steht im Log nur die Zahl der Dateien
  bzw. Zeichen, nie der Inhalt.

Prüfung: `tools/dunkel-pruefung/szenario-ziehen.sh` (Querencia Files
isoliert neben dem Terminal, eine und zwei Dateien mit Leerzeichen und
Apostroph; die Shell-Funktion `zeig` schreibt die empfangenen Argumente mit).

## Reiter (0.3.0, wie GNOME Terminal 2.x)

- **Reiterleiste** ab dem zweiten Reiter; Ansicht → „Reiterleiste immer
  anzeigen“ zeigt sie auch mit einem Reiter. Gespeichert beim Umschalten
  in `$XDG_CONFIG_HOME/querencia/terminal.ini` (`[Reiter]
  Leiste=automatisch|immer`, Vorgabe automatisch); andere Gruppen bleiben
  erhalten.
- **Kürzel** (Fenster, Capture-Phase vor VTE, `tabs.rs`):
  Strg+Umschalt+T neuer Reiter, Strg+Umschalt+W Reiter schließen,
  Strg+Umschalt+Q Fenster schließen, Strg+Bild auf/ab vorheriger/nächster
  Reiter, Strg+Umschalt+Bild auf/ab Reiter nach links/rechts, Alt+1 …
  Alt+9 Reiter 1 … 9. Wie bei den gesperrten Aktionen von GNOME Terminal
  2.x greift ein Kürzel nur, wenn sein Befehl möglich ist; sonst geht die
  Taste an die Shell (kein Umlauf am ersten und letzten Reiter; mit einem
  Reiter bekommt die Shell Strg+Bild auf/ab und Alt+Ziffer).
- **Maus:** Mittelklick auf einen Reiter schließt ihn, Doppelklick auf
  die freie Reiterleiste öffnet einen neuen, Ziehen verschiebt Reiter,
  Schließknopf je Reiter. Reiter sind 20 Zeichen breit, damit rechts
  Platz für den Doppelklick bleibt.
- **Titel** aus dem Fenstertitel der Shell (OSC 0/2, VTE
  `window-title-changed`), ohne Titel „Terminal“; der Fenstertitel bleibt
  „Querencia Terminal“.
- **Arbeitsverzeichnis** neuer Reiter: `current-directory-uri` des
  aktuellen Reiters (OSC 7, nur Pfade dieses Rechners), sonst
  `/proc/<shell>/cwd`, sonst das Home-Verzeichnis.
- **Nachfrage beim Schließen**, wenn in einem Reiter außer der Shell ein
  Programm im Vordergrund läuft (Feld `tpgid` in `/proc/<shell>/stat` ≠
  Shell-PID, wie GNOME Terminal 2.x): Reiter bzw. Fenster mit einem Reiter
  „Dieses Terminal schließen?“, Fenster mit mehreren Reitern „Dieses
  Fenster schließen?“; vorgewählt ist „Abbrechen“. Hintergrundjobs zählen
  nicht. Dialoge werden jedes Mal neu gebaut.
- **Menüleiste** Datei | Bearbeiten | Ansicht | Reiter | Hilfe
  (`menus.rs`), Kürzel nur als Anzeige im Menü. F10 und Alt+Buchstabe
  öffnen sie nicht (`handle-menubar-accel` aus, Menüköpfe ohne
  Zugriffstaste), damit Midnight Commander, htop und readline die Tasten
  behalten; in geöffneten Menüs gelten die Zugriffstasten. Nach einem
  Menüeintrag oder Dialog geht der Fokus zurück ins Terminal.
- **Kontextmenü** zusätzlich „Neuer Reiter“ und „Reiter schließen“ unter
  den bisherigen Zeilen.
- **Hilfe → Tastenkürzel** listet alle Kürzel; Hilfe → Info nennt
  Version und Lizenz.

Noch nicht umgesetzt:

- M3.0b: Menü „Terminal“, „Neues Fenster“, Reiter in ein anderes Fenster
  ziehen, Reiterliste im Menü „Reiter“, Suche, Menüleiste ausblenden,
  Zugriffstasten der Menüleiste einschaltbar, Profile im Kontextmenü.
- M3.0c: Profile, Schrift, Farbschemata, gespeicherte Einstellungen,
  Standardterminal (dafür fehlt noch `-e`/`-x` für einen Befehl statt der
  Shell).

## Prüfstand

Stand 20.10.2026 (0.3.0, Reiter), geprüft in der Entwicklungsumgebung:

- 49 Unit-Tests (Reitermodell, Kürzel, Menüs, Zugriffstasten je Sprache,
  Einstellung, Titel, Arbeitsverzeichnis, Vordergrundprozess), fmt und
  Clippy sauber, `i18n.py pruefen` PASS (61 Texte).
- Isolierte GUI-Prüfung `szenario-reiter.sh` (R1–R19, 40 Prüfpunkte) de
  und en, je 1024×768 und 1920×1080: je 40 von 40. Regression der
  0.2.0-Szenarien mit `QT_ZELLE="9 18 1 54"` (das Terminal beginnt wegen
  der Menüleiste 26 px tiefer): Sprache je 9 von 9, Kopieren/Einfügen je
  50 von 51 (P2 wie bei 0.2.0), Schließen 3 von 3 (gleich 0.2.0;
  `szenario-schliessen.sh` klickt jetzt `$SCHLIESSEN_X` statt der festen
  Stelle der alten Zelle 8×17).

Stand 20.10.2026 (0.2.0, deutsch und englisch), gegen VTE 0.84.0 und
GTK 4.22.4 der VM:

- `cargo fmt --all --check`, `cargo build`, `cargo test` (28 Tests) und
  `cargo clippy --all-targets -- -D warnings` laufen ohne Fehler, auch mit
  der lokalen `vte4`-Kopie.
- `python3 werkzeug/i18n.py pruefen`: PASS (10 Texte); `msgfmt -c`
  fehlerfrei.
- Isolierte GUI-Prüfung Kopieren/Einfügen
  (`tools/isolierte-gui-pruefung/pruefen.sh`, Szenario A–P, je mit
  `QT_LANGUAGE=de` und `QT_LANGUAGE=en`, F2 mit der Zugriffstaste der
  Sprache): je 50 von 51;
  offen ist P2 (mittlere Maustaste fügt PRIMARY ein, das ein anderes
  Programm per `wl-copy --primary` gesetzt hat), beim Stand vor der
  App-ID-Umstellung ebenso.
- Sprache (`szenario-sprache.sh`, S1–S8: Katalog in `/proc/<pid>/maps`,
  Kontextmenü, Zugriffstaste je Sprache, Startfehler): de und en, je
  1024×768 und 1920×1080, 9 von 9 (Binary mit `share/locale` daneben).
- Isolierte Sitzung mit Panel und Desktop (`pruefen-appid.sh`): Start aus
  dem Anwendungsmenü, Name und Symbol in Fensterliste und Titelleiste,
  1024×768 und 1920×1080 (Belegbilder, Auswertung von Hand).
- Stand 14.10.2026: Ohne Display endet das Programm mit
  `Failed to open display` und Exit-Code 1.
- Stand 14.10.2026: Die Freigabe der `child_setup`-Daten ist mit isolierten Proben geprüft:
  richtige Größe, genau ein Drop, kein Abbruch in einem fremden Thread.
  Zur Laufzeit lief das über den `Pty`-Pfad, die vier korrigierten
  Funktionen sind wörtlich gleich. `Terminal::spawn_async` selbst ist erst
  mit der GUI-Abnahme zur Laufzeit geprüft.

Die Funktionen an der grafischen Oberfläche sind noch nicht abgenommen.
Die Anleitung dazu steht in [m3-0a-gui-abnahme.md](m3-0a-gui-abnahme.md).

## Bekannte Einschränkungen

- **Titelleiste:** GTK 4 nutzt unter Wayland Serverdekoration, wenn der
  Compositor sie bevorzugt (labwc: KDE-Protokoll `server-decoration`).
  Sonst zeichnet GTK seine eigene Standard-Titelleiste. Die Anwendung
  setzt keine eigene.
- **Fenster schließen bei laufender Shell:** Rückfrage nur, wenn ein
  Programm im Vordergrund läuft (siehe „Reiter“). Beim Schließen wird das
  PTY geschlossen, und der Kernel schickt der Shell SIGHUP. Ob VTE zusätzlich selbst SIGHUP sendet, ist nicht geprüft.
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
src/main.rs    Application, App-ID, Stil
src/tabs.rs    Reitermodell: Befehle, Kürzel, Leiste, Titel, Arbeitsverzeichnis, Vordergrundprozess
src/menus.rs   Menüleiste (Datei, Bearbeiten, Ansicht, Reiter, Hilfe)
src/dialogs.rs Nachfrage beim Schließen, Tastenkürzel, Info
src/i18n.rs    gettext: Textdomäne, Katalogsuche, Platzhalter
po/            querencia-terminal.pot, de.po, it.po, es.po, fr.po, LINGUAS
werkzeug/i18n.py     po-Pflege (pot, update, desktop, mo, pruefen)
data/org.querencia.Terminal.desktop.in  Vorlage der Desktop-Datei (englisch)
data/org.querencia.Terminal.desktop  Desktop-Datei mit [de] (Exec ohne Pfad)
data/org.querencia.Terminal.metainfo.xml  AppStream-MetaInfo
src/window.rs  Fenster, Reiter (Notebook), Aktionen, Maus auf der Leiste, Prozessende
src/clipboard.rs     Kürzel und Zwischenablage (Kopieren, Einfügen)
src/dnd.rs           Dateien und Text ins Terminal ziehen (0.3.2)
src/context_menu.rs  klassisches Kontextmenü
tools/isolierte-gui-pruefung/  isolierte GUI-Prüfung (labwc 0.9.8)
tools/dunkel-pruefung/  isolierte Prüfung hell/dunkel und Ziehen aus Files (eigenes HOME)
integration/   Integration in der Entwicklungsumgebung (App-ID-Umstellung,
               0.2.0 mit Katalog, 0.3.0 Reiter; pruefen|aktivieren|zurueck),
               Vorlagen, Probe; feste Pfade, nicht zur Veröffentlichung
src/shell.rs   Shell-Auswahl, Start über VTE, Fehlermeldung, Wartestatus
vendor/vte4/   lokale Kopie von vte4 0.10.0 mit Patch (Lizenz MIT)
```
