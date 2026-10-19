#!/bin/bash
# Querencia Terminal: Kopieren und Einfügen, Prüfungen A–P (siehe Bericht).
# Läuft in der isolierten Sitzung (pruefen.sh -> isoliert.py). Jede Prüfung
# vergleicht echte Inhalte: CLIPBOARD/PRIMARY über wl-paste, eingefügten
# Text über Dateien, die die Shell im Terminal (cat) bzw. foot/Textprobe
# schreiben.
. "$HIER/lib.sh"

# Kontextmenü: Rechtsklick hier, Zeilenmitten relativ dazu (aus Bild E1)
RX=405; RY=205
MX=$((RX + 40)); M_KOPIEREN=$((RY + 14)); M_EINFUEGEN=$((RY + 38)); M_ALLES=$((RY + 69))

soll() { printf "$2" > "$OUT/soll-$1.txt"; }
soll A 'Grüße aus Köln: äöüß ÄÖÜ – 10 € „Zitat“\nZweite Zeile mit  zwei Leerzeichen\nDritte Zeile Ende'
soll E 'Zweite Zeile mit  zwei Leerzeichen'

notiz "Start $(date '+%F %T'), WAYLAND_DISPLAY=$WAYLAND_DISPLAY, XKB_DEFAULT_LAYOUT=$XKB_DEFAULT_LAYOUT"
wl-copy --clear
start_terminal
notiz "Terminal-PID $QT_PID"

# --- P4/P5: Bestand: Größe 80x24, Tab-Vervollständigung, Ausgabe
tippen 'tput cols > $OUT/groesse; tput lines >> $OUT/groesse'
printf '80\n24\n' > "$OUT/soll-groesse"
vergleich "P4 Terminalgröße 80x24 (tput)" "$OUT/groesse" "$OUT/soll-groesse"
q "type echo /us
key Tab
wait 400
type  > \$OUT/tab.txt
key Return
wait 500"
printf '/usr/\n' > "$OUT/soll-tab"
vergleich "P5 Tab-Vervollständigung (echo /us<Tab>)" "$OUT/tab.txt" "$OUT/soll-tab"

# --- T: was erreicht die Shell? (Rohmodus, erstes Byte; danach ein x)
# CLIPBOARD leer, PRIMARY = "P": so zeigt sich auch, was Umschalt+Einfg liest.
wl-copy --clear; wl-copy --primary 'P'
roh_pruefung() {  # <name> <qeingabe-akkord> <soll-hex> <text>
  tippen "roh $1"
  # Kam schon ein Byte an, landet das x an der Eingabezeile: Strg+U löscht es.
  q "$2
wait 400
type x
wait 400
chord ctrl u
wait 200"
  printf '%s\n' "$3" > "$OUT/soll-$1.roh"
  vergleich "$4" "$OUT/$1.roh" "$OUT/soll-$1.roh"
}
roh_pruefung t1 'chord ctrl c' 03 "T1 Strg+C sendet ETX (03) wie bisher"
roh_pruefung t2 'chord ctrl v' 16 "T2 Strg+V sendet SYN (16) wie bisher"
roh_pruefung t3 'chord ctrl+shift c' 78 "T3 Strg+Umschalt+C erreicht die Shell nicht"
roh_pruefung t4 'chord ctrl+shift v' 78 "T4 Strg+Umschalt+V (leere Zwischenablage) erreicht die Shell nicht"
roh_pruefung t5 'chord shift Insert' 78 "T5 Umschalt+Einfg liest CLIPBOARD (leer), nicht PRIMARY"
roh_pruefung t6 'chord ctrl d' 04 "T6 Strg+D sendet EOT (04) wie bisher"

# --- A: markieren (Maus), Hervorhebung, PRIMARY getrennt von CLIPBOARD
wl-copy 'VORHER-A'
tippen clear; tippen zeige
ziehen 0 1 16 3
q "shot $OUT/A-markiert.png"
klippe > "$OUT/A-clipboard.txt"; printf 'VORHER-A' > "$OUT/soll-A-clipboard.txt"
vergleich "A1 Markieren allein ändert CLIPBOARD nicht" "$OUT/A-clipboard.txt" "$OUT/soll-A-clipboard.txt"
primaer > "$OUT/A-primary.txt"
vergleich "A2 Markierung liegt in PRIMARY (3 Zeilen, Umlaute)" "$OUT/A-primary.txt" "$OUT/soll-A.txt"

