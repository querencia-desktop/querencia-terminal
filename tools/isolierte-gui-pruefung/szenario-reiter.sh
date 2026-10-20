#!/bin/bash
# Querencia Terminal 0.3.0: Reiter (R1–R19). Läuft in der isolierten Sitzung
# (pruefen.sh -> isoliert.py), je einmal mit QT_LANGUAGE=de und =en und mit
# QT_GROESSE=1024x768 bzw. 1920x1080:
#   QT_LANGUAGE=en QT_GROESSE=1920x1080 bash pruefen.sh <binary> <profil> szenario-reiter.sh 300
# Jede Shell trägt beim Start ihre PID in out/shells ein (Zeile n = n-ter
# geöffneter Reiter); „jetzt“ schreibt die PID der Shell im aktiven Reiter.
# Belegbilder: Leiste, drei Reiter, Wechsel, Kontextmenü, Menüs, Nachfrage.
. "$HIER/lib.sh"

SPRACHE=${LANGUAGE%%:*}
read -r BREITE HOEHE <<<"${QT_GROESSE/x/ }"
BREITE=${BREITE:-1280}; HOEHE=${HOEHE:-720}
F=1; [ "$BREITE" -gt 1280 ] && F=2
# Koordinaten in Pixeln des Ausgangs; Fenster bei 0,0.
qx() { q "extent $BREITE $HOEHE
$1"; }
# Lage seit 0.3.0 (Titelleiste 27 px, Menüleiste bis y=53, Reiterleiste
# bis y=87): Menüköpfe, Reiter (Breite 228 px ab x=10), freie Leiste.
MENUE_Y=40; DATEI_X=30; BEARB_X=107; ANSICHT_X=190; REITER_X=256; HILFE_X=311
LEISTE_Y=70
reiter_x() { echo $((10 + 228 * $1 + 100)); }
FREI_X=650
# Zugriffstaste „Schließen“ in der Nachfrage: de „Terminal _schließen“,
# „Fenster _schließen“; en „C_lose Terminal“, „C_lose Window“.
case $SPRACHE in en) TASTE_SCHLIESSEN=l ;; *) TASTE_SCHLIESSEN=s ;; esac

# eigene Shell: testrc von pruefen.sh plus Hilfen für die Reiter
cat > "$P/reiterrc" <<X
. '$P/testrc'
echo \$\$ >> "\$OUT/shells"
jetzt() { echo \$\$ > "\$OUT/jetzt"; }
titel() { printf '\\033]0;%s\\007' "\$1"; }
X
printf '#!/bin/bash\nexport INPUTRC=%q\nexec /bin/bash --noprofile --rcfile %q -i\n' "$P/inputrc" "$P/reiterrc" > "$P/qshell-reiter"
chmod +x "$P/qshell-reiter"
: > "$OUT/shells"

