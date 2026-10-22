import json, sys
R = json.load(open(sys.argv[1], encoding="utf-8"))
BUY = {"goods": 20, "influence": 30, "labor": 20, "magic": 100}
K = ["goods", "influence", "labor", "magic"]
fmt = lambda c: " ".join(f"{c[k]}{k[0].upper()}" for k in K if c[k])
gp = lambda c: sum(c[k] * BUY[k] for k in K)

print("== 1. Prix po imprimé ≠ somme des coûts d'achat ==")
for kind in ("salles", "extensions", "equipes", "batiments", "organisations"):
    for r in R[kind]:
        if r["cout"] and r["prix_po"] is not None and gp(r["cout"]) != r["prix_po"]:
            print(f"  {kind:13} {r['nom']:30} p.{r['source']['page']}  {fmt(r['cout'])} imprimé {r['prix_po']} po, calculé {gp(r['cout'])} po")

print("== 2. Coût du modèle ≠ somme de ses éléments ==")
idx = {r["id"]: r for k in ("salles", "equipes") for r in R[k]}
ext = {r["id"]: r for r in R["extensions"]}
for kind in ("batiments", "organisations"):
    for b in R[kind]:
        tot = {k: 0 for k in K}
        for it in b["composition"]:
            src = ext[it["extension"]] if "extension" in it else idx[it["id"]]
            for k in K: tot[k] += src["cout"][k] * it["quantite"]
        if tot != b["cout"]:
            print(f"  {kind:13} {b['nom']:30} p.{b['source']['page']}  imprimé {fmt(b['cout'])} | somme {fmt(tot)}")

print("== 3. Liens d'amélioration non réciproques ==")
for kind in ("salles", "equipes"):
    by = {r["id"]: r for r in R[kind]}
    for r in R[kind]:
        for t in r["amelioration_en"]:
            if r["id"] not in by[t]["amelioration_de"]:
                print(f"  {r['nom']} → {by[t]['nom']} (absent de « amélioration de » de la cible)")
        for t in r["amelioration_de"]:
            if r["id"] not in by[t]["amelioration_en"]:
                print(f"  {by[t]['nom']} → {r['nom']} (absent de « amélioration en » de la source)")

print("== 4. Recettes ==")
from collections import Counter
print(" ", Counter((r["recettes"] or {}).get("mode", "aucune") for k in ("salles", "equipes", "extensions") for r in R[k]))
print("== 5. Gestionnaires ==", [(g["nom"], g["salaire_po_jour"]) for g in R["gestionnaires"] if not g["salaire_po_jour"]])
