#!/bin/bash
# Querencia Terminal, Kopieren/Einfügen: isolierte GUI-Prüfung in einem
# eigenen labwc 0.9.8 (headless, eigener D-Bus und XDG_RUNTIME_DIR), mit
# echten virtuellen Tastatur- und Zeigerereignissen (qeingabe) und echten
# Wayland-Programmen (wl-copy/wl-paste, foot, GTK-4-Textprobe).
#   pruefen.sh <terminal-binary> <profilname> <szenario.sh> [sekunden]
# Ergebnis: <gui-test>/<profil>/logs/ergebnis.txt
set -euo pipefail
HIER="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QT_BIN=$(realpath -e "${1:?Terminal-Binary}"); PROFIL="${2:?Profilname}"; SZ=$(realpath -e "${3:?Szenario}")
G=~/projects/querencia-terminal-werkstatt/gui-test
bash "$HIER/bauen.sh" "$G/bin" >/dev/null
TEXTPROBE="$G/textprobe-target/debug/textprobe"
[ -x "$TEXTPROBE" ] || { echo "textprobe fehlt: (cd $HIER/textprobe && CARGO_TARGET_DIR=$G/textprobe-target cargo build --offline)" >&2; exit 2; }
. "$HIER/testprofil.sh"; testprofil_neu "$G" "$PROFIL"
P="$G/$PROFIL"; mkdir -p "$P/logs" "$P/out" "$P/config/labwc" "$P/config/foot"
# Fensterlage fest: Terminal links oben, Textprobe rechts oben, foot rechts unten.
cat > "$P/config/labwc/rc.xml" <<X
<?xml version="1.0"?>
<labwc_config>
  <core><decoration>server</decoration></core>
  <theme><name>Clearlooks</name></theme>
  <placement><policy>automatic</policy></placement>
  <windowRules>
    <windowRule identifier="org.example.Querencia.Terminal"><action name="MoveTo" x="0" y="0"/></windowRule>
    <windowRule identifier="org.example.Querencia.Textprobe"><action name="MoveTo" x="760" y="0"/></windowRule>
    <windowRule identifier="foot"><action name="MoveTo" x="760" y="330"/></windowRule>
  </windowRules>
</labwc_config>
X
: > "$P/config/foot/foot.ini"
# Testtexte (UTF-8, Umlaute, doppelte Leerzeichen, mehrere Zeilen)
printf 'Grüße aus Köln: äöüß ÄÖÜ – 10 € „Zitat“\nZweite Zeile mit  zwei Leerzeichen\nDritte Zeile Ende\n' > "$P/probe.txt"
printf 'Aus der Textprobe: Übermaß an Öl\nzweite  Zeile' > "$P/textprobe-vorgabe.txt"
# Shell des Terminals: bash mit eigener rc, ohne ~/.bashrc und ~/.inputrc
: > "$P/inputrc"
cat > "$P/testrc" <<X
PS1='\$ '
HISTFILE=/dev/null
OUT='$P/out'
zeige() { cat '$P/probe.txt'; }
ziel() { cat > "\$OUT/\$1.txt"; echo "\$?" > "\$OUT/\$1.rc"; }
rc() { echo "\$?" > "\$OUT/\$1.rc"; }
# roh NAME: erstes Byte, das die Shell ungefiltert (stty raw) erreicht, als Hex
roh() { stty raw -echo; head -c1 | od -An -tx1 | tr -d ' ' > "\$OUT/\$1.roh"; stty sane; }
X
printf '#!/bin/bash\nexport INPUTRC=%q\nexec /bin/bash --noprofile --rcfile %q -i\n' "$P/inputrc" "$P/testrc" > "$P/qshell"
chmod +x "$P/qshell"
echo "Terminal: $QT_BIN ($(sha256sum "$QT_BIN" | cut -c1-16))" > "$P/logs/ergebnis.txt"
# Sprache wie die Live-Sitzung (runtime environment)
LANG=en_US.UTF-8 LANGUAGE=de QT_BIN="$QT_BIN" QE="$G/bin/qeingabe" TEXTPROBE="$TEXTPROBE" QT_PROFIL="$P" HIER="$HIER" \
  /usr/bin/python3 "$HIER/isoliert.py" "$P" "$SZ" "${4:-240}" || true
cat "$P/logs/ergebnis.txt"
