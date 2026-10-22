//! Contrôles de cohérence du référentiel. Ils ne modifient rien : ils produisent des
//! avertissements que l'interface affiche (désactivables dans les paramètres).

use crate::calculs::valeur_achat;
use crate::referentiel::Referentiel;
use domaine::fiches::{ElementModele, Modele, StatutPoint, Unite};
use domaine::CoutCapitaux;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Avertissement {
    /// Prix imprimé ≠ somme des coûts d'achat des capitaux (Table 2-1).
    PrixIncoherent {
        fiche: String,
        page: Option<u32>,
        imprime: u32,
        calcule: u32,
    },
    /// Coût total d'un modèle ≠ somme de ses éléments.
    ModeleIncoherent {
        modele: String,
        page: Option<u32>,
        imprime: CoutCapitaux,
        somme: CoutCapitaux,
    },
    /// Lien d'amélioration présent d'un côté seulement, ou vers un id inconnu.
    LienAmelioration { de: String, vers: String },
    /// Élément de modèle introuvable dans le référentiel.
    ElementInconnu { modele: String, id: String },
    /// Point à confirmer encore ouvert.
    PointOuvert { code: String, question: String },
}

impl std::fmt::Display for Avertissement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let p = |p: &Option<u32>| p.map(|n| format!(" (p. {n})")).unwrap_or_default();
        match self {
            Avertissement::PrixIncoherent { fiche, page, imprime, calcule } => write!(
                f,
                "{fiche}{} : prix imprimé {imprime} po, mais ses capitaux valent {calcule} po à l'achat",
                p(page)
            ),
            Avertissement::ModeleIncoherent { modele, page, imprime, somme } => {
                write!(f, "{modele}{} : total imprimé {imprime}, somme des éléments {somme}", p(page))
            }
            Avertissement::LienAmelioration { de, vers } => {
                write!(f, "amélioration {de} → {vers} : lien non réciproque ou inconnu")
            }
            Avertissement::ElementInconnu { modele, id } => write!(f, "{modele} : élément inconnu « {id} »"),
            Avertissement::PointOuvert { code, question } => write!(f, "{code} à confirmer : {question}"),
        }
    }
}

fn prix(r: &Referentiel, u: &Unite, out: &mut Vec<Avertissement>) {
    if let (Some(c), Some(imprime)) = (u.cout, u.prix_po) {
        let calcule = valeur_achat(r, &c);
        if calcule != imprime {
            out.push(Avertissement::PrixIncoherent {
                fiche: u.nom.clone(),
                page: u.source.page,
                imprime,
                calcule,
            });
        }
    }
}

fn modele<'a>(
    r: &'a Referentiel,
    m: &Modele,
    unites: impl Fn(&str) -> Option<&'a Unite>,
    out: &mut Vec<Avertissement>,
) {
    let mut somme = CoutCapitaux::default();
    for e in &m.composition {
        let (u, q) = match e {
            ElementModele::Unite { id, quantite } => (unites(id), quantite),
            ElementModele::Extension { id, quantite, .. } => (r.extension(id), quantite),
        };
        let id = match e {
            ElementModele::Unite { id, .. } | ElementModele::Extension { id, .. } => id,
        };
        match u.and_then(|u| u.cout) {
            Some(c) => somme = somme + c.fois(*q),
            None => out.push(Avertissement::ElementInconnu {
                modele: m.nom.clone(),
                id: id.clone(),
            }),
        }
    }
    if somme != m.cout {
        out.push(Avertissement::ModeleIncoherent {
            modele: m.nom.clone(),
            page: m.source.page,
            imprime: m.cout,
            somme,
        });
    }
    let calcule = valeur_achat(r, &m.cout);
    if calcule != m.prix_po {
        out.push(Avertissement::PrixIncoherent {
            fiche: m.nom.clone(),
            page: m.source.page,
            imprime: m.prix_po,
            calcule,
        });
    }
}

