"""Construit data/referentiel_officiel.json à partir de ref.json (sortie de normalize.py)
et des tables générales relevées à la main dans le Guide de campagne.

Usage : python3 -I build_data.py ref.json ../../data/referentiel_officiel.json
"""
import json, sys

R = json.load(open(sys.argv[1], encoding="utf-8"))
GC, GDM = "GC", "GDM"

def src(page, section, livre=GC):
    return {"livre": livre, "page": page, "section": section}

RES = {"gp": "Po", "goods": "Marchandises", "influence": "Influence", "labor": "Travail", "magic": "Magie"}

def cout(c):
    return None if c is None else {"marchandises": c["goods"], "influence": c["influence"],
                                   "travail": c["labor"], "magie": c["magic"]}

def recettes(r):
    if not r:
        return None
    mode = r["mode"]
    adm = ({"type": "Liste", "ressources": [RES[t] for t in r["types"]]} if mode == "liste" else
           {"type": {"tous": "Toutes", "types_existants": "TypesDuBatiment", "salle_hote": "SalleHote"}[mode]})
    return {"bonus": r["bonus"], "admissibles": adm, "texte": r["texte"]}

def taille(t):
    if not t: return None
    if "min_cases" in t: return {"type": "Cases", "min": t["min_cases"], "max": t["max_cases"]}
    if "personnes" in t: return {"type": "Personnes", "nombre": t["personnes"]}
    return {"type": "Texte", "texte": t["texte"]}

def unite(r, section):
    return {"id": r["id"], "nom": r["nom"], "source": src(r["source"]["page"], section),
            "cout": cout(r["cout"]), "prix_po": r["prix_po"], "duree_jours": r["duree_jours"],
            "recettes": recettes(r["recettes"]), "avantages": r.get("avantages"),
            "taille": taille(r["taille"]), "amelioration_de": r["amelioration_de"],
            "amelioration_en": r["amelioration_en"], "point_a_confirmer": None}

salles = [unite(r, "Les salles") for r in R["salles"]]
for s in salles:
    if s["id"] == "labyrinthe": s["point_a_confirmer"] = "PC-07"
extensions = [unite(r, "Extensions de salle") for r in R["extensions"]]
equipes = [unite(r, "Employés") for r in R["equipes"]]
gestionnaires = [{"id": g["id"], "nom": g["nom"], "source": src(g["source"]["page"], "Exemples de gestionnaires"),
                  "salaire_po_jour": g["salaire_po_jour"], "competences": g["competences"]} for g in R["gestionnaires"]]

def modele(b, section, correction=None):
    comp = []
    for it in b["composition"]:
        if "extension" in it:
            comp.append({"type": "Extension", "id": it["extension"], "quantite": it["quantite"], "sur": it["sur"]})
        else:
            comp.append({"type": "Unite", "id": it["id"], "quantite": it["quantite"]})
    m = {"id": b["id"], "nom": b["nom"], "source": src(b["source"]["page"], section), "cout": cout(b["cout"]),
         "prix_po": b["prix_po"], "composition": comp, "composition_imprimee": b["composition_texte"],
         "point_a_confirmer": None}
    if correction:
        correction(m)
    return m

def arene(m):  # PC-08 : la VF ajoute un jardin absent de la VO ; le total imprimé l'exclut
    m["composition"] = [c for c in m["composition"] if c["id"] != "jardin"]
    m["point_a_confirmer"] = "PC-08"

def caravaniers(m):  # PC-09 : la VF indique 1 conducteur d'attelage, la VO 2 ; le total imprimé en compte 2
    for c in m["composition"]:
        if c["id"] == "conducteur-d-attelage": c["quantite"] = 2
    m["point_a_confirmer"] = "PC-09"

batiments = [modele(b, "Les bâtiments", arene if b["id"] == "arene" else None) for b in R["batiments"]]
organisations = [modele(o, "Les organisations", caravaniers if o["id"] == "caravaniers" else None)
                 for o in R["organisations"]]

capitaux = [{"ressource": n, "cout_achat_po": a, "cout_produit_po": p, "source": src(78, "Table 2-1 : Valeurs des capitaux")}
            for n, a, p in (("Marchandises", 20, 10), ("Influence", 30, 15), ("Travail", 20, 10), ("Magie", 100, 50))]

COMM = [("lieu-dit", "Lieu-dit", 0, 20, 2), ("hameau", "Hameau", 21, 60, 4), ("village", "Village", 61, 200, 10),
        ("petite-bourgade", "Petite bourgade", 201, 2000, 15), ("grosse-bourgade", "Grosse bourgade", 2001, 5000, 25),
        ("petite-ville", "Petite ville", 5001, 10000, 35), ("grande-ville", "Grande ville", 10001, 25000, 50),
        ("metropole", "Métropole", 25001, None, 65)]
