//! Fiches de personnage dans la campagne : réglages de campagne, registre des règles
//! optionnelles, lien PJ ↔ fiche, Prestige calculé.
//!
//! Chaque PJ peut avoir une fiche (module `fiches`) dont le journal est enveloppé dans le journal
//! de la campagne par `Evenement::Fiche` : annuler/rétablir et l'historique restent uniques.

use crate::etat::{Erreur, Etat};
use crate::modele::IdPersonnage;
use fiches::generation::Methode;
use fiches::journal::{EtatPersonnage, EvenementFiche, Journal};
use fiches::progression::{xp_requis, MethodePv, ReglesFiche, Vitesse};
use fiches::Referentiel as ReferentielFiches;
use regles::options::OptionsRegles;

/// Réglages de campagne propres aux personnages (journalisés : `Evenement::ReglagePersonnages`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct ReglesPersonnages {
    pub vitesse: Vitesse,
    pub methode_pv: MethodePv,
    pub multiclassage: bool,
    pub classe_de_predilection: bool,
    pub generation: Methode,
}

impl Default for ReglesPersonnages {
    fn default() -> Self {
        let f = ReglesFiche::default();
        ReglesPersonnages {
            vitesse: f.vitesse,
            methode_pv: f.methode_pv,
            multiclassage: f.multiclassage,
            classe_de_predilection: f.classe_de_predilection,
            generation: Methode::Achat { points: 15 },
        }
    }
}

impl ReglesPersonnages {
    pub fn fiche(&self) -> ReglesFiche {
        ReglesFiche {
            vitesse: self.vitesse,
            methode_pv: self.methode_pv,
            multiclassage: self.multiclassage,
            classe_de_predilection: self.classe_de_predilection,
        }
    }
}

/// Une règle optionnelle de campagne, telle que l'interface la présente.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct RegleInfo {
    pub id: &'static str,
    pub nom: &'static str,
    pub description: &'static str,
    /// Règle du livre (vrai) ou règle personnalisée décidée par le joueur (faux).
    pub officielle: bool,
    /// Ce que le réglage change concrètement.
    pub impact: &'static str,
    /// Valeur par défaut et valeur actuelle (identifiants de `choix`).
    pub defaut: &'static str,
    pub valeur: String,
    /// (identifiant, libellé) de chaque valeur possible.
    pub choix: Vec<(&'static str, &'static str)>,
}

const OUI_NON: [(&str, &str); 2] = [("oui", "Oui"), ("non", "Non")];

fn bool_id(b: bool) -> String {
    (if b { "oui" } else { "non" }).to_string()
}

fn vitesse_id(v: Vitesse) -> &'static str {
    match v {
        Vitesse::Lente => "lente",
        Vitesse::Moyenne => "moyenne",
        Vitesse::Rapide => "rapide",
        Vitesse::Jalons => "jalons",
    }
}

fn pv_id(m: MethodePv) -> &'static str {
    match m {
        MethodePv::Jet => "jet",
        MethodePv::Moyenne => "moyenne",
        MethodePv::MoyenneArrondie => "moyenne_arrondie",
        MethodePv::Maximum => "maximum",
    }
}

fn generation_id(m: Methode) -> String {
    match m {
        Methode::Achat { points } => format!("achat_{points}"),
        Methode::Standard => "standard".into(),
        Methode::Classique => "classique".into(),
        Methode::Heroique => "heroique".into(),
        Methode::ReserveDeDes { des } => format!("reserve_{des}"),
        Methode::Libre => "libre".into(),
    }
}

fn generation_depuis(id: &str) -> Option<Methode> {
    match id {
        "standard" => Some(Methode::Standard),
        "classique" => Some(Methode::Classique),
        "heroique" => Some(Methode::Heroique),
        "libre" => Some(Methode::Libre),
        _ => {
            if let Some(p) = id.strip_prefix("achat_") {
                p.parse().ok().map(|points| Methode::Achat { points })
            } else {
                id.strip_prefix("reserve_")?
                    .parse()
                    .ok()
                    .map(|des| Methode::ReserveDeDes { des })
            }
        }
    }
}

