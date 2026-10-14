# M3.0a – Anleitung GUI-Abnahme

Die Abnahme führt nur der Projektinhaber durch. Getestet wird in der
laufenden Classic2-Sitzung (greetd, VT 7) oder in der labwc-Testsitzung
auf tty2, nie in beiden gleichzeitig. Weder die Sitzung noch ihre
Konfiguration werden verändert. Querencia Terminal wird nur als normales
Fenster aus einem vorhandenen Terminal (z. B. foot) gestartet.

Voraussetzung ist ein gebautes Binary: `cargo build` in
`~/projects/querencia-terminal-dev`.

## Start

Im vorhandenen Terminal:

```
cd ~/projects/querencia-terminal-dev
./target/debug/querencia-terminal 2>&1 | tee ~/qt-m3-0a-1.log
```

Das startende Terminal bleibt bis zum Ende blockiert und zeigt das Log.

## Prüfschritte

| Nr. | Aktion | Erwartung |
|---|---|---|
| 1 | Start wie oben | Fenster „Querencia Terminal“ mit labwc-Titelleiste, ohne GTK-Headerbar, Scrollleiste rechts |
| 2 | Sofort tippen, ohne zu klicken | Prompt ist sichtbar, und die Eingabe kommt an. Im Log: `starte /bin/bash in ~` und `läuft als PID …`. Die PID notieren. |
| 3 | `pwd` | `~` |
| 4 | `echo test` | `test` |
| 5 | `tput cols; tput lines` (vor jeder Größenänderung) | `80` und `24` |
| 6 | `ls --color=always /usr/bin \| head` und `for i in 0 1 2 3 4 5 6 7; do printf '\e[3%sm Farbe%s \e[0m' $i $i; done; echo` | farbige Ausgabe |
| 7 | Tippen, Rücktaste, Pfeil links/rechts, Pfeil hoch (Verlauf), `ls /us` + Tab | Bearbeiten der Zeile, Verlauf und Vervollständigung funktionieren |
| 8 | `sleep 100`, dann Ctrl+C; `cat`, dann Ctrl+D | `sleep` bricht ab; `cat` endet, die Shell läuft weiter |
| 9 | `less /etc/passwd`, blättern, `q` | Vollbildprogramm funktioniert |
| 10 | Fenster am Rand in der Größe ändern, dann `tput cols; tput lines` | neue Werte; Inhalt bricht um |
| 11 | `seq 1 3000`, mit Mausrad, Shift+PageUp und Scrollleiste zurückblättern, dann eine Taste drücken | ältere Zeilen sichtbar; Tastendruck springt ans Ende |
| 12 | `exit` | Fenster schließt sich, im Log steht `Shell beendet (Exit-Code 0)`, das startende Terminal zeigt wieder den Prompt |
| 13 | Neu starten, `exit 3` | Log `Shell beendet (Exit-Code 3)` |
| 14 | Neu starten, PID notieren, `sleep 1000 &`, dann das Fenster über die Titelleiste schließen | Fenster zu, Programm beendet; siehe Prozessprüfung |

Prozessprüfung nach Schritt 14 im startenden Terminal (`<PID>` ersetzen):

```
ps -o pid,stat,cmd -p <PID>
pgrep -af 'sleep 1000'
systemctl --user list-units 'vte-spawn-*' --no-legend
```

Erwartung: Alle drei Befehle zeigen nichts mehr an. Das Log enthält
keine `CRITICAL`- oder `WARNING`-Zeilen von GTK oder VTE.

## Fehlerfälle

| Nr. | Start | Erwartung |
|---|---|---|
| F1 | `SHELL=/nicht/vorhanden ./target/debug/querencia-terminal` | Rückfall auf die passwd-Shell: Log `starte /bin/bash …`, Shell nutzbar |
| F2 | `printf '#!/nicht/vorhanden\n' > /tmp/qt-kaputt.sh; chmod +x /tmp/qt-kaputt.sh` und dann `SHELL=/tmp/qt-kaputt.sh ./target/debug/querencia-terminal` | Fenster bleibt offen, rote Zeile `Fehler: /tmp/qt-kaputt.sh konnte nicht gestartet werden: …`, gleiche Meldung im Log; Schließen über die Titelleiste beendet das Programm |
| F3 | `HOME=/nicht/vorhanden ./target/debug/querencia-terminal` | Fenster bleibt offen, Fehlerzeile mit „Failed to change to directory“ |

Danach `/tmp/qt-kaputt.sh` wieder löschen.

## Ergebnis festhalten

Für jede Nummer PASS oder FAIL mit einer kurzen Bemerkung notieren. Dazu
gehören die Logs `~/qt-m3-0a-*.log` und, wo es hilft, ein Screenshot.
Ein Screenshot belegt keine Tastaturbedienung; die Schritte 2, 7 und 8
müssen tatsächlich getippt werden.
