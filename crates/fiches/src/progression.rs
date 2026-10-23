//! Progression de niveau : points d'expérience, « montée en attente » et assistant de montée
//! (Core Rulebook, tableau 3-1, p. 31, et « Advancing Your Character », p. 30).

use crate::caracteristiques::Carac;
use crate::donnees::Referentiel;
use crate::fiche::{BonusPredilection, Fiche, NiveauPris, Probleme};
use std::collections::BTreeMap;

/// Vitesse de progression, choix de campagne (Core p. 30) ; `Jalons` : le MJ accorde les niveaux.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Vitesse {
    Lente,
    Moyenne,
    Rapide,
    Jalons,
}

/// Comment sont obtenus les dés de vie après le niveau 1 (réglage de campagne).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MethodePv {
    /// Le joueur lance le dé.
    Jet,
    /// Moyenne arrondie vers le bas (d6 : 3).
    Moyenne,
    /// Moyenne arrondie vers le haut (d6 : 4).
    MoyenneArrondie,
    Maximum,
}

/// Réglages de campagne utilisés par les fiches (le registre complet viendra en phase 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReglesFiche {
    pub vitesse: Vitesse,
    pub methode_pv: MethodePv,
    /// Un personnage peut prendre des niveaux dans plusieurs classes.
    pub multiclassage: bool,
    /// Bonus de classe de prédilection (+1 PV ou +1 rang par niveau).
    pub classe_de_predilection: bool,
}

impl Default for ReglesFiche {
    fn default() -> Self {
        ReglesFiche {
            vitesse: Vitesse::Moyenne,
            methode_pv: MethodePv::MoyenneArrondie,
            multiclassage: false,
            classe_de_predilection: true,
        }
    }
}

/// XP nécessaires pour atteindre `niveau` (`None` : jalons, ou niveau hors 1-20).
pub fn xp_requis(r: &Referentiel, vitesse: Vitesse, niveau: u32) -> Option<u32> {
    let l = r.ligne_progression(niveau)?;
    match vitesse {
        Vitesse::Lente => Some(l.xp_lente),
        Vitesse::Moyenne => Some(l.xp_moyenne),
        Vitesse::Rapide => Some(l.xp_rapide),
        Vitesse::Jalons => None,
    }
}

/// Niveau que les XP (ou les jalons accordés) permettent d'atteindre, de 1 à 20.
pub fn niveau_atteignable(r: &Referentiel, vitesse: Vitesse, xp: u32, jalon: u32) -> u32 {
    if vitesse == Vitesse::Jalons {
        return jalon.clamp(1, 20);
    }
    (1..=20)
        .rev()
        .find(|&n| xp_requis(r, vitesse, n).is_some_and(|req| xp >= req))
        .unwrap_or(1)
}

/// Dé de vie obtenu selon la méthode : `jet` n'est lu que pour la méthode `Jet`.
pub fn de_de_vie(methode: MethodePv, de: u32, jet: Option<u32>) -> Result<u32, ErreurMontee> {
    match methode {
        MethodePv::Jet => match jet {
            Some(j) if (1..=de).contains(&j) => Ok(j),
            Some(j) => Err(ErreurMontee::JetHorsLimites { jet: j, de }),
            None => Err(ErreurMontee::JetManquant),
        },
        MethodePv::Moyenne => Ok(de.div_ceil(2)),
        MethodePv::MoyenneArrondie => Ok(de / 2 + 1),
        MethodePv::Maximum => Ok(de),
    }
}

/// Choix faits dans l'assistant pour un niveau.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Montee {
    pub classe: String,
    /// Dé de vie déjà résolu (voir [`de_de_vie`]).
    pub de_vie: u32,
    pub rangs: BTreeMap<String, u32>,
    pub predilection: Option<BonusPredilection>,
    pub dons: Vec<String>,
    pub augmentation: Option<Carac>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurMontee {
    NiveauPasEncoreAtteint { niveau: u32, atteignable: u32 },
    NiveauMaximum,
    ClasseInconnue(String),
    MulticlassageDesactive { classe: String },
    JetManquant,
    JetHorsLimites { jet: u32, de: u32 },
    DonsAttendus { attendus: u32, fournis: u32 },
    AugmentationAttendue,
    AugmentationInattendue,
    PredilectionAttendue,
    PredilectionInattendue,
    Fiche(Probleme),
}