# --- B/C: Strg+Umschalt+C, Inhalt der Zwischenablage
q "chord ctrl+shift c
wait 500"
klippe > "$OUT/B-clipboard.txt"
vergleich "B/C Strg+Umschalt+C: CLIPBOARD == Markierung (Zeilenumbrüche, UTF-8)" "$OUT/B-clipboard.txt" "$OUT/soll-A.txt"
grep -q 'Grüße aus Köln: äöüß ÄÖÜ – 10 € „Zitat“' "$OUT/B-clipboard.txt" && notiz "H1 Umlaute/€/„“ in CLIPBOARD byte-genau"

# --- D: Strg+Umschalt+V (mehrzeilig) in cat
tippen 'ziel d'
q "chord ctrl+shift v
wait 600
key Return
chord ctrl d
wait 500"
{ cat "$OUT/soll-A.txt"; echo; } > "$OUT/soll-d.txt"
vergleich "D Strg+Umschalt+V: eingefügt == CLIPBOARD (+Enter), I mehrzeilig, H Umlaute" "$OUT/d.txt" "$OUT/soll-d.txt"
echo 0 > "$OUT/soll-rc0"; vergleich "D cat regulär beendet (rc 0)" "$OUT/d.rc" "$OUT/soll-rc0"

# --- E: Kopieren über das Kontextmenü
wl-copy 'VORHER-E'
tippen clear; tippen zeige
ziehen 0 2 33 2
q "move $RX $RY
wait 300
rclick
wait 700
shot $OUT/E1-menue-mit-auswahl.png
move $MX $M_KOPIEREN
wait 400
shot $OUT/E2-menue-kopieren-hervorgehoben.png
click
wait 500"
klippe > "$OUT/E-clipboard.txt"
vergleich "E Kontextmenü Kopieren: CLIPBOARD == Markierung (2 Leerzeichen erhalten)" "$OUT/E-clipboard.txt" "$OUT/soll-E.txt"
ergebnis "E1 Bild: Kopieren wählbar" zeile_ist "$OUT/E1-menue-mit-auswahl.png" $MX $M_KOPIEREN aktiv
ergebnis "E1 Bild: Einfügen wählbar" zeile_ist "$OUT/E1-menue-mit-auswahl.png" $MX $M_EINFUEGEN aktiv

# --- E3: Kontextmenü per Tastatur (Umschalt+F10, erste Zeile, Enter)
wl-copy 'VORHER-E3'
ziehen 0 3 16 3
q "chord shift F10
wait 700
shot $OUT/E3-menue-tastatur.png
key Return
wait 500"
printf 'Dritte Zeile Ende' > "$OUT/soll-E3.txt"; klippe > "$OUT/E3-clipboard.txt"
vergleich "E3 Umschalt+F10, Enter: Kopieren" "$OUT/E3-clipboard.txt" "$OUT/soll-E3.txt"

# --- E4: Alles auswählen über das Kontextmenü, dann Strg+Umschalt+C
q "move $RX $RY
wait 300
rclick
wait 700
move $MX $M_ALLES
wait 400
click
wait 400
shot $OUT/E4-alles-ausgewaehlt.png
chord ctrl+shift c
wait 500"
klippe > "$OUT/E4-clipboard.txt"
ergebnis "E4 Alles auswählen: CLIPBOARD enthält den ganzen Inhalt" bash -c "grep -q '^\\$ zeige' '$OUT/E4-clipboard.txt' && grep -q '^Grüße aus Köln' '$OUT/E4-clipboard.txt' && grep -q '^Dritte Zeile Ende' '$OUT/E4-clipboard.txt'"

# --- F: Einfügen über das Kontextmenü
printf 'Kontextmenü: Größe ändern\nZeile  zwei' | wl-copy
tippen 'ziel f'
q "move $RX $RY
wait 300
rclick
wait 700
shot $OUT/F1-menue-einfuegen.png
move $MX $M_EINFUEGEN
wait 400
click
wait 600
key Return
chord ctrl d
wait 500"
printf 'Kontextmenü: Größe ändern\nZeile  zwei\n' > "$OUT/soll-f.txt"
vergleich "F Kontextmenü Einfügen" "$OUT/f.txt" "$OUT/soll-f.txt"

