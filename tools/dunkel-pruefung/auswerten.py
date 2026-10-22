#!/usr/bin/env python3
"""Querencia Terminal 0.3.1: Auswertung der Prüfung hell/dunkel (QD-109).

Liest die Fotos eines Laufs von lauf.sh und prüft
- Farbproben an festen Stellen (Menüleiste, Reiterleiste, aktiver Reiter,
  geöffnetes Menü, Kontextmenü) gegen die Werte des Themas
  Querencia-Klassisch hell bzw. dunkel; Satz a/c im Startschema, Satz b im
  anderen (Wechsel im laufenden Programm);
- Satz c (zurückgeschaltet) pixelgleich zu Satz a (ohne die blinkende
  Schreibmarke, MASKE);
- keine GTK-Meldungen außer den erwarteten Portal-Hinweisen.

Aufruf:  auswerten.py <lauf-ordner> [<vergleichs-lauf> …]
Mit Vergleichsläufen zusätzlich: gleichnamige Fotos pixelgleich bzw. Zahl
und Rahmen der abweichenden Pixel (z. B. 0.3.0 gegen 0.3.1, hell).
"""

import os
import sys

import gi

gi.require_version("GdkPixbuf", "2.0")
from gi.repository import GdkPixbuf  # noqa: E402

# Erwartete Farben (Thema Querencia-Klassisch: gtk.css / gtk-dark.css)
SOLL = {
    "hell": {"leiste": "f2f1f0", "aktiv": "ffffff", "menue": "f5f4f2"},
    "dunkel": {"leiste": "3c3b37", "aktiv": "2b2a27", "menue": "353430"},
}
# Probenstellen (Ausgang 1280x800, Fenster bei 0,0); x des Hilfe-Menüs je Sprache
PROBEN = [
    ("01-reiter", "Menüleiste frei", 600, 40, "leiste"),
    ("01-reiter", "Reiterleiste frei", 780, 70, "leiste"),
    ("01-reiter", "aktiver Reiter", 650, 62, "aktiv"),
    ("03-menue-hilfe", "Hilfe-Menü offen (Fläche)", None, 100, "menue"),
    ("02-menue-datei", "Datei-Menü offen (Fläche)", 18, 100, "menue"),
    ("06-kontext", "Kontextmenü (Fläche)", 310, 410, "menue"),
]
HILFE_MENUE_X = {"de": 312, "en": 205}
# Schreibmarke der Shell in Zeile 2 (blinkt, Zustand hängt vom Zeitpunkt
# ab): beim Vergleich zweier Fotos ausgenommen und als Hinweis gezählt.
MASKE = (20, 104, 36, 128)


def bild(pfad):
    pb = GdkPixbuf.Pixbuf.new_from_file(pfad)
    return pb.get_width(), pb.get_height(), pb.get_n_channels(), pb.get_rowstride(), pb.get_pixels()


def pixel(b, x, y):
    _, _, n, rs, d = b
    o = y * rs + x * n
    return "%02x%02x%02x" % (d[o], d[o + 1], d[o + 2])


def abweichung(a, b, maske=MASKE):
    """Zahl und Rahmen der abweichenden Pixel außerhalb der Maske, dazu die
    Zahl der abweichenden Pixel in der Maske."""
    wa, ha, na, ra, da = a
    wb, hb, nb, rb, db = b
    if (wa, ha) != (wb, hb):
        return -1, None, 0
    zahl, maskiert, x0, y0, x1, y1 = 0, 0, wa, ha, -1, -1
    mx0, my0, mx1, my1 = maske
    for y in range(ha):
        za = da[y * ra:y * ra + wa * na]
        zb = db[y * rb:y * rb + wb * nb]
        if za == zb:
            continue
        for x in range(wa):
            if za[x * na:x * na + 3] != zb[x * nb:x * nb + 3]:
                if mx0 <= x <= mx1 and my0 <= y <= my1:
                    maskiert += 1
                    continue
                zahl += 1
                x0, y0, x1, y1 = min(x0, x), min(y0, y), max(x1, x), max(y1, y)
    return zahl, (x0, y0, x1, y1) if zahl else None, maskiert


def umgebung(lauf):
    werte = {}
    with open(os.path.join(lauf, "out", "umgebung.txt")) as f:
        for teil in f.read().replace(";", "\n").split("\n"):
            teil = teil.strip()
            if teil.startswith("Schema "):
                werte["schema"] = teil.split()[1]
            elif teil.startswith("Sprache "):
                werte["sprache"] = teil.split()[1]
    return werte


def main():
    lauf = sys.argv[1]
    out = os.path.join(lauf, "out")
    u = umgebung(lauf)
    start = u["schema"]
    anders = "dunkel" if start == "hell" else "hell"
    ergebnisse = []

    def ergebnis(ok, text):
        ergebnisse.append(("PASS" if ok else "FAIL") + " " + text)

    for satz, schema in (("a", start), ("b", anders), ("c", start)):
        for foto, was, x, y, art in PROBEN:
            pfad = os.path.join(out, f"{satz}-{foto}.png")
            if x is None:
                x = HILFE_MENUE_X[u["sprache"]]
            ist = pixel(bild(pfad), x, y)
            soll = SOLL[schema][art]
            ergebnis(ist == soll, f"{satz} ({schema}) {was} bei {x},{y}: #{ist} (soll #{soll})")
    for name in sorted(os.listdir(out)):
        if name.startswith("a-") and name.endswith(".png"):
            c = name.replace("a-", "c-", 1)
            zahl, rahmen, m = abweichung(bild(os.path.join(out, name)), bild(os.path.join(out, c)))
            ergebnis(zahl == 0, f"zurückgeschaltet {c} = {name}" + ("" if zahl == 0 else f" ({zahl} Pixel, {rahmen})")
                     + (f" [Schreibmarke: {m} Pixel]" if m else ""))
    with open(os.path.join(out, "gtk-meldungen.txt")) as f:
        n = int(f.read().strip() or "0")
    ergebnis(n == 0, f"GTK-Meldungen außer Portal-Hinweisen: {n}")
    for vergleich in sys.argv[2:]:
        vout = os.path.join(vergleich, "out")
        for name in sorted(os.listdir(out)):
            if not (name.startswith("a-") and name.endswith(".png")) or not os.path.exists(os.path.join(vout, name)):
                continue
            zahl, rahmen, m = abweichung(bild(os.path.join(out, name)), bild(os.path.join(vout, name)))
            ergebnisse.append(f"VERGLEICH {name} gegen {os.path.basename(vergleich)}: "
                              + ("pixelgleich" if zahl == 0 else f"{zahl} Pixel anders, Rahmen {rahmen}")
                              + (f" [Schreibmarke: {m} Pixel]" if m else ""))
    text = "\n".join(ergebnisse) + "\n"
    with open(os.path.join(out, "auswertung.txt"), "w") as f:
        f.write(text)
    print(text, end="")
    return 0 if not any(z.startswith("FAIL") for z in ergebnisse) else 1


if __name__ == "__main__":
    sys.exit(main())