shell_pid() { sed -n "${1}p" "$OUT/shells"; }
lebt() { kill -0 "$(shell_pid "$1")" 2>/dev/null; }
tot() { ! lebt "$1"; }
anzahl_shells() { wc -l < "$OUT/shells"; }
# aktiv_ist N: der aktive Reiter gehört zur N-ten Shell
aktiv_ist() {
  rm -f "$OUT/jetzt"
  tippen jetzt
  sleep 0.3
  [ "$(cat "$OUT/jetzt" 2>/dev/null)" = "$(shell_pid "$1")" ]
}
# hell X Y: mittlere Helligkeit um (X,Y) im Bild $1
hell() {
  python3 -c 'import sys, gi
gi.require_version("GdkPixbuf", "2.0")
from gi.repository import GdkPixbuf
pb = GdkPixbuf.Pixbuf.new_from_file(sys.argv[1])
n, rs, d = pb.get_n_channels(), pb.get_rowstride(), pb.get_pixels()
x, y = int(sys.argv[2]), int(sys.argv[3])
lum = [sum(d[yy * rs + xx * n:yy * rs + xx * n + 3]) // 3
       for yy in range(y - 3, y + 3) for xx in range(x - 5, x + 5)]
print(sum(lum) // len(lum))' "$@"
}
leiste_sichtbar() { [ "$(hell "$1" "$FREI_X" "$LEISTE_Y")" -ge 180 ]; }
leiste_verborgen() { [ "$(hell "$1" "$FREI_X" "$LEISTE_Y")" -le 60 ]; }
log_zaehlt() { [ "$(grep -c -- "$1" "$LOGS/terminal.log")" -ge "$2" ]; }

notiz "Start $(date '+%F %T'), LANGUAGE=$LANGUAGE, Ausgang ${BREITE}x${HOEHE}"
if [ -n "${QT_GROESSE:-}" ]; then
  "$QA" "$QT_GROESSE" >>"$LOGS/szenario.txt" 2>&1 && notiz "Ausgang auf $QT_GROESSE gesetzt" || notiz "qausgabe fehlgeschlagen"
  sleep 1
fi
SHELL="$P/qshell-reiter" QUERENCIA_TERMINAL_DEBUG=1 "$QT_BIN" >>"$LOGS/terminal.log" 2>&1 &
QT_PID=$!
sleep $((3 + 2 * F))
notiz "Terminal-PID $QT_PID"

# --- R1: ein Reiter, keine Reiterleiste
q "mark R1"
qx "shot $OUT/r01-ein-reiter-$SPRACHE.png"
ergebnis "R1 ein Reiter: Reiterleiste verborgen" leiste_verborgen "$OUT/r01-ein-reiter-$SPRACHE.png"
ergebnis "R1 eine Shell" test "$(anzahl_shells)" = 1

# --- R2: Strg+Umschalt+T öffnet Reiter 2 im Arbeitsverzeichnis von Reiter 1
q "mark R2"
tippen "cd $OUT"
qx "chord ctrl+shift t
wait $((1500 * F))
shot $OUT/r02-zwei-reiter-leiste-$SPRACHE.png"
ergebnis "R2 Strg+Umschalt+T: zweite Shell" test "$(anzahl_shells)" = 2
ergebnis "R2 Reiterleiste ab dem zweiten Reiter sichtbar" leiste_sichtbar "$OUT/r02-zwei-reiter-leiste-$SPRACHE.png"
ergebnis "R2 neuer Reiter im Arbeitsverzeichnis des aktuellen" grep -q "in $OUT\$" "$LOGS/terminal.log"
ergebnis "R2 neuer Reiter ist aktiv" aktiv_ist 2

# --- R3: Kontextmenü „Neuer Reiter“ öffnet Reiter 3
q "mark R3"
RX=405; RY=205; MX=$((RX + 40))
qx "move $RX $RY
wait $((300 * F))
rclick
wait $((900 * F))
shot $OUT/r03-kontextmenue-$SPRACHE.png
move $MX $((RY + 103))
wait $((500 * F))
shot $OUT/r03-kontextmenue-neuer-reiter-$SPRACHE.png
click
wait $((1500 * F))"
ergebnis "R3 Kontextmenü „Neuer Reiter“: dritte Shell" test "$(anzahl_shells)" = 3
ergebnis "R3 Kontextmenü: Zeile „Neuer Reiter“ wählbar" zeile_ist "$OUT/r03-kontextmenue-$SPRACHE.png" $MX $((RY + 103)) aktiv
ergebnis "R3 Kontextmenü: Zeile „Reiter schließen“ wählbar" zeile_ist "$OUT/r03-kontextmenue-$SPRACHE.png" $MX $((RY + 127)) aktiv
ergebnis "R3 dritter Reiter aktiv" aktiv_ist 3

# --- R4: Reitertitel aus dem Fenstertitel der Shell
q "mark R4"
tippen 'titel Reiter-Drei'
sleep 0.5
qx "shot $OUT/r04-drei-reiter-titel-$SPRACHE.png"
ergebnis "R4 Titel gesetzt (Reitertitel mit 11 Zeichen im Log)" grep -q 'Reitertitel (11 Zeichen)' "$LOGS/terminal.log"

# --- R5/R6: wechseln mit Strg+Bild auf/ab und Alt+Ziffer
q "mark R5"
qx "chord ctrl Page_Up
wait $((600 * F))
shot $OUT/r05-wechsel-strg-bild-auf-$SPRACHE.png"
ergebnis "R5 Strg+Bild auf: Reiter 2" aktiv_ist 2
qx "chord ctrl Page_Down
wait $((600 * F))"
ergebnis "R5 Strg+Bild ab: Reiter 3" aktiv_ist 3
qx "chord alt 1
wait $((600 * F))
shot $OUT/r06-wechsel-alt-1-$SPRACHE.png"
ergebnis "R6 Alt+1: Reiter 1" aktiv_ist 1
qx "chord alt 3
wait $((600 * F))"
ergebnis "R6 Alt+3: Reiter 3" aktiv_ist 3

# --- R7: am letzten Reiter geht Strg+Bild ab an die Shell (kein Umlauf)
q "mark R7"
tippen 'roh r7'
qx "chord ctrl Page_Down
wait $((500 * F))
chord ctrl u
wait 300"
ergebnis "R7 Strg+Bild ab am letzten Reiter: Taste an die Shell (ESC)" test "$(cat "$OUT/r7.roh" 2>/dev/null)" = 1b
ergebnis "R7 Reiter 3 bleibt aktiv" aktiv_ist 3

# --- R8: Strg+Umschalt+Bild auf verschiebt Reiter 3 nach links
q "mark R8"
qx "chord ctrl+shift Page_Up
wait $((800 * F))
shot $OUT/r08-verschoben-$SPRACHE.png"
qx "chord alt 2
wait $((600 * F))"
ergebnis "R8 verschoben: Alt+2 zeigt jetzt Shell 3" aktiv_ist 3
qx "chord alt 3
wait $((600 * F))"
ergebnis "R8 verschoben: Alt+3 zeigt jetzt Shell 2" aktiv_ist 2

# --- R9: Mittelklick auf Reiter 3 (Shell 2) schließt ihn
q "mark R9"
qx "move $(reiter_x 2) $LEISTE_Y
wait $((300 * F))
mclick
wait $((1500 * F))
shot $OUT/r09-mittelklick-$SPRACHE.png"
ergebnis "R9 Mittelklick: Shell 2 beendet" tot 2
ergebnis "R9 Shells 1 und 3 laufen" bash -c "kill -0 $(shell_pid 1) && kill -0 $(shell_pid 3)"

# --- R10: Doppelklick auf die freie Reiterleiste öffnet Reiter (Shell 4)
q "mark R10"
qx "move $FREI_X $LEISTE_Y
wait $((300 * F))
dclick
wait $((1500 * F))
shot $OUT/r10-doppelklick-leiste-$SPRACHE.png"
ergebnis "R10 Doppelklick auf freie Leiste: vierte Shell" test "$(anzahl_shells)" = 4
ergebnis "R10 neuer Reiter aktiv" aktiv_ist 4

# --- R11: Strg+Umschalt+W mit laufendem Programm fragt nach
q "mark R11"
tippen 'sleep 1000'
sleep 0.5
SCHLAF=$(pgrep -P "$(shell_pid 4)" -x sleep)
qx "chord ctrl+shift w
wait $((1000 * F))
shot $OUT/r11-nachfrage-reiter-$SPRACHE.png
key Escape
wait $((700 * F))"
ergebnis "R11 Nachfrage „Terminal“ erschienen" grep -q 'Nachfrage beim Schließen (Terminal)' "$LOGS/terminal.log"
ergebnis "R11 Escape: Reiter und sleep laufen weiter" bash -c "kill -0 $(shell_pid 4) && kill -0 ${SCHLAF:-0}"
qx "chord ctrl+shift w
wait $((1000 * F))
chord alt $TASTE_SCHLIESSEN
wait $((1500 * F))"
ergebnis "R11 bestätigt: Shell 4 und sleep beendet" bash -c "! kill -0 $(shell_pid 4) 2>/dev/null && ! kill -0 ${SCHLAF:-0} 2>/dev/null"

# --- R12: Alt+Buchstabe und F10 bleiben bei der Shell
q "mark R12"
tippen 'roh r12a'
qx "chord alt b
wait 500
chord ctrl u
wait 300"
ergebnis "R12 Alt+B an die Shell (ESC b)" test "$(cat "$OUT/r12a.roh" 2>/dev/null)" = 1b
tippen 'roh r12b'
qx "key F10
wait 500
chord ctrl u
wait 300
shot $OUT/r12-f10-ohne-menue-$SPRACHE.png"
ergebnis "R12 F10 an die Shell" test "$(cat "$OUT/r12b.roh" 2>/dev/null)" = 1b

# --- R13: Menüs (Bilder) und Menü „Reiter“ per Maus
q "mark R13"
for m in "$DATEI_X datei" "$BEARB_X bearbeiten" "$ANSICHT_X ansicht" "$REITER_X reiter" "$HILFE_X hilfe"; do
  set -- $m
  qx "move $1 $MENUE_Y
wait 200
click
wait $((800 * F))
shot $OUT/r13-menue-$2-$SPRACHE.png
key Escape
wait $((400 * F))"
done
AKTIV_VOR=$(cat "$OUT/jetzt")
qx "move $REITER_X $MENUE_Y
wait 200
click
wait $((800 * F))
move $((REITER_X + 40)) 73
wait 300
click
wait $((800 * F))"
ergebnis "R13 Reiter → Vorheriger Reiter: Shell 1" aktiv_ist 1
notiz "vorher aktiv: $AKTIV_VOR"

# --- R14: Hilfe → Tastenkürzel
q "mark R14"
qx "move $HILFE_X $MENUE_Y
wait 200
click
wait $((800 * F))
move $((HILFE_X + 40)) 73
wait 300
click
wait $((1200 * F))
shot $OUT/r14-tastenkuerzel-$SPRACHE.png
key Escape
wait $((700 * F))"

# --- R15: Fenster schließen (Titelleiste) mit zwei Reitern und laufendem
q "mark R15"
# Programm fragt nach „Fenster“; Abbrechen lässt alles laufen
tippen 'sleep 1000'
sleep 0.5
SCHLAF=$(pgrep -P "$(shell_pid 1)" -x sleep)
qx "move $SCHLIESSEN_X 13
wait 300
click
wait $((1000 * F))
shot $OUT/r15-nachfrage-fenster-$SPRACHE.png
key Escape
wait $((700 * F))"
ergebnis "R15 Nachfrage „Fenster“ erschienen" grep -q 'Nachfrage beim Schließen (Window)' "$LOGS/terminal.log"
ergebnis "R15 abgebrochen: Terminal und sleep laufen" bash -c "kill -0 $QT_PID && kill -0 ${SCHLAF:-0}"
qx "chord ctrl c
wait 500"

# --- R16: Reiter ohne laufendes Programm schließt ohne Nachfrage
q "mark R16"
qx "chord ctrl+shift w
wait $((1200 * F))
shot $OUT/r16-ein-reiter-wieder-$SPRACHE.png"
ergebnis "R16 Strg+Umschalt+W ohne Programm: ohne Nachfrage geschlossen" test "$(grep -c 'Nachfrage beim Schließen' "$LOGS/terminal.log")" = 3
ergebnis "R16 ein Reiter: Leiste wieder verborgen" leiste_verborgen "$OUT/r16-ein-reiter-wieder-$SPRACHE.png"

# --- R17: Ansicht → Reiterleiste immer anzeigen (und gespeichert)
q "mark R17"
qx "move $ANSICHT_X $MENUE_Y
wait 200
click
wait $((800 * F))
move $((ANSICHT_X + 40)) 73
wait 300
click
wait $((1000 * F))
shot $OUT/r17-leiste-immer-$SPRACHE.png"
ergebnis "R17 Leiste mit einem Reiter sichtbar" leiste_sichtbar "$OUT/r17-leiste-immer-$SPRACHE.png"
ergebnis "R17 terminal.ini: Leiste=immer" grep -qx 'Leiste=immer' "$P/config/querencia/terminal.ini"
cp "$P/config/querencia/terminal.ini" "$OUT/terminal.ini" 2>/dev/null

# --- R18: letzter Reiter mit Programm: Strg+Umschalt+W fragt, Bestätigen
q "mark R18"
# schließt das Fenster
tippen jetzt
sleep 0.3
LETZTE=$(cat "$OUT/jetzt")
tippen 'sleep 1000'
sleep 0.5
SCHLAF=$(pgrep -P "$LETZTE" -x sleep)
qx "chord ctrl+shift w
wait $((1000 * F))
shot $OUT/r18-nachfrage-letzter-reiter-$SPRACHE.png
chord alt $TASTE_SCHLIESSEN
wait $((2000 * F))"
ergebnis "R18 Nachfrage „Terminal“ beim letzten Reiter" test "$(grep -c 'Nachfrage beim Schließen (Terminal)' "$LOGS/terminal.log")" = 3
ergebnis "R18 bestätigt: Terminalprozess beendet" bash -c "! kill -0 $QT_PID 2>/dev/null"
ergebnis "R18 sleep beendet" bash -c "! kill -0 ${SCHLAF:-0} 2>/dev/null"

# --- R19: Einstellung gilt beim nächsten Start
q "mark R19"
SHELL="$P/qshell-reiter" QUERENCIA_TERMINAL_DEBUG=1 "$QT_BIN" >>"$LOGS/terminal.log" 2>&1 &
QT_PID=$!
sleep $((3 + 2 * F))
qx "shot $OUT/r19-neustart-leiste-immer-$SPRACHE.png"
ergebnis "R19 nach Neustart: Leiste mit einem Reiter sichtbar" leiste_sichtbar "$OUT/r19-neustart-leiste-immer-$SPRACHE.png"
tippen exit
sleep $((1 * F))
ergebnis "R19 exit im letzten Reiter beendet das Terminal" bash -c "! kill -0 $QT_PID 2>/dev/null"
fertig
