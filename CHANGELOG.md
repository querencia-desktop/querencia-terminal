# Änderungen

## 0.3.2 – 2026-10-22

Dateien ins Terminal ziehen (QD-125):

- Dateien aus Querencia Files (oder einem anderen Programm) ins
  Terminalfenster ziehen fügt ihre Pfade an der Schreibmarke ein, wie im
  GNOME Terminal: jeder Pfad in einfachen Anführungszeichen und mit einem
  Leerzeichen dahinter, mehrere nacheinander. Andere Adressen (z. B.
  sftp://) bleiben als Adresse, gezogener Text wird unverändert eingefügt.
- Eingefügt wird wie beim Einfügen aus der Zwischenablage (Bracketed Paste):
  ein gezogener Text mit Zeilenumbruch wird nicht sofort ausgeführt.
- Das Terminal nimmt nur „Kopieren“ an; Files verschiebt beim Ziehen ins
  Terminal nie eine Datei.
- Isolierte Prüfung `tools/dunkel-pruefung/szenario-ziehen.sh` mit Files.

## 0.3.1 – 2026-10-22

Dunkelmodus und Hilfe:

- Menüleiste, Reiterleiste, die Menüs der Menüleiste und das Kontextmenü
  folgen dem Dunkelmodus: Flächen, Schrift und Ränder kommen als benannte
  Farben aus dem GTK-Thema (`@theme_bg_color`, `@theme_fg_color`,
  `@theme_base_color`, `@menu_bg_color`, `@menu_fg_color`, `@borders`),
  ohne eigenen `@media`-Block. Unter Querencia-Klassisch hell bleiben die
  Flächen pixelgleich; nur Trennlinie und Untermenüpfeil im Kontextmenü
  sind aus Themenfarben gemischt (#dbd8d4 statt #dcd8d3, #686868 statt
  #686463). Der Terminalbereich selbst (VTE) ist unverändert.
- Hilfe-Menü: neu „Online-Hilfe“ (https://querencialinux.org/de/docs/ bzw.
  /en/docs/, je nach Sprache der Oberfläche) und „Fehler melden“
  (Meldestelle auf GitHub); beide öffnen den Browser.
- Info-Fenster: „Alpha – nicht für den Alltag freigegeben“, Verweis auf
  https://querencialinux.org und „© 2026 Querencia Desktop contributors“.
- MetaInfo: Homepage https://querencialinux.org.
- 5 neue Texte englisch/deutsch (66 insgesamt).
- Isolierte Prüfung hell/dunkel: `tools/dunkel-pruefung/` (eigenes HOME,
  Farbschema per gsettings im Profil, Portal-Ersatz, Adress-Probe statt
  Browser, Auswertung der Farben).

## 0.3.0 – 2026-10-20

Reiter wie im GNOME Terminal 2.x (Anlass: Werkstatt QREL-06 A4.4, QD-079):

- Reiterleiste ab dem zweiten Reiter; Ansicht → „Reiterleiste immer
  anzeigen“ zeigt sie auch mit einem Reiter (gespeichert in
  `~/.config/querencia/terminal.ini`, `[Reiter] Leiste=automatisch|immer`).
- Strg+Umschalt+T neuer Reiter, Strg+Umschalt+W Reiter schließen,
  Strg+Umschalt+Q Fenster schließen, Strg+Bild auf/ab vorheriger/nächster
  Reiter, Strg+Umschalt+Bild auf/ab Reiter verschieben, Alt+1 … Alt+9
  Reiter direkt. Wie bei GNOME Terminal 2.x ohne Umlauf an den Enden; ist
  ein Befehl gerade nicht möglich (z. B. mit nur einem Reiter), geht die
  Taste an die Shell.
- Mittelklick auf einen Reiter schließt ihn, Doppelklick auf die freie
  Reiterleiste öffnet einen neuen, Reiter mit der Maus verschiebbar,
  Schließknopf je Reiter.
- Reitertitel aus dem Fenstertitel der Shell; ohne Titel „Terminal“. Der
  Fenstertitel bleibt „Querencia Terminal“.
- Neue Reiter beginnen im Arbeitsverzeichnis des aktuellen (VTE
  `current-directory-uri`, sonst `/proc/<shell>/cwd`).
- Nachfrage beim Schließen, wenn in einem Reiter außer der Shell noch ein
  Programm im Vordergrund läuft („Dieses Terminal schließen?“ bzw. „Dieses
  Fenster schließen?“). Hintergrundjobs zählen nicht. Bis 0.2.0 schloss
  das Fenster ohne Frage. Der letzte Reiter schließt das Fenster.
- Menüleiste Datei | Bearbeiten | Ansicht | Reiter | Hilfe mit
  „Neuer Reiter“, „Reiter schließen“, „Fenster schließen“, Kopieren,
  Einfügen, Alles auswählen, Reiterleiste, Vorheriger/Nächster Reiter,
  Reiter nach links/rechts verschieben, Tastenkürzel, Info. F10 und
  Alt+Buchstabe bleiben bei der Shell (Midnight Commander, readline).
- Kontextmenü mit „Neuer Reiter“ und „Reiter schließen“ unter den
  bisherigen Zeilen (Zugriffstasten deutsch R, S; englisch T, L).
- Hilfe → Tastenkürzel listet alle Kürzel.
- 51 neue Texte englisch/deutsch (61 insgesamt).
- Isolierte GUI-Prüfung: `szenario-reiter.sh` (R1–R19), `qeingabe` kennt
  Bild auf/ab.
- Farbschema: einmal nach dem Start `gtk-interface-color-scheme` neu
  melden, damit GTK 4.22 bei `prefer-dark` das dunkle CSS auch für ein neu
  gestartetes Programm lädt (QD-081; betrifft Menü- und Reiterleiste,
  nicht die Farben von VTE).

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
