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
    /// Nouveaux sorts connus choisis à ce niveau (noms anglais du catalogue).
    #[cfg_attr(feature = "serde", serde(default))]
    pub sorts_appris: Vec<String>,
    /// Sort connu échangé contre un autre du même niveau : (ancien, nouveau).
    #[cfg_attr(feature = "serde", serde(default))]
    pub echange: Option<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErreurMontee {
    EchangeImpossible(String),
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
            ErreurMontee::EchangeImpossible(m) => write!(f, "échange de sort impossible : {m}"),
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
    /// Parmi ces dons, nombre à prendre dans la liste du lignage.
    pub dons_de_lignage: u32,
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

/// Dons de lignage à prendre (parmi les dons à choisir) en prenant le niveau `nc` de `classe`.
fn dons_de_lignage(classe: &crate::donnees::Classe, nc: u32) -> u32 {
    u32::from(
        !classe.lignages.is_empty()
            && table_a(classe, nc).is_some_and(|l| l.special_en.contains("Bloodline feat")),
    )
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
        dons_de_lignage: dons_de_lignage(classe, nc),
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

    let ap = apercu(r, f, &m.classe);
    if let Some((vieux, neuf)) = &m.echange {
        let permis = ap.as_ref().is_some_and(|a| a.echange_de_sort);
        let cle = classe.nom_en.to_lowercase();
        let niveau_de = |s: &str| r.sort(s).and_then(|x| x.niveau_pour(&cle));
        if !permis {
            e.push(ErreurMontee::EchangeImpossible(
                "pas d'échange possible à ce niveau".into(),
            ));
        } else if !f.sorts_connus.iter().any(|s| s.eq_ignore_ascii_case(vieux)) {
            e.push(ErreurMontee::EchangeImpossible(format!(
                "{vieux} n'est pas un sort connu (les sorts de lignage ne s'échangent pas)"
            )));
        } else if niveau_de(vieux).is_none() || niveau_de(vieux) != niveau_de(neuf) {
            e.push(ErreurMontee::EchangeImpossible(
                "le nouveau sort doit être du même niveau que l'ancien".into(),
            ));
        }
    }
    if !e.is_empty() {
        return Err(e);
    }

    let mut nouvelle = f.clone();
    if let Some((vieux, neuf)) = &m.echange {
        nouvelle
            .sorts_connus
            .retain(|s| !s.eq_ignore_ascii_case(vieux));
        nouvelle.sorts_connus.push(neuf.clone());
    }
    nouvelle.sorts_connus.extend(m.sorts_appris.iter().cloned());
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
    // Un sort de lignage déjà connu avant ce niveau (choisi quand il n'était pas encore
    // accordé) n'empêche pas de monter : il reste signalé sur la fiche, à remplacer ensuite.
    // Seul un sort de lignage choisi maintenant est refusé.
    let choisis_ici: Vec<&String> = m
        .sorts_appris
        .iter()
        .chain(m.echange.as_ref().map(|(_, neuf)| neuf))
        .collect();
    let problemes: Vec<Probleme> = nouvelle
        .valider(r)
        .into_iter()
        .filter(|p| match p {
            Probleme::SortDeLignageChoisi(s) => {
                choisis_ici.iter().any(|c| c.eq_ignore_ascii_case(s))
            }
            _ => true,
        })
        .collect();
    if problemes.is_empty() {
        Ok(nouvelle)
    } else {
        Err(problemes.into_iter().map(ErreurMontee::Fiche).collect())
    }
}

/// Niveau valide pour les tests : choisit automatiquement les sorts, dons et l'augmentation requis.
#[cfg(test)]
pub(crate) fn montee_auto(r: &Referentiel, f: &Fiche, classe: &str, de: u32) -> Montee {
    let ap = apercu(r, f, classe).unwrap();
    let cle = r.classe(classe).unwrap().nom_en.to_lowercase();
    let mut pris = f.sorts_connus.clone();
    // les sorts accordés par le lignage ne se choisissent pas
    pris.extend(
        f.lignage
            .as_ref()
            .and_then(|id| r.classe(classe)?.lignages.iter().find(|l| &l.id == id))
            .map(|l| {
                l.sorts_bonus
                    .iter()
                    .map(|s| s.nom_en.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    );
    let mut neufs = Vec::new();
    for (sn, nb) in &ap.sorts_connus_gagnes {
        for _ in 0..*nb {
            let s = r
                .sorts
                .iter()
                .filter(|s| s.niveau_pour(&cle) == Some(*sn))
                .find(|s| !pris.contains(&s.nom_en))
                .expect("assez de sorts dans le catalogue");
            pris.push(s.nom_en.clone());
            neufs.push(s.nom_en.clone());
        }
    }
    let deja = f.dons_choisis();
    let lignage = f.lignage.as_ref().and_then(|id| {
        r.classe(classe)
            .and_then(|c| c.lignages.iter().find(|l| &l.id == id))
    });
    let mut dons: Vec<String> = Vec::new();
    if let (Some(l), true) = (lignage, ap.dons_de_lignage > 0) {
        // don de lignage : le premier de la liste que la fiche n'a pas encore
        let choisi = l
            .dons_bonus_en
            .iter()
            .find(|d| !deja.contains(d))
            .expect("assez de dons de lignage");
        dons.push(choisi.clone());
    }
    let libres = ap.dons_a_choisir as usize - dons.len();
    let autres: Vec<String> = r
        .dons
        .iter()
        .filter(|d| !d.repetable() && d.prerequis.is_empty() && !deja.contains(&d.nom_en))
        .filter(|d| !d.a_un_choix())
        .filter(|d| !dons.contains(&d.nom_en))
        .map(|d| d.nom_en.clone())
        .take(libres)
        .collect();
    dons.extend(autres);
    Montee {
        classe: classe.to_string(),
        de_vie: de,
        rangs: BTreeMap::new(),
        predilection: ap
            .predilection_possible
            .then_some(BonusPredilection::PointDeVie),
        dons,
        augmentation: ap.augmentation.then_some(Carac::Cha),
        sorts_appris: neufs,
        echange: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::donnees::Referentiel;

    /// Fiche d'exemple dont le sort « Mage Armor » (accordé par le lignage au niveau 3 de
    /// classe) est remplacé par un autre sort de niveau 1 : on ne le choisit pas en double.
    fn base() -> Fiche {
        let mut f = crate::fiche::tests::elfe_ensorceleuse();
        let r = Referentiel::core();
        let neuf = r
            .sorts
            .iter()
            .find(|s| {
                s.niveau_pour("sorcerer") == Some(1)
                    && !f.sorts_connus.contains(&s.nom_en)
                    && s.nom_en != "Mage Armor"
            })
            .unwrap()
            .nom_en
            .clone();
        for s in &mut f.sorts_connus {
            if s == "Mage Armor" {
                *s = neuf.clone();
            }
        }
        f
    }

    fn montee(r: &Referentiel, f: &Fiche, de: u32) -> Montee {
        montee_auto(r, f, "ensorceleur", de)
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

        let n = appliquer(&r, &ReglesFiche::default(), &f, &montee(&r, &f, 4), 2).unwrap();
        assert_eq!(n.niveau_de_personnage(), 2);
        assert_eq!(n.sorts_connus.len(), 5 + 2);
        assert_eq!(n.calculer(&r).unwrap().bba.total, 1);
        // la fiche d'origine n'est pas modifiée
        assert_eq!(f.niveau_de_personnage(), 1);
    }

    #[test]
    fn tant_que_le_niveau_n_est_pas_atteint_on_refuse() {
        let r = Referentiel::core();
        let f = base();
        let e = appliquer(&r, &ReglesFiche::default(), &f, &montee(&r, &f, 4), 1).unwrap_err();
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
        f = appliquer(&r, &regles, &f, &montee(&r, &f, 4), 20).unwrap();
        // niveau 3 : un don, sinon erreur
        let mut m = montee(&r, &f, 4);
        m.dons.clear();
        let e = appliquer(&r, &regles, &f, &m, 20).unwrap_err();
        assert_eq!(
            e,
            vec![ErreurMontee::DonsAttendus {
                attendus: 1,
                fournis: 0
            }]
        );
        f = appliquer(&r, &regles, &f, &montee(&r, &f, 4), 20).unwrap();
        // niveau 4 : augmentation obligatoire
        let mut m = montee(&r, &f, 4);
        m.augmentation = None;
        let e = appliquer(&r, &regles, &f, &m, 20).unwrap_err();
        assert_eq!(e, vec![ErreurMontee::AugmentationAttendue]);
        let a = apercu(&r, &f, "ensorceleur").unwrap();
        assert!(a.augmentation && a.echange_de_sort);
        f = appliquer(&r, &regles, &f, &montee(&r, &f, 4), 20).unwrap();
        assert_eq!(f.augmentations, vec![(4, Carac::Cha)]);
        assert_eq!(f.calculer(&r).unwrap().scores[&Carac::Cha].total, 17);
    }

    #[test]
    fn de_lignage_au_niveau_7() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut f = base();
        for niv in 2..=7 {
            let m = montee(&r, &f, 4);
            if niv == 7 {
                assert_eq!(m.dons.len(), 2, "don de niveau impair + don de lignage");
            }
            f = appliquer(&r, &regles, &f, &m, 20)
                .unwrap_or_else(|e| panic!("niveau {niv} : {e:?}"));
        }
        assert_eq!(f.niveau_de_personnage(), 7);
    }

    #[test]
    fn don_de_lignage_limite_a_la_liste_du_lignage() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut f = base();
        for _ in 2..=6 {
            f = appliquer(&r, &regles, &f, &montee(&r, &f, 4), 20).unwrap();
        }
        let a = apercu(&r, &f, "ensorceleur").unwrap();
        assert_eq!((a.dons_a_choisir, a.dons_de_lignage), (2, 1));
        // deux dons quelconques hors de la liste du lignage : refusé
        let mut m = montee(&r, &f, 4);
        let deja = f.dons_choisis();
        m.dons = r
            .dons
            .iter()
            .filter(|d| {
                !d.repetable()
                    && d.prerequis.is_empty()
                    && !d.a_un_choix()
                    && !deja.contains(&d.nom_en)
            })
            .filter(|d| {
                let l = &r.classe("ensorceleur").unwrap().lignages;
                let id = f.lignage.as_ref().unwrap();
                !l.iter()
                    .find(|x| &x.id == id)
                    .unwrap()
                    .autorise_don(&d.nom_en)
            })
            .take(2)
            .map(|d| d.nom_en.clone())
            .collect();
        let e = appliquer(&r, &regles, &f, &m, 20).unwrap_err();
        assert!(
            e.iter().any(|x| matches!(
                x,
                ErreurMontee::Fiche(Probleme::DonDeLignageManquant { niveau: 7, .. })
            )),
            "{e:?}"
        );
        // un des deux dons dans la liste : accepté
        let f7 = appliquer(&r, &regles, &f, &montee(&r, &f, 4), 20).unwrap();
        assert_eq!(f7.niveau_de_personnage(), 7);
    }

    #[test]
    fn liste_de_dons_du_lignage() {
        let r = Referentiel::core();
        let l = &r.classe("ensorceleur").unwrap().lignages;
        let draconique = l.iter().find(|x| x.id == "draconic").unwrap();
        assert!(draconique.autorise_don("Great Fortitude"));
        assert!(draconique.autorise_don("great fortitude"));
        assert!(draconique.autorise_don("Skill Focus (Fly)"));
        assert!(!draconique.autorise_don("Skill Focus (Acrobatics)"));
        assert!(!draconique.autorise_don("Dodge"));
        // une entrée sans précision accepte toutes les précisions (Spell Focus, Weapon Focus…)
        let feerique = l.iter().find(|x| x.id == "fey").unwrap();
        assert!(!feerique.autorise_don("Iron Will"));
        assert!(feerique.autorise_don("Dodge"));
        let arcanique = l.iter().find(|x| x.id == "arcane").unwrap();
        assert!(arcanique.autorise_don("Spell Focus (evocation)"));
    }

    #[test]
    fn multiclassage_desactive_par_defaut() {
        let r = Referentiel::core();
        let f = base();
        let mut m = montee(&r, &f, 4);
        m.classe = "magicien".into();
        let e = appliquer(&r, &ReglesFiche::default(), &f, &m, 20).unwrap_err();
        assert_eq!(e, vec![ErreurMontee::ClasseInconnue("magicien".into())]);
    }

    #[test]
    fn predilection_obligatoire_dans_la_classe_de_predilection() {
        let r = Referentiel::core();
        let f = base();
        let mut m = montee(&r, &f, 4);
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
        let mut m = montee(&r, &f, 4);
        m.rangs.insert("bluff".into(), 5);
        let e = appliquer(&r, &ReglesFiche::default(), &f, &m, 20).unwrap_err();
        assert!(matches!(
            e[0],
            ErreurMontee::Fiche(Probleme::RangsDepasses { niveau: 2, .. })
        ));
    }

    #[test]
    fn sorts_a_choisir_a_chaque_niveau() {
        let r = Referentiel::core();
        let f = base();
        let mut m = montee(&r, &f, 4);
        m.sorts_appris.clear();
        let e = appliquer(&r, &ReglesFiche::default(), &f, &m, 20).unwrap_err();
        assert!(e.iter().any(|x| matches!(
            x,
            ErreurMontee::Fiche(Probleme::SortsAChoisir {
                niveau_de_sort: 0,
                attendus: 5,
                choisis: 4,
                ..
            })
        )));
        // un sort absent de la liste de l'ensorceleur est refusé
        let mut m = montee(&r, &f, 4);
        m.sorts_appris = vec!["Cure Light Wounds".into()];
        let e = appliquer(&r, &ReglesFiche::default(), &f, &m, 20).unwrap_err();
        assert!(e.iter().any(|x| matches!(
            x,
            ErreurMontee::Fiche(Probleme::SortInconnu(_) | Probleme::SortHorsListe { .. })
        )));
    }

    #[test]
    fn echange_de_sort_a_partir_du_niveau_4() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let mut f = base();
        for _ in 2..=3 {
            f = appliquer(&r, &regles, &f, &montee(&r, &f, 4), 20).unwrap();
        }
        // niveau 4 : échange permis ; ici contre un sort de niveau 1 du catalogue
        let vieux = f
            .sorts_connus
            .iter()
            .find(|s| r.sort(s).unwrap().niveau_pour("sorcerer") == Some(1))
            .unwrap()
            .clone();
        let neuf = r
            .sorts
            .iter()
            .find(|s| s.niveau_pour("sorcerer") == Some(1) && !f.sorts_connus.contains(&s.nom_en))
            .unwrap()
            .nom_en
            .clone();
        let mut m = montee(&r, &f, 4);
        m.echange = Some((vieux.clone(), neuf.clone()));
        let n = appliquer(&r, &regles, &f, &m, 20).unwrap();
        assert!(n.sorts_connus.contains(&neuf) && !n.sorts_connus.contains(&vieux));
        // pas d'échange au niveau 2 (niveau d'échange : 4, 6, 8…)
        let f2 = appliquer(&r, &regles, &base(), &montee(&r, &base(), 4), 20).unwrap();
        let mut m = montee(&r, &f2, 4);
        m.echange = Some((vieux, neuf));
        let e = appliquer(&r, &regles, &f2, &m, 20).unwrap_err();
        assert!(matches!(e[0], ErreurMontee::EchangeImpossible(_)));
    }

    /// Fiche de niveau 2 (lignage draconique) ; au niveau 3 de classe, « Mage Armor » est accordé.
    fn niveau_2() -> Fiche {
        let r = Referentiel::core();
        let f = base();
        appliquer(&r, &ReglesFiche::default(), &f, &montee(&r, &f, 4), 20).unwrap()
    }

    #[test]
    fn sort_de_lignage_non_rechoisissable_a_la_montee() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let f = niveau_2();
        let mut m = montee(&r, &f, 4);
        // un niveau de plus : le lignage accorde « Mage Armor »
        let niveau_1 = m
            .sorts_appris
            .iter()
            .position(|s| r.sort(s).and_then(|d| d.niveau_pour("sorcerer")) == Some(1));
        // niveau 3 : un seul sort de niveau 1 gagné ? sinon on ajoute le choix interdit à la main
        match niveau_1 {
            Some(i) => m.sorts_appris[i] = "Mage Armor".into(),
            None => {
                m.sorts_appris.pop();
                m.sorts_appris.push("Mage Armor".into());
            }
        }
        let e = appliquer(&r, &regles, &f, &m, 20).unwrap_err();
        assert!(e.iter().any(|x| matches!(
            x,
            ErreurMontee::Fiche(Probleme::SortDeLignageChoisi(s)) if s == "Mage Armor"
        )));
    }

    #[test]
    fn sort_de_lignage_choisi_avant_d_etre_accorde_est_a_remplacer() {
        use crate::journal::{EvenementFiche, Journal};
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        // niveau 1 : « Mage Armor » choisi, pas encore accordé : valide
        let f = crate::fiche::tests::elfe_ensorceleuse();
        assert!(f.sorts_connus.contains(&"Mage Armor".to_string()));
        assert!(f.valider(&r).is_empty());
        assert!(f.remplacements_de_sorts(&r).is_empty());
        let mut j = Journal::nouveau(f);
        j.ajouter(
            EvenementFiche::XpGagnes {
                nombre: 100_000,
                motif: String::new(),
            },
            &r,
            &regles,
        )
        .unwrap();
        // les vieilles campagnes restent lisibles : la montée au niveau 3 passe
        for _ in 2..=3 {
            let etat = j.etat(&r, &regles).unwrap();
            let m = montee(&r, &etat.fiche, 4);
            j.ajouter(EvenementFiche::MonteeDeNiveau(m), &r, &regles)
                .unwrap();
        }
        // mais le doublon avec le lignage est signalé
        let etat = j.etat(&r, &regles).unwrap();
        assert!(etat
            .fiche
            .valider(&r)
            .contains(&Probleme::SortDeLignageChoisi("Mage Armor".into())));
        let rempl = etat.fiche.remplacements_de_sorts(&r);
        assert_eq!(rempl.len(), 1);
        assert_eq!(rempl[0].ancien, "Mage Armor");
        assert_eq!(rempl[0].niveau_de_sort, 1);
        let choix = rempl[0].propositions[0].clone();
        assert_eq!(r.sort(&choix).unwrap().niveau_pour("sorcerer"), Some(1));
        // un remplacement hors liste (autre niveau, ou sort du lignage) est refusé
        let mauvais = r
            .sorts
            .iter()
            .find(|s| s.niveau_pour("sorcerer") == Some(2))
            .unwrap()
            .nom_en
            .clone();
        for nouveau in [mauvais, "Mage Armor".to_string()] {
            let ev = EvenementFiche::SortRemplace {
                ancien: "Mage Armor".into(),
                nouveau,
            };
            assert!(j.ajouter(ev, &r, &regles).is_err());
        }
        // le bon remplacement rend la fiche valide
        j.ajouter(
            EvenementFiche::SortRemplace {
                ancien: "Mage Armor".into(),
                nouveau: choix.clone(),
            },
            &r,
            &regles,
        )
        .unwrap();
        let f = j.etat(&r, &regles).unwrap().fiche;
        assert!(f.valider(&r).is_empty());
        assert!(f.sorts_connus.contains(&choix));
        assert!(f.remplacements_de_sorts(&r).is_empty());
    }

    #[test]
    fn sort_de_lignage_choisi_a_la_montee_est_refuse() {
        let r = Referentiel::core();
        let regles = ReglesFiche::default();
        let f = niveau_2();
        let mut m = montee(&r, &f, 4);
        m.sorts_appris.pop();
        m.sorts_appris.push("Mage Armor".into());
        let e = appliquer(&r, &regles, &f, &m, 20).unwrap_err();
        assert!(e.iter().any(|x| matches!(
            x,
            ErreurMontee::Fiche(Probleme::SortDeLignageChoisi(s)) if s == "Mage Armor"
        )));
    }

    #[test]
    fn sort_en_double_propose_un_remplacement_du_meme_niveau() {
        let r = Referentiel::core();
        let mut f = niveau_2();
        let doublon = f
            .sorts_connus
            .iter()
            .find(|s| r.sort(s).unwrap().niveau_pour("sorcerer") == Some(1))
            .unwrap()
            .clone();
        f.sorts_connus.push(doublon.clone());
        assert!(f
            .valider(&r)
            .iter()
            .any(|p| matches!(p, Probleme::SortEnDouble(_))));
        let rempl = f.remplacements_de_sorts(&r);
        assert_eq!(rempl.len(), 1);
        assert_eq!(rempl[0].ancien, doublon);
        assert!(!rempl[0].propositions.is_empty());
        for p in &rempl[0].propositions {
            assert_eq!(r.sort(p).unwrap().niveau_pour("sorcerer"), Some(1));
            assert!(!f.sorts_connus.contains(p), "{p} déjà connu");
            assert_ne!(p, "Mage Armor", "sort de lignage proposé");
        }
        // appliquer la première proposition rend la fiche valide
        f.sorts_connus.pop();
        assert!(f.valider(&r).is_empty());
    }
}
