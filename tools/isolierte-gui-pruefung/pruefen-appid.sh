#!/bin/bash
# App-ID org.querencia.Terminal: isolierte Sitzung mit Panel und Desktop der
# Live-Sitzung (nur deren Binaries, nur gelesen), den Startern aus
# integration/vorschlag (neuer Starter + Brücke unter der alten ID) und einer
# Kopie von desktop-symbole (nennt die alte ID). Geprüft wird: Start aus dem
# Anwendungsmenü, Name und Symbol in Fensterliste und Titelleiste, das
# Desktop-Symbol über die Brücke.
#   pruefen-appid.sh <panel> <desktop> <profil> <BREITExHÖHE> <szenario.txt> [ohne-bruecke]
# Ergebnis und Bilder: <gui-test>/<profil>/out, Protokolle in logs/.
set -euo pipefail
HIER="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PANEL=$(realpath -e "${1:?Panel-Binary}"); DESKTOP=$(realpath -e "${2:?Desktop-Binary}")
PROFIL="${3:?Profilname}"; GROESSE="${4:?BREITExHÖHE}"; SZQ=$(realpath -e "${5:?Szenario}")
VORSCHLAG="$HIER/../../integration/vorschlag"
G=${QT_GUI_TEST:-~/projects/querencia-terminal-gui-test}
mkdir -p "$G"
bash "$HIER/bauen.sh" "$G/bin" >/dev/null
. "$HIER/testprofil.sh"; testprofil_neu "$G" "$PROFIL"
P="$G/$PROFIL"
mkdir -p "$P/logs" "$P/out" "$P/config/labwc" "$P/config/dconf" "$P/data/applications" "$P/data/themes" \
    "$P/state/querencia" "$P/schreibtisch"
# labwc wie die Sitzung: Theme Querencia-Klassisch, Symbolthema gnome-human
cat > "$P/config/labwc/rc.xml" <<X
<?xml version="1.0"?>
<labwc_config>
  <core><decoration>server</decoration></core>
  <theme><name>Querencia-Klassisch</name><icon>gnome-human</icon></theme>
  <placement><policy>center</policy></placement>
</labwc_config>
X
ln -s "$(readlink -f ~/.local/share/themes/Querencia-Klassisch)" "$P/data/themes/Querencia-Klassisch"
# GTK-Einstellungen (Symbolthema, Schrift) als Kopie der Sitzung
[ -f ~/.config/dconf/user ] && cp ~/.config/dconf/user "$P/config/dconf/user"
printf 'XDG_DESKTOP_DIR="%s"\n' "$P/schreibtisch" > "$P/config/user-dirs.dirs"
# Starter: neuer Starter, Brücke (außer „ohne-bruecke“), Lage der Desktop-Symbole wie live
cp "$VORSCHLAG/org.querencia.Terminal.desktop" "$P/data/applications/"
[ "${6:-}" = ohne-bruecke ] || cp "$VORSCHLAG/org.example.Querencia.Terminal.desktop" "$P/data/applications/"
cp ~/.local/state/querencia/desktop-symbole "$P/state/querencia/desktop-symbole"
{
    echo "Panel:   $PANEL ($(sha256sum "$PANEL" | cut -c1-16))"
    echo "Desktop: $DESKTOP ($(sha256sum "$DESKTOP" | cut -c1-16))"
    echo "Größe:   $GROESSE; Brücke: ${6:-mit}"
    echo "Starter: $(cd "$P/data/applications" && sha256sum -- *.desktop | tr '\n' ' ')"
} > "$P/logs/ergebnis.txt"
SZ="$P/logs/szenario.txt.in"
HOEHE=${GROESSE#*x}
# Desktop-Symbol „Querencia Terminal“ (desktop-symbole: links 8, unten 8)
sed "s#OUT#$P/out#g; s#GROESSE#${GROESSE/x/ }#g; s#SYMBOLY#$((HOEHE - 104))#g" "$SZQ" > "$SZ"
cat > "$P/sitzung.sh" <<S
#!/bin/bash
exec >>"$P/logs/sitzung.txt" 2>&1
set -x
"$G/bin/qausgabe" "$GROESSE"
sleep 1
"$DESKTOP" >"$P/logs/desktop.log" 2>&1 &
sleep 2
QUERENCIA_MENU_DEBUG=1 "$PANEL" >"$P/logs/panel.log" 2>&1 &
sleep 5
"$G/bin/qeingabe" < "$SZ"
echo "qeingabe exit \$?"
sleep 1
for p in \$(pgrep -f 'opt/querencia-terminal/'); do
    tr '\0' '\n' </proc/\$p/environ 2>/dev/null | grep -qx "XDG_RUNTIME_DIR=\$XDG_RUNTIME_DIR" || continue
    echo "Terminal PID \$p exe \$(readlink /proc/\$p/exe)"
done
touch "$P/logs/FERTIG"
sleep 30
S
HMP04_DATA_DIRS="$P/data:/usr/local/share:/usr/share" LANG=en_US.UTF-8 LANGUAGE=de \
    /usr/bin/python3 "$HIER/isoliert.py" "$P" "$P/sitzung.sh" "${QT_ZEIT:-150}" || true
cat "$P/logs/ergebnis.txt"; ls "$P/out"
