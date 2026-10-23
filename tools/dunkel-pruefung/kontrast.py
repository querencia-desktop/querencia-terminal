#!/usr/bin/env python3
"""Querencia Terminal 0.3.1: WCAG-Kontrast der Farbpaare von Menüleiste,
Reiterleiste und Menüs in Querencia-Klassisch hell und dunkel (Werte aus
gtk.css bzw. gtk-dark.css des installierten Themas, nur gelesen).
Aufruf: kontrast.py [<thema-ordner gtk-4.0>]"""
import os
import re
import sys

ORDNER = sys.argv[1] if len(sys.argv) > 1 else os.path.realpath(
    os.path.expanduser("~/.local/share/themes/Querencia-Klassisch/gtk-4.0"))
PAARE = [
    ("Menüleiste: Schrift auf Fläche", "theme_fg_color", "theme_bg_color"),
    ("Reiterleiste: Reitertitel auf Fläche", "theme_fg_color", "theme_bg_color"),
    ("aktiver Reiter: Titel auf Fläche", "theme_fg_color", "theme_base_color"),
    ("Menüs: Eintrag auf Fläche", "menu_fg_color", "menu_bg_color"),
    ("Menüs: gesperrter Eintrag auf Fläche", "insensitive_fg_color", "menu_bg_color"),
]


def farben(datei, letzte):
    """gtk.css: die erste Definition (spätere gehören zu dunklen Teilen),
    gtk-dark.css: die letzte (frühere sind die hellen Grundwerte)."""
    werte = {}
    for name, wert in re.findall(r"^@define-color (\w+) (#[0-9a-fA-F]{6});", open(datei).read(), re.M):
        if letzte or name not in werte:
            werte[name] = wert
    return werte


def lum(h):
    def k(c):
        c = int(c, 16) / 255
        return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4
    r, g, b = (k(h[i:i + 2]) for i in (1, 3, 5))
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def kontrast(a, b):
    la, lb = sorted((lum(a), lum(b)), reverse=True)
    return (la + 0.05) / (lb + 0.05)


for schema, datei in (("hell", "gtk.css"), ("dunkel", "gtk-dark.css")):
    f = farben(os.path.join(ORDNER, datei), schema == "dunkel")
    for was, vorn, grund in PAARE:
        k = kontrast(f[vorn], f[grund])
        stufe = "AA" if k >= 4.5 else ("AA groß" if k >= 3 else "unter AA")
        if "gesperrt" in was:
            stufe += " (gesperrt, WCAG nimmt aus)"
        print(f"{schema:6} {was}: {f[vorn]} auf {f[grund]} = {k:.2f}:1 {stufe}")
