#!/bin/bash
# Querencia Terminal 0.3.1: ein isolierter Lauf der Prüfung hell/dunkel
# (QD-109). Eigenes HOME, XDG_RUNTIME_DIR, Bus und labwc (isoliert.py),
# Farbschema über die gsettings des Profils, die portal-ersatz.py wie das
# echte Settings-Portal an GTK meldet. GTK-Thema: die Live-Ablage
# Querencia-Klassisch (nur gelesen, als Link); Symbolthemen aus einem
# einmaligen Schnappschuss (schnappschuss-symbole/), damit Änderungen anderer
# Sessions die Vergleiche nicht stören. Adressen (Hilfe-Menü, Info-Dialog)
# öffnet statt eines Browsers die Probe uri-probe, die sie in out/uri.log
# schreibt.
#   lauf.sh <binary> <name> hell|dunkel de|en [BxH] [zeitlimit]
# QTD_AKTIONEN=nein: Hilfe-Einträge und Verweis nicht auslösen (Gegenprobe
# mit 0.3.0, das sie nicht hat).
# Ergebnis und Fotos: <gui-test>/<name>/out/
set -euo pipefail
HIER=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
BIN=$(realpath -e "${1:?Binary}"); NAME=${2:?Name}; SCHEMA=${3:?hell|dunkel}; SPRACHE=${4:?de|en}
RES=${5:-1280x800}; T=${6:-300}
case $SCHEMA in hell | dunkel) ;; *) echo "Schema hell|dunkel" >&2; exit 2 ;; esac
case $SPRACHE in de | en) ;; *) echo "Sprache de|en" >&2; exit 2 ;; esac
G=${QT_GUI_TEST:-~/projects/querencia-terminal-gui-test}
mkdir -p "$G"
bash "$HIER/../isolierte-gui-pruefung/bauen.sh" "$G/bin" >/dev/null
. "$HIER/../isolierte-gui-pruefung/testprofil.sh"; testprofil_neu "$G" "$NAME"
P=$G/$NAME
S=$G/schnappschuss-symbole
if [ ! -f "$S/zeitpunkt.txt" ]; then
    mkdir -p "$S"
    cp -a ~/.local/share/icons/Querencia ~/.local/share/icons/hicolor "$S/"
    date '+%F %T' >"$S/zeitpunkt.txt"
fi
THEMA=$(readlink -f ~/.local/share/themes/Querencia-Klassisch)
[ -f "$THEMA/gtk-4.0/gtk-dark.css" ] || { echo "Thema ohne Dunkelfassung: $THEMA" >&2; exit 2; }
mkdir -p "$P"/{config/labwc,config/gtk-4.0,data/themes,data/icons,data/applications,state,logs,out,home}
cat >"$P/config/labwc/rc.xml" <<X
<?xml version="1.0"?>
<labwc_config>
  <core><decoration>server</decoration></core>
  <theme><name>Querencia-Klassisch</name></theme>
  <placement><policy>automatic</policy></placement>
  <windowRules>
    <windowRule identifier="org.querencia.Terminal" title="Querencia Terminal"><action name="MoveTo" x="0" y="0"/></windowRule>
  </windowRules>
</labwc_config>
X
ln -s "$THEMA" "$P/data/themes/Querencia-Klassisch"
cp -a "$S/Querencia" "$S/hicolor" "$P/data/icons/"
schrift=$(gsettings get org.gnome.desktop.interface font-name | tr -d "'")
mono=$(gsettings get org.gnome.desktop.interface monospace-font-name | tr -d "'")
printf '[Settings]\ngtk-font-name=%s\ngtk-icon-theme-name=Querencia\ngtk-theme-name=Querencia-Klassisch\n' "$schrift" >"$P/config/gtk-4.0/settings.ini"
# Probe statt Browser für http(s)-Adressen
printf '#!/bin/sh\nprintf "%%s\\n" "$1" >>%q\n' "$P/out/uri.log" >"$P/uri-probe"
chmod +x "$P/uri-probe"
cat >"$P/data/applications/qtd-uri-probe.desktop" <<X
[Desktop Entry]
Type=Application
Name=URI-Probe
Exec=$P/uri-probe %u
MimeType=x-scheme-handler/http;x-scheme-handler/https;
NoDisplay=true
X
printf '[Default Applications]\nx-scheme-handler/http=qtd-uri-probe.desktop\nx-scheme-handler/https=qtd-uri-probe.desktop\n' >"$P/config/mimeapps.list"
# Shell der Reiter: bash ohne ~/.bashrc, mit festem Prompt und Titel-Hilfe
: >"$P/inputrc"
cat >"$P/testrc" <<X
PS1='\$ '
HISTFILE=/dev/null
unset PROMPT_COMMAND
titel() { printf '\\033]0;%s\\007' "\$1"; }
X
printf '#!/bin/bash\nexport INPUTRC=%q\nexec /bin/bash --noprofile --rcfile %q -i\n' "$P/inputrc" "$P/testrc" >"$P/qshell"
chmod +x "$P/qshell"
cat >"$P/env.sh" <<X
W=$HIER
Q=$G/bin/qeingabe
QA=$G/bin/qausgabe
OUT=$P/out
RES=$RES
SCHEMA=$SCHEMA
SPRACHE=$SPRACHE
BIN=$BIN
AKTIONEN=${QTD_AKTIONEN:-ja}
SCHRIFT='$schrift'
MONO='$mono'
export GSK_RENDERER=cairo GDK_DISABLE=gl,vulkan LANG=en_US.UTF-8 LANGUAGE=$SPRACHE
X
{
    echo "Binary $BIN $(sha256sum "$BIN" | cut -d' ' -f1)"
    echo "Thema $THEMA; Symbole $(cat "$S/zeitpunkt.txt"); Schema $SCHEMA; Sprache $SPRACHE; Ausgang $RES"
} >"$P/out/umgebung.txt"
QTD_PORTAL=1 python3 -I "$HIER/isoliert.py" "$P" "$HIER/${QTD_SZENARIO:-szenario.sh}" "$T" || true
cat "$P/out/ergebnis.txt" 2>/dev/null || echo "kein Ergebnis" >&2
