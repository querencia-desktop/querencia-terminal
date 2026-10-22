#!/bin/bash
# Querencia Terminal 0.3.1, Prüfung hell/dunkel: läuft IN der isolierten
# Sitzung (lauf.sh). Satz a im Startschema, dann Wechsel per gsettings im
# laufenden Programm (Satz b), zurück (Satz c = a).
set -u
. "$HMP04_PROFILE/env.sh"
LOG=$OUT/szenario.log
exec >>"$LOG" 2>&1
B=${RES%x*}; H=${RES#*x}
ERG=$OUT/ergebnis.txt
: >"$ERG"
ergebnis() { local id=$1; shift; if "$@"; then echo "PASS $id" >>"$ERG"; else echo "FAIL $id ($*)" >>"$ERG"; fi; }
notiz() { echo "INFO $*" >>"$ERG"; }
schema_setzen() {
    case $1 in dunkel) gsettings set org.gnome.desktop.interface color-scheme prefer-dark ;;
    *) gsettings set org.gnome.desktop.interface color-scheme default ;; esac
    echo "$(date +%T.%N | cut -c1-12) color-scheme $(gsettings get org.gnome.desktop.interface color-scheme)"
}
# Werte der Live-Sitzung (nur gelesen) in die gsettings des Profils
gsettings set org.gnome.desktop.interface font-name "$SCHRIFT"
gsettings set org.gnome.desktop.interface monospace-font-name "$MONO"
gsettings set org.gnome.desktop.interface gtk-theme Querencia-Klassisch
gsettings set org.gnome.desktop.interface icon-theme Querencia
# Schreibmarke ohne Blinken: Fotos hängen dann nicht vom Zeitpunkt ab
gsettings set org.gnome.desktop.interface cursor-blink false
schema_setzen "$SCHEMA"
python3 -I "$W/portal-ersatz.py" "$HMP04_PROFILE/logs/portal-bereit" >>"$OUT/portal.log" 2>&1 &
for i in $(seq 50); do [ -f "$HMP04_PROFILE/logs/portal-bereit" ] && break; sleep 0.1; done
"$QA" "$RES"; sleep 1
. "$W/eingabe.sh"
q "extent $B $H"
foto() { grim "$OUT/$1.png"; echo "$(date +%T.%N | cut -c1-12) Foto $1"; }
weg() { q "move $((B - 2)) $((H - 2))" "wait 600"; }
tippen() { q "type $1" "key Return" "wait 400"; }
SHELL=$HMP04_PROFILE/qshell QUERENCIA_TERMINAL_DEBUG=1 "$BIN" >>"$OUT/terminal.log" 2>&1 &
TPID=$!
notiz "Terminal-PID $TPID, Schema $SCHEMA, Sprache $SPRACHE, Ausgang $RES"
q "wait 4000"
tippen "titel Reiter-Eins"
q "chord ctrl+shift t" "wait 1500"
tippen "titel Reiter-Zwei"
q "chord ctrl+shift t" "wait 1500"
tippen "titel Reiter-Drei"
case $SPRACHE in
en) DATEI_X=24; HILFE_X=225; T_ONLINE=o; T_MELDEN=r; T_INFO=a; T_KUERZEL=k ;;
*) DATEI_X=32; HILFE_X=332; T_ONLINE=o; T_MELDEN=f; T_INFO=i; T_KUERZEL=t ;;
esac
MENUE_Y=40
hilfe() { q "move $HILFE_X $MENUE_Y" "wait 400" "click" "wait 1000"; }
uri_zeilen() { [ -f "$OUT/uri.log" ] && wc -l <"$OUT/uri.log" || echo 0; }
# uri_erwartet <kennung> <adresse>: die Probe hat seit uri_merken genau
# diese Adresse als nächste Zeile bekommen (Wartezeit bis 5 s)
uri_merken() { URI_VORHER=$(uri_zeilen); }
uri_erwartet() {
    local n=$((URI_VORHER + 1)) i
    for i in $(seq 50); do [ "$(uri_zeilen)" -ge "$n" ] && break; q "wait 100"; done
    ergebnis "$1" test "$(sed -n "${n}p" "$OUT/uri.log" 2>/dev/null)" = "$2"
}
case $SPRACHE in de) HILFE_URL=https://querencialinux.org/de/docs/ ;; *) HILFE_URL=https://querencialinux.org/en/docs/ ;; esac
satz() {
    local p=$1 aktionen=$2
    echo "--- Satz $p ($(gsettings get org.gnome.desktop.interface color-scheme))"
    weg; foto "$p-01-reiter"
    q "move $DATEI_X $MENUE_Y" "wait 400" "click" "wait 1000"; weg; foto "$p-02-menue-datei"
    q "key Down" "wait 600"; foto "$p-02b-menue-datei-auswahl"
    q "key Escape" "wait 400" "key Escape" "wait 800"
    hilfe; weg; foto "$p-03-menue-hilfe"
    if [ "$aktionen" = ja ]; then
        uri_merken; q "key $T_ONLINE" "wait 1500"
        uri_erwartet "$p Hilfe → Online-Hilfe öffnet $HILFE_URL" "$HILFE_URL"
        hilfe; uri_merken; q "key $T_MELDEN" "wait 1500"
        uri_erwartet "$p Hilfe → Fehler melden öffnet die Meldestelle" "https://github.com/querencia-desktop/querencia/issues/new/choose"
        hilfe
    fi
    q "key $T_INFO" "wait 1500"; weg; foto "$p-04-info"
    if [ "$aktionen" = ja ]; then
        uri_merken; q "chord shift Tab" "wait 400" "key Return" "wait 1500"
        uri_erwartet "$p Info: Verweis öffnet die Webseite" "https://querencialinux.org"
    fi
    q "key Escape" "wait 1000"
    hilfe; q "key $T_KUERZEL" "wait 1500"; weg; foto "$p-05-kuerzel"
    q "key Escape" "wait 1000"
    # Foto erst, wenn das Programm das Menü meldet (Eingaben können sich
    # unter Last verzögern), dann 800 ms für das Zeichnen
    local menues i
    menues=$(grep -c "Kontextmenü offen" "$OUT/terminal.log")
    q "move 300 300" "wait 400" "rclick"
    for i in $(seq 50); do [ "$(grep -c "Kontextmenü offen" "$OUT/terminal.log")" -gt "$menues" ] && break; q "wait 100"; done
    q "wait 800"; foto "$p-06-kontext"
    q "key Down" "wait 600"; foto "$p-06b-kontext-auswahl"
    q "key Escape" "wait 800"; weg
    foto "$p-07-ende"
}
satz a "$AKTIONEN"
case $SCHEMA in hell) ANDERS=dunkel ;; *) ANDERS=hell ;; esac
schema_setzen "$ANDERS"; q "wait 3000"
satz b nein
schema_setzen "$SCHEMA"; q "wait 3000"
satz c nein
ergebnis "Terminal läuft bis zum Ende" kill -0 "$TPID"
ergebnis "drei Reiter mit je einer Shell" test "$(grep -c 'qshell läuft als PID' "$OUT/terminal.log")" = 3
kill "$TPID" 2>/dev/null; wait "$TPID" 2>/dev/null
# GTK-Meldungen ohne die erwarteten Portal-Hinweise (der Ersatz bietet nur
# Settings an)
grep -i -E "gtk-(warning|critical)|gdk-(warning|critical)" "$OUT/terminal.log" | grep -v -c "Cannot get portal" >"$OUT/gtk-meldungen.txt"
touch "$HMP04_PROFILE/logs/FERTIG"; sleep 30