/// Le registre complet des règles optionnelles : réglages des personnages puis règles de
/// l'intermède déjà existantes.
pub fn registre(o: &OptionsRegles, p: &ReglesPersonnages) -> Vec<RegleInfo> {
    let oui_non = || OUI_NON.to_vec();
    vec![
        RegleInfo {
            id: "pj.vitesse",
            nom: "Vitesse de progression",
            description: "Table d'expérience utilisée pour monter de niveau (Core p. 31), ou niveaux accordés par le MJ.",
            officielle: true,
            impact: "Fixe les XP nécessaires pour chaque niveau.",
            defaut: "moyenne",
            valeur: vitesse_id(p.vitesse).into(),
            choix: vec![
                ("lente", "Lente"),
                ("moyenne", "Moyenne"),
                ("rapide", "Rapide"),
                ("jalons", "Jalons (niveaux accordés par le MJ)"),
            ],
        },
        RegleInfo {
            id: "pj.methode_pv",
            nom: "Points de vie aux niveaux suivants",
            description: "Comment est obtenu le dé de vie après le niveau 1 (le niveau 1 est toujours au maximum).",
            officielle: true,
            impact: "Le dé de vie proposé à la montée de niveau.",
            defaut: "moyenne_arrondie",
            valeur: pv_id(p.methode_pv).into(),
            choix: vec![
                ("jet", "Jet de dé"),
                ("moyenne", "Moyenne (arrondie en bas)"),
                ("moyenne_arrondie", "Moyenne (arrondie en haut)"),
                ("maximum", "Maximum"),
            ],
        },
        RegleInfo {
            id: "pj.generation",
            nom: "Génération des caractéristiques",
            description: "Méthode proposée à la création d'un personnage (Core p. 15).",
            officielle: true,
            impact: "Les scores de base proposés ou contrôlés à la création.",
            defaut: "achat_15",
            valeur: generation_id(p.generation),
            choix: vec![
                ("achat_10", "Achat de points : 10 (faible)"),
                ("achat_15", "Achat de points : 15 (standard)"),
                ("achat_20", "Achat de points : 20 (élevé)"),
                ("achat_25", "Achat de points : 25 (épique)"),
                ("standard", "Standard (4d6, on retire le plus bas)"),
                ("classique", "Classique (3d6)"),
                ("heroique", "Héroïque (2d6 + 6)"),
                ("reserve_24", "Réserve de dés (24 dés)"),
                ("reserve_28", "Réserve de dés (28 dés)"),
                ("libre", "Saisie libre"),
            ],
        },
        RegleInfo {
            id: "pj.multiclassage",
            nom: "Multiclassage",
            description: "Un personnage peut prendre des niveaux dans plusieurs classes.",
            officielle: true,
            impact: "La montée de niveau propose ou non d'autres classes.",
            defaut: "non",
            valeur: bool_id(p.multiclassage),
            choix: oui_non(),
        },
        RegleInfo {
            id: "pj.classe_de_predilection",
            nom: "Bonus de classe de prédilection",
            description: "+1 point de vie ou +1 rang de compétence à chaque niveau dans la classe de prédilection (Core p. 31).",
            officielle: true,
            impact: "Un choix de plus à chaque montée de niveau.",
            defaut: "oui",
            valeur: bool_id(p.classe_de_predilection),
            choix: oui_non(),
        },
        RegleInfo {
            id: "inter.repartition_stricte",
            nom: "Répartition stricte des bonus",
            description: "Une salle ou une équipe donne tout son bonus à un seul type de capital par jour (officiel : répartition libre entre types admis, GC p. 94).",
            officielle: false,
            impact: "Recettes des salles et des équipes.",
            defaut: "oui",
            valeur: bool_id(o.repartition_stricte),
            choix: oui_non(),
        },
        RegleInfo {
            id: "inter.construction_parallele",
            nom: "Construction en parallèle",
            description: "Les salles d'un même étage peuvent être construites en même temps.",
            officielle: false,
            impact: "Chantiers.",
            defaut: "oui",
            valeur: bool_id(o.construction_parallele_autorisee),
            choix: oui_non(),
        },
        RegleInfo {
            id: "inter.production_pendant_amelioration",
            nom: "Production pendant une amélioration",
            description: "Une salle en travaux d'amélioration continue de produire.",
            officielle: false,
            impact: "Recettes des salles améliorées.",
            defaut: "non",
            valeur: bool_id(o.production_pendant_amelioration),
            choix: oui_non(),
        },
        RegleInfo {
            id: "inter.moitie_equipe_sans_batiment",
            nom: "Équipe sans bâtiment : recettes divisées par 2",
            description: "Règle officielle (GC p. 94). Désactivée, l'équipe garde son bonus entier.",
            officielle: true,
            impact: "Recettes des équipes sans lieu de travail.",
            defaut: "oui",
            valeur: bool_id(o.moitie_equipe_sans_batiment),
            choix: oui_non(),
        },
        RegleInfo {
            id: "inter.delocalisation_jet_unique",
            nom: "Délocalisation : un seul jet",
            description: "Un seul jet de 1d6 pour toute l'organisation, au lieu d'un par groupe d'employés (GC p. 95).",
            officielle: false,
            impact: "Durée de préparation d'une délocalisation.",
            defaut: "non",
            valeur: bool_id(o.delocalisation_jet_unique),
            choix: oui_non(),
        },
        RegleInfo {
            id: "inter.capitaux_produits_payants",
            nom: "Capitaux produits payants",
            description: "Les capitaux produits par les bâtiments et organisations se paient au Coût produit (GC p. 94). Désactivé : gratuits (décision du 23/10/2026).",
            officielle: true,
            impact: "Coût des capitaux gardés après les tests de recettes.",
            defaut: "non",
            valeur: bool_id(o.capitaux_produits_payants),
            choix: oui_non(),
        },
    ]
}

