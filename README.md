# Querencia Terminal

Terminalemulator für Querencia Desktop, eine Desktopumgebung im Stil von
GNOME 2 und MATE. Vorbild für Aussehen und Bedienung ist MATE Terminal
(Referenz: Fork `pottrauschen/mate-terminal`). Es wird kein C-Code
portiert.

> **Alpha.** Entwicklungsstufe. Keine Sicherheits- oder
> Datenverlustgarantie.

Das Repository ist eigenständig und vom Desktop-Workspace getrennt. Die
Desktop-Datei liegt unter `data/org.querencia.Terminal.desktop`
(erzeugt aus `data/org.querencia.Terminal.desktop.in`); eine Installation als
Paket und eine Einrichtung als Standardterminal (`x-terminal-emulator`)
gibt es noch nicht.

## Stand

| | |
|---|---|
| Version | 0.3.1: Menü- und Reiterleiste folgen dem Dunkelmodus, Hilfe-Menü mit Online-Hilfe und Fehlermeldung (Tag `terminal/0.3.1`); davor 0.3.0 mit Reitern (Tag `terminal/0.3.0`), 0.2.0 (Tags `terminal/0.2.0`, `terminal/0.2.0-1`), App-ID `org.querencia.Terminal` (Tag `terminal/appid-0.1-2`) |
| Auf der Entwicklungs-VM | 0.3.0 live; 0.3.1 geprüft, wartet auf Freigabe |
| Letzte Prüfung | 22.10.2026 (0.3.1): 52 Unit-Tests, `i18n.py pruefen` (66 Texte); isolierte Prüfung hell/dunkel mit drei Reitern (`tools/dunkel-pruefung/`) de/en, Wechsel im laufenden Programm, Gegenprobe 0.3.0 |
| Bekannte Lücken | P2 (mittlere Maustaste mit PRIMARY eines anderen Programms) offen; weitere in [docs/technik.md](docs/technik.md) |
| Getestet auf | Ubuntu 26.04, labwc 0.9.8, GTK 4.22, VTE 0.84, Rust 1.93.1 |

Änderungen je Version: [CHANGELOG.md](CHANGELOG.md).

## Bauen und Prüfen

`cargo build --locked`, danach `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked` und `python3 werkzeug/i18n.py pruefen`.
Voraussetzungen, lokaler vte4-Patch, Sprachen, Umfang, Prüfstand und isolierte GUI-Prüfung: [docs/technik.md](docs/technik.md); Abnahme von Hand: [docs/m3-0a-gui-abnahme.md](docs/m3-0a-gui-abnahme.md).

## Lizenz

GPL-3.0-or-later, siehe [LICENSE](LICENSE) (reiner Lizenztext). Copyright
2026 Querencia Desktop contributors; Urheberhinweis in
[COPYRIGHT](COPYRIGHT).

`vendor/vte4/` behält seine eigene MIT-Lizenz (`vendor/vte4/LICENSE`); die
Querencia-Änderung darin ist mit „Querencia patch“ markiert und in
[docs/technik.md](docs/technik.md) unter „Lokaler vte4-Patch“ beschrieben. Die MetaInfo-Datei steht unter FSFAP.