impl std::fmt::Display for ErreurMontee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErreurMontee::NiveauPasEncoreAtteint {
                niveau,
                atteignable,
            } => write!(
                f,
                "niveau {niveau} pas encore atteint (niveau atteignable : {atteignable})"
            ),
            ErreurMontee::NiveauMaximum => write!(f, "le niveau 20 est le maximum"),
            ErreurMontee::ClasseInconnue(c) => write!(f, "classe inconnue : {c}"),
            ErreurMontee::MulticlassageDesactive { classe } => write!(
                f,
                "multiclassage désactivé dans cette campagne ({classe} est une nouvelle classe)"
            ),
            ErreurMontee::JetManquant => write!(f, "saisir le résultat du dé de vie"),
            ErreurMontee::JetHorsLimites { jet, de } => {
                write!(f, "dé de vie {jet} impossible pour un d{de}")
            }
            ErreurMontee::DonsAttendus { attendus, fournis } => {
                write!(f, "{attendus} don(s) à choisir, {fournis} fourni(s)")
            }
            ErreurMontee::AugmentationAttendue => {
                write!(f, "choisir la caractéristique à augmenter de 1")
            }
            ErreurMontee::AugmentationInattendue => {
                write!(f, "pas d'augmentation de caractéristique à ce niveau")
            }
            ErreurMontee::PredilectionAttendue => {
                write!(f, "classe de prédilection : choisir +1 PV ou +1 rang")
            }
            ErreurMontee::PredilectionInattendue => {
                write!(f, "ce niveau n'est pas dans une classe de prédilection")
            }
            ErreurMontee::Fiche(p) => write!(f, "{p}"),
        }
    }
}

/// Ce que l'assistant montre avant de valider un niveau.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApercuMontee {
    pub niveau_de_personnage: u32,
    pub classe: String,
    pub niveau_de_classe: u32,
    pub de_vie: u32,
    /// Bonus de base à l'attaque, jets de sauvegarde (Vigueur, Réflexes, Volonté) : avant, après.
    pub bba: (i32, i32),
    pub vigueur: (i32, i32),
    pub reflexes: (i32, i32),
    pub volonte: (i32, i32),
    pub dons_a_choisir: u32,
    pub augmentation: bool,
    pub predilection_possible: bool,
    /// Capacités de classe gagnées (texte du tableau de classe).
    pub capacites_en: String,
    /// Sorts connus gagnés par niveau de sort (0 à 9) : (niveau de sort, nombre).
    pub sorts_connus_gagnes: Vec<(u32, u32)>,
    /// Un sort connu peut être échangé contre un autre du même niveau.
    pub echange_de_sort: bool,
}

fn table_a(classe: &crate::donnees::Classe, nc: u32) -> Option<&crate::donnees::ProgressionNiveau> {
    nc.checked_sub(1)
        .and_then(|i| classe.progression.get(i as usize))
}

/// Dons à choisir au niveau de personnage `niveau` en prenant le niveau `nc` de `classe`.
fn dons_attendus(r: &Referentiel, f: &Fiche, classe: &crate::donnees::Classe, niveau: u32) -> u32 {
    let nc = f.niveau_de_classe(&classe.id) + 1;
    let mut n = 0;
    if r.ligne_progression(niveau).is_some_and(|l| l.don) {
        n += 1;
    }
    if niveau == 1 {
        n += r.race(&f.race).map(|x| x.bonus_don).unwrap_or(0);
    }
    if table_a(classe, nc).is_some_and(|l| l.special_en.contains("Bloodline feat")) {
        n += 1;
    }
    n
}

