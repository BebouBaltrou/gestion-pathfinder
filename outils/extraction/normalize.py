"""entries.json -> référentiel structuré (ref.json) + rapport d'anomalies."""
import json, re, sys, unicodedata

E = json.load(open(sys.argv[1], encoding="utf-8"))
anom = []

def slug(s):
    s = unicodedata.normalize("NFD", s.lower().replace("’", "-").replace("'", "-"))
    s = "".join(c for c in s if unicodedata.category(c) != "Mn")
    return re.sub(r"[^a-z0-9]+", "-", s).strip("-")

def nom_propre(s):
    s = s.replace(" (EXTENSION)", "").lower()
    return s[:1].upper() + s[1:]

CAP = {"Marchandises": "goods", "Influence": "influence", "Travail": "labor", "Magie": "magic"}
BUY = {"goods": 20, "influence": 30, "labor": 20, "magic": 100}

def parse_cost(txt, who):
    txt = txt.lstrip(". ").replace(" ", " ").replace("\xa0", " ")
    cost = {k: 0 for k in BUY}
    m = re.search(r"\(([\d ]+)\s*po\)", txt)
    gp = int(m.group(1).replace(" ", "")) if m else None
    for n, k in re.findall(r"(\d+)\s+(Marchandises|Influence|Travail|Magie)", txt):
        cost[CAP[k]] += int(n)
    if gp is None:
        anom.append(f"{who}: prix en po illisible « {txt} »")
    return cost, gp

def parse_days(txt, who):
    m = re.match(r"(\d+)\s+jours?", txt or "")
    if not m:
        anom.append(f"{who}: durée non numérique « {txt} »"); return None
    return int(m.group(1))

KINDS = {"po": "gp", "Marchandises": "goods", "Influence": "influence", "Travail": "labor", "Magie": "magic"}
def parse_earn(txt, who):
    if not txt:
        return None
    m = re.match(r"\+(\d+)\s*(.*)", txt)
    if not m:
        anom.append(f"{who}: recettes illisibles « {txt} »"); return {"raw": txt}
    bonus, rest = int(m.group(1)), m.group(2).strip()
    if rest.lower().startswith("capitaux"):
        if "génère" in rest or "produit déjà" in rest:
            return {"bonus": bonus, "mode": "types_existants", "texte": txt}
        return {"bonus": bonus, "mode": "tous", "texte": txt}
    if rest.startswith("au test"):
        return {"bonus": bonus, "mode": "salle_hote", "texte": txt}
    types = [KINDS[w] for w in re.findall(r"po|Marchandises|Influence|Travail|Magie", rest)]
    if not types:
        anom.append(f"{who}: recettes sans type « {txt} »")
    return {"bonus": bonus, "mode": "liste", "types": types, "texte": txt}

def parse_size(txt, unit):
    m = re.match(r"(\d+)\s*-\s*(\d+)\s*cases", txt or "")
    if m: return {"min_cases": int(m.group(1)), "max_cases": int(m.group(2))}
    m = re.match(r"(\d+)\s*personnes?", txt or "")
    if m: return {"personnes": int(m.group(1))}
    return {"texte": txt} if txt else None

def upgrades(ch):
    de, en = [], []
    items = list(ch.items())
    for k, v in items:
        if not k.startswith("Amélioration") and k != "en":
            continue
        key = k.replace("Améliorations", "Amélioration")
        if key == "Amélioration" and v == "":
            continue
        if key == "en" or key.startswith("Amélioration en"):
            extra = key[len("Amélioration en"):].strip() if key.startswith("Amélioration en") else ""
            en.append((extra + v).strip())
        else:
            v = re.sub(r"^(la|le|les)\s+", "", v)
            de.append(v)
    split = lambda xs: [x.strip() for s in xs for x in re.split(r",| et ", s) if x.strip()]
    return split(de), split(en)