fn liens(liste: &[Unite], out: &mut Vec<Avertissement>) {
    let get = |id: &str| liste.iter().find(|u| u.id == id);
    for u in liste {
        for v in &u.amelioration_en {
            if !get(v).is_some_and(|c| c.amelioration_de.contains(&u.id)) {
                out.push(Avertissement::LienAmelioration {
                    de: u.id.clone(),
                    vers: v.clone(),
                });
            }
        }
        for v in &u.amelioration_de {
            if !get(v).is_some_and(|c| c.amelioration_en.contains(&u.id)) {
                out.push(Avertissement::LienAmelioration {
                    de: v.clone(),
                    vers: u.id.clone(),
                });
            }
        }
    }
}

/// Tous les contrôles, dans l'ordre : prix, modèles, liens d'amélioration, points ouverts.
pub fn controler(r: &Referentiel) -> Vec<Avertissement> {
    let mut out = Vec::new();
    for u in r.salles.iter().chain(&r.extensions).chain(&r.equipes) {
        prix(r, u, &mut out);
    }
    for m in &r.modeles_batiments {
        modele(r, m, |id| r.salle(id), &mut out);
    }
    for m in &r.modeles_organisations {
        modele(r, m, |id| r.equipe(id), &mut out);
    }
    liens(&r.salles, &mut out);
    liens(&r.equipes, &mut out);
    for p in r
        .points_a_confirmer
        .iter()
        .filter(|p| p.statut == StatutPoint::Ouvert)
    {
        out.push(Avertissement::PointOuvert {
            code: p.code.clone(),
            question: p.question.clone(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::referentiel_officiel;

    #[test]
    fn effectifs_du_referentiel() {
        let r = referentiel_officiel();
        assert_eq!(r.salles.len(), 78);
        assert_eq!(r.extensions.len(), 3);
        assert_eq!(r.equipes.len(), 22);
        assert_eq!(r.gestionnaires.len(), 17);
        assert_eq!(r.modeles_batiments.len(), 59);
        assert_eq!(r.modeles_organisations.len(), 14);
        assert!(r
            .salles
            .iter()
            .chain(&r.equipes)
            .all(|u| u.source.page.is_some()));
    }

    #[test]
    fn seule_incoherence_de_prix_le_labyrinthe() {
        let r = referentiel_officiel();
        let a = controler(&r);
        let prix: Vec<_> = a
            .iter()
            .filter(|x| matches!(x, Avertissement::PrixIncoherent { .. }))
            .collect();
        assert_eq!(
            prix,
            vec![&Avertissement::PrixIncoherent {
                fiche: "Labyrinthe".into(),
                page: Some(102),
                imprime: 370,
                calcule: 600
            }]
        );
    }

    #[test]
    fn modeles_et_liens_coherents() {
        let r = referentiel_officiel();
        let a = controler(&r);
        assert!(
            !a.iter().any(|x| matches!(
                x,
                Avertissement::ModeleIncoherent { .. }
                    | Avertissement::LienAmelioration { .. }
                    | Avertissement::ElementInconnu { .. }
            )),
            "{a:#?}"
        );
    }

    #[test]
    fn salaires_des_gestionnaires() {
        let r = referentiel_officiel();
        assert_eq!(r.gestionnaire("comptable").unwrap().salaire_po_jour, 3);
        assert_eq!(r.gestionnaire("capitaine").unwrap().salaire_po_jour, 5);
        assert_eq!(
            r.gestionnaires
                .iter()
                .map(|g| g.salaire_po_jour)
                .sum::<u32>(),
            55
        );
    }

    #[test]
    fn categories_de_communaute() {
        let r = referentiel_officiel();
        assert_eq!(r.categorie_pour_population(150).unwrap().id, "village");
        assert_eq!(r.categorie_pour_population(20).unwrap().id, "lieu-dit");
        assert_eq!(
            r.categorie_pour_population(80_000)
                .unwrap()
                .limite_depense_jour,
            65
        );
    }
}