/// Aperçu du prochain niveau dans `classe_id`, ou `None` au niveau 20 / classe inconnue.
pub fn apercu(r: &Referentiel, f: &Fiche, classe_id: &str) -> Option<ApercuMontee> {
    let niveau = f.niveau_de_personnage() + 1;
    if niveau > 20 {
        return None;
    }
    let classe = r.classe(classe_id)?;
    let nc = f.niveau_de_classe(classe_id) + 1;
    let apres = table_a(classe, nc)?;
    let precedent = nc
        .checked_sub(2)
        .and_then(|i| classe.progression.get(i as usize));
    let (b0, v0, r0, w0) = match precedent {
        Some(l) => (l.bba[0], l.vigueur, l.reflexes, l.volonte),
        None => (0, 0, 0, 0),
    };
    let mut gagnes = Vec::new();
    for sn in 0..=9usize {
        let apres_n = apres.sorts_connus.get(sn).copied().flatten().unwrap_or(0);
        let avant = precedent
            .and_then(|l| l.sorts_connus.get(sn).copied().flatten())
            .unwrap_or(0);
        if apres_n > avant {
            gagnes.push((sn as u32, apres_n - avant));
        }
    }
    let echange =
        classe
            .incantation
            .as_ref()
            .is_some_and(|i| match (i.echange_niveau_min, i.echange_pas) {
                (Some(min), Some(pas)) if pas > 0 => nc >= min && (nc - min).is_multiple_of(pas),
                _ => false,
            });
    Some(ApercuMontee {
        niveau_de_personnage: niveau,
        classe: classe_id.to_string(),
        niveau_de_classe: nc,
        de_vie: classe.de_vie,
        bba: (b0, apres.bba[0]),
        vigueur: (v0, apres.vigueur),
        reflexes: (r0, apres.reflexes),
        volonte: (w0, apres.volonte),
        dons_a_choisir: dons_attendus(r, f, classe, niveau),
        augmentation: r.ligne_progression(niveau).is_some_and(|l| l.augmentation),
        predilection_possible: f.classes_de_predilection.iter().any(|c| c == classe_id),
        capacites_en: apres.special_en.clone(),
        sorts_connus_gagnes: gagnes,
        echange_de_sort: echange,
    })
}

