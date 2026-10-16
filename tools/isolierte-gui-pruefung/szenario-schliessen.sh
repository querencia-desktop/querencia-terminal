#!/bin/bash
# Bestand (M3.0a-Abnahme Nr. 14): Fenster mit laufender Shell und
# Hintergrundprozess über die Titelleiste schließen. Für alten und neuen
# Stand gleich auszuführen; das Ergebnis muss übereinstimmen.
. "$HIER/lib.sh"
start_terminal
tippen 'echo $$ > $OUT/shellpid; sleep 1000 & echo $! > $OUT/sleeppid'
sleep 0.5
q "move 643 13
wait 300
click
wait 2000"
ergebnis "S1 Terminalprozess beendet" bash -c "! kill -0 $QT_PID 2>/dev/null"
ergebnis "S2 Shell beendet" bash -c "! kill -0 $(cat "$OUT/shellpid") 2>/dev/null"
ergebnis "S3 sleep 1000 beendet" bash -c "! kill -0 $(cat "$OUT/sleeppid") 2>/dev/null"
grep 'Shell beendet' "$LOGS/terminal.log" | sed 's/^/INFO /' >>"$ERG"
fertig