# --- F2: Menü-Taste, Zugriffstaste von „Einfügen“ (de E, en P)
printf 'Zugriffstaste E: Äußerst' | wl-copy
tippen 'ziel f2'
q "key Menu
wait 700
key $TASTE_EINFUEGEN
wait 600
key Return
chord ctrl d
wait 500"
printf 'Zugriffstaste E: Äußerst\n' > "$OUT/soll-f2.txt"
vergleich "F2 Menü-Taste + $TASTE_EINFUEGEN: Einfügen" "$OUT/f2.txt" "$OUT/soll-f2.txt"

# --- G: Umschalt+Einfg fügt CLIPBOARD ein (PRIMARY enthält etwas anderes)
printf 'Umschalt+Einfg: ÄÖÜ äöü ß' | wl-copy
primaer > "$OUT/G-primary.txt"
tippen 'ziel g'
q "chord shift Insert
wait 600
key Return
chord ctrl d
wait 500"
printf 'Umschalt+Einfg: ÄÖÜ äöü ß\n' > "$OUT/soll-g.txt"
vergleich "G Umschalt+Einfg fügt CLIPBOARD ein" "$OUT/g.txt" "$OUT/soll-g.txt"
cmp -s "$OUT/G-primary.txt" "$OUT/soll-g.txt" && notiz "G: PRIMARY war gleich CLIPBOARD (Prüfung nicht trennscharf)"

# --- I: mehrzeilig an der Shell-Eingabe: Bracketed Paste, nichts läuft ohne Enter
rm -f "$OUT/b1" "$OUT/b2"
printf 'touch "$OUT/b1"\ntouch "$OUT/b2"' | wl-copy
tippen clear
q "chord ctrl+shift v
wait 1500
shot $OUT/I1-eingefuegt-nicht-ausgefuehrt.png"
ergebnis "I1 mehrzeiliger Text an der Eingabe nicht vorzeitig ausgeführt" test ! -e "$OUT/b1" -a ! -e "$OUT/b2"
q "key Return
wait 1000"
ergebnis "I2 nach Enter beide Zeilen ausgeführt" test -e "$OUT/b1" -a -e "$OUT/b2"

# --- J: Strg+C bricht einen laufenden Befehl ab
tippen clear
tippen 'sleep 60'
q "wait 800
chord ctrl c
wait 600"
tippen 'rc j'
echo 130 > "$OUT/soll-rc130"
vergleich "J Strg+C bricht sleep ab (rc 130 = SIGINT)" "$OUT/j.rc" "$OUT/soll-rc130"
ergebnis "J kein sleep 60 mehr in der Testsitzung" bash -c '! pgrep -f "^sleep 60$" -u "$(id -u)" -a | while read pid rest; do tr "\0" "\n" < /proc/$pid/environ | grep -qx "XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR" && echo $pid; done | grep -q .'

# --- K: Kopieren ohne Auswahl
wl-copy 'VORHER-K'
tippen clear
q "move $(cx 40) $(cy 15)
click
wait 300"
tippen 'ziel k'
q "chord ctrl+shift c
wait 500
chord ctrl d
wait 500"
printf 'VORHER-K' > "$OUT/soll-K.txt"; klippe > "$OUT/K-clipboard.txt"
vergleich "K1 Strg+Umschalt+C ohne Auswahl: CLIPBOARD unverändert" "$OUT/K-clipboard.txt" "$OUT/soll-K.txt"
: > "$OUT/soll-leer.txt"
vergleich "K2 kein Zeichen an die Shell (cat las nichts)" "$OUT/k.txt" "$OUT/soll-leer.txt"
vergleich "K3 kein Strg+C ausgelöst (cat rc 0)" "$OUT/k.rc" "$OUT/soll-rc0"
q "move $RX $RY
wait 300
rclick
wait 700
shot $OUT/K4-menue-ohne-auswahl.png
key Escape
wait 400"
grep -q 'Kontextmenü offen (Maus): Kopieren=aus, Einfügen=an' "$LOGS/terminal.log" \
  && echo "PASS K4 Menü ohne Auswahl: Kopieren aus, Einfügen an (Log)" >>"$ERG" \
  || echo "FAIL K4 Menüzustand ohne Auswahl" >>"$ERG"
ergebnis "K5 Bild: Kopieren ausgegraut" zeile_ist "$OUT/K4-menue-ohne-auswahl.png" $MX $M_KOPIEREN grau
ergebnis "K6 Bild: Einfügen wählbar" zeile_ist "$OUT/K4-menue-ohne-auswahl.png" $MX $M_EINFUEGEN aktiv