/// Applique un niveau : renvoie la nouvelle fiche, ou toutes les erreurs trouvées.
/// `atteignable` est le niveau permis par les XP ou les jalons.
pub fn appliquer(
    r: &Referentiel,
    regles: &ReglesFiche,
    f: &Fiche,
    m: &Montee,
    atteignable: u32,
) -> Result<Fiche, Vec<ErreurMontee>> {
    let niveau = f.niveau_de_personnage() + 1;
    if niveau > 20 {
        return Err(vec![ErreurMontee::NiveauMaximum]);
    }
    if niveau > atteignable {
        return Err(vec![ErreurMontee::NiveauPasEncoreAtteint {
            niveau,
            atteignable,
        }]);
    }
    let Some(classe) = r.classe(&m.classe) else {
        return Err(vec![ErreurMontee::ClasseInconnue(m.classe.clone())]);
    };
    let mut e = Vec::new();
    if !regles.multiclassage && f.niveaux.iter().any(|n| n.classe != m.classe) {
        e.push(ErreurMontee::MulticlassageDesactive {
            classe: m.classe.clone(),
        });
    }
    if m.de_vie < 1 || m.de_vie > classe.de_vie {
        e.push(ErreurMontee::JetHorsLimites {
            jet: m.de_vie,
            de: classe.de_vie,
        });
    }
    let attendus = dons_attendus(r, f, classe, niveau);
    if attendus != m.dons.len() as u32 {
        e.push(ErreurMontee::DonsAttendus {
            attendus,
            fournis: m.dons.len() as u32,
        });
    }
    let aug = r.ligne_progression(niveau).is_some_and(|l| l.augmentation);
    match (aug, m.augmentation) {
        (true, None) => e.push(ErreurMontee::AugmentationAttendue),
        (false, Some(_)) => e.push(ErreurMontee::AugmentationInattendue),
        _ => {}
    }
    let pred = f.classes_de_predilection.iter().any(|c| c == &m.classe);
    if regles.classe_de_predilection {
        match (pred, m.predilection) {
            (true, None) => e.push(ErreurMontee::PredilectionAttendue),
            (false, Some(_)) => e.push(ErreurMontee::PredilectionInattendue),
            _ => {}
        }
    } else if m.predilection.is_some() {
        e.push(ErreurMontee::PredilectionInattendue);
    }
    if !e.is_empty() {
        return Err(e);
    }

    let mut nouvelle = f.clone();
    nouvelle.niveaux.push(NiveauPris {
        classe: m.classe.clone(),
        de_vie_obtenu: m.de_vie,
        rangs: m.rangs.clone(),
        predilection: m.predilection,
        dons: m.dons.clone(),
    });
    if let Some(c) = m.augmentation {
        nouvelle.augmentations.push((niveau, c));
    }
    let problemes = nouvelle.valider(r);
    if problemes.is_empty() {
        Ok(nouvelle)
    } else {
        Err(problemes.into_iter().map(ErreurMontee::Fiche).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::donnees::Referentiel;

    fn base() -> Fiche {
        crate::fiche::tests::elfe_ensorceleuse()
    }

    fn montee(_f: &Fiche, de: u32) -> Montee {
        Montee {
            classe: "ensorceleur".into(),
            de_vie: de,
            rangs: BTreeMap::new(),
            predilection: Some(BonusPredilection::PointDeVie),
            dons: vec![],
            augmentation: None,
        }
    }

    #[test]
    fn tableau_3_1_xp() {
        let r = Referentiel::core();
        assert_eq!(xp_requis(&r, Vitesse::Moyenne, 2), Some(2000));
        assert_eq!(xp_requis(&r, Vitesse::Lente, 2), Some(3000));
        assert_eq!(xp_requis(&r, Vitesse::Rapide, 2), Some(1300));
        assert_eq!(xp_requis(&r, Vitesse::Moyenne, 20), Some(3_600_000));
        assert_eq!(xp_requis(&r, Vitesse::Jalons, 5), None);
    }

    #[test]
    fn niveau_atteignable_selon_xp_et_jalons() {
        let r = Referentiel::core();
        assert_eq!(niveau_atteignable(&r, Vitesse::Moyenne, 0, 0), 1);
        assert_eq!(niveau_atteignable(&r, Vitesse::Moyenne, 1999, 0), 1);
        assert_eq!(niveau_atteignable(&r, Vitesse::Moyenne, 2000, 0), 2);
        assert_eq!(niveau_atteignable(&r, Vitesse::Moyenne, 5000, 0), 3);
        assert_eq!(niveau_atteignable(&r, Vitesse::Moyenne, 99_999_999, 0), 20);
        assert_eq!(niveau_atteignable(&r, Vitesse::Jalons, 99_999, 4), 4);
        assert_eq!(niveau_atteignable(&r, Vitesse::Jalons, 0, 0), 1);
    }

    #[test]
    fn methodes_de_points_de_vie() {
        use MethodePv::*;
        assert_eq!(de_de_vie(Maximum, 6, None), Ok(6));
        assert_eq!(de_de_vie(MoyenneArrondie, 6, None), Ok(4));
        assert_eq!(de_de_vie(Moyenne, 6, None), Ok(3));
        assert_eq!(de_de_vie(Moyenne, 8, None), Ok(4));
        assert_eq!(de_de_vie(MoyenneArrondie, 8, None), Ok(5));
        assert_eq!(de_de_vie(Jet, 6, Some(5)), Ok(5));
        assert_eq!(
            de_de_vie(Jet, 6, Some(7)),
            Err(ErreurMontee::JetHorsLimites { jet: 7, de: 6 })
        );
        assert_eq!(de_de_vie(Jet, 6, None), Err(ErreurMontee::JetManquant));
    }

    #[test]
    fn niveau_2_apercu_et_application() {
        let r = Referentiel::core();
        let f = base();
        let a = apercu(&r, &f, "ensorceleur").unwrap();
        assert_eq!(a.niveau_de_personnage, 2);
        assert_eq!(a.bba, (0, 1));
        assert_eq!(a.volonte, (2, 3));
        assert_eq!(a.dons_a_choisir, 0);
        assert!(!a.augmentation);
        assert_eq!(a.sorts_connus_gagnes, vec![(0, 1)]); // 4 -> 5 tours de magie
        assert!(!a.echange_de_sort);

        let regles = ReglesFiche::default();
        let m = montee(&f, 4);
        let n = appliquer(&r, &regles, &f, &m, 2).unwrap();
        assert_eq!(n.niveau_de_personnage(), 2);
        assert_eq!(n.calculer(&r).unwrap().bba.total, 1);
        // la fiche d'origine n'est pas modifiée
        assert_eq!(f.niveau_de_personnage(), 1);
    }

    #[test]
    fn tant_que_le_niveau_n_est_pas_atteint_on_refuse() {
        let r = Referentiel::core();
        let f = base();
        let e = appliquer(&r, &ReglesFiche::default(), &f, &montee(&f, 4), 1).unwrap_err();
        assert_eq!(
            e,
            vec![ErreurMontee::NiveauPasEncoreAtteint {
                niveau: 2,
                atteignable: 1
            }]
        );
    }

    #[test]
    fn don_au_niveau_3_et_augmentation_au_niveau_4() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut f = base();
        f = appliquer(&r, &regles, &f, &montee(&f, 4), 20).unwrap();
        // niveau 3 : un don, sinon erreur
        let e = appliquer(&r, &regles, &f, &montee(&f, 4), 20).unwrap_err();
        assert_eq!(
            e,
            vec![ErreurMontee::DonsAttendus {
                attendus: 1,
                fournis: 0
            }]
        );
        let mut m = montee(&f, 4);
        m.dons = vec!["Toughness".into()];
        f = appliquer(&r, &regles, &f, &m, 20).unwrap();
        // niveau 4 : augmentation obligatoire
        let e = appliquer(&r, &regles, &f, &montee(&f, 4), 20).unwrap_err();
        assert_eq!(e, vec![ErreurMontee::AugmentationAttendue]);
        let mut m = montee(&f, 4);
        m.augmentation = Some(Carac::Cha);
        let a = apercu(&r, &f, "ensorceleur").unwrap();
        assert!(a.augmentation && a.echange_de_sort);
        f = appliquer(&r, &regles, &f, &m, 20).unwrap();
        assert_eq!(f.augmentations, vec![(4, Carac::Cha)]);
        assert_eq!(f.calculer(&r).unwrap().scores[&Carac::Cha].total, 17);
    }

    #[test]
    fn de_lignage_au_niveau_7() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut f = base();
        for niv in 2..=7 {
            let mut m = montee(&f, 4);
            if niv % 2 == 1 {
                m.dons = vec!["Don".into()];
            }
            if niv == 4 {
                m.augmentation = Some(Carac::Cha);
            }
            if niv == 7 {
                m.dons.push("Don de lignage".into()); // niveau impair + lignage
            }
            f = appliquer(&r, &regles, &f, &m, 20)
                .unwrap_or_else(|e| panic!("niveau {niv} : {e:?}"));
        }
        assert_eq!(f.niveau_de_personnage(), 7);
    }

    #[test]
    fn multiclassage_desactive_par_defaut() {
        let r = Referentiel::core();
        let f = base();
        // une seule classe disponible dans le référentiel : on simule une nouvelle classe inconnue
        let mut m = montee(&f, 4);
        m.classe = "magicien".into();
        let e = appliquer(&r, &ReglesFiche::default(), &f, &m, 20).unwrap_err();
        assert_eq!(e, vec![ErreurMontee::ClasseInconnue("magicien".into())]);
    }

    #[test]
    fn predilection_obligatoire_dans_la_classe_de_predilection() {
        let r = Referentiel::core();
        let f = base();
        let mut m = montee(&f, 4);
        m.predilection = None;
        let e = appliquer(&r, &ReglesFiche::default(), &f, &m, 20).unwrap_err();
        assert_eq!(e, vec![ErreurMontee::PredilectionAttendue]);
        let regles = ReglesFiche {
            classe_de_predilection: false,
            ..ReglesFiche::default()
        };
        assert!(appliquer(&r, &regles, &f, &m, 20).is_ok());
    }

    #[test]
    fn rangs_en_trop_refuses_a_la_montee() {
        let r = Referentiel::core();
        let f = base();
        let mut m = montee(&f, 4);
        m.rangs.insert("bluff".into(), 5); // 2+1 disponibles, +1 prédilection non choisie
        let e = appliquer(&r, &ReglesFiche::default(), &f, &m, 20).unwrap_err();
        assert!(matches!(
            e[0],
            ErreurMontee::Fiche(Probleme::RangsDepasses { niveau: 2, .. })
        ));
    }
}
