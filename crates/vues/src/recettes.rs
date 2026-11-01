//! Page Recettes : tests de capitaux des bâtiments et organisations d'une localité
//! (Guide de campagne p. 79, 84 et 94). L'interface envoie sa saisie (`EntreeRecettes`) ;
//! le cœur calcule tout : participants, répartition valide, jets lus, bilan.

use crate::batiments::{id_ressource, ressource};
use crate::localites::Choix;
use crate::{Capitaux, Somme};
use campagne::evenements::Evenement;
use campagne::modele::*;
use campagne::recettes::{DemandeRecettes, UniteProductrice};
use domaine::{CoutCapitaux, Ressource, TypeCapital};
use regles::Referentiel;
use std::collections::{BTreeMap, BTreeSet};

// ------------------------------------------------------------------ saisie

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CleU {
    /// « salle » ou « equipe ».
    pub genre: GenreUnite,
    pub id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum GenreUnite {
    Salle,
    Equipe,
}

impl From<CleUnite> for CleU {
    fn from(c: CleUnite) -> Self {
        match c {
            CleUnite::Salle(s) => CleU {
                genre: GenreUnite::Salle,
                id: s.0,
            },
            CleUnite::Equipe(e) => CleU {
                genre: GenreUnite::Equipe,
                id: e.0,
            },
        }
    }
}

impl From<CleU> for CleUnite {
    fn from(c: CleU) -> Self {
        match c.genre {
            GenreUnite::Salle => CleUnite::Salle(IdSalle(c.id)),
            GenreUnite::Equipe => CleUnite::Equipe(IdEquipe(c.id)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Points {
    pub ressource: String,
    pub points: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Repartition {
    pub cle: CleU,
    pub points: Vec<Points>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SaisieJet {
    pub ressource: String,
    pub prendre10: bool,
    /// Résultats de d20, « 12 5 18 ».
    pub des: String,
}

/// Tout ce que l'interface a saisi. Les champs absents (`None`) prennent leur valeur par défaut.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase", default))]
pub struct EntreeRecettes {
    pub localite: Option<u32>,
    pub jours: Option<String>,
    pub batiments: Option<Vec<u32>>,
    pub organisations: Option<Vec<u32>>,
    pub repartition: Vec<Repartition>,
    pub jets: Vec<SaisieJet>,
    pub conserver_tout: Option<bool>,
    pub conserves: Capitaux,
    /// PJ qui paie le Coût produit (sinon la trésorerie de la localité).
    pub payeur: Option<u32>,
}

// ------------------------------------------------------------------ vue

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VueRecettes {
    pub localites: Vec<Choix>,
    /// Aucune localité : rien à afficher.
    pub localite: Option<u32>,
    pub jours_en_attente: u32,
    /// Saisie effective (valeurs par défaut comprises), à reprendre côté interface.
    pub jours: String,
    pub batiments: Vec<Participant>,
    pub organisations: Vec<Participant>,
    /// Unités écartées faute de type de revenu coché.
    pub sans_type: Vec<String>,
    pub repartition_stricte: bool,
    pub unites: Vec<VueUnite>,
    pub tests: Vec<VueJet>,
    pub gratuits: bool,
    pub conserver_tout: bool,
    pub payeurs: Vec<Choix>,
    pub tresorerie: String,
    pub bilan: Option<VueBilan>,
    /// Calcul impossible avec la saisie actuelle (jets, répartition…).
    pub erreur: Option<String>,
    /// Les recettes peuvent être encaissées.
    pub encaissable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Participant {
    pub id: u32,
    pub nom: String,
    pub coche: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VueUnite {
    pub cle: CleU,
    pub libelle: String,
    pub bonus: u32,
    pub admises: Vec<String>,
    /// Répartition retenue (valide ou en cours de saisie).
    pub points: Vec<Points>,
    pub somme: u32,
    /// « Tout en po quand c'est possible ».
    pub defaut: Vec<Points>,
    /// Répartition des dernières recettes perçues, si elle est encore valide.
    pub precedente: Option<Vec<Points>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VueJet {
    pub ressource: String,
    pub prendre10: bool,
    pub des: String,
    pub erreur: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TestJour {
    pub ressource: String,
    pub lignes: String,
    pub prendre10: bool,
    pub formule: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Conservable {
    pub capital: String,
    pub max: u32,
    pub garde: u32,
    /// Coût produit d'une unité, en po.
    pub cout_unite_po: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VueBilan {
    pub premier_jour: Vec<TestJour>,
    pub detail_jours: Vec<String>,
    pub jours: u32,
    pub po_brut: Somme,
    pub capitaux_bruts: Capitaux,
    pub absence: Option<String>,
    pub po_net: Somme,
    pub capitaux_nets: Capitaux,
    pub conservables: Vec<Conservable>,
    pub cout_conserves: Somme,
}

// ------------------------------------------------------------------ calculs

/// Répartition par défaut : tout le bonus sur le premier type admis (po s'il est admis).
fn defaut(u: &UniteProductrice) -> Vec<(Ressource, u32)> {
    let r = if u.admises.contains(&Ressource::Po) {
        Some(Ressource::Po)
    } else {
        u.admises.iter().next().copied()
    };
    r.map(|r| vec![(r, u.bonus)]).unwrap_or_default()
}

fn valide(aff: &[(Ressource, u32)], u: &UniteProductrice, stricte: bool) -> bool {
    let somme: u32 = aff.iter().map(|(_, p)| p).sum();
    let utiles = aff.iter().filter(|(_, p)| *p > 0).count();
    somme == u.bonus
        && aff.iter().all(|(r, p)| *p == 0 || u.admises.contains(r))
        && (!stricte || utiles <= 1)
}

fn points(a: &[(Ressource, u32)]) -> Vec<Points> {
    a.iter()
        .map(|(r, p)| Points {
            ressource: id_ressource(*r).into(),
            points: *p,
        })
        .collect()
}

fn lire_points(a: &[Points]) -> Vec<(Ressource, u32)> {
    a.iter()
        .filter_map(|p| ressource(&p.ressource).map(|r| (r, p.points)))
        .collect()
}

/// Lit « 12 5 18 » ou « 12, 5, 18 » : des d20.
fn lire_des(t: &str) -> Option<Vec<u8>> {
    t.split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .filter(|x| !x.is_empty())
        .map(|x| x.parse::<u8>().ok().filter(|d| (1..=20).contains(d)))
        .collect()
}

fn capitaux_de(c: CoutCapitaux) -> Capitaux {
    c.into()
}

pub fn vue_recettes(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    entree: &EntreeRecettes,
) -> VueRecettes {
    let regles = campagne::modele::ReglesTest::depuis_options(&c.options);
    let etat = c.etat(r).clone();
    let localites: Vec<Choix> = etat
        .localites
        .values()
        .map(|l| Choix {
            id: l.id.0,
            libelle: l.nom.clone(),
        })
        .collect();
    let vide = |localite: Option<u32>| VueRecettes {
        localites: localites.clone(),
        localite,
        jours_en_attente: 0,
        jours: String::new(),
        batiments: vec![],
        organisations: vec![],
        sans_type: vec![],
        repartition_stricte: regles.repartition_stricte,
        unites: vec![],
        tests: vec![],
        gratuits: regles.capitaux_produits_gratuits,
        conserver_tout: true,
        payeurs: vec![],
        tresorerie: String::new(),
        bilan: None,
        erreur: None,
        encaissable: false,
    };
    let Some(l) = entree
        .localite
        .map(IdLocalite)
        .filter(|l| etat.localites.contains_key(l))
        .or_else(|| etat.localites.keys().next().copied())
    else {
        return vide(None);
    };
    let attente = etat.jours_sans_recettes(l);
    let jours_txt = entree
        .jours
        .clone()
        .unwrap_or_else(|| attente.max(1).to_string());
    let jours = jours_txt.trim().parse::<u32>().unwrap_or(0);

    let unites: Vec<UniteProductrice> = etat.unites_productrices(l, r, regles, true).collect();
    let coches_b: BTreeSet<u32> = match &entree.batiments {
        Some(v) => v.iter().copied().collect(),
        None => etat.batiments_localite(l).map(|b| b.id.0).collect(),
    };
    let coches_o: BTreeSet<u32> = match &entree.organisations {
        Some(v) => v.iter().copied().collect(),
        None => etat.organisations.keys().map(|o| o.0).collect(),
    };
    let orgs: BTreeSet<IdOrganisation> = unites.iter().filter_map(|u| u.organisation).collect();
    let batiments = etat
        .batiments_localite(l)
        .filter(|b| unites.iter().any(|u| u.batiment == Some(b.id)))
        .map(|b| Participant {
            id: b.id.0,
            nom: b.nom.clone(),
            coche: coches_b.contains(&b.id.0),
        })
        .collect();
    let organisations = orgs
        .iter()
        .map(|o| Participant {
            id: o.0,
            nom: etat
                .organisations
                .get(o)
                .map(|x| x.nom.clone())
                .unwrap_or_default(),
            coche: coches_o.contains(&o.0),
        })
        .collect();
    let sans_type: Vec<String> = unites
        .iter()
        .filter(|u| u.admises.is_empty())
        .map(|u| u.libelle.clone())
        .collect();
    let actives: Vec<&UniteProductrice> = unites
        .iter()
        .filter(|u| !u.admises.is_empty())
        .filter(|u| match u.cle {
            CleUnite::Salle(_) => u.batiment.is_some_and(|b| coches_b.contains(&b.0)),
            CleUnite::Equipe(_) => {
                u.organisation.is_some_and(|o| coches_o.contains(&o.0))
                    || u.batiment.is_some_and(|b| coches_b.contains(&b.0))
            }
        })
        .collect();

    let mut v = vide(Some(l.0));
    v.jours_en_attente = attente;
    v.jours = jours_txt;
    v.batiments = batiments;
    v.organisations = organisations;
    v.sans_type = sans_type;
    v.payeurs = crate::batiments::payeurs(&etat);
    v.tresorerie = etat
        .localites
        .get(&l)
        .map(|x| format!("Trésorerie de {} ({})", x.nom, x.tresorerie))
        .unwrap_or_default();
    v.conserver_tout = entree.conserver_tout.unwrap_or(true);

    // ---------- répartition : on garde la saisie si elle est utilisable
    let stricte = regles.repartition_stricte;
    let saisie: BTreeMap<CleUnite, Vec<(Ressource, u32)>> = entree
        .repartition
        .iter()
        .map(|x| (CleUnite::from(x.cle), lire_points(&x.points)))
        .collect();
    let mut retenue: Vec<(CleUnite, Vec<(Ressource, u32)>)> = Vec::new();
    for u in &actives {
        let precedente = etat
            .repartitions
            .get(&u.cle)
            .filter(|a| valide(a, u, stricte))
            .cloned();
        let utilisable = saisie.get(&u.cle).filter(|a| {
            a.iter().all(|(r, p)| *p == 0 || u.admises.contains(r))
                && (!stricte || a.iter().filter(|(_, p)| *p > 0).count() <= 1)
        });
        let aff = utilisable
            .cloned()
            .or_else(|| precedente.clone())
            .unwrap_or_else(|| defaut(u));
        v.unites.push(VueUnite {
            cle: u.cle.into(),
            libelle: u.libelle.clone(),
            bonus: u.bonus,
            admises: u.admises.iter().map(|x| id_ressource(*x).into()).collect(),
            points: points(&aff),
            somme: aff.iter().map(|(_, p)| p).sum(),
            defaut: points(&defaut(u)),
            precedente: precedente.as_deref().map(points),
        });
        retenue.push((u.cle, aff));
    }
    if actives.is_empty() {
        return v;
    }

    // ---------- jets
    let ressources: BTreeSet<Ressource> = retenue
        .iter()
        .flat_map(|(_, a)| a.iter().filter(|(_, p)| *p > 0).map(|(r, _)| *r))
        .collect();
    let mut jets: Vec<(Ressource, Vec<JetJour>)> = Vec::new();
    let mut jets_ok = jours > 0;
    for res in &ressources {
        let id = id_ressource(*res);
        let s = entree.jets.iter().find(|j| j.ressource == id);
        let prendre10 = s.is_none_or(|j| j.prendre10);
        let des = s.map(|j| j.des.clone()).unwrap_or_default();
        let mut erreur = None;
        if prendre10 {
            jets.push((*res, vec![None; jours as usize]));
        } else {
            match lire_des(&des) {
                Some(d) if d.len() == jours as usize => {
                    jets.push((*res, d.into_iter().map(Some).collect()));
                }
                Some(d) => {
                    jets_ok = false;
                    erreur = Some(format!("{} / {jours} jet(s)", d.len()));
                }
                None => {
                    jets_ok = false;
                    erreur = Some("valeurs de 1 à 20".into());
                }
            }
        }
        v.tests.push(VueJet {
            ressource: id.into(),
            prendre10,
            des,
            erreur,
        });
    }
    if ressources.is_empty() {
        v.erreur = Some("Aucun point affecté.".into());
        return v;
    }
    if jours == 0 {
        v.erreur = Some("Indiquez le nombre de jours à traiter.".into());
        return v;
    }
    if !jets_ok {
        return v;
    }
    if let Some(u) = v.unites.iter().find(|u| u.somme != u.bonus) {
        v.erreur = Some(format!(
            "{} : {} point(s) affecté(s) sur {}",
            u.libelle, u.somme, u.bonus
        ));
        return v;
    }

    // ---------- bilan
    let mut demande = DemandeRecettes {
        localite: l,
        jours,
        affectations: retenue,
        jets,
        conserves: CoutCapitaux::default(),
        payeur: None,
        regles,
    };
    let brut = match etat.calculer_recettes(&demande, r) {
        Ok(b) => b,
        Err(e) => {
            v.erreur = Some(e.to_string());
            return v;
        }
    };
    let mut conserves = if v.conserver_tout {
        brut.capitaux_nets
    } else {
        entree.conserves.into()
    };
    for t in TypeCapital::TOUS {
        let max = brut.capitaux_nets.get(t);
        let x = conserves.get_mut(t);
        *x = (*x).min(max);
    }
    demande.conserves = conserves;
    demande.payeur = Some(match entree.payeur {
        Some(p) if !regles.capitaux_produits_gratuits => Payeur::Personnage(IdPersonnage(p)),
        _ => Payeur::Localite(l),
    });
    let bilan = match etat.calculer_recettes(&demande, r) {
        Ok(b) => b,
        Err(e) => {
            v.erreur = Some(e.to_string());
            return v;
        }
    };
    v.bilan = Some(VueBilan {
        premier_jour: brut
            .par_jour
            .first()
            .map(|tests| {
                tests
                    .iter()
                    .map(|t| TestJour {
                        ressource: id_ressource(t.ressource).into(),
                        lignes: t
                            .lignes
                            .iter()
                            .map(|(n, p)| format!("{n} +{p}"))
                            .collect::<Vec<_>>()
                            .join(" · "),
                        prendre10: t.prendre10,
                        formule: t.formule.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        detail_jours: brut
            .par_jour
            .iter()
            .enumerate()
            .map(|(i, tests)| {
                let txt: Vec<String> = tests
                    .iter()
                    .map(|t| format!("{} : {}", t.ressource, t.formule))
                    .collect();
                format!("Jour {} — {}", brut.du + i as u32, txt.join(" ; "))
            })
            .collect(),
        jours,
        po_brut: brut.po_brut.into(),
        capitaux_bruts: capitaux_de(brut.capitaux_bruts),
        absence: if brut.tranches_absence > 0 {
            Some(format!(
                "Absence : {} jour(s) sans PJ → {} tranche(s) de 7 jours : −{} po et −{} par capital (GC p. 84)",
                brut.jours_absents,
                brut.tranches_absence,
                7 * brut.tranches_absence,
                brut.tranches_absence
            ))
        } else if brut.jours_absents > 0 {
            Some(format!(
                "{} jour(s) sans PJ, moins d'une tranche de 7 : aucune réduction",
                brut.jours_absents
            ))
        } else {
            None
        },
        po_net: brut.po_net.into(),
        capitaux_nets: capitaux_de(brut.capitaux_nets),
        conservables: TypeCapital::TOUS
            .into_iter()
            .filter(|t| brut.capitaux_nets.get(*t) > 0)
            .map(|t| Conservable {
                capital: id_ressource(t.into()).into(),
                max: brut.capitaux_nets.get(t),
                garde: conserves.get(t),
                cout_unite_po: r.valeur(t.into()).map(|x| x.cout_produit_po).unwrap_or(0),
            })
            .collect(),
        cout_conserves: bilan.cout_conserves.into(),
    });
    v.encaissable = true;
    v
}

/// Événement d'encaissement construit à partir de la même saisie que l'aperçu.
pub fn encaisser(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    entree: &EntreeRecettes,
) -> Result<(Evenement, String), String> {
    let vue = vue_recettes(c, r, entree);
    if !vue.encaissable {
        return Err(vue.erreur.unwrap_or_else(|| {
            "Les recettes ne peuvent pas être calculées avec cette saisie".into()
        }));
    }
    let regles = campagne::modele::ReglesTest::depuis_options(&c.options);
    let l = IdLocalite(vue.localite.ok_or("Aucune localité")?);
    let jours: u32 = vue.jours.trim().parse().map_err(|_| "Jours illisibles")?;
    let affectations = vue
        .unites
        .iter()
        .map(|u| (CleUnite::from(u.cle), lire_points(&u.points)))
        .collect();
    let jets = vue
        .tests
        .iter()
        .filter_map(|t| {
            let res = ressource(&t.ressource)?;
            let jets = if t.prendre10 {
                vec![None; jours as usize]
            } else {
                lire_des(&t.des)?.into_iter().map(Some).collect()
            };
            Some((res, jets))
        })
        .collect();
    let bilan = vue.bilan.ok_or("Pas de bilan")?;
    let mut conserves = CoutCapitaux::default();
    for x in &bilan.conservables {
        if let Some(t) = ressource(&x.capital).and_then(|r| r.capital()) {
            *conserves.get_mut(t) = x.garde;
        }
    }
    let payeur = Some(match entree.payeur {
        Some(p) if !regles.capitaux_produits_gratuits => Payeur::Personnage(IdPersonnage(p)),
        _ => Payeur::Localite(l),
    });
    Ok((
        Evenement::RecettesPercues {
            localite: l,
            jours,
            affectations,
            jets,
            conserves,
            payeur,
            regles,
        },
        "Recettes encaissées".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::modeles_perso::ModelesPerso;
    use campagne::Campagne;

    /// Une localité, un PJ présent, un bâtiment vide avec une salle construite qui génère
    /// une ressource : la page propose un bilan et l'encaissement le crédite.
    fn campagne_productive(r: &Referentiel) -> Campagne {
        let (mut c, l) = crate::localites::campagne_sandpoint(r);
        let pj = c.nouvel_id_personnage();
        c.executer(
            crate::personnages::creation(pj, "Seoni", "", "").unwrap(),
            r,
        )
        .unwrap();
        c.executer(crate::personnages::deplacement(pj.0, Some(l.0)), r)
            .unwrap();
        let perso = ModelesPerso::default();
        let (ev, _) = crate::batiments::creation(
            &mut c,
            r,
            &perso,
            &crate::batiments::SaisieBatiment {
                localite: Some(l.0),
                nom: "Auberge".into(),
                facteur: 1,
                ..Default::default()
            },
        )
        .unwrap();
        c.executer(ev, r).unwrap();
        // une salle qui admet au moins un type de revenu
        let reference = r
            .salles
            .iter()
            .find(|u| u.recettes.is_some() && u.cout.is_some())
            .expect("une salle à recettes")
            .id
            .clone();
        let (ev, _) = crate::actions_batiments::action(
            &mut c,
            r,
            crate::actions_batiments::ActionBatiment::AjouterSalle {
                batiment: 1,
                etage: 1,
                reference,
                deja: true,
            },
        )
        .unwrap();
        c.executer(ev, r).unwrap();
        c
    }

    #[test]
    fn page_vide_sans_localite() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Vide");
        let v = vue_recettes(&mut c, &r, &EntreeRecettes::default());
        assert!(v.localite.is_none());
        assert!(!v.encaissable);
    }

    #[test]
    fn des_et_jours_sont_verifies() {
        assert_eq!(lire_des("12 5, 18"), Some(vec![12, 5, 18]));
        assert_eq!(lire_des("0 5"), None);
        assert_eq!(lire_des("21"), None);
    }

    #[test]
    fn bilan_puis_encaissement() {
        let r = regles::referentiel_officiel();
        let mut c = campagne_productive(&r);
        // avancer pour avoir des jours en attente
        c.executer(Evenement::JoursAvances { nombre: 3 }, &r)
            .unwrap();
        let mut entree = EntreeRecettes::default();
        let v = vue_recettes(&mut c, &r, &entree);
        assert_eq!(v.localite, Some(1));
        assert_eq!(v.jours, v.jours_en_attente.to_string());
        assert!(v.jours_en_attente >= 3);
        // sans type de revenu coché, la salle est écartée : on le coche
        if v.unites.is_empty() {
            assert!(!v.sans_type.is_empty());
            let sb = c.etat(&r).salles.values().next().unwrap().clone();
            let admises = c.etat(&r).admises_salle(&sb, &r);
            let res = *admises.iter().next().expect("un type admis");
            let (ev, _) = crate::actions_batiments::action(
                &mut c,
                &r,
                crate::actions_batiments::ActionBatiment::ReglerSalle {
                    salle: sb.id.0,
                    ressource: Some(id_ressource(res).into()),
                    revenus_actifs: true,
                    brisee: false,
                },
            )
            .unwrap();
            c.executer(ev, &r).unwrap();
        }
        entree.jours = None;
        let v = vue_recettes(&mut c, &r, &entree);
        assert!(v.encaissable, "erreur : {:?}", v.erreur);
        assert!(v.bilan.is_some());
        let (ev, msg) = encaisser(&mut c, &r, &entree).unwrap();
        assert_eq!(msg, "Recettes encaissées");
        c.executer(ev, &r).unwrap();
        // plus aucun jour en attente
        let v = vue_recettes(&mut c, &r, &EntreeRecettes::default());
        assert_eq!(v.jours_en_attente, 0);
    }

    #[test]
    fn jets_incomplets_refuses() {
        let r = regles::referentiel_officiel();
        let mut c = campagne_productive(&r);
        c.executer(Evenement::JoursAvances { nombre: 2 }, &r)
            .unwrap();
        let sb = c.etat(&r).salles.values().next().unwrap().clone();
        let res = *c.etat(&r).admises_salle(&sb, &r).iter().next().unwrap();
        let (ev, _) = crate::actions_batiments::action(
            &mut c,
            &r,
            crate::actions_batiments::ActionBatiment::ReglerSalle {
                salle: sb.id.0,
                ressource: Some(id_ressource(res).into()),
                revenus_actifs: true,
                brisee: false,
            },
        )
        .unwrap();
        c.executer(ev, &r).unwrap();
        let entree = EntreeRecettes {
            jets: vec![SaisieJet {
                ressource: id_ressource(res).into(),
                prendre10: false,
                des: "12".into(), // plusieurs jours en attente, un seul jet
            }],
            ..Default::default()
        };
        let v = vue_recettes(&mut c, &r, &entree);
        assert!(!v.encaissable);
        assert_eq!(
            v.tests[0].erreur,
            Some(format!("1 / {} jet(s)", v.jours_en_attente))
        );
        assert!(encaisser(&mut c, &r, &entree).is_err());
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests_json {
    use super::*;

    #[test]
    fn saisie_lue_comme_l_interface_l_envoie() {
        let e: EntreeRecettes = serde_json::from_str(
            r#"{"localite":1,"jours":"3","batiments":[1],"organisations":null,
                "repartition":[{"cle":{"genre":"salle","id":2},"points":[{"ressource":"po","points":3}]}],
                "jets":[{"ressource":"po","prendre10":false,"des":"12 5 18"}],
                "conserverTout":false,"conserves":{"marchandises":1,"influence":0,"travail":0,"magie":0},
                "payeur":null}"#,
        )
        .unwrap();
        assert_eq!(e.repartition[0].cle.genre, GenreUnite::Salle);
        assert_eq!(e.conserves.marchandises, 1);
        let vide: EntreeRecettes = serde_json::from_str("{}").unwrap();
        assert!(vide.localite.is_none());
    }
}
