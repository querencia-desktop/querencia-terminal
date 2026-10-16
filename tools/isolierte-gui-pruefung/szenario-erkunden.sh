#!/bin/bash
# Erkundung: Fensterlage und Zellraster (nur Bilder).
. "$HIER/lib.sh"
start_terminal
q "type zeige
key Return
wait 500
shot $OUT/erkunden-1.png"
"$TEXTPROBE" "$P/out/tp.txt" >"$LOGS/textprobe.log" 2>&1 &
sleep 2
foot --window-size-pixels=500x360 bash --noprofile --norc >"$LOGS/foot.log" 2>&1 &
sleep 2
q "shot $OUT/erkunden-2.png"
fertig
