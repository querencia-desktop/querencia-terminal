# Gemeinsame Teile der Szenarien (läuft IN der isolierten Sitzung).
P="$QT_PROFIL"; OUT="$P/out"; LOGS="$P/logs"; ERG="$LOGS/ergebnis.txt"
exec >>"$LOGS/szenario.txt" 2>&1
set -u
# qeingabe-Befehle; das erste Ereignis nach dem Anlegen der virtuellen
# Tastatur ging verloren (Keymap-Wechsel), daher zuerst warten.
q() { printf 'wait 300\n%s\n' "$1" | "$QE"; }
# Zellmitte im Terminal (Fenster bei 0,0; Zelle 8x17 ab 1,27)
cx() { echo $((1 + 8 * $1 + 4)); }
cy() { echo $((27 + 17 * $1 + 8)); }
# tippen <text>: Text und Enter
tippen() { q "type $1
key Return
wait 400"; }
# ziehen <spalte1> <zeile1> <spalte2> <zeile2>: markieren mit gedrückter linker Taste
# (Start im linken, Ende im rechten Teil der Zelle: beide Zellen gehören dazu)
ziehen() { q "move $((1 + 8 * $1 + 1)) $(cy $2)
wait 200
press
glide $((1 + 8 * $3 + 7)) $(cy $4) 400
release
wait 300"; }
# klippe: CLIPBOARD lesen (wl-paste über data-control, ohne Fokus)
# (ausdrücklich UTF-8: bei „text/plain“ ohne Zeichensatz wandelt GTK um)
klippe() { wl-paste -n -t 'text/plain;charset=utf-8' 2>/dev/null; }
primaer() { wl-paste -n --primary -t 'text/plain;charset=utf-8' 2>/dev/null; }
# zeile <png> <menü-x> <zeilen-y>: "aktiv" (dunkler Text auf Menühintergrund),
# "grau" (nur heller Text) oder "kein-menue" (kein heller Menühintergrund)
zeile() {
  python3 -c 'import sys; from PIL import Image
x, y = int(sys.argv[2]), int(sys.argv[3])
im = Image.open(sys.argv[1]).convert("RGB").crop((x - 15, y - 8, x + 95, y + 8))
lum = [sum(p) // 3 for p in im.getdata()]
hell = sum(1 for l in lum if l >= 235) / len(lum)
print("kein-menue" if hell < 0.6 else "aktiv" if min(lum) < 110 else "grau")' "$@"
}
zeile_ist() { [ "$(zeile "$1" "$2" "$3")" = "$4" ]; }
# vergleich <kennung> <ist-datei> <soll-datei>
vergleich() {
  if cmp -s -- "$2" "$3"; then echo "PASS $1" >>"$ERG"
  else echo "FAIL $1: $(od -c "$2" | head -3 | tr '\n' ' ') // soll $(od -c "$3" | head -3 | tr '\n' ' ')" >>"$ERG"; fi
}
notiz() { echo "INFO $*" >>"$ERG"; }
start_terminal() {
  SHELL="$P/qshell" QUERENCIA_TERMINAL_DEBUG=1 "$QT_BIN" >>"$LOGS/terminal.log" 2>&1 &
  QT_PID=$!
  sleep 3
}
# ergebnis <Kennung> <Bedingung als Befehl...>
ergebnis() {
  local id=$1; shift
  if "$@"; then echo "PASS $id" >>"$ERG"; else echo "FAIL $id ($*)" >>"$ERG"; fi
}
# gleich <datei> <erwartete-datei>: Bytevergleich
gleich() { cmp -s -- "$1" "$2"; }
fertig() { echo "--- Terminal-Log"; cat "$LOGS/terminal.log"; touch "$LOGS/FERTIG"; sleep 30; }
