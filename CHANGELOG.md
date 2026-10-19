# Änderungen

## 0.2.0 – 2026-10-20

Englisch und Deutsch über GNU gettext (G2-AH, Muster Querencia Editor 0.2.0):

- Texte der Oberfläche englisch im Code (msgid), Deutsch aus `po/de.po`
  mit dem bisherigen Wortlaut: Kontextmenü („_Copy“, „_Paste“,
  „Select _All“ / „_Kopieren“, „_Einfügen“, „Alles _auswählen“),
  Startfehler im Terminal, Comment und Keywords der Desktop-Datei.
- Zugriffstasten des Kontextmenüs kommen jetzt aus der Übersetzung
  (deutsch K, E, A wie bisher; englisch C, P, A).
- Kataloge neben der Binary (`share/locale`), im Präfix oder in
  `/usr/share/locale`; `QUERENCIA_TERMINAL_LOCALEDIR` für Tests.
- `werkzeug/i18n.py` für die po-Pflege; Desktop-Datei aus
  `data/org.querencia.Terminal.desktop.in` erzeugt (wie `msgfmt --desktop`).
- AppStream-MetaInfo `data/org.querencia.Terminal.metainfo.xml` mit der
  alten ID `org.example.Querencia.Terminal` unter `<provides>`.
- Lizenzkopf in `LICENSE` im README (Copyright „Querencia
  Desktop contributors“).
- Isolierte GUI-Prüfung: Sprache wählbar (`QT_LANGUAGE`), Zugriffstaste
  „Einfügen“ je Sprache.

## appid-0.1 – 2026-10-20

App-ID `org.querencia.Terminal` statt `org.example.Querencia.Terminal`
(Entscheidung 7); Desktop-ID und `StartupWMClass` gleich. Integration mit
Brücke unter der alten Desktop-ID (`NoDisplay`), weil der Desktop sie noch
fest kennt.

## Kopieren und Einfügen – 2026-10-17

Strg+Umschalt+C/V, Umschalt+Einfg und ein klassisches Kontextmenü wie im
GNOME Terminal 2.x.

## 0.1.0 (M3.0a) – 2026-10-14

Erste Fassung: GTK-4-Fenster mit VTE-Terminal, Shell-Start über VTE,
Scrollleiste, 80×24, lokaler `vte4`-Patch.
