"""Génère crates/regles/src/donnees_officielles.rs depuis data/referentiel_officiel.json.
Ne pas modifier le fichier généré à la main : modifier le JSON puis relancer ce script.

Usage : python3 -I gen_rust.py ../../data/referentiel_officiel.json ../../crates/regles/src/donnees_officielles.rs
"""
import json, sys

D = json.load(open(sys.argv[1], encoding="utf-8"))
q = lambda s: json.dumps(s, ensure_ascii=False)  # littéral de chaîne Rust valide (échappements compatibles)
opt = lambda v, f=str: "None" if v is None else f"Some({f(v)})"
qs = lambda v: f"{q(v)}.to_string()"

def src(s):
    return f"s({q(s['livre'])}, {opt(s['page'])}, {q(s['section'])})"

def cout(c):
    return f"c({c['marchandises']}, {c['influence']}, {c['travail']}, {c['magie']})"

def rec(r):
    if r is None: return "None"
    a = r["admissibles"]
    adm = {"Toutes": "Admissibles::Toutes", "TypesDuBatiment": "Admissibles::TypesDuBatiment",
           "SalleHote": "Admissibles::SalleHote"}.get(a["type"])
    if adm is None:
        adm = "Admissibles::Liste { ressources: vec![" + ", ".join(f"Ressource::{x}" for x in a["ressources"]) + "] }"
    return f"Some(Recettes {{ bonus: {r['bonus']}, admissibles: {adm}, texte: {qs(r['texte'])} }})"

def taille(t):
    if t is None: return "None"
    if t["type"] == "Cases": return f"Some(Taille::Cases {{ min: {t['min']}, max: {t['max']} }})"
    if t["type"] == "Personnes": return f"Some(Taille::Personnes {{ nombre: {t['nombre']} }})"
    return f"Some(Taille::Texte {{ texte: {qs(t['texte'])} }})"

ids = lambda xs: "&[" + ", ".join(q(x) for x in xs) + "]"

def unite(u):
    return (f"        u({q(u['id'])}, {q(u['nom'])}, {src(u['source'])}, {opt(u['cout'], cout)}, {opt(u['prix_po'])}, "
            f"{opt(u['duree_jours'])}, {rec(u['recettes'])}, {opt(u['avantages'], q)}, {taille(u['taille'])}, "
            f"{ids(u['amelioration_de'])}, {ids(u['amelioration_en'])}, {opt(u['point_a_confirmer'], q)}),")

def elem(e):
    if e["type"] == "Extension":
        return f"ElementModele::Extension {{ id: {qs(e['id'])}, quantite: {e['quantite']}, sur: {qs(e['sur'])} }}"
    return f"ElementModele::Unite {{ id: {qs(e['id'])}, quantite: {e['quantite']} }}"

def modele(m):
    return (f"        m({q(m['id'])}, {q(m['nom'])}, {src(m['source'])}, {cout(m['cout'])}, {m['prix_po']}, vec![" +
            ", ".join(elem(e) for e in m["composition"]) + f"], {q(m['composition_imprimee'])}, {opt(m['point_a_confirmer'], q)}),")

L = ["// @generated par outils/extraction/gen_rust.py depuis data/referentiel_officiel.json — ne pas éditer.",
     "#![allow(clippy::too_many_arguments)]",
     "use crate::referentiel::Referentiel;",
     "use domaine::fiches::*;",
     "use domaine::{CoutCapitaux, Ressource, Source};",
     "",
     "fn s(livre: &str, page: Option<u32>, section: &str) -> Source {",
     "    Source { livre: livre.to_string(), page, section: section.to_string() }",
     "}",
     "fn c(marchandises: u32, influence: u32, travail: u32, magie: u32) -> CoutCapitaux {",
     "    CoutCapitaux { marchandises, influence, travail, magie }",
     "}",
     "fn u(id: &str, nom: &str, source: Source, cout: Option<CoutCapitaux>, prix_po: Option<u32>, duree_jours: Option<u32>,",
     "     recettes: Option<Recettes>, avantages: Option<&str>, taille: Option<Taille>, de: &[&str], en: &[&str], pc: Option<&str>) -> Unite {",
     "    Unite { id: id.to_string(), nom: nom.to_string(), source, cout, prix_po, duree_jours, recettes,",
     "            avantages: avantages.map(str::to_string), taille,",
     "            amelioration_de: de.iter().map(|x| x.to_string()).collect(),",
     "            amelioration_en: en.iter().map(|x| x.to_string()).collect(),",
     "            point_a_confirmer: pc.map(str::to_string) }",
     "}",
     "fn m(id: &str, nom: &str, source: Source, cout: CoutCapitaux, prix_po: u32, composition: Vec<ElementModele>,",
     "     imprimee: &str, pc: Option<&str>) -> Modele {",
     "    Modele { id: id.to_string(), nom: nom.to_string(), source, cout, prix_po, composition,",
     "             composition_imprimee: imprimee.to_string(), point_a_confirmer: pc.map(str::to_string) }",
     "}",
     "",
     "/// Référentiel officiel tel que relevé dans le Guide de campagne.",
     "pub fn referentiel_officiel() -> Referentiel {",
     "    Referentiel {",
     "    sources: vec!["]
for x in D["sources"]:
    L.append(f"        LivreSource {{ code: {qs(x['code'])}, titre: {qs(x['titre'])}, editeur: {qs(x['editeur'])}, url: {qs(x['url'])} }},")
L.append("    ],\n    capitaux: vec![")
for x in D["capitaux"]:
    L.append(f"        ValeurCapital {{ ressource: Ressource::{x['ressource']}, cout_achat_po: {x['cout_achat_po']}, cout_produit_po: {x['cout_produit_po']}, source: {src(x['source'])} }},")
L.append("    ],\n    communautes: vec![")
for x in D["communautes"]:
    L.append(f"        CategorieCommunaute {{ id: {qs(x['id'])}, nom: {qs(x['nom'])}, population_min: {x['population_min']}, "
             f"population_max: {opt(x['population_max'])}, limite_depense_jour: {x['limite_depense_jour']}, "
             f"source_population: {src(x['source_population'])}, source_limite: {src(x['source_limite'])}, "
             f"point_a_confirmer: {opt(x['point_a_confirmer'], qs)} }},")
for key in ("salles", "extensions", "equipes"):
    L.append(f"    ],\n    {key}: vec![")
    L += [unite(x) for x in D[key]]
L.append("    ],\n    gestionnaires: vec![")
for g in D["gestionnaires"]:
    L.append(f"        Gestionnaire {{ id: {qs(g['id'])}, nom: {qs(g['nom'])}, source: {src(g['source'])}, "
             f"salaire_po_jour: {g['salaire_po_jour']}, competences: {qs(g['competences'])} }},")
for key in ("modeles_batiments", "modeles_organisations"):
    L.append(f"    ],\n    {key}: vec![")
    L += [modele(x) for x in D[key]]
L.append("    ],\n    points_a_confirmer: vec![")
for p in D["points_a_confirmer"]:
    L.append(f"        PointAConfirmer {{ code: {qs(p['code'])}, question: {qs(p['question'])}, source: {src(p['source'])}, "
             f"lecture_appliquee: {qs(p['lecture_appliquee'])}, statut: StatutPoint::{p['statut']}, decision: {opt(p['decision'], qs)} }},")
L.append("    ],\n    }\n}\n")
open(sys.argv[2], "w", encoding="utf-8").write("\n".join(L))
print("écrit", sys.argv[2], len(L), "lignes")