# --- L: Einfügen bei leerer Zwischenablage
wl-copy --clear; sleep 0.5
if wl-paste -n >/dev/null 2>&1; then notiz "L: Zwischenablage nicht leer?"; fi
tippen 'ziel l'
q "chord ctrl+shift v
wait 500
chord shift Insert
wait 500
chord ctrl d
wait 500"
vergleich "L1 leere Zwischenablage: nichts eingefügt" "$OUT/l.txt" "$OUT/soll-leer.txt"
vergleich "L2 cat regulär beendet" "$OUT/l.rc" "$OUT/soll-rc0"
q "move $RX $RY
wait 300
rclick
wait 700
shot $OUT/L3-menue-leere-zwischenablage.png
move $MX $M_EINFUEGEN
wait 300
click
wait 400
key Escape
wait 300"
grep -q 'Kontextmenü offen (Maus): Kopieren=aus, Einfügen=aus' "$LOGS/terminal.log" \
  && echo "PASS L3 Menü bei leerer Zwischenablage: Einfügen aus (Log)" >>"$ERG" \
  || echo "FAIL L3 Menüzustand leere Zwischenablage" >>"$ERG"
ergebnis "L4 Bild: Einfügen ausgegraut" zeile_ist "$OUT/L3-menue-leere-zwischenablage.png" $MX $M_EINFUEGEN grau
ergebnis "L5 Bild: Alles auswählen wählbar" zeile_ist "$OUT/L3-menue-leere-zwischenablage.png" $MX $M_ALLES aktiv
vergleich "L6 Klick auf ausgegrautes Einfügen fügt nichts ein" "$OUT/l.txt" "$OUT/soll-leer.txt"

# --- M: Terminal -> andere Wayland-Programme (GTK-4-Textprobe, foot)
tippen clear; tippen zeige
ziehen 0 1 38 1
q "chord ctrl+shift c
wait 500"
printf 'Grüße aus Köln: äöüß ÄÖÜ – 10 € „Zitat“' > "$OUT/soll-M.txt"
klippe > "$OUT/M-clipboard.txt"; vergleich "M0 CLIPBOARD == Zeile 1" "$OUT/M-clipboard.txt" "$OUT/soll-M.txt"
"$TEXTPROBE" "$OUT/m-textprobe.txt" >>"$LOGS/textprobe.log" 2>&1 &
TP1=$!
sleep 2
q "chord ctrl v
wait 800
shot $OUT/M1-textprobe.png"
vergleich "M1 Terminal -> GTK-4-Textprobe (Strg+V)" "$OUT/m-textprobe.txt" "$OUT/soll-M.txt"
kill $TP1; sleep 0.5
foot --window-size-pixels=500x360 bash --noprofile --norc -c "cat > '$OUT/m-foot.txt'" >>"$LOGS/foot.log" 2>&1 &
sleep 2
q "chord ctrl+shift v
wait 800
key Return
chord ctrl d
wait 800"
{ cat "$OUT/soll-M.txt"; echo; } > "$OUT/soll-m-foot.txt"
vergleich "M2 Terminal -> foot (Strg+Umschalt+V in foot)" "$OUT/m-foot.txt" "$OUT/soll-m-foot.txt"

# --- N: andere Wayland-Programme -> Terminal
"$TEXTPROBE" "$OUT/n-textprobe.txt" "$P/textprobe-vorgabe.txt" >>"$LOGS/textprobe.log" 2>&1 &
TP2=$!
sleep 2
q "chord ctrl c
wait 600"
klippe > "$OUT/N-clipboard.txt"
vergleich "N0 Textprobe Strg+C -> CLIPBOARD" "$OUT/N-clipboard.txt" "$P/textprobe-vorgabe.txt"
q "move $(cx 40) $(cy 20)
click
wait 300"
tippen 'ziel n'
q "chord ctrl+shift v
wait 600
key Return
chord ctrl d
wait 500"
{ cat "$P/textprobe-vorgabe.txt"; echo; } > "$OUT/soll-n.txt"
vergleich "N1 GTK-4-Textprobe -> Terminal" "$OUT/n.txt" "$OUT/soll-n.txt"
kill $TP2; sleep 0.5
foot --window-size-pixels=500x360 bash --noprofile --norc -c "printf 'Aus foot: Äpfel und Birnen\n'; sleep 40" >>"$LOGS/foot.log" 2>&1 &
FOOT=$!
sleep 2
q "move 763 366
wait 200
press
glide 990 366 400
release
wait 300
chord ctrl+shift c
wait 600
shot $OUT/N2-foot-markiert.png"
klippe > "$OUT/N2-clipboard.txt"
notiz "N2 CLIPBOARD aus foot: $(cat "$OUT/N2-clipboard.txt" | head -c 60)"
q "move $(cx 40) $(cy 20)
click
wait 300"
tippen 'ziel n2'
q "chord ctrl+shift v
wait 600
key Return
chord ctrl d
wait 500"
{ cat "$OUT/N2-clipboard.txt"; echo; } > "$OUT/soll-n2.txt"
vergleich "N2 foot -> Terminal (== CLIPBOARD aus foot)" "$OUT/n2.txt" "$OUT/soll-n2.txt"
grep -q '^Aus foot: Äpfel' "$OUT/n2.txt" && echo "PASS N2 Text aus foot im Terminal angekommen" >>"$ERG" || echo "FAIL N2 Text aus foot" >>"$ERG"
kill $FOOT 2>/dev/null

