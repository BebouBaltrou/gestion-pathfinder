"""Extraction par coordonnées (2 colonnes) des pages PDF 97-119 du Guide de campagne.
Produit lines.json : [{page_livre, col, top, font, size, text}] dans l'ordre de lecture."""
import json, sys, pdfplumber

pdf_path, out = sys.argv[1], sys.argv[2]
first, last = int(sys.argv[3]), int(sys.argv[4])
res = []
with pdfplumber.open(pdf_path) as pdf:
    for i in range(first - 1, last):
        pg = pdf.pages[i]
        mid = pg.width / 2
        words = pg.extract_words(extra_attrs=["fontname", "size"], keep_blank_chars=False, use_text_flow=False)
        rows = {}
        for w in words:
            if w["top"] > pg.height - 45:  # pied de page (numéro + filigrane)
                continue
            col = 0 if w["x0"] < mid else 1
            key = (col, round(w["top"] / 2))
            rows.setdefault(key, []).append(w)
        for (col, _), ws in sorted(rows.items()):
            ws.sort(key=lambda w: w["x0"])
            res.append({
                "page": i,  # page livre = page PDF (1-based) - 1 = index 0-based
                "col": col, "top": round(ws[0]["top"], 1),
                "font": ws[0]["fontname"].split("+")[-1], "size": round(ws[0]["size"], 1),
                "text": " ".join(w["text"] for w in ws),
                "fonts": sorted({w["fontname"].split("+")[-1] for w in ws}),
                # mots en gras de champ préfixés par § (libellés : Recettes, Durée…)
                "marked": " ".join(("§" if w["fontname"].split("+")[-1] in ("Dax-ExtraBold", "Dax-Bold") and round(w["size"], 1) == 8.5 else "") + w["text"] for w in ws),
            })
json.dump(res, open(out, "w", encoding="utf-8"), ensure_ascii=False, indent=0)
print(len(res), "lignes")