/// Valide un réglage proposé et renvoie les nouveaux réglages de personnages ou d'intermède.
pub enum Reglage {
    Personnages(ReglesPersonnages),
    Intermede(OptionsRegles),
}

pub fn changer(
    id: &str,
    valeur: &str,
    o: &OptionsRegles,
    p: &ReglesPersonnages,
) -> Result<Reglage, String> {
    let oui = |v: &str| match v {
        "oui" => Ok(true),
        "non" => Ok(false),
        _ => Err(format!("valeur inconnue : {v}")),
    };
    let mut p2 = *p;
    let mut o2 = o.clone();
    match id {
        "pj.vitesse" => {
            p2.vitesse = match valeur {
                "lente" => Vitesse::Lente,
                "moyenne" => Vitesse::Moyenne,
                "rapide" => Vitesse::Rapide,
                "jalons" => Vitesse::Jalons,
                v => return Err(format!("valeur inconnue : {v}")),
            }
        }
        "pj.methode_pv" => {
            p2.methode_pv = match valeur {
                "jet" => MethodePv::Jet,
                "moyenne" => MethodePv::Moyenne,
                "moyenne_arrondie" => MethodePv::MoyenneArrondie,
                "maximum" => MethodePv::Maximum,
                v => return Err(format!("valeur inconnue : {v}")),
            }
        }
        "pj.generation" => {
            p2.generation =
                generation_depuis(valeur).ok_or_else(|| format!("valeur inconnue : {valeur}"))?
        }
        "pj.multiclassage" => p2.multiclassage = oui(valeur)?,
        "pj.classe_de_predilection" => p2.classe_de_predilection = oui(valeur)?,
        "inter.repartition_stricte" => o2.repartition_stricte = oui(valeur)?,
        "inter.construction_parallele" => o2.construction_parallele_autorisee = oui(valeur)?,
        "inter.production_pendant_amelioration" => {
            o2.production_pendant_amelioration = oui(valeur)?
        }
        "inter.moitie_equipe_sans_batiment" => o2.moitie_equipe_sans_batiment = oui(valeur)?,
        "inter.delocalisation_jet_unique" => o2.delocalisation_jet_unique = oui(valeur)?,
        "inter.capitaux_produits_payants" => o2.capitaux_produits_payants = oui(valeur)?,
        _ => return Err(format!("règle inconnue : {id}")),
    }
    Ok(if id.starts_with("pj.") {
        Reglage::Personnages(p2)
    } else {
        Reglage::Intermede(o2)
    })
}

/// Nombre de rounds dans un jour (1 round = 6 secondes, 24 h) : le temps de campagne fait
/// avancer les effets temporaires des fiches.
pub const ROUNDS_PAR_JOUR: u32 = 14_400;

