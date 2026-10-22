"""Découpe lines.json en fiches structurées (entries.json)."""
import json, re, sys
L = json.load(open(sys.argv[1], encoding="utf-8"))

SECTION_MARK = {"LESSALLES": "salle", "EMPLOYÉS": "employe", "BÂTIMENTS": "batiment",
                "ORGANISATIONS": "organisation", "EXEMPLESDEGESTIONNAIRES": "gestionnaire"}
section, entries, cur, field = None, [], None, None
for l in L:
    t = l["text"].replace("\x08", " ")
    if l["font"] == "JubileeBold":
        section = SECTION_MARK.get(t.replace(" ", ""), section if t.replace(" ", "") not in
                                   ("COMMENTLIRELEPROFIL", "D’UNEUNITÉ?", "RETARDSSURLES", "CONSTRUCTIONSETLE",
                                    "RECRUTEMENT", "DÉLOCALISERUNE", "ORGANISATION") else None)
        cur = None; continue
    if l["font"] == "JubileeMedium" and t.startswith("Extensions"):
        section = "extension"; cur = None; continue
    if l["font"].startswith("CCTreacherous") or l["font"] in ("Pathfinder", "MasonSerifBold") or l["size"] > 12:
        continue
    if t.startswith("Les évènements d’intermède"):
        section = None; cur = None
    if section is None:
        continue
    if l["font"] == "DaxCondensed-Bold" and l["size"] == 11.0:
        if cur and not cur["champs"] and not cur["desc"] and cur["section"] == section:
            cur["nom"] += " " + t; continue  # titre sur deux lignes
        cur = {"nom": t, "section": section, "page": l["page"], "champs": {}, "desc": []}
        entries.append(cur); field = None; continue
    if cur is None:
        continue
    m = l["marked"].replace("\x08", " ")
    prev = cur["champs"].get(field, "") if field else ""
    cont = field and not cur["desc"] and (
        l["font"] == "Dax-Medium" or re.match(r"^[a-zà-ÿ(\d]", t) or re.search(r"([,(\d]| et| de| à| la| du| des| en| d’| aux)$", prev))
    if field == "Compétences" and not (l["font"] == "Dax-Medium"):
        cont = False
    if "§" in m or cont:
        # découpe sur les libellés en gras ; une ligne de continuation garde le libellé courant
        parts = re.split(r"((?:§\S+ ?)+)", m)
        label = field if not m.startswith("§") else None
        for p in parts:
            if not p.strip():
                continue
            if p.startswith("§"):
                label = p.replace("§", "").strip().rstrip(".").rstrip(":").strip()
                cur["champs"].setdefault(label, "")
            elif label:
                cur["champs"][label] = (cur["champs"][label] + " " + p.strip(" ;")).strip()
        field = label; continue
    if l["font"] == "Dax-Medium" and field:
        cur["champs"][field] = (cur["champs"][field] + " " + t.strip(" ;")).strip(); continue
    field = None
    cur["desc"].append(t)

for e in entries:
    e["description"] = " ".join(e.pop("desc"))
    # recoller les mots coupés « ab- cd »
    e["description"] = re.sub(r"(\w)- (\w)", r"\1\2", e["description"])
json.dump(entries, open(sys.argv[2], "w", encoding="utf-8"), ensure_ascii=False, indent=1)
from collections import Counter
print(Counter(e["section"] for e in entries))
print(Counter(k for e in entries for k in e["champs"]))