out = {"salles": [], "extensions": [], "equipes": [], "gestionnaires": [], "batiments": [], "organisations": []}
SRC = "Guide de campagne (Ultimate Campaign VF)"
for e in E:
    sec, ch, nom = e["section"], e["champs"], nom_propre(e["nom"])
    who = f"{sec} « {nom} » p.{e['page']}"
    base = {"id": slug(nom), "nom": nom, "source": {"livre": SRC, "page": e["page"]}, "statut": "officiel"}
    if sec in ("salle", "extension", "employe"):
        rec = dict(base)
        if "Création" in ch:
            rec["cout"], rec["prix_po"] = parse_cost(ch["Création"], who)
        else:
            rec["cout"], rec["prix_po"] = None, None
            anom.append(f"{who}: coût « voir description » (pas de valeur fixe)")
        rec["duree_jours"] = parse_days(ch.get("Durée"), who) if "Création" in ch else None
        rec["recettes"] = parse_earn(ch.get("Recettes"), who)
        av = ch.get("Avantages") or ch.get("Avantage")
        if av: rec["avantages"] = av
        rec["taille"] = parse_size(ch.get("Taille"), sec)
        rec["amelioration_de"], rec["amelioration_en"] = upgrades(ch)
        out[{"salle": "salles", "extension": "extensions", "employe": "equipes"}[sec]].append(rec)
    elif sec == "gestionnaire":
        m = re.match(r"(\d+)\s*po", ch.get("Salaire", ""))
        rec = dict(base, salaire_po_jour=int(m.group(1)) if m else None,
                   competences=ch.get("Compétences", "").rstrip(", "))
        out["gestionnaires"].append(rec)
    else:
        rec = dict(base)
        rec["cout"], rec["prix_po"] = parse_cost(ch.get("Création", ""), who)
        comp = ch.get("Salles") or ch.get("Employés") or ""
        rec["composition_texte"] = comp.rstrip(".")
        out["batiments" if sec == "batiment" else "organisations"].append(rec)

# ---- résolution des noms (pluriels) ----
def candidates(s):
    s = s.strip().rstrip(".").lower().replace("’", "’")
    yield s
    words = s.split()
    sing = [re.sub(r"(?<=[a-zé])x$", "", re.sub(r"(?<=[a-zéè])s$", "", w)) if w not in ("de", "du", "des", "aux", "à") else w for w in words]
    yield " ".join(sing)
    yield " ".join([sing[0]] + words[1:])
    yield s.replace(" ", "-")
    yield " ".join(sing).replace(" ", "-")

ALIAS = {"chambre forte": "chambre-forte", "chambres fortes": "chambre-forte", "dstatue": "statue",
         "salles de bain": "salle de bain", "salles des trophées": "salle des trophées", "salles de combat": "salle de combat",
         "salles de classe": "salle de classe", "chambres à coucher": "chambre à coucher", "postes de travail": "poste de travail",
         "abris pour animaux": "abri pour animaux", "murs défensifs": "mur défensif", "sites funéraires": "site funéraire",
         "archers d’élite": "archers d’élite", "soldats d’élite": "soldats d’élite", "conducteurs d’attelage": "conducteur d’attelage",
         "laquai": "laquais", "cambrioleur": "cambrioleurs", "curé": "curé", "acolytes": "acolyte", "apprentis-mages": "apprenti-mage",
         "garde": "gardes", "travailleur": "travailleurs", "marin": "marins", "fraudeur": "fraudeurs", "archer": "archers",
         "bureaucrate": "bureaucrates", "artisan": "artisans", "soldat": "soldats", "tire-laine": "tire-laine",
         "mage": "mage", "sage": "sage", "auditorium": "auditorium", "étalage (voir ci-dessous)": "étalage", "archer d’élite": "archers d’élite", "observatoire": "dôme d’observation"}

def resolve(name, pool, who):
    by = {r["nom"].lower(): r["id"] for r in pool}
    n = ALIAS.get(name.lower().strip(), name)
    for c in candidates(n):
        if c in by: return by[c]
    anom.append(f"{who}: nom non résolu « {name} »")
    return None

pools = {"salles": out["salles"], "equipes": out["equipes"]}
for kind in ("salles", "equipes"):
    for r in out[kind]:
        who = f"{kind} « {r['nom']} »"
        r["amelioration_de"] = [x for x in (resolve(n, pools[kind], who) for n in r["amelioration_de"]) if x]
        r["amelioration_en"] = [x for x in (resolve(n, pools[kind], who) for n in r["amelioration_en"]) if x]

def composition(txt, who, rooms):
    res = []
    for part in [p.strip() for p in re.split(r",(?![^()]*\))", txt) if p.strip()]:
        m = re.match(r"(\d+)\s+(.*)", part)
        if not m:
            anom.append(f"{who}: élément de composition illisible « {part} »"); continue
        q, name = int(m.group(1)), m.group(2).strip()
        ext = re.match(r"(ameublements?|fortifications?)\s*\((.*)\)", name)
        if ext and rooms:
            res.append({"extension": "ameublement" if name.startswith("ameub") else "fortification", "quantite": q, "sur": ext.group(2)})
            continue
        rid = resolve(name, pools["salles" if rooms else "equipes"], who)
        if rid: res.append({"id": rid, "quantite": q})
    return res

for kind, rooms in (("batiments", True), ("organisations", False)):
    for r in out[kind]:
        r["composition"] = composition(r["composition_texte"], f"{kind} « {r['nom']} »", rooms)

json.dump(out, open(sys.argv[2], "w", encoding="utf-8"), ensure_ascii=False, indent=1)
json.dump(anom, open(sys.argv[3], "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print({k: len(v) for k, v in out.items()}, len(anom), "anomalies")
for a in anom: print(" -", a)