impl Etat {
    /// Applique un événement du journal d'un personnage (création comprise).
    pub(crate) fn appliquer_fiche(
        &mut self,
        pj: IdPersonnage,
        ev: &EvenementFiche,
    ) -> Result<(), Erreur> {
        self.pj(pj)?;
        self.verifier_fiche_d_acolyte(pj, ev)?;
        let r = ReferentielFiches::partage();
        let regles = self.regles_personnages.fiche();
        let resultat = self.appliquer_fiche_brute(pj, ev, r, &regles);
        if resultat.is_ok() {
            if let EvenementFiche::XpGagnes { nombre, .. } = ev {
                self.partager_l_xp_avec_l_acolyte(pj, *nombre);
            }
        }
        resultat
    }

    fn appliquer_fiche_brute(
        &mut self,
        pj: IdPersonnage,
        ev: &EvenementFiche,
        r: &ReferentielFiches,
        regles: &ReglesFiche,
    ) -> Result<(), Erreur> {
        match self.fiches.get_mut(&pj) {
            None => {
                let EvenementFiche::Creation(f) = ev else {
                    return Err(Erreur::Fiche(
                        "ce personnage n'a pas encore de fiche : la créer d'abord".into(),
                    ));
                };
                if let Some(p) = f.valider(r).first() {
                    return Err(Erreur::Fiche(p.to_string()));
                }
                let j = Journal::nouveau((**f).clone());
                j.etat(r, regles)
                    .map_err(|e| Erreur::Fiche(e.to_string()))?;
                self.fiches.insert(pj, j);
            }
            Some(j) => {
                let mut copie = j.clone();
                copie
                    .ajouter(ev.clone(), r, regles)
                    .map_err(|e| Erreur::Fiche(e.to_string()))?;
                *j = copie;
            }
        }
        Ok(())
    }

    /// Fait passer `jours` jours sur les effets temporaires de toutes les fiches.
    pub(crate) fn faire_passer_le_temps_des_fiches(&mut self, jours: u32) {
        let r = ReferentielFiches::partage();
        let regles = self.regles_personnages.fiche();
        let rounds = jours.saturating_mul(ROUNDS_PAR_JOUR);
        for j in self.fiches.values_mut() {
            let a_duree = j.etat(r, &regles).is_ok_and(|e| {
                e.fiche
                    .temporaires
                    .iter()
                    .any(|t| t.rounds_restants.is_some())
            });
            if a_duree {
                let _ = j.ajouter(EvenementFiche::TempsEcoule { rounds }, r, &regles);
            }
        }
    }

    /// État recalculé de la fiche d'un PJ, s'il en a une.
    pub fn fiche_de(&self, pj: IdPersonnage) -> Option<EtatPersonnage> {
        let j = self.fiches.get(&pj)?;
        j.etat(
            ReferentielFiches::partage(),
            &self.regles_personnages.fiche(),
        )
        .ok()
    }

    /// Événement qui donne à un personnage de niveau 1 de quoi atteindre `niveau` dès sa création
    /// (personnage créé à un niveau supérieur à 1) : les XP du niveau choisi selon la vitesse de la
    /// campagne, ou le jalon en mode jalons. `Ok(None)` : rien à accorder. Les niveaux se montent
    /// ensuite un à un par l'assistant de montée de niveau, comme en jeu.
    pub fn evenement_niveau_de_depart(
        &self,
        pj: IdPersonnage,
        niveau: u32,
    ) -> Result<Option<EvenementFiche>, String> {
        if !(1..=20).contains(&niveau) {
            return Err("Le niveau doit être compris entre 1 et 20.".into());
        }
        if let Some(maitre) = self.acolytes.get(&pj) {
            let plafond = self.plafond_acolyte(*maitre).unwrap_or(0);
            if niveau > plafond {
                return Err(format!(
                    "Un acolyte ne peut pas dépasser le niveau {plafond} (tableau du don Prestige \
                     et au moins deux niveaux sous son maître)."
                ));
            }
        }
        let e = self
            .fiche_de(pj)
            .ok_or("Ce personnage n'a pas encore de fiche.")?;
        let vitesse = self.regles_personnages.fiche().vitesse;
        let motif = format!("Création au niveau {niveau}");
        if vitesse == Vitesse::Jalons {
            return Ok((niveau > e.jalon).then_some(EvenementFiche::JalonAccorde { niveau }));
        }
        let requis = xp_requis(ReferentielFiches::partage(), vitesse, niveau)
            .ok_or("Niveau inconnu du tableau de progression.")?;
        Ok((requis > e.xp).then(|| EvenementFiche::XpGagnes {
            nombre: requis - e.xp,
            motif,
        }))
    }

