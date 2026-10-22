#!/bin/bash
# Querencia Terminal 0.3.2 (QD-125): Dateien aus Querencia Files ins Terminal
# ziehen. Läuft IN der isolierten Sitzung (lauf.sh mit
# QTD_SZENARIO=szenario-ziehen.sh, Ausgang 1600x900). Terminal bei 0,0,
# Files bei 830,0 mit ~/Ziehen (einfach.txt, it's.txt, mit Leerzeichen.txt).
# Die Shell-Funktion zeig schreibt jedes empfangene Argument als <…> nach
# out/ziehen.txt; so prüft das Szenario das Quoting über die echte Shell.
set -u
. "$HMP04_PROFILE/env.sh"
LOG=$OUT/szenario.log
exec >>"$LOG" 2>&1
B=${RES%x*}; H=${RES#*x}
ERG=$OUT/ergebnis.txt
: >"$ERG"
ergebnis() { local id=$1; shift; if "$@"; then echo "PASS $id" >>"$ERG"; else echo "FAIL $id ($*)" >>"$ERG"; fi; }
gsettings set org.gnome.desktop.interface font-name "$SCHRIFT"
gsettings set org.gnome.desktop.interface monospace-font-name "$MONO"
gsettings set org.gnome.desktop.interface gtk-theme Querencia-Klassisch
gsettings set org.gnome.desktop.interface icon-theme Querencia
gsettings set org.gnome.desktop.interface cursor-blink false
python3 -I "$W/portal-ersatz.py" "$HMP04_PROFILE/logs/portal-bereit" >>"$OUT/portal.log" 2>&1 &
for i in $(seq 50); do [ -f "$HMP04_PROFILE/logs/portal-bereit" ] && break; sleep 0.1; done
"$QA" "$RES"; sleep 1
. "$W/eingabe.sh"
q "extent $B $H"
foto() { grim "$OUT/$1.png"; echo "$(date +%T.%N | cut -c1-12) Foto $1"; }
Z=$HOME/Ziehen
SHELL=$HMP04_PROFILE/qshell QUERENCIA_TERMINAL_DEBUG=1 "$BIN" >>"$OUT/terminal.log" 2>&1 &
TPID=$!
q "wait 3000"
"$FILES" "$Z" >>"$OUT/files.log" 2>&1 &
FPID=$!
q "wait 6000"
EINFACH="1104 200"; ITS="1246 200"; LEER="1388 200"; TERM="400 300"
ins_terminal() { q "move $TERM" "wait 300" "click" "wait 500"; }
ziehen() {  # <x> <y>: von Files ins Terminal, mit Zwischenschritten
    q "move $1 $2" "wait 400" "press" "wait 200" "glide $(($1 - 30)) $(($2 + 10)) 200" \
      "glide 700 260 600" "glide ${TERM% *} ${TERM#* } 600" "wait 400" "release" "wait 1500"
}
abgelegt() { grep -c "Ablegen: $1 Datei(en)" "$OUT/terminal.log"; }
# 1: eine Datei
ins_terminal; q "type zeig" "key space" "wait 300"
foto z0-vorher
ziehen $EINFACH
foto z1-eine-datei-abgelegt
ergebnis "1 eine Datei: Terminal meldet das Ablegen" test "$(abgelegt 1)" -ge 1
ins_terminal; q "key Return" "wait 800"
foto z1b-ausgefuehrt
# 2: zwei Dateien (Klick, Strg+Klick), eine mit Apostroph, eine mit Leerzeichen
ins_terminal; q "type zeig" "key space" "wait 300"
q "move $ITS" "wait 300" "click" "wait 500" "move $LEER" "wait 300" "keydown Control_L" "click" "keyup Control_L" "wait 600"
foto z2-auswahl
ziehen $ITS
foto z2-zwei-dateien-abgelegt
ergebnis "2 zwei Dateien: Terminal meldet das Ablegen" test "$(abgelegt 2)" -ge 1
ins_terminal; q "key Return" "wait 800"
foto z2b-ausgefuehrt
printf '<%s>\n---\n<%s>\n<%s>\n---\n' "$Z/einfach.txt" "$Z/it's.txt" "$Z/mit Leerzeichen.txt" >"$OUT/ziehen-soll.txt"
ergebnis "Shell bekam genau die Pfade (Quoting mit Leerzeichen und Apostroph)" cmp -s "$OUT/ziehen.txt" "$OUT/ziehen-soll.txt"
ergebnis "nichts verschoben: alle drei Dateien noch da" bash -c "[ -f '$Z/einfach.txt' ] && [ -f \"$Z/it's.txt\" ] && [ -f '$Z/mit Leerzeichen.txt' ]"
ergebnis "Terminal läuft bis zum Ende" kill -0 "$TPID"
kill "$FPID" "$TPID" 2>/dev/null; wait "$TPID" 2>/dev/null
grep -i -E "gtk-(warning|critical)|gdk-(warning|critical)" "$OUT/terminal.log" | grep -v -c "Cannot get portal" >"$OUT/gtk-meldungen.txt"
ergebnis "keine GTK-Meldungen im Terminal außer Portal-Hinweisen" test "$(cat "$OUT/gtk-meldungen.txt")" = 0
touch "$HMP04_PROFILE/logs/FERTIG"; sleep 30
