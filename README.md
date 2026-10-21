# Querencia Terminal

Terminalemulator für alle, die Querencia Desktop benutzen. Teil von
[Querencia Desktop](https://github.com/querencia-desktop/querencia) – einer
klassischen Desktopumgebung im GNOME-2-Stil für Wayland (labwc, Rust, GTK 4).

*English summary:* A terminal emulator in the style of MATE Terminal and GNOME
Terminal 2, built with GTK 4 and VTE. Part of Querencia Desktop, a classic
GNOME-2-style desktop for Wayland. Alpha software; documentation is in German,
the user interface is available in English and German.

> **Alpha.** Stand 0.3.0: Fenster mit Menüleiste und Reitern, Shell,
> Kopieren/Einfügen und Kontextmenü; Profile, Suche und Farbschemata fehlen
> noch. Keine Sicherheits- oder Datenverlustgarantie.

## Zweck

- Ein Fenster je Start mit VTE-Terminal (80×24, 10 000 Zeilen Scrollback),
  Scrollleiste rechts wie bei MATE, Shell-Auswahl wie MATE Terminal
  (`$SHELL`, Login-Shell, `/bin/sh`), Startfehler rot im Terminal.
- Kopieren und Einfügen wie GNOME Terminal 2.x (Strg+Umschalt+C/V,
  Umschalt+Einfg, PRIMARY über die mittlere Maustaste) und ein klassisches
  Kontextmenü.
- Reiter wie GNOME Terminal 2.x (Strg+Umschalt+T/W, Strg+Bild auf/ab,
  Alt+1 … Alt+9, Reiterleiste ab dem zweiten Reiter) und Menüleiste
  Datei | Bearbeiten | Ansicht | Reiter | Hilfe; Nachfrage beim Schließen,
  wenn noch ein Programm im Vordergrund läuft.
- Oberfläche auf Englisch und Deutsch über GNU gettext (englische msgid,
  `po/de.po`); Auswahl über `LANGUAGE`/`LANG`.
- Bewusst **noch nicht** Teil dieses Programms: Suche, Profile,
  Farbschemata, `-e`/`-x` (Befehl statt Shell), Einrichtung als
  Standardterminal.

Anwendungskennung (App-ID): `org.querencia.Terminal` (bis Oktober 2026
`org.example.Querencia.Terminal`; die MetaInfo nennt die alte ID als
`<provides>`) · Binary: `querencia-terminal` · Debian-Paket:
`querencia-terminal` (Paketierung in `querencia-packaging`)

| Datei | Inhalt |
|---|---|
| `src/` | Application und App-ID, Fenster, Reiter, Menüs und Dialoge, Shell-Start, Zwischenablage, Kontextmenü, Übersetzung |
| `po/`, `werkzeug/i18n.py` | Übersetzungen und ihre Pflege ohne GNU gettext |
| `data/` | Desktop-Datei (aus `.desktop.in`), AppStream-MetaInfo |
| `vendor/vte4/` | lokale, korrigierte Kopie von `vte4` 0.10.0 (MIT), siehe [docs/technik.md](docs/technik.md) |
| `tools/isolierte-gui-pruefung/` | isolierte GUI-Prüfung (labwc 0.9.8, virtuelle Eingabe, Szenarien) |
| `docs/` | Technik, Patch, Umfang, Prüfstand, Einschränkungen (`technik.md`); Anleitung GUI-Abnahme |

## Stand

| | |
|---|---|
| Version | 0.3.0 mit Reitern (Tag `terminal/0.3.0`); davor 0.2.0 (Tags `terminal/0.2.0`, `terminal/0.2.0-1`) |
| Letzte Prüfung | 20.10.2026 (0.3.0): 49 Unit-Tests, `i18n.py pruefen` (61 Texte); isolierte GUI-Prüfung Reiter (R1–R19) de/en bei 1024×768 und 1920×1080, Regression der 0.2.0-Szenarien (Kopieren/Einfügen 50 von 51, Sprache 9 von 9, Schließen 3 von 3) |
| Bekannte Lücken | P2 (mittlere Maustaste mit PRIMARY eines anderen Programms) offen; weitere in [docs/technik.md](docs/technik.md) |
| Getestet auf | Ubuntu 26.04, labwc 0.9.8, GTK 4.22, VTE 0.84, Rust 1.93.1 |
| Nicht getestet | andere Distributionen, HiDPI, Mehrmonitor, X11 |

Änderungen je Version: [CHANGELOG.md](CHANGELOG.md).

## Bauen

Voraussetzungen (Ubuntu 26.04):

```bash
sudo apt install build-essential pkg-config rustc cargo libgtk-4-dev libvte-2.91-gtk4-dev
```

```bash
cargo build --release --locked
```

Das Ergebnis liegt unter `target/release/querencia-terminal`; es braucht eine
laufende Wayland-Sitzung. `Cargo.toml` ersetzt `vte4` per
`[patch.crates-io]` durch `vendor/vte4/` (Grund und Rückkehrweg:
[docs/technik.md](docs/technik.md)). Kataloge: `python3 werkzeug/i18n.py mo
<ziel>`; ohne `de.mo` zeigt das Programm Englisch.

## Prüfen

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
python3 werkzeug/i18n.py pruefen
```

Diese Befehle laufen ohne Bildschirm; die ersten drei sind auch die CI
(`.github/workflows/ci.yml`). Die **GUI-Prüfung** läuft in einer
abgeschotteten Sitzung (eigener labwc ohne Bildschirm, eigenes `HOME` und
`XDG_RUNTIME_DIR`), niemals in der laufenden Desktopsitzung:

```bash
bash tools/isolierte-gui-pruefung/pruefen.sh <binary> <profil> szenario-zwischenablage.sh 300
```

Szenarien und Prüfstand: [docs/technik.md](docs/technik.md), Abnahme von Hand:
[docs/m3-0a-gui-abnahme.md](docs/m3-0a-gui-abnahme.md). Arbeitsweise:
[CONTRIBUTING.md](CONTRIBUTING.md).

## Vorbild und Abgrenzung

Vorbild für Bedienung und Aussehen ist **MATE Terminal bzw. GNOME Terminal
2.30 aus GNOME 2.30**. Querencia übernimmt davon die Bedienideen (Kürzel,
Kontextmenü, Shell-Auswahl), **keinen Quellcode, keine Texte, keine
Grafiken**; es wird kein C-Code portiert. Terminalemulation übernimmt VTE. Das
Projekt ist mit MATE, GNOME und deren Autoren weder verbunden noch von ihnen
unterstützt.

Abweichungen vom Vorbild und Dinge, die bewusst fehlen:

- GTK 4 und VTE für GTK 4, keine Headerbar, kein libadwaita; Titelleiste vom
  Compositor (labwc).
- Umschalt+Einfg liest CLIPBOARD (GNOME Terminal 2.x: PRIMARY).
- Noch keine Profile, Suche und Farbschemata (siehe „Zweck“).

Fremdbestandteile: `vendor/vte4/` (vte4 0.10.0 der gtk-rs-Entwickler, MIT,
eigene `LICENSE`, Querencia-Änderung mit „Querencia patch“ markiert; Meldung
an Upstream steht aus); Rust-Abhängigkeiten (gtk4-rs, gettext-rs und Umfeld)
unter MIT, MIT OR Apache-2.0 und verwandten freien Lizenzen, Liste mit
`cargo metadata --locked`. Dynamisch gelinkt: GTK 4, VTE (LGPL-3.0-or-later)
und das gettext der glibc. In `tools/isolierte-gui-pruefung/` liegt eine
gekürzte Fassung des wlroots-Protokolls `virtual-keyboard-unstable-v1.xml`
mit eigenem Vermerk (© 2008–2018 Kristian Høgsberg, Intel Corporation,
Collabora, Ltd., Purism SPC; MIT, siehe Dateikopf).

## Lizenz

Programm: GPL-3.0-or-later, siehe [LICENSE](LICENSE). Copyright 2026
Querencia Desktop contributors; Urheberhinweis in
[COPYRIGHT](COPYRIGHT). `vendor/vte4/` behält seine MIT-Lizenz
(`vendor/vte4/LICENSE`), die Protokolldatei ihren eigenen Vermerk; die
MetaInfo-Datei steht unter FSFAP.

Name und Logo „Querencia“ sind keine freien Inhalte: [MARKE.md](MARKE.md).

## Fehler melden

Fehler, Wünsche und Fragen zu Querencia gehen an die gemeinsame Meldestelle:
<https://github.com/querencia-desktop/querencia/issues/new/choose>.
Sicherheitslücken bitte nicht öffentlich melden, siehe [SECURITY.md](SECURITY.md).