communautes = [{"id": i, "nom": n, "population_min": a, "population_max": b, "limite_depense_jour": l,
                "source_population": {"livre": GDM, "page": None, "section": "Le profil d’une communauté — Fourchette de population des communautés"},
                "source_limite": src(81, "Limites des dépenses"),
                "point_a_confirmer": {"metropole": "PC-02", "lieu-dit": "PC-11"}.get(i)} for i, n, a, b, l in COMM]

def pc(code, question, source, lecture, statut, decision=None):
    return {"code": code, "question": question, "source": source, "lecture_appliquee": lecture,
            "statut": statut, "decision": decision}

V = "Valide"
points = [
    pc("PC-01", "La Magie compte-t-elle dans la limite des dépenses quotidienne ?", src(81, "Limites des dépenses"),
       "Oui, comme l'en-tête du tableau (Marchandises, Influence, Travail ou Magie).", V, "Validé le 22/10/2026"),
    pc("PC-02", "La « Capitale » du tableau des limites (65) correspond-elle à la Métropole ?", src(81, "Limites des dépenses"),
       "Oui : Capitale = Métropole.", V, "Validé le 22/10/2026"),
    pc("PC-03", "Les po dépensées pour acheter une salle terminée comptent-elles dans la limite ?", src(85, "Construction de bâtiments"),
       "Non : la limite porte sur les unités de capital.", V, "Validé le 22/10/2026"),
    pc("PC-04", "Une salle « +12 » sans type ne produit-elle que des po ?", src(94, "Recettes"),
       "Oui ; confirmé par le PDF (« +12 po »).", V, "Validé le 22/10/2026"),
    pc("PC-05", "Que signifie « un type que le bâtiment génère déjà » (Clocher, Cour, Étalage) ?", src(98, "Les salles"),
       "Un type admis par une autre salle du même bâtiment.", V, "Validé le 22/10/2026"),
    pc("PC-06", "Un gestionnaire peut-il gérer plusieurs établissements pour un seul salaire ?", src(89, "Les gestionnaires"),
       "Oui : salaire compté une fois par jour (règle de l'utilisateur).", V, "Validé le 22/10/2026"),
    pc("PC-07", "Labyrinthe : 370 po imprimés pour 15 Marchandises + 15 Travail (600 po calculés).", src(102, "Labyrinthe"),
       "Garder 370 po (texte imprimé, identique en VO), avec avertissement d'incohérence.", V, "Validé le 22/10/2026"),
    pc("PC-08", "Arène : la VF liste « 1 jardin » absent de la VO ; le total imprimé (109 M, 20 I, 96 T, 1 Mg) l'exclut.",
       src(111, "Arène"), "Composition sans jardin (conforme à la VO et au total imprimé).", "Ouvert"),
    pc("PC-09", "Caravaniers : la VF indique 1 conducteur d'attelage, la VO 2 ; le total imprimé (6 M, 3 I, 7 T) en compte 2.",
       src(118, "Caravaniers"), "2 conducteurs d'attelage (conforme à la VO et au total imprimé).", "Ouvert"),
    pc("PC-10", "Accélération : comment arrondir une durée divisée qui ne tombe pas juste (ex. 40 jours ÷ 3) ?",
       src(95, "Durée"), "Arrondi au jour supérieur (14 jours).", "Ouvert"),
    pc("PC-11", "Population de 20 exactement : lieu-dit (« moins de 20 ») ou hameau (« 21-60 ») ?",
       {"livre": GDM, "page": None, "section": "Fourchette de population des communautés"}, "Lieu-dit.", "Ouvert"),
]

out = {
    "format": 1,
    "sources": [
        {"code": GC, "titre": "Guide de campagne (Pathfinder JdR) — VF de Ultimate Campaign", "editeur": "Black Book Editions / Paizo",
         "url": "https://www.pathfinder-fr.org/Wiki/Pathfinder-RPG.Interm%c3%a8des.ashx"},
        {"code": GDM, "titre": "Guide du maître (Pathfinder JdR) — VF de GameMastery Guide", "editeur": "Black Book Editions / Paizo",
         "url": "https://www.pathfinder-fr.org/Wiki/Pathfinder-RPG.Villes.ashx"},
    ],
    "capitaux": capitaux, "communautes": communautes, "salles": salles, "extensions": extensions,
    "equipes": equipes, "gestionnaires": gestionnaires, "modeles_batiments": batiments,
    "modeles_organisations": organisations, "points_a_confirmer": points,
}
json.dump(out, open(sys.argv[2], "w", encoding="utf-8"), ensure_ascii=False, indent=1)
print({k: len(v) for k, v in out.items() if isinstance(v, list)})