    /// Prestige du PJ : la valeur saisie si elle existe, sinon niveau + modificateur de Charisme
    /// de sa fiche (GC p. 93), sinon rien.
    pub fn prestige_effectif(&self, pj: IdPersonnage) -> Option<i32> {
        let saisi = self.personnages.get(&pj)?.prestige;
        if saisi.is_some() {
            return saisi;
        }
        let e = self.fiche_de(pj)?;
        let v = e.fiche.calculer(ReferentielFiches::partage()).ok()?;
        Some(v.niveau_de_personnage as i32 + v.modificateurs[&fiches::Carac::Cha])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evenements::Evenement;
    use crate::modele::IdPersonnage;
    use crate::Campagne;
    use domaine::Montant;
    use fiches::etats::EffetTemporaire;
    use fiches::journal::EvenementFiche;
    use fiches::{Effet, TypeBonus};

    fn campagne_avec_pj() -> (Campagne, regles::Referentiel, IdPersonnage) {
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
        (c, r, pj)
    }

    fn fiche_ev(pj: IdPersonnage, e: EvenementFiche) -> Evenement {
        Evenement::Fiche { pj, evenement: e }
    }

    #[test]
    fn un_personnage_peut_etre_cree_a_un_niveau_superieur_a_un() {
        let (mut c, r, pj) = campagne_avec_pj();
        let fiche = fiches::exemple::elfe_ensorceleuse();
        c.executer(fiche_ev(pj, EvenementFiche::Creation(Box::new(fiche))), &r)
            .unwrap();
        // niveau 1 : rien à accorder
        assert_eq!(c.etat(&r).evenement_niveau_de_depart(pj, 1), Ok(None));
        assert!(c.etat(&r).evenement_niveau_de_depart(pj, 21).is_err());
        let ev = c
            .etat(&r)
            .evenement_niveau_de_depart(pj, 5)
            .unwrap()
            .unwrap();
        c.executer(fiche_ev(pj, ev), &r).unwrap();
        assert_eq!(c.etat(&r).fiche_de(pj).unwrap().niveau_atteignable, 5);
        // une seule annulation défait l'octroi
        c.annuler().unwrap();
        assert_eq!(c.etat(&r).fiche_de(pj).unwrap().niveau_atteignable, 1);
    }

    #[test]
    fn la_fiche_vit_dans_le_journal_de_la_campagne() {
        let (mut c, r, pj) = campagne_avec_pj();
        // Avant la création de la fiche, rien d'autre n'est accepté.
        assert!(c.executer(fiche_ev(pj, EvenementFiche::Repos), &r).is_err());
        let fiche = fiches::exemple::elfe_ensorceleuse();
        c.executer(fiche_ev(pj, EvenementFiche::Creation(Box::new(fiche))), &r)
            .unwrap();
        // Prestige calculé : niveau 1 + modificateur de CHA (+3) = 4.
        assert_eq!(c.etat(&r).prestige_effectif(pj), Some(4));
        // Une valeur saisie l'emporte.
        c.executer(
            Evenement::PersonnageModifie {
                id: pj,
                nom: "Lyra".into(),
                prestige: Some(9),
            },
            &r,
        )
        .unwrap();
        assert_eq!(c.etat(&r).prestige_effectif(pj), Some(9));
        // XP, objet, état : tout passe par la campagne et s'annule avec elle.
        c.executer(
            fiche_ev(
                pj,
                EvenementFiche::XpGagnes {
                    nombre: 2000,
                    motif: "dragon".into(),
                },
            ),
            &r,
        )
        .unwrap();
        assert_eq!(c.etat(&r).fiche_de(pj).unwrap().xp, 2000);
        assert_eq!(c.etat(&r).fiche_de(pj).unwrap().niveau_atteignable, 2);
        c.executer(
            fiche_ev(pj, EvenementFiche::EtatAjoute("shaken".into())),
            &r,
        )
        .unwrap();
        assert!(c
            .executer(fiche_ev(pj, EvenementFiche::EtatAjoute("bidon".into())), &r)
            .is_err());
        c.annuler();
        assert!(c.etat(&r).fiche_de(pj).unwrap().fiche.etats.is_empty());
    }

    #[test]
    fn les_reglages_changent_la_progression_et_le_temps_use_les_effets() {
        let (mut c, r, pj) = campagne_avec_pj();
        c.executer(
            fiche_ev(
                pj,
                EvenementFiche::Creation(Box::new(fiches::exemple::elfe_ensorceleuse())),
            ),
            &r,
        )
        .unwrap();
        // 2000 XP : niveau 2 à vitesse moyenne ; à vitesse lente, niveau 1 seulement.
        let xp = |c: &mut Campagne| {
            c.executer(
                fiche_ev(
                    pj,
                    EvenementFiche::XpGagnes {
                        nombre: 2000,
                        motif: String::new(),
                    },
                ),
                &r,
            )
            .unwrap()
        };
        xp(&mut c);
        assert_eq!(c.etat(&r).fiche_de(pj).unwrap().niveau_atteignable, 2);
        let p = ReglesPersonnages {
            vitesse: Vitesse::Lente,
            ..Default::default()
        };
        c.executer(Evenement::ReglagePersonnages { regles: p }, &r)
            .unwrap();
        assert_eq!(c.etat(&r).fiche_de(pj).unwrap().niveau_atteignable, 1);
        // Un effet de 2 jours disparaît après 3 jours de campagne ; un autre, sans durée, reste.
        for (nom, duree) in [("Aide", Some(2 * ROUNDS_PAR_JOUR)), ("Bénédiction", None)] {
            c.executer(
                fiche_ev(
                    pj,
                    EvenementFiche::EffetTemporaireAjoute(EffetTemporaire {
                        nom: nom.into(),
                        effets: vec![Effet::new("attaque", TypeBonus::Moral, 1, nom)],
                        rounds_restants: duree,
                    }),
                ),
                &r,
            )
            .unwrap();
        }
        c.executer(Evenement::JoursAvances { nombre: 3 }, &r)
            .unwrap();
        let t = c.etat(&r).fiche_de(pj).unwrap().fiche.temporaires;
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].nom, "Bénédiction");
    }

    #[test]
    fn le_registre_decrit_et_modifie_les_regles() {
        let o = OptionsRegles::default();
        let p = ReglesPersonnages::default();
        let reg = registre(&o, &p);
        assert!(reg.iter().all(|x| {
            x.choix.iter().any(|(v, _)| *v == x.defaut)
                && x.choix.iter().any(|(v, _)| *v == x.valeur)
                && !x.description.is_empty()
                && !x.impact.is_empty()
        }));
        // Chaque valeur proposée est acceptée en retour.
        for x in &reg {
            for (v, _) in &x.choix {
                assert!(changer(x.id, v, &o, &p).is_ok(), "{} = {v}", x.id);
            }
        }
        assert!(changer("pj.vitesse", "turbo", &o, &p).is_err());
        assert!(changer("inconnue", "oui", &o, &p).is_err());
        match changer("pj.generation", "reserve_28", &o, &p).unwrap() {
            Reglage::Personnages(n) => {
                assert_eq!(n.generation, Methode::ReserveDeDes { des: 28 })
            }
            _ => panic!(),
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn une_campagne_avec_fiche_survit_a_la_sauvegarde() {
        let (mut c, r, pj) = campagne_avec_pj();
        c.executer(
            fiche_ev(
                pj,
                EvenementFiche::Creation(Box::new(fiches::exemple::elfe_ensorceleuse())),
            ),
            &r,
        )
        .unwrap();
        let json = crate::fichier::vers_json(&c).unwrap();
        let mut c2 = crate::fichier::depuis_json(&json).unwrap();
        assert_eq!(c2.etat(&r).fiche_de(pj), c.etat(&r).fiche_de(pj));
        assert!(c2.etat(&r).fiche_de(pj).is_some());
    }
}
