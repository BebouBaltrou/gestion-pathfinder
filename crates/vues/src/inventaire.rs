//! Pages Inventaire et Équipement : ce que l'interface montre, calculé à partir de l'état de la
//! campagne. Les poids, capacités et emplacements viennent du moteur (`crates/inventaire`).

use crate::Somme;
use campagne::catalogue_inventaire::catalogue_complet;
use campagne::equipement_fiche::{personnage_de_ref, PROPRIETAIRE_PERSONNAGE};
use campagne::etat::Etat;
use domaine::Montant;
use inventaire::{Id, Modele, Registre};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PageInventaire {
    pub inventaires: Vec<InventaireVue>,
    pub emplacements: Vec<EmplacementVue>,
    /// Personnages qui n'ont pas encore d'inventaire.
    pub sans_inventaire: Vec<PersonnageVue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PersonnageVue {
    pub id: u32,
    pub nom: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EmplacementVue {
    pub id: String,
    pub zone: String,
    pub nom: String,
    /// 0 = illimité.
    pub capacite: u32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct InventaireVue {
    pub racine: u64,
    pub proprietaire: String,
    pub type_proprietaire: String,
    /// Identifiant de l'entité propriétaire (numéro du personnage pour un PJ).
    pub ref_proprietaire: String,
    pub poids: PoidsVue,
    pub valeur: Somme,
    pub objets: Vec<ObjetVue>,
    pub jeux: Vec<String>,
    /// Charge portée, pour un personnage qui a une fiche (limites de la Force actuelle).
    pub charge: Option<ChargeVue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PoidsVue {
    pub grammes: i64,
    pub texte: String,
}

impl PoidsVue {
    fn de(grammes: i64) -> Self {
        PoidsVue {
            grammes,
            texte: texte_poids(grammes),
        }
    }
}

/// « 250 g », « 1,5 kg », « 12 kg ».
pub fn texte_poids(grammes: i64) -> String {
    if grammes.abs() < 1000 {
        return format!("{grammes} g");
    }
    let kilos = grammes / 1000;
    let reste = (grammes % 1000).abs();
    if reste == 0 {
        format!("{kilos} kg")
    } else {
        let decimales = format!("{reste:03}");
        format!("{kilos},{} kg", decimales.trim_end_matches('0'))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ObjetVue {
    pub id: u64,
    pub nom: String,
    pub type_objet: String,
    pub quantite: u32,
    /// Conteneur qui le contient (l'inventaire racine pour un objet posé « sur soi »).
    pub parent: Option<u64>,
    pub est_conteneur: bool,
    /// Poids de l'objet avec son contenu quand il compte.
    pub poids: PoidsVue,
    /// Poids du contenu, pour un conteneur.
    pub contenu: Option<PoidsVue>,
    pub capacite: Option<PoidsVue>,
    pub contenu_compte: bool,
    pub valeur_unite: Option<Somme>,
    pub note: String,
    pub source: String,
    pub colle: bool,
    /// Emplacements occupés quand l'objet est équipé.
    pub equipe: Vec<String>,
    pub emplacements_possibles: Vec<String>,
    pub proprietes: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ChargeVue {
    /// « légère », « intermédiaire », « lourde » ou « surcharge ».
    pub nom: String,
    /// Poids de l'inventaire en livres (1 livre = 500 g), comme les limites de la fiche.
    pub poids_lb: f64,
    /// Limites légère / intermédiaire / lourde, en livres.
    pub limites_lb: [f64; 3],
}

/// La charge d'un personnage : le poids de son inventaire classé par le moteur de la fiche.
fn charge_du_proprietaire(
    etat: &Etat,
    type_id: &str,
    ref_id: &str,
    grammes: i64,
) -> Option<ChargeVue> {
    if type_id != PROPRIETAIRE_PERSONNAGE {
        return None;
    }
    let pj = personnage_de_ref(ref_id)?;
    let fiche = etat.fiche_de(pj)?;
    let valeurs = fiche.fiche.calculer(fiches::Referentiel::partage()).ok()?;
    let limites = valeurs.equipement.limites?;
    let poids_lb = grammes as f64 / 500.0;
    Some(ChargeVue {
        nom: fiches::equipement::Charge::selon_limites(Some(limites), poids_lb)
            .nom()
            .to_string(),
        poids_lb,
        limites_lb: [limites.0, limites.1, limites.2],
    })
}

fn proprietaire_lisible(etat: &Etat, type_id: &str, ref_id: &str) -> String {
    if type_id == PROPRIETAIRE_PERSONNAGE {
        if let Some(pj) = personnage_de_ref(ref_id).and_then(|id| etat.pj(id).ok()) {
            return pj.nom.clone();
        }
    }
    format!("{type_id} {ref_id}")
}

fn objet_vue(registre: &Registre, id: Id, equipe: &BTreeMap<Id, Vec<String>>) -> Option<ObjetVue> {
    let e = registre.exemplaire(id).ok()?;
    let modele = e.modele.as_ref().and_then(|m| registre.modele(m));
    let contenu = e
        .conteneur
        .as_ref()
        .map(|_| PoidsVue::de(registre.poids_brut_contenu(id).0));
    Some(ObjetVue {
        id,
        nom: modele.map_or_else(|| "Inventaire".to_string(), |m| m.nom.clone()),
        type_objet: modele.map(|m| m.type_objet.clone()).unwrap_or_default(),
        quantite: e.quantite,
        parent: e.parent,
        est_conteneur: e.conteneur.is_some(),
        poids: PoidsVue::de(registre.poids_total(id).0),
        contenu,
        capacite: e
            .conteneur
            .as_ref()
            .and_then(|c| c.capacite)
            .map(|c| PoidsVue::de(c.0)),
        contenu_compte: e.conteneur.as_ref().is_none_or(|c| c.contenu_compte),
        valeur_unite: modele.and_then(|m| m.valeur).map(Somme::from),
        note: e.note.clone(),
        source: modele.map(|m| m.source.clone()).unwrap_or_default(),
        colle: e.colle,
        equipe: equipe.get(&id).cloned().unwrap_or_default(),
        emplacements_possibles: registre.emplacements_possibles(id).unwrap_or_default(),
        proprietes: modele
            .map(|m| {
                m.proprietes
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default(),
    })
}

pub fn page_inventaire(etat: &Etat) -> PageInventaire {
    let registre = &etat.inventaire;
    let mut inventaires = Vec::new();
    let mut avec_inventaire = std::collections::BTreeSet::new();
    for (racine, proprietaire) in registre.inventaires() {
        if proprietaire.type_id == PROPRIETAIRE_PERSONNAGE {
            if let Some(n) = personnage_de_ref(&proprietaire.ref_id) {
                avec_inventaire.insert(n.0);
            }
        }
        let equipe: BTreeMap<Id, Vec<String>> = registre
            .equipement_de(racine)
            .into_iter()
            .map(|e| (e.objet, e.emplacements))
            .collect();
        let objets: Vec<ObjetVue> = registre
            .exemplaires()
            .map(|e| e.id)
            .filter(|id| registre.racine_de(*id) == Ok(racine) && *id != racine)
            .filter_map(|id| objet_vue(registre, id, &equipe))
            .collect();
        let valeur: Montant = objets
            .iter()
            .filter_map(|o| {
                o.valeur_unite
                    .as_ref()
                    .map(|v| Montant::pc(v.pc * i64::from(o.quantite)))
            })
            .sum();
        inventaires.push(InventaireVue {
            racine,
            proprietaire: proprietaire_lisible(etat, &proprietaire.type_id, &proprietaire.ref_id),
            type_proprietaire: proprietaire.type_id.clone(),
            ref_proprietaire: proprietaire.ref_id.clone(),
            poids: PoidsVue::de(registre.poids_total(racine).0),
            valeur: valeur.into(),
            objets,
            jeux: registre.noms_des_jeux(racine),
            charge: charge_du_proprietaire(
                etat,
                &proprietaire.type_id,
                &proprietaire.ref_id,
                registre.poids_total(racine).0,
            ),
        });
    }
    PageInventaire {
        inventaires,
        emplacements: registre
            .regles_emplacements()
            .emplacements
            .iter()
            .map(|e| EmplacementVue {
                id: e.id.clone(),
                zone: e.zone.clone(),
                nom: e.nom.clone(),
                capacite: e.capacite,
            })
            .collect(),
        sans_inventaire: etat
            .personnages
            .values()
            .filter(|p| !avec_inventaire.contains(&p.id.0))
            .map(|p| PersonnageVue {
                id: p.id.0,
                nom: p.nom.clone(),
            })
            .collect(),
    }
}

/// Un modèle du catalogue, pour la recherche et l'ajout rapide.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ModeleVue {
    pub id: String,
    pub nom: String,
    pub type_objet: String,
    pub valeur: Option<Somme>,
    pub poids: Option<PoidsVue>,
    pub source: String,
}

impl ModeleVue {
    fn de(m: &Modele) -> Self {
        ModeleVue {
            id: m.id.clone(),
            nom: m.nom.clone(),
            type_objet: m.type_objet.clone(),
            valeur: m.valeur.map(Somme::from),
            poids: m.poids.map(|p| PoidsVue::de(p.0)),
            source: m.source.clone(),
        }
    }
}

/// Texte comparable : minuscules, sans accents.
fn simple(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            '’' => '\'',
            c => c,
        })
        .collect()
}

/// Cherche dans le catalogue par nom (tous les mots doivent y être) et par type d'objet.
pub fn rechercher_catalogue(
    requete: &str,
    type_objet: Option<&str>,
    limite: usize,
) -> Vec<ModeleVue> {
    let mots: Vec<String> = simple(requete)
        .split_whitespace()
        .map(str::to_string)
        .collect();
    catalogue_complet()
        .iter()
        .filter(|m| type_objet.is_none_or(|t| m.type_objet == t))
        .filter(|m| {
            let nom = simple(&m.nom);
            mots.iter().all(|mot| nom.contains(mot))
        })
        .take(limite)
        .map(ModeleVue::de)
        .collect()
}

/// Les types d'objets du catalogue, sans doublon.
pub fn types_du_catalogue() -> Vec<String> {
    let mut types: Vec<String> = catalogue_complet()
        .iter()
        .map(|m| m.type_objet.clone())
        .collect();
    types.sort();
    types.dedup();
    types
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poids_lisibles() {
        assert_eq!(texte_poids(250), "250 g");
        assert_eq!(texte_poids(1500), "1,5 kg");
        assert_eq!(texte_poids(12_000), "12 kg");
        assert_eq!(texte_poids(2250), "2,25 kg");
        assert_eq!(texte_poids(0), "0 g");
    }

    #[test]
    fn la_recherche_ignore_accents_et_casse() {
        let r = rechercher_catalogue("epee longue", None, 10);
        assert!(r.iter().any(|m| m.nom == "Épée longue"), "{r:?}");
        let armes = rechercher_catalogue("", Some("arme"), 1000);
        assert!(armes.len() > 50);
        assert!(armes.iter().all(|m| m.type_objet == "arme"));
        assert!(types_du_catalogue().contains(&"objet-merveilleux".to_string()));
    }

    #[test]
    fn la_page_montre_les_objets_le_poids_et_la_charge() {
        use campagne::catalogue_inventaire::emplacements_par_defaut;
        use campagne::evenements::Evenement;
        use campagne::Campagne;
        use fiches::journal::EvenementFiche;
        use inventaire::{Commande, Conteneur, Proprietaire};
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let pj = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id: pj,
                nom: "Lyra".into(),
                compte_initial: Montant::po(100),
                prestige: None,
            },
            &r,
        )
        .unwrap();
        c.executer(
            Evenement::Fiche {
                pj,
                evenement: EvenementFiche::Creation(Box::new(fiches::exemple::elfe_ensorceleuse())),
            },
            &r,
        )
        .unwrap();
        let armure = rechercher_catalogue("armure de cuir", Some("armure"), 5)
            .into_iter()
            .find(|m| m.nom == "Armure de cuir")
            .unwrap();
        let modele = catalogue_complet()
            .iter()
            .find(|m| m.id == armure.id)
            .unwrap();
        assert!(page_inventaire(c.etat(&r)).sans_inventaire.len() == 1);
        c.executer_inventaires(
            vec![
                Commande::DefinirEmplacements(emplacements_par_defaut()),
                Commande::AjouterModele(modele.clone()),
                Commande::CreerInventaire {
                    proprietaire: Proprietaire {
                        type_id: PROPRIETAIRE_PERSONNAGE.into(),
                        ref_id: pj.to_string(),
                    },
                    conteneur: Conteneur {
                        capacite: None,
                        types_autorises: None,
                        contenu_compte: true,
                    },
                },
                Commande::Ajouter {
                    modele: modele.id.clone(),
                    quantite: 1,
                    dans: 1,
                },
                Commande::Equiper {
                    id: 2,
                    emplacement: "corps".into(),
                },
            ],
            &r,
        )
        .unwrap();
        let page = page_inventaire(c.etat(&r));
        assert!(page.sans_inventaire.is_empty());
        assert_eq!(page.emplacements.len(), 14);
        let inv = &page.inventaires[0];
        assert_eq!(inv.proprietaire, "Lyra");
        assert_eq!(inv.objets.len(), 1);
        let objet = &inv.objets[0];
        assert_eq!(objet.nom, "Armure de cuir");
        assert_eq!(objet.equipe, vec!["corps"]);
        assert_eq!(objet.poids.grammes, 7_500);
        assert_eq!(inv.poids.texte, "7,5 kg");
        assert_eq!(inv.valeur.pc, 1_000);
        let charge = inv.charge.as_ref().expect("charge");
        assert_eq!(charge.nom, "légère");
        assert_eq!(charge.poids_lb, 15.0);
    }
}
