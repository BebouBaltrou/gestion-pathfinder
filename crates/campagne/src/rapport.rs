//! Récapitulatif lisible d'une campagne : page HTML autonome (imprimable en PDF depuis le
//! navigateur) et historique au format CSV (tableur).

use crate::modele::*;
use crate::Campagne;
use domaine::{CoutCapitaux, TypeCapital};
use regles::Referentiel;

fn echapper(t: &str) -> String {
    t.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn capitaux(c: &CoutCapitaux) -> String {
    TypeCapital::TOUS
        .iter()
        .map(|t| format!("{} {t}", c.get(*t)))
        .collect::<Vec<_>>()
        .join(", ")
}

fn ligne(cellules: &[String]) -> String {
    let mut s = String::from("<tr>");
    for c in cellules {
        s.push_str("<td>");
        s.push_str(c);
        s.push_str("</td>");
    }
    s.push_str("</tr>\n");
    s
}

fn entete(cellules: &[&str]) -> String {
    let mut s = String::from("<tr>");
    for c in cellules {
        s.push_str(&format!("<th>{c}</th>"));
    }
    s.push_str("</tr>\n");
    s
}

const STYLE: &str = "body{font-family:Georgia,serif;max-width:1000px;margin:24px auto;padding:0 16px;color:#222}\
h1{color:#7a5a1e;margin-bottom:0}h2{color:#7a5a1e;border-bottom:2px solid #d6aa5c;padding-bottom:4px;margin-top:32px}\
h3{margin-bottom:4px}table{border-collapse:collapse;width:100%;margin:8px 0 16px}\
th,td{border:1px solid #ccc;padding:4px 8px;text-align:left;vertical-align:top;font-size:14px}\
th{background:#f3ead8}.neg{color:#c0392b;font-weight:bold}.doux{color:#777}\
@media print{h2{page-break-after:avoid}table{page-break-inside:auto}}";

fn montant(m: domaine::Montant) -> String {
    if m.est_negatif() {
        format!("<span class=\"neg\">{m}</span>")
    } else {
        m.to_string()
    }
}

/// Page HTML complète décrivant l'état actuel et l'historique de la campagne.
pub fn html(c: &mut Campagne, r: &Referentiel) -> String {
    let etat = c.etat(r).clone();
    let date = c.date_courante(r);
    let mut h = String::new();
    h.push_str(&format!(
        "<!doctype html><html lang=\"fr\"><head><meta charset=\"utf-8\"><title>{}</title><style>{STYLE}</style></head><body>\n",
        echapper(&c.nom)
    ));
    h.push_str(&format!(
        "<h1>{}</h1><p class=\"doux\">{} — règles : Guide de campagne (VF d'Ultimate Campaign)</p>\n",
        echapper(&c.nom),
        echapper(&date)
    ));

    // Personnages
    h.push_str("<h2>Personnages</h2><table>");
    h.push_str(&entete(&["Nom", "Compte", "Lieu", "Prestige"]));
    for p in etat.personnages.values() {
        let lieu = p
            .localite
            .and_then(|l| etat.localites.get(&l))
            .map(|l| l.nom.clone())
            .unwrap_or_else(|| "hors localité".into());
        h.push_str(&ligne(&[
            echapper(&p.nom),
            montant(p.compte),
            echapper(&lieu),
            p.prestige.map(|x| x.to_string()).unwrap_or_default(),
        ]));
    }
    h.push_str("</table>\n");

    // Localités
    h.push_str("<h2>Localités</h2><table>");
    h.push_str(&entete(&[
        "Localité",
        "Catégorie",
        "Trésorerie",
        "Capitaux",
        "Salaires / jour",
        "Limite / jour",
    ]));
    for l in etat.localites.values() {
        let cat = match &l.categorie {
            CategorieLocalite::Officielle { id } => r
                .communaute(id)
                .map(|x| x.nom.clone())
                .unwrap_or(id.clone()),
            CategorieLocalite::Personnalisee { nom, .. } => format!("{nom} (personnalisée)"),
        };
        h.push_str(&ligne(&[
            echapper(&l.nom),
            echapper(&cat),
            montant(l.tresorerie),
            capitaux(&l.capitaux),
            etat.salaires_localite(l.id).to_string(),
            etat.limite_depense(l.id, r)
                .map(|x| x.to_string())
                .unwrap_or_default(),
        ]));
    }
    h.push_str("</table>\n");

    // Bâtiments
    h.push_str("<h2>Bâtiments</h2>\n");
    if etat.batiments.is_empty() {
        h.push_str("<p class=\"doux\">Aucun.</p>");
    }
    for b in etat.batiments.values() {
        let loc = etat
            .localites
            .get(&b.localite)
            .map(|l| l.nom.clone())
            .unwrap_or_default();
        h.push_str(&format!(
            "<h3>{} <span class=\"doux\">— {}{}</span></h3><table>",
            echapper(&b.nom),
            echapper(&loc),
            if b.classement.is_empty() {
                String::new()
            } else {
                format!(", {}", echapper(&b.classement))
            }
        ));
        h.push_str(&entete(&[
            "Étage",
            "Salle",
            "État",
            "Génère",
            "Recettes (livre)",
        ]));
        for e in &b.etages {
            for s in etat.salles_etage(e.id) {
                let u = r.salle(&s.reference);
                let statut = match &s.statut {
                    StatutSalle::AConstruire => "à construire".to_string(),
                    StatutSalle::EnTravaux(ch) => {
                        format!("en travaux, jour {}/{}", ch.jours_ecoules, ch.duree)
                    }
                    StatutSalle::Construite => "construite".to_string(),
                };
                let statut = if s.brisee {
                    format!("{statut}, brisée")
                } else {
                    statut
                };
                let gen: Vec<String> = s.ressources.iter().map(|x| x.to_string()).collect();
                h.push_str(&ligne(&[
                    echapper(&e.nom),
                    echapper(&etat.nom_salle(s, r)),
                    statut,
                    if s.revenus_actifs {
                        gen.join(", ")
                    } else {
                        "revenus désactivés".into()
                    },
                    u.and_then(|u| u.recettes.as_ref())
                        .map(|x| echapper(&x.texte))
                        .unwrap_or_default(),
                ]));
            }
        }
        h.push_str("</table>\n");
    }

    // Organisations
    h.push_str("<h2>Organisations</h2>\n");
    if etat.organisations.is_empty() {
        h.push_str("<p class=\"doux\">Aucune.</p>");
    }
    for o in etat.organisations.values() {
        let loc = etat
            .localites
            .get(&o.localite)
            .map(|l| l.nom.clone())
            .unwrap_or_default();
        h.push_str(&format!(
            "<h3>{} <span class=\"doux\">— siège : {}</span></h3><table>",
            echapper(&o.nom),
            echapper(&loc)
        ));
        h.push_str(&entete(&[
            "Équipe",
            "État",
            "Localité",
            "Lieu de travail",
            "Génère",
        ]));
        for e in etat.equipes_organisation(o.id) {
            let statut = match &e.statut {
                StatutEquipe::ARecruter => "à recruter".to_string(),
                StatutEquipe::EnRecrutement(ch) => {
                    format!("en recrutement, jour {}/{}", ch.jours_ecoules, ch.duree)
                }
                StatutEquipe::Recrutee => "recrutée".to_string(),
                StatutEquipe::EnDelocalisation(d) => format!(
                    "en délocalisation vers {} ({} j)",
                    etat.localites
                        .get(&d.vers)
                        .map(|l| l.nom.clone())
                        .unwrap_or_default(),
                    d.jours_restants()
                ),
            };
            let gen: Vec<String> = e.ressources.iter().map(|x| x.to_string()).collect();
            h.push_str(&ligne(&[
                echapper(&etat.nom_equipe(e, r)),
                echapper(&statut),
                echapper(
                    &etat
                        .localites
                        .get(&e.localite)
                        .map(|l| l.nom.clone())
                        .unwrap_or_default(),
                ),
                echapper(
                    &e.lieu
                        .and_then(|b| etat.batiments.get(&b))
                        .map(|b| b.nom.clone())
                        .unwrap_or_else(|| "sans bâtiment".into()),
                ),
                if e.revenus_actifs {
                    gen.join(", ")
                } else {
                    "revenus désactivés".into()
                },
            ]));
        }
        h.push_str("</table>\n");
    }

    // Gestionnaires
    h.push_str("<h2>Gestionnaires</h2><table>");
    h.push_str(&entete(&[
        "Nom",
        "Salaire / jour",
        "Payé par",
        "S'occupe de",
        "Actif",
    ]));
    for g in etat.gestionnaires.values() {
        let aff: Vec<String> = g
            .affectations
            .iter()
            .map(|a| match a {
                Affectation::Batiment(b) => etat
                    .batiments
                    .get(b)
                    .map(|x| x.nom.clone())
                    .unwrap_or_default(),
                Affectation::Gestionnaire(g) => etat
                    .gestionnaires
                    .get(g)
                    .map(|x| x.nom.clone())
                    .unwrap_or_default(),
                Affectation::Organisation(o) => etat
                    .organisations
                    .get(o)
                    .map(|x| x.nom.clone())
                    .unwrap_or_default(),
            })
            .collect();
        h.push_str(&ligne(&[
            echapper(&g.nom),
            etat.salaire_quotidien(g).to_string(),
            echapper(
                &etat
                    .localites
                    .get(&g.localite)
                    .map(|l| l.nom.clone())
                    .unwrap_or_default(),
            ),
            echapper(&aff.join(", ")),
            if g.actif { "oui" } else { "non" }.into(),
        ]));
    }
    h.push_str("</table>\n");

    // Historique
    h.push_str("<h2>Historique</h2><table>");
    h.push_str(&entete(&["N°", "Jour", "Date", "Événement"]));
    let entrees: Vec<_> = c.historique().cloned().collect();
    for e in entrees.iter().rev() {
        let d = c.decrire(&e.evenement, r);
        h.push_str(&ligne(&[
            e.numero.to_string(),
            e.jour.to_string(),
            echapper(&c.calendrier.libelle(e.jour)),
            echapper(&d),
        ]));
    }
    h.push_str("</table>\n</body></html>\n");
    h
}

fn csv_champ(t: &str) -> String {
    if t.contains([';', '"', '\n']) {
        format!("\"{}\"", t.replace('"', "\"\""))
    } else {
        t.to_string()
    }
}

/// Historique au format CSV (séparateur « ; », UTF-8 avec BOM pour Excel en français).
pub fn historique_csv(c: &mut Campagne, r: &Referentiel) -> String {
    let mut s = String::from("\u{feff}N°;Jour;Date;Événement\n");
    let entrees: Vec<_> = c.historique().cloned().collect();
    for e in entrees.iter().rev() {
        let d = c.decrire(&e.evenement, r);
        s.push_str(&format!(
            "{};{};{};{}\n",
            e.numero,
            e.jour,
            csv_champ(&c.calendrier.libelle(e.jour)),
            csv_champ(&d)
        ));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evenements::Evenement;
    use domaine::Montant;

    #[test]
    fn rapport_et_csv() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Rise <of> the Runelords");
        let pj = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id: pj,
                nom: "Seoni; la sorcière".into(),
                compte_initial: Montant::po(-5),
                prestige: None,
            },
            &r,
        )
        .unwrap();
        let h = html(&mut c, &r);
        assert!(h.contains("Rise &lt;of&gt; the Runelords"));
        assert!(h.contains("class=\"neg\""));
        let csv = historique_csv(&mut c, &r);
        assert!(csv.starts_with('\u{feff}'));
        assert!(csv.contains("\"Création du personnage Seoni; la sorcière"));
    }
}