# --- P: Bestandsfunktionen: Strg+V bleibt Steuerzeichen, mittlere Taste fügt PRIMARY ein
wl-copy 'CLIPBOARD-P'
tippen clear
tippen 'ziel p'
q "chord ctrl v
type x
key Return
chord ctrl d
wait 500"
printf 'x\n' > "$OUT/soll-p.txt"
vergleich "P1 Strg+V fügt nicht ein (Steuerzeichen lnext an die Shell)" "$OUT/p.txt" "$OUT/soll-p.txt"
wl-copy --primary 'PRIMARY-P2: Öse'
tippen 'ziel p2'
q "move $(cx 40) $(cy 20)
mclick
wait 600
key Return
chord ctrl d
wait 500"
printf 'PRIMARY-P2: Öse\n' > "$OUT/soll-p2.txt"
vergleich "P2 mittlere Maustaste fügt PRIMARY ein" "$OUT/p2.txt" "$OUT/soll-p2.txt"

# --- O: Fenster schließen und neu öffnen
q "move $(cx 10) $(cy 1)
click
wait 300"
tippen clear; tippen zeige
ziehen 0 2 33 2
q "chord ctrl+shift c
wait 400
move $SCHLIESSEN_X 13
wait 300
click
wait 1500"
ergebnis "O1 Fenster über die Titelleiste geschlossen, Prozess beendet" bash -c "! kill -0 $QT_PID 2>/dev/null"
if wl-paste -n >/dev/null 2>&1; then notiz "O: CLIPBOARD nach dem Schließen noch belegt"; else notiz "O: CLIPBOARD nach dem Schließen leer (Wayland: Inhalt gehört dem Programm, kein Zwischenablage-Manager)"; fi
start_terminal
notiz "neues Terminal PID $QT_PID"
tippen clear; tippen zeige
ziehen 0 2 33 2
q "chord ctrl+shift c
wait 500"
klippe > "$OUT/O-clipboard.txt"
vergleich "O2 nach Neustart: Kopieren" "$OUT/O-clipboard.txt" "$OUT/soll-E.txt"
tippen 'ziel o'
q "chord ctrl+shift v
wait 600
key Return
chord ctrl d
wait 500"
{ cat "$OUT/soll-E.txt"; echo; } > "$OUT/soll-o.txt"
vergleich "O3 nach Neustart: Einfügen" "$OUT/o.txt" "$OUT/soll-o.txt"

# --- P3: exit schließt das Fenster wie bisher
tippen exit
sleep 1.5
ergebnis "P3 exit beendet Shell und Fenster" bash -c "! kill -0 $QT_PID 2>/dev/null"
grep -c 'Shell beendet (Exit-Code 0)' "$LOGS/terminal.log" | sed 's/^/INFO Shell-Ende Exit-Code 0, Anzahl: /' >>"$ERG"

# Keine Inhalte im Terminal-Log (Stichproben aller Testtexte)
if grep -q -e 'Grüße' -e 'VORHER' -e 'Kontextmenü: Größe' -e 'Umschalt+Einfg:' -e 'Äpfel' -e 'Übermaß' -e 'touch' "$LOGS/terminal.log"; then
  echo "FAIL Q Zwischenablage-Inhalt im Terminal-Log" >>"$ERG"
else
  echo "PASS Q keine Zwischenablage-Inhalte im Terminal-Log" >>"$ERG"
fi
notiz "Ende $(date '+%F %T')"
fertig
