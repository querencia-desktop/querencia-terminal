#!/bin/bash
# Querencia Terminal 0.2.0: Sprache der Oberfläche (gettext), Prüfungen S1–S8.
# Läuft in der isolierten Sitzung (pruefen.sh -> isoliert.py), je einmal mit
# QT_LANGUAGE=de und =en und mit QT_GROESSE=1024x768 bzw. 1920x1080:
#   QT_LANGUAGE=en QT_GROESSE=1920x1080 bash pruefen.sh <binary> <profil> szenario-sprache.sh 240
# Belegbilder: Kontextmenü mit Markierung, hervorgehobene Zeile, Startfehler.
# Ausgewertet werden Zugriffstaste je Sprache, Wählbarkeit der Zeilen (Bild),
# der eingeblendete Katalog (/proc/<pid>/maps) und die Fehlermeldung (Log).
. "$HIER/lib.sh"

SPRACHE=${LANGUAGE%%:*}
read -r BREITE HOEHE <<<"${QT_GROESSE/x/ }"
BREITE=${BREITE:-1280}; HOEHE=${HOEHE:-720}
# Bei 1920x1080 zeichnet der Software-Renderer langsamer: längere Pausen.
F=1; [ "$BREITE" -gt 1280 ] && F=2
# Koordinaten in Pixeln des Ausgangs (qeingabe rechnet sonst in 1280x720).
qx() { q "extent $BREITE $HOEHE
$1"; }
RX=405; RY=205
MX=$((RX + 40)); M_KOPIEREN=$((RY + 14)); M_EINFUEGEN=$((RY + 38)); M_ALLES=$((RY + 69))

notiz "Start $(date '+%F %T'), LANGUAGE=$LANGUAGE, Ausgang ${BREITE}x${HOEHE}"
if [ -n "${QT_GROESSE:-}" ]; then
  "$QA" "$QT_GROESSE" >>"$LOGS/szenario.txt" 2>&1 && notiz "Ausgang auf $QT_GROESSE gesetzt" || notiz "qausgabe fehlgeschlagen"
  sleep 1
fi
wl-copy --clear
start_terminal
sleep $((F - 1))
notiz "Terminal-PID $QT_PID"

# --- S1: Katalog: deutsch eingeblendet, englisch nicht
if grep -q 'LC_MESSAGES/querencia-terminal.mo' "/proc/$QT_PID/maps"; then KAT=ja; else KAT=nein; fi
notiz "Katalog in /proc/maps: $KAT ($(grep -o '[^ ]*querencia-terminal.mo' "/proc/$QT_PID/maps" | head -1))"
case $SPRACHE in
de) ergebnis "S1 de: querencia-terminal.mo eingeblendet" test "$KAT" = ja ;;
*) ergebnis "S1 $SPRACHE: kein querencia-terminal.mo eingeblendet" test "$KAT" = nein ;;
esac

# --- S2/S3: Kontextmenü mit Markierung (Bild), Kopieren hervorgehoben
tippen zeige
ziehen 0 3 16 3
qx "move $RX $RY
wait $((300 * F))
rclick
wait $((900 * F))
shot $OUT/S2-menue-$SPRACHE.png
move $MX $M_KOPIEREN
wait $((500 * F))
shot $OUT/S3-menue-kopieren-hervorgehoben-$SPRACHE.png
key Escape
wait 400"
ergebnis "S2 Bild: Kopieren wählbar" zeile_ist "$OUT/S2-menue-$SPRACHE.png" $MX $M_KOPIEREN aktiv
ergebnis "S2 Bild: Einfügen ausgegraut (Zwischenablage leer)" zeile_ist "$OUT/S2-menue-$SPRACHE.png" $MX $M_EINFUEGEN grau
ergebnis "S2 Bild: Alles auswählen wählbar" zeile_ist "$OUT/S2-menue-$SPRACHE.png" $MX $M_ALLES aktiv

# --- S4: Menü-Taste und Zugriffstaste der Sprache fügen ein
printf 'Sprache %s: Grüße' "$SPRACHE" | wl-copy
tippen 'ziel s4'
q "key Menu
wait $((700 * F))
shot $OUT/S4-menue-tastatur-$SPRACHE.png
key $TASTE_EINFUEGEN
wait $((600 * F))
key Return
chord ctrl d
wait 500"
printf 'Sprache %s: Grüße\n' "$SPRACHE" > "$OUT/soll-s4.txt"
vergleich "S4 Menü-Taste + $TASTE_EINFUEGEN: Einfügen" "$OUT/s4.txt" "$OUT/soll-s4.txt"

# --- S5: die Zugriffstaste der anderen Sprache tut nichts
case $SPRACHE in de) ANDERE=p ;; *) ANDERE=e ;; esac
printf 'nicht einfuegen' | wl-copy
tippen 'ziel s5'
q "key Menu
wait $((700 * F))
key $ANDERE
wait $((400 * F))
key Escape
wait 300
type ende
key Return
chord ctrl d
wait 500"
printf 'ende\n' > "$OUT/soll-s5.txt"
vergleich "S5 Menü-Taste + $ANDERE (andere Sprache): kein Einfügen" "$OUT/s5.txt" "$OUT/soll-s5.txt"

# --- S6: Ende der Shell schließt das Fenster
tippen exit
sleep $((1 * F))
ergebnis "S6 exit beendet das Terminal" bash -c "! kill -0 $QT_PID 2>/dev/null"

# --- S7/S8: Startfehler (Home-Verzeichnis fehlt): Meldung in der Sprache
HOME=/nicht/vorhanden-$$ SHELL="$P/qshell" "$QT_BIN" >>"$LOGS/fehler.log" 2>&1 &
FEHLER_PID=$!
sleep $((3 * F))
q "shot $OUT/S7-startfehler-$SPRACHE.png"
cat "$LOGS/fehler.log" >>"$LOGS/terminal.log"
case $SPRACHE in
de) MUSTER='konnte nicht gestartet werden' ;;
*) MUSTER='could not be started' ;;
esac
ergebnis "S7 Startfehler-Meldung in $SPRACHE (stderr: „$MUSTER“)" grep -q "$MUSTER" "$LOGS/fehler.log"
ergebnis "S8 Fenster bleibt nach dem Startfehler offen" kill -0 "$FEHLER_PID"
kill "$FEHLER_PID" 2>/dev/null
fertig
