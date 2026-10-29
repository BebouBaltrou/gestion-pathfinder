//! Modèles d'affichage des personnages : liste, fiche complète, catalogue de création.
//! Tout est calculé par le cœur (`fiches`) ; l'interface n'a rien à recalculer.

use crate::Somme;
use campagne::etat::Etat;
use campagne::modele::IdPersonnage;
use fiches::fiche::{Fiche, Probleme};
use fiches::journal::EtatPersonnage;
use fiches::progression::xp_requis;
use fiches::traductions::Traductions;
use fiches::{Carac, Detail, Referentiel};

macro_rules! vue {
    ($(#[$m:meta])* $v:vis struct $n:ident { $($(#[$fm:meta])* $fv:vis $f:ident : $t:ty),* $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize))]
        #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
        $v struct $n { $($(#[$fm])* $fv $f : $t),* }
    };
}

vue! {
    /// Un nombre et le détail de son calcul.
    pub struct ValeurVue {
        pub total: i32,
        pub explication: String,
        pub termes: Vec<TermeVue>,
    }
}
vue! {
    pub struct TermeVue {
        pub libelle: String,
        pub valeur: i32,
        pub compte: bool,
    }
}

fn valeur(d: &Detail) -> ValeurVue {
    ValeurVue {
        total: d.total,
        explication: d.explication(),
        termes: d
            .termes
            .iter()
            .map(|t| TermeVue {
                libelle: t.libelle.clone(),
                valeur: t.valeur,
                compte: t.compte,
            })
            .collect(),
    }
}

vue! {
    pub struct ResumeFiche {
        pub race: String,
        pub classes: String,
        pub niveau: u32,
        pub pv: i32,
        pub ca: i32,
        pub xp: u32,
        pub niveau_atteignable: u32,
        pub niveaux_en_attente: u32,
    }
}
vue! {
    pub struct LignePersonnage {
        pub id: u32,
        pub nom: String,
        pub compte: Somme,
        pub lieu: Option<String>,
        /// Prestige saisi, ou niveau + modificateur de Charisme de la fiche.
        pub prestige: Option<i32>,
        pub prestige_calcule: bool,
        /// Nom du maître, si ce personnage est l'acolyte d'un autre (don Prestige).
        pub acolyte_de: Option<String>,
        pub fiche: Option<ResumeFiche>,
    }
}

fn classes_texte(r: &Referentiel, f: &Fiche) -> String {
    let mut vues: Vec<&str> = Vec::new();
    for n in &f.niveaux {
        if !vues.contains(&n.classe.as_str()) {
            vues.push(&n.classe);
        }
    }
    vues.iter()
        .map(|id| {
            let nom = r
                .classe(id)
                .map(|c| c.nom.clone())
                .unwrap_or(id.to_string());
            format!("{nom} {}", f.niveau_de_classe(id))
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn resume(r: &Referentiel, e: &EtatPersonnage) -> Option<ResumeFiche> {
    let v = e.fiche.calculer(r).ok()?;
    Some(ResumeFiche {
        race: r
            .race(&e.fiche.race)
            .map(|x| x.nom.clone())
            .unwrap_or_default(),
        classes: classes_texte(r, &e.fiche),
        niveau: v.niveau_de_personnage,
        pv: v.pv_max.total,
        ca: v.ca.total,
        xp: e.xp,
        niveau_atteignable: e.niveau_atteignable,
        niveaux_en_attente: e.niveaux_en_attente(),
    })
}

/// Liste des PJ avec le résumé de leur fiche.
pub fn liste_personnages(etat: &Etat) -> Vec<LignePersonnage> {
    let r = Referentiel::partage();
    etat.personnages
        .values()
        .map(|p| {
            let e = etat.fiche_de(p.id);
            LignePersonnage {
                id: p.id.0,
                nom: p.nom.clone(),
                compte: p.compte.into(),
                lieu: p
                    .localite
                    .and_then(|l| etat.localites.get(&l))
                    .map(|l| l.nom.clone()),
                prestige: etat.prestige_effectif(p.id),
                prestige_calcule: p.prestige.is_none() && e.is_some(),
                acolyte_de: etat
                    .acolytes
                    .get(&p.id)
                    .and_then(|m| etat.personnages.get(m))
                    .map(|m| m.nom.clone()),
                fiche: e.as_ref().and_then(|e| resume(r, e)),
            }
        })
        .collect()
}

// ------------------------------------------------------------------ fiche complète

vue! {
    pub struct CaracVue {
        pub id: String,
        pub nom: String,
        pub score: ValeurVue,
        pub modificateur: i32,
    }
}
vue! {
    pub struct CompetenceVue {
        pub cle: String,
        pub nom: String,
        pub valeur: ValeurVue,
        pub rangs: u32,
        pub de_classe: bool,
        pub formation_requise: bool,
        pub caracteristique: String,
    }
}
vue! {
    pub struct DonVue {
        pub nom_en: String,
        pub nom: String,
        pub resume: Option<String>,
        pub avantage: Option<String>,
        pub source: Option<String>,
        pub note_livre: Option<String>,
        /// Prérequis non remplis : le don est inutilisable (Core p. 111).
        pub inutilisable: Option<String>,
    }
}
vue! {
    pub struct TexteVue {
        pub nom: String,
        pub texte: String,
    }
}
vue! {
    pub struct EmplacementVue {
        pub niveau_de_sort: u32,
        pub base: Option<u32>,
        pub bonus: u32,
        pub par_jour: Option<u32>,
        pub restants: Option<u32>,
        pub connus: u32,
        pub dd: i32,
    }
}
vue! {
    pub struct RemplacementSortVue {
        pub ancien_en: String,
        pub ancien: String,
        pub niveau_de_sort: u32,
        pub propositions: Vec<SortProposeVue>,
    }
}
vue! {
    pub struct SortProposeVue {
        pub nom_en: String,
        pub nom: String,
    }
}
vue! {
    pub struct SortVue {
        pub nom_en: String,
        pub nom: String,
        pub niveau: u32,
        pub ecole: String,
        /// Obtenu automatiquement par le lignage.
        pub de_lignage: bool,
        /// Statistiques du sort au niveau de lanceur du personnage (affichées au clic).
        pub stats: Option<StatsSortFicheVue>,
    }
}
vue! {
    pub struct StatsSortFicheVue {
        pub niveau_de_lanceur: u32,
        pub temps_d_incantation: Option<String>,
        pub composantes: Option<String>,
        pub portee: Option<String>,
        pub portee_calculee: Option<String>,
        pub cible: Option<String>,
        pub duree: Option<String>,
        pub duree_calculee: Option<String>,
        pub jet_de_sauvegarde: Option<String>,
        /// DD du jet de sauvegarde pour ce niveau de sort.
        pub dd: Option<i32>,
        pub resistance_a_la_magie: Option<String>,
        pub nombre: Option<String>,
        pub degats: Vec<crate::aventure::DegatsSortVue>,
        pub description: String,
        pub source: Option<String>,
        pub note_livre: Option<String>,
    }
}
vue! {
    pub struct IncantateurVue {
        pub classe: String,
        pub niveau_de_lanceur: u32,
        pub emplacements: Vec<EmplacementVue>,
        pub sorts: Vec<SortVue>,
    }
}
vue! {
    pub struct ArmeVue {
        pub nom_en: String,
        pub nom: String,
        pub maitrise: bool,
        pub attaque: ValeurVue,
        pub degats: String,
        pub critique: Option<String>,
        pub portee: Option<String>,
        pub type_degats: Option<String>,
    }
}
vue! {
    pub struct ObjetVue {
        pub nom_en: String,
        pub nom: String,
        pub quantite: u32,
    }
}
vue! {
    /// Bonus d'altération (magique) d'une armure, d'un bouclier ou d'une arme portés.
    pub struct BonusMagiqueVue {
        pub nom_en: String,
        pub bonus: i32,
    }
}
vue! {
    /// Objet magique porté, avec ses effets saisis par le joueur.
    pub struct ObjetMagiqueVue {
        pub nom: String,
        pub effets: Vec<String>,
    }
}
vue! {
    pub struct EquipementVue {
        pub bonus_magiques: Vec<BonusMagiqueVue>,
        pub objets_magiques: Vec<ObjetMagiqueVue>,
        pub armure: Option<ObjetVue>,
        pub bouclier: Option<ObjetVue>,
        pub armes: Vec<ArmeVue>,
        pub objets: Vec<ObjetVue>,
        pub poids_total: f64,
        pub poids_inconnu: Vec<String>,
        pub charge: String,
        /// Limites légère / intermédiaire / lourde de la Force actuelle, en livres.
        pub limites: Option<(f64, f64, f64)>,
        pub dex_max: Option<String>,
        pub malus_d_armure: i32,
        pub source_malus: String,
        pub echec_sorts_pct: u32,
        pub non_maitrises: Vec<String>,
    }
}
vue! {
    pub struct EtatVue {
        pub id: String,
        pub nom: String,
        pub texte: String,
        pub note: Option<String>,
    }
}
vue! {
    pub struct TemporaireVue {
        pub nom: String,
        pub rounds_restants: Option<u32>,
        pub effets: Vec<String>,
    }
}
vue! {
    pub struct IdentiteVue {
        pub nom: String,
        pub race: String,
        pub classes: String,
        pub lignage: Option<String>,
        pub niveau: u32,
        pub xp: u32,
        /// XP du prochain niveau (absent aux jalons ou au niveau 20).
        pub xp_suivant: Option<u32>,
        pub niveau_atteignable: u32,
        pub niveaux_en_attente: u32,
        pub prestige: Option<i32>,
        pub prestige_calcule: bool,
        pub compte: Somme,
    }
}
vue! {
    pub struct PouvoirDomaineVue {
        /// Identifiant du pouvoir limité par jour (vide pour le catalogue).
        pub cle: String,
        pub nom: String,
        pub genre: String,
        pub niveau: u32,
        pub actif: bool,
        /// Utilisations par jour, quand le livre en donne.
        pub utilisations: Option<i32>,
        pub texte: String,
    }
}
vue! {
    pub struct SortDomaineVue {
        pub niveau: u32,
        pub nom: String,
        pub nom_en: Option<String>,
        pub note: Option<String>,
        pub accessible: bool,
    }
}
vue! {
    pub struct DomaineVue {
        pub id: String,
        pub nom: String,
        pub description: String,
        pub pouvoirs: Vec<PouvoirDomaineVue>,
        pub sorts: Vec<SortDomaineVue>,
    }
}
vue! {
    pub struct CanalisationVue {
        /// Some(true) positive, Some(false) négative, None : à choisir.
        pub positive: Option<bool>,
        pub des: u32,
        pub dd: i32,
        pub utilisations: i32,
    }
}
vue! {
    /// Foi du personnage : dieu, alignement, domaines, énergie canalisée.
    pub struct DivinVue {
        pub divinite: Option<String>,
        pub alignement: Option<String>,
        pub arme_de_predilection: Option<String>,
        pub domaines: Vec<DomaineVue>,
        pub canalisation: Option<CanalisationVue>,
        /// Dés d'attaque sournoise du roublard (0 hors roublard).
        pub attaque_sournoise_des: u32,
        /// Prêtre Neutre : l'énergie canalisée reste à choisir.
        pub energie_a_choisir: bool,
    }
}
vue! {
    pub struct SortPrepareVue {
        pub niveau: u32,
        pub nom_en: String,
        pub nom: String,
        pub domaine: bool,
        pub utilise: bool,
    }
}
vue! {
    pub struct CapacitePreparation {
        pub niveau: u32,
        /// Sorts ordinaires préparables (oraisons au niveau 0).
        pub normaux: u32,
        /// Emplacements de sort de domaine (0 ou 1).
        pub domaine: u32,
        pub restants: Option<u32>,
    }
}
vue! {
    /// Sorts préparés d'un lanceur qui prépare (prêtre, druide).
    pub struct PreparationVue {
        pub classe_id: String,
        pub classe: String,
        pub capacites: Vec<CapacitePreparation>,
        pub prepares: Vec<SortPrepareVue>,
        /// Sorts de domaine permis (niveaux accessibles seulement).
        pub sorts_de_domaine: Vec<SortDomaineVue>,
        /// Descripteurs d'alignement interdits (good, evil, lawful, chaotic).
        pub interdits: Vec<String>,
        /// Conversion spontanée possible : Some(true) en soins, Some(false) en blessures.
        pub conversion: Option<bool>,
    }
}
vue! {
    /// Un pouvoir limité par jour et ce qu'il en reste.
    pub struct PouvoirJournalierVue {
        pub cle: String,
        pub nom: String,
        /// `None` : à volonté.
        pub max: Option<i32>,
        pub restants: Option<i32>,
    }
}
vue! {
    pub struct FormeVue {
        pub id: String,
        pub nom: String,
        pub categorie: String,
        pub taille: Option<String>,
        pub niveau: u32,
        /// Le niveau de druide suffit.
        pub disponible: bool,
        pub effets: Vec<String>,
        pub texte: String,
    }
}
vue! {
    /// Forme animale du druide.
    pub struct FormesVue {
        pub formes: Vec<FormeVue>,
        /// Nom de la forme prise en ce moment.
        pub en_cours: Option<String>,
        pub rounds_restants: Option<u32>,
        /// Durée de chaque transformation, en heures (1 par niveau de druide).
        pub duree_heures: u32,
    }
}
vue! {
    pub struct CompagnonVue {
        pub animal_id: String,
        pub animal: String,
        pub nom: String,
        pub taille: String,
        pub vitesse: String,
        pub dv: u32,
        pub pv: i32,
        pub bba: i32,
        pub caracteristiques: Vec<CaracVue>,
        pub armure_naturelle: i32,
        pub ca: i32,
        pub ca_contact: i32,
        pub ca_depourvu: i32,
        pub reflexes: i32,
        pub vigueur: i32,
        pub volonte: i32,
        pub attaques: String,
        pub bonus_attaque: i32,
        pub competences: u32,
        pub dons: u32,
        pub tours: u32,
        pub capacites: Vec<TexteVue>,
        pub particularites: String,
        pub developpement_applique: bool,
        pub ignore_developpement: bool,
        pub augmentations_permises: u32,
        pub augmentations: Vec<String>,
    }
}
vue! {
    pub struct FicheVue {
        pub identite: IdentiteVue,
        pub caracteristiques: Vec<CaracVue>,
        pub pv: ValeurVue,
        pub ca: ValeurVue,
        pub ca_contact: ValeurVue,
        pub ca_depourvu: ValeurVue,
        pub initiative: ValeurVue,
        pub vigueur: ValeurVue,
        pub reflexes: ValeurVue,
        pub volonte: ValeurVue,
        pub bba: ValeurVue,
        pub attaques_iteratives: Vec<i32>,
        pub bmo: ValeurVue,
        pub dmd: ValeurVue,
        pub vitesse: u32,
        pub competences: Vec<CompetenceVue>,
        pub rangs_par_niveau: Vec<(u32, u32, u32)>,
        pub dons: Vec<DonVue>,
        /// Dons choisis de chaque niveau (noms anglais), pour les réattribuer niveau par niveau.
        pub dons_par_niveau: Vec<Vec<String>>,
        /// Style de combat du rôdeur (« tir_a_distance » ou « deux_armes »), absent tant qu'il n'est pas choisi.
        pub style_de_combat: Option<String>,
        /// Grâces, talents et ennemis jurés : pris et à prendre.
        pub choix_classes: ChoixClassesVue,
        /// Clé anglaise minuscule de la classe de chaque niveau, pour filtrer les dons par classe.
        pub classes_par_niveau: Vec<String>,
        /// Sorts connus choisis (noms anglais), pour les réattribuer.
        pub sorts_connus: Vec<String>,
        pub traits: Vec<TexteVue>,
        pub capacites: Vec<TexteVue>,
        pub divin: Option<DivinVue>,
        pub pouvoirs: Vec<PouvoirJournalierVue>,
        pub formes: Option<FormesVue>,
        /// Pacte avec la nature du druide : domaine ou compagnon.
        pub pacte: Option<String>,
        pub compagnon: Option<CompagnonVue>,
        pub preparation: Option<PreparationVue>,
        pub incantateurs: Vec<IncantateurVue>,
        pub equipement: EquipementVue,
        pub etats: Vec<EtatVue>,
        pub temporaires: Vec<TemporaireVue>,
        pub notes_etats: Vec<String>,
        /// Ce qui rend la fiche invalide ou incomplète.
        pub problemes: Vec<String>,
        /// Sorts connus à remplacer (doublon ou sort déjà accordé par le lignage), avec propositions.
        pub remplacements_de_sorts: Vec<RemplacementSortVue>,
        pub historique: Vec<String>,
    }
}

fn nom_objet(r: &Referentiel, nom_en: &str) -> String {
    let c = &r.equipement;
    c.armure(nom_en)
        .map(|a| a.nom.clone())
        .or_else(|| c.arme(nom_en).map(|a| a.nom.clone()))
        .or_else(|| c.objet(nom_en).map(|a| a.nom.clone()))
        .unwrap_or_else(|| nom_en.to_string())
}

fn objet_vue(r: &Referentiel, nom_en: &str, quantite: u32) -> ObjetVue {
    ObjetVue {
        nom_en: nom_en.to_string(),
        nom: nom_objet(r, nom_en),
        quantite,
    }
}

/// Construit la fiche complète d'un personnage à partir de son état.
/// Choix génériques des classes : ce qui est pris et ce qui reste à prendre.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct ChoixClassesVue {
    /// Nombre de grâces attendues au niveau de paladin actuel.
    pub graces_attendues: u32,
    pub graces: Vec<String>,
    /// Grâces accessibles au niveau de paladin actuel, pas encore prises.
    pub graces_disponibles: Vec<String>,
    pub talents: Vec<String>,
    /// Talents de roublard et de maître pas encore pris.
    pub talents_disponibles: Vec<String>,
    /// Texte du livre de chaque talent (nom sans astérisque).
    pub talents_textes: std::collections::BTreeMap<String, String>,
    /// Nombre de talents de roublard attendus au niveau actuel (dont des talents de maître dès le niveau 10).
    pub talents_attendus: u32,
    /// Nombre d'ennemis jurés attendus au niveau de rôdeur actuel.
    pub ennemis_attendus: u32,
    pub ennemis_jures: Vec<String>,
    pub types_ennemis: Vec<String>,
    /// Augmentations de +2 attendues (une de moins que de choix d’ennemi) et déjà choisies.
    pub augmentations_attendues: u32,
    pub augmentations_ennemis: Vec<String>,
    /// Nombre de pouvoirs de rage attendus au niveau de barbare actuel.
    pub rage_attendus: u32,
    pub rage_choisis: Vec<PouvoirRageVue>,
    /// Pouvoirs de rage accessibles et pas encore pris. `mjra` : source APG (règles avancées).
    pub rage_disponibles: Vec<PouvoirRageVue>,
    /// Niveau de barbare (0 hors barbare) et rage en cours.
    pub niveau_barbare: u32,
    pub en_rage: bool,
    /// Valeurs chiffrées des capacités conditionnelles (inspirations, courage).
    pub capacites_chiffrees: Vec<String>,
    /// Rounds passés dans la rage en cours et rounds de fatigue restants après une rage.
    pub rounds_de_rage: u32,
    pub fatigue_rage: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PouvoirRageVue {
    pub nom: String,
    pub mjra: bool,
    /// Texte du livre (vide s'il n'est pas saisi).
    pub texte: String,
}

fn pouvoir_rage_vue(rage: &fiches::choix_classe::PouvoirsRage, nom: &str) -> PouvoirRageVue {
    PouvoirRageVue {
        nom: nom.to_string(),
        mjra: rage.sources.get(nom).is_some_and(|s| s == "APG"),
        texte: rage.textes.get(nom).cloned().unwrap_or_default(),
    }
}

fn choix_classes(r: &Referentiel, f: &Fiche) -> ChoixClassesVue {
    let c = &r.choix;
    let niveau_paladin = f.niveau_de_classe("paladin");
    let t = &c.talents_roublard;
    let graces_disponibles = c
        .grace_paladin
        .listes
        .iter()
        .filter(|l| l.a_partir_du <= niveau_paladin)
        .flat_map(|l| l.graces.iter())
        .filter(|g| !f.graces.contains(g))
        .cloned()
        .collect();
    let talents_disponibles = t
        .talents
        .iter()
        .chain(&t.talents_de_maitre)
        .filter(|x| !f.talents_roublard.iter().any(|p| p == *x))
        .cloned()
        .collect();
    let niveau_barbare = f.niveau_de_classe("barbare");
    let rage = &r.rage;
    let rage_disponibles = rage
        .niveau_de_table
        .keys()
        .filter(|n| rage.encore_prenable(n, &f.pouvoirs_rage))
        .filter(|n| rage.accessible(n, niveau_barbare, &f.pouvoirs_rage))
        .map(|n| pouvoir_rage_vue(rage, n))
        .collect();
    ChoixClassesVue {
        rage_attendus: fiches::choix_classe::PouvoirsRage::attendus(niveau_barbare),
        niveau_barbare,
        en_rage: f.en_rage,
        rounds_de_rage: f.rounds_de_rage,
        fatigue_rage: f.fatigue_rage,
        capacites_chiffrees: fiches::choix_classe::capacites_chiffrees(f),
        rage_choisis: f
            .pouvoirs_rage
            .iter()
            .map(|n| pouvoir_rage_vue(rage, n))
            .collect(),
        rage_disponibles,
        graces_attendues: c.nombre_graces(niveau_paladin),
        graces: f.graces.clone(),
        graces_disponibles,
        talents: f.talents_roublard.clone(),
        talents_disponibles,
        talents_textes: c.talents_roublard.textes.clone(),
        talents_attendus: c.nombre_talents(f.niveau_de_classe("roublard")),
        ennemis_attendus: c.nombre_ennemis_jures(f.niveau_de_classe("rodeur")),
        ennemis_jures: f.ennemis_jures.clone(),
        types_ennemis: c.ennemis_jures_rodeur.types.clone(),
        augmentations_attendues: c
            .nombre_ennemis_jures(f.niveau_de_classe("rodeur"))
            .saturating_sub(1),
        augmentations_ennemis: f.augmentations_ennemis.clone(),
    }
}

pub fn fiche_complete(
    e: &EtatPersonnage,
    historique: Vec<String>,
    identite: (String, Somme, Option<i32>, bool),
    problemes_supplementaires: Vec<String>,
) -> Result<FicheVue, String> {
    let r = Referentiel::partage();
    let t = Traductions::partage();
    let f = &e.fiche;
    let v = f.calculer(r).map_err(|p| p.to_string())?;
    let mut problemes: Vec<String> = f.valider(r).iter().map(Probleme::to_string).collect();
    problemes.extend(problemes_supplementaires);
    let (nom_pj, compte, prestige, prestige_calcule) = identite;
    let nom_sort = |nom_en: &str| {
        t.sort(nom_en)
            .map(|s| s.nom.clone())
            .unwrap_or_else(|| nom_en.to_string())
    };
    let remplacements: Vec<RemplacementSortVue> = f
        .remplacements_de_sorts(r)
        .into_iter()
        .map(|x| RemplacementSortVue {
            ancien: nom_sort(&x.ancien),
            ancien_en: x.ancien,
            niveau_de_sort: x.niveau_de_sort,
            propositions: x
                .propositions
                .into_iter()
                .map(|p| SortProposeVue {
                    nom: nom_sort(&p),
                    nom_en: p,
                })
                .collect(),
        })
        .collect();

    let classe_principale = f.niveaux.first().and_then(|n| r.classe(&n.classe));
    let lignage = f.lignage.as_ref().and_then(|id| {
        t.lignage(id)
            .map(|l| l.nom.to_string())
            .or_else(|| Some(id.clone()))
    });

    let caracteristiques = Carac::TOUTES
        .into_iter()
        .map(|c| CaracVue {
            id: c.abrege().to_lowercase(),
            nom: nom_carac(c).to_string(),
            score: valeur(&v.scores[&c]),
            modificateur: v.modificateurs[&c],
        })
        .collect();

    // Compétences : toutes, avec leur détail.
    let de_classe_ids: Vec<String> = f.competences_de_classe(r);
    let mut competences: Vec<CompetenceVue> = v
        .competences
        .iter()
        .map(|(cle, d)| {
            let base = cle.split(':').next().unwrap_or(cle);
            let def = r.competence(base);
            let rangs: u32 = f.niveaux.iter().filter_map(|n| n.rangs.get(cle)).sum();
            let nom_en = def.map(|x| x.nom_en.as_str()).unwrap_or(base);
            CompetenceVue {
                cle: cle.clone(),
                nom: t.competence_avec_sous_type(cle).unwrap_or(cle.clone()),
                valeur: valeur(d),
                rangs,
                de_classe: de_classe_ids
                    .iter()
                    .any(|c| c.to_lowercase().starts_with(&nom_en.to_lowercase())),
                formation_requise: def.is_some_and(|x| x.formation_requise),
                caracteristique: def.map(|x| x.caracteristique.clone()).unwrap_or_default(),
            }
        })
        .collect();
    competences.sort_by_key(|c| c.nom.to_lowercase());

    // Dons : automatiques puis choisis, avec prérequis non remplis.
    let invalides = fiches::prerequis::dons_invalides(r, f, &v);
    let mut dons = Vec::new();
    for d in f.dons_automatiques(r).into_iter().chain(f.dons_choisis()) {
        let fr = t.don(&d);
        let nom = match d.split_once(" (") {
            Some((_, choix)) => format!("{} ({}", fr.map(|x| x.nom.as_str()).unwrap_or(&d), choix),
            None => fr.map(|x| x.nom.clone()).unwrap_or(d.clone()),
        };
        let inutilisable = invalides.iter().find(|(n, _)| *n == d).map(|(_, ver)| {
            ver.details
                .iter()
                .filter_map(|(txt, s)| match s {
                    fiches::prerequis::Statut::Manquant(m) => Some(format!("{txt} : {m}")),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ; ")
        });
        dons.push(DonVue {
            nom_en: d.clone(),
            nom,
            resume: fr.and_then(|x| x.resume.clone()),
            avantage: fr.and_then(|x| x.avantage.clone()),
            source: t.source_vf("dons", &d),
            note_livre: t.note_livre("dons", &d),
            inutilisable,
        });
    }

    // Traits raciaux et capacités de classe.
    let traits = r
        .race(&f.race)
        .map(|race| {
            race.traits
                .iter()
                .map(|tr| match t.trait_racial(&f.race, &tr.nom_en) {
                    Some((n, x)) => TexteVue {
                        nom: n.to_string(),
                        texte: x.to_string(),
                    },
                    None => TexteVue {
                        nom: tr.nom_en.clone(),
                        texte: tr.texte_en.clone(),
                    },
                })
                .collect()
        })
        .unwrap_or_default();
    let mut capacites = Vec::new();
    if let Some(c) = classe_principale {
        let nc = f.niveau_de_classe(&c.id) as usize;
        // Le texte du livre n'est donné qu'à la première apparition d'une capacité qui progresse.
        let mut deja = std::collections::HashSet::new();
        for l in c.progression.iter().take(nc) {
            for cap in t.capacites_de_niveau(&l.special_en) {
                // Texte du livre quand la classe en donne un (« forme animale (2/jour) » → « forme animale »).
                let cle = cle_capacite(&cap);
                let mut texte = match c.textes_capacites.get(&cle) {
                    Some(desc) if deja.insert(cle.clone()) => format!("{cap} — {desc}"),
                    _ => cap,
                };
                // Empathie sauvage : 1d20 + niveau de druide + modificateur de Charisme.
                if cle == "empathie sauvage" {
                    texte.push_str(&format!(
                        " Bonus actuel : {:+} (niveau {} + modificateur de Charisme).",
                        nc as i32 + v.modificateurs[&Carac::Cha],
                        nc
                    ));
                }
                capacites.push(TexteVue {
                    nom: format!("Niveau {}", l.niveau),
                    texte,
                });
            }
        }
    }

    // Sorts.
    let restants = e.emplacements_restants(r);
    let mut incantateurs = Vec::new();
    for inc in &v.incantateurs {
        let classe = r.classe(&inc.classe);
        let cle = classe.map(|c| c.nom_en.to_lowercase()).unwrap_or_default();
        let mut sorts: Vec<SortVue> = Vec::new();
        let mut ajouter = |nom_en: &str, de_lignage: bool| {
            let def = r.sort(nom_en).or_else(|| r.sort_approx(nom_en));
            let fr = def.and_then(|d| t.sort(&d.nom_en));
            sorts.push(SortVue {
                nom_en: def.map(|d| d.nom_en.clone()).unwrap_or(nom_en.to_string()),
                nom: fr.map(|s| s.nom.clone()).unwrap_or(nom_en.to_string()),
                niveau: def.and_then(|d| d.niveau_pour(&cle)).unwrap_or(0),
                ecole: fr.map(|s| s.ecole.clone()).unwrap_or_default(),
                de_lignage,
                stats: def.map(|d| {
                    let cl = inc.niveau_de_lanceur;
                    let st = combat::stats_sorts::statistiques(d, cl);
                    let niveau = d.niveau_pour(&cle).unwrap_or(0);
                    StatsSortFicheVue {
                        niveau_de_lanceur: cl,
                        temps_d_incantation: fr.and_then(|s| s.temps_d_incantation.clone()),
                        composantes: fr.and_then(|s| s.composantes.clone()),
                        portee: fr.and_then(|s| s.portee.clone()),
                        portee_calculee: st.portee,
                        cible: fr.and_then(|s| s.cible.clone()),
                        duree: fr.and_then(|s| s.duree.clone()),
                        duree_calculee: d
                            .duree_en
                            .as_deref()
                            .and_then(|x| combat::sorts::duree_suggeree(x, cl))
                            .map(|x| x.texte()),
                        jet_de_sauvegarde: fr.and_then(|s| s.jet_de_sauvegarde.clone()),
                        dd: inc
                            .emplacements
                            .iter()
                            .find(|e| e.niveau_de_sort == niveau)
                            .map(|e| e.dd),
                        resistance_a_la_magie: fr.and_then(|s| s.resistance_a_la_magie.clone()),
                        nombre: st.nombre,
                        degats: st
                            .degats
                            .iter()
                            .map(|x| crate::aventure::DegatsSortVue {
                                libelle: x.libelle.clone(),
                                formule: x.formule.clone(),
                                regle: x.regle.clone(),
                                a_confirmer: x.a_confirmer,
                            })
                            .collect(),
                        description: fr.map(|s| s.description.clone()).unwrap_or_default(),
                        source: t.source_vf("sorts", &d.nom_en),
                        note_livre: t.note_livre("sorts", &d.nom_en),
                    }
                }),
            });
        };
        for s in &f.sorts_connus {
            ajouter(s, false);
        }
        for (s, _) in &inc.sorts_de_lignage {
            ajouter(s, true);
        }
        sorts.sort_by_key(|s| (s.niveau, s.nom.clone()));
        incantateurs.push(IncantateurVue {
            classe: classe.map(|c| c.nom.clone()).unwrap_or(inc.classe.clone()),
            niveau_de_lanceur: inc.niveau_de_lanceur,
            emplacements: inc
                .emplacements
                .iter()
                .map(|em| EmplacementVue {
                    niveau_de_sort: em.niveau_de_sort,
                    base: em.base,
                    bonus: em.bonus,
                    par_jour: em.par_jour,
                    restants: restants
                        .iter()
                        .find(|(c, n, _, _)| *c == inc.classe && *n == em.niveau_de_sort)
                        .map(|(_, _, rest, _)| *rest),
                    connus: em.connus,
                    dd: em.dd,
                })
                .collect(),
            sorts,
        });
    }

    // Foi : divinité, domaines, canalisation d'énergie.
    let divin = (classe_principale.is_some_and(|c| c.domaines_choisis > 0)
        || f.pacte.as_deref() == Some("domaine"))
    .then(|| {
        let dieu = f.divinite.as_deref().and_then(|d| r.divinite(d));
        DivinVue {
            divinite: dieu.map(|d| d.nom.clone()),
            alignement: f.alignement.clone(),
            arme_de_predilection: dieu.map(|d| d.arme_de_predilection.clone()),
            domaines: v
                .domaines
                .iter()
                .map(|d| DomaineVue {
                    id: d.id.clone(),
                    nom: d.nom.clone(),
                    description: r
                        .domaine(&d.id)
                        .map(|x| x.description.clone())
                        .unwrap_or_default(),
                    pouvoirs: d
                        .pouvoirs
                        .iter()
                        .map(|p| PouvoirDomaineVue {
                            cle: p.cle.clone(),
                            nom: p.nom.clone(),
                            genre: p.genre.clone(),
                            niveau: p.niveau,
                            actif: p.actif,
                            utilisations: p.utilisations,
                            texte: p.texte.clone(),
                        })
                        .collect(),
                    sorts: d
                        .sorts
                        .iter()
                        .map(|s| SortDomaineVue {
                            niveau: s.niveau,
                            nom: s.nom_fr.clone(),
                            nom_en: s.nom_en.clone(),
                            note: s.note.clone(),
                            accessible: s.accessible,
                        })
                        .collect(),
                })
                .collect(),
            attaque_sournoise_des: v.attaque_sournoise,
            canalisation: v.canalisation.as_ref().map(|c| CanalisationVue {
                positive: c.positive,
                des: c.des,
                dd: c.dd,
                utilisations: c.utilisations,
            }),
            energie_a_choisir: v
                .canalisation
                .as_ref()
                .is_some_and(|c| c.positive.is_none()),
        }
    });

    // Pouvoirs limités par jour, avec ce qu'il en reste.
    let pouvoirs: Vec<PouvoirJournalierVue> = v
        .pouvoirs
        .iter()
        .map(|p| PouvoirJournalierVue {
            cle: p.cle.clone(),
            nom: p.nom.clone(),
            max: p.max,
            restants: match &p.partage {
                // Réserve partagée : ce qui reste de la réserve, divisé par le coût d'un usage.
                Some((reserve, cout)) => v
                    .pouvoirs
                    .iter()
                    .find(|q| &q.cle == reserve)
                    .and_then(|q| q.max)
                    .map(|m| {
                        let pris = e.pouvoirs_utilises.get(reserve).copied().unwrap_or(0) as i32;
                        ((m - pris) / (*cout).max(1) as i32).max(0)
                    }),
                None => p.max.map(|m| {
                    (m - e.pouvoirs_utilises.get(&p.cle).copied().unwrap_or(0) as i32).max(0)
                }),
            },
        })
        .collect();

    // Forme animale du druide.
    let formes = classe_principale
        .filter(|c| c.pouvoirs.iter().any(|p| p.cle == "forme_animale"))
        .map(|c| {
            let nc = f.niveau_de_classe(&c.id);
            let en_cours = f
                .temporaires
                .iter()
                .find(|t| t.nom.starts_with(fiches::formes::PREFIXE_FORME));
            FormesVue {
                formes: r
                    .formes
                    .iter()
                    .map(|x| FormeVue {
                        id: x.id.clone(),
                        nom: x.nom.clone(),
                        categorie: x.categorie.clone(),
                        taille: x.taille.clone(),
                        niveau: x.niveau_druide,
                        disponible: nc >= x.niveau_druide,
                        effets: x
                            .effets
                            .iter()
                            .map(|e| {
                                let cible = match e.cible.as_str() {
                                    "carac.for" => "Force",
                                    "carac.dex" => "Dextérité",
                                    "carac.con" => "Constitution",
                                    "ca" => "CA",
                                    autre => autre,
                                };
                                format!("{:+} {} ({})", e.valeur, cible, e.type_bonus.nom())
                            })
                            .collect(),
                        texte: x.texte.clone(),
                    })
                    .collect(),
                en_cours: en_cours.map(|t| t.nom.clone()),
                rounds_restants: en_cours.and_then(|t| t.rounds_restants),
                duree_heures: nc,
            }
        });

    // Compagnon animal du druide.
    let compagnon = v.compagnon.as_ref().map(|c| {
        let choix = f.compagnon.as_ref();
        CompagnonVue {
            animal_id: choix.map(|x| x.animal.clone()).unwrap_or_default(),
            animal: c.animal.clone(),
            nom: c.nom.clone(),
            taille: c.taille.clone(),
            vitesse: c.vitesse.clone(),
            dv: c.dv,
            pv: c.pv,
            bba: c.bba,
            caracteristiques: c
                .caracteristiques
                .iter()
                .map(|(car, score, m)| CaracVue {
                    id: car.abrege().to_lowercase(),
                    nom: nom_carac(*car).to_string(),
                    score: ValeurVue {
                        total: *score,
                        explication: score.to_string(),
                        termes: vec![],
                    },
                    modificateur: *m,
                })
                .collect(),
            armure_naturelle: c.armure_naturelle,
            ca: c.ca,
            ca_contact: c.ca_contact,
            ca_depourvu: c.ca_depourvu,
            reflexes: c.reflexes,
            vigueur: c.vigueur,
            volonte: c.volonte,
            attaques: c.attaques.clone(),
            bonus_attaque: c.bonus_attaque,
            competences: c.competences,
            dons: c.dons,
            tours: c.tours,
            capacites: c
                .capacites
                .iter()
                .flat_map(|s| s.split(", ").map(str::to_string).collect::<Vec<_>>())
                .map(|nom| {
                    let cle = nom.to_lowercase();
                    TexteVue {
                        texte: r.compagnons.textes.get(&cle).cloned().unwrap_or_default(),
                        nom,
                    }
                })
                .collect(),
            particularites: c.particularites.clone(),
            developpement_applique: c.developpement_applique,
            ignore_developpement: choix.is_some_and(|x| x.ignore_developpement),
            augmentations_permises: c.augmentations_permises,
            augmentations: choix
                .map(|x| {
                    x.augmentations
                        .iter()
                        .map(|k| k.abrege().to_lowercase())
                        .collect()
                })
                .unwrap_or_default(),
        }
    });

    // Sorts préparés (lanceur qui prépare).
    let preparation = v
        .incantateurs
        .iter()
        .find(|inc| {
            r.classe(&inc.classe)
                .and_then(|c| c.incantation.as_ref())
                .is_some_and(|i| i.prepare())
        })
        .and_then(|inc| {
            let classe = r.classe(&inc.classe)?;
            let liste = e.prepares.get(&inc.classe).cloned().unwrap_or_default();
            let utilises = e.prepares_utilises.get(&inc.classe);
            Some(PreparationVue {
                classe_id: inc.classe.clone(),
                classe: classe.nom.clone(),
                capacites: inc
                    .emplacements
                    .iter()
                    .map(|em| CapacitePreparation {
                        niveau: em.niveau_de_sort,
                        normaux: if em.niveau_de_sort == 0 {
                            em.base.unwrap_or(0)
                        } else {
                            em.par_jour.unwrap_or(0)
                        },
                        domaine: em.domaine,
                        restants: restants
                            .iter()
                            .find(|(c, n, _, _)| *c == inc.classe && *n == em.niveau_de_sort)
                            .map(|(_, _, rest, _)| *rest),
                    })
                    .collect(),
                prepares: liste
                    .iter()
                    .enumerate()
                    .map(|(k, s)| SortPrepareVue {
                        niveau: s.niveau,
                        nom: t
                            .sort(&s.sort)
                            .map(|x| x.nom.clone())
                            .unwrap_or(s.sort.clone()),
                        nom_en: s.sort.clone(),
                        domaine: s.domaine,
                        utilise: utilises.and_then(|u| u.get(k)).copied().unwrap_or(false),
                    })
                    .collect(),
                sorts_de_domaine: v
                    .domaines
                    .iter()
                    .flat_map(|d| d.sorts.iter())
                    .filter(|s| s.accessible)
                    .map(|s| SortDomaineVue {
                        niveau: s.niveau,
                        nom: s.nom_fr.clone(),
                        nom_en: s.nom_en.clone(),
                        note: s.note.clone(),
                        accessible: true,
                    })
                    .collect(),
                interdits: fiches::preparation::descripteurs_interdits(f, r)
                    .into_iter()
                    .map(String::from)
                    .collect(),
                conversion: v.canalisation.as_ref().and_then(|c| c.positive),
            })
        });

    let eq = &f.equipement;
    let be = &v.equipement;
    let equipement = EquipementVue {
        bonus_magiques: eq
            .bonus_magiques
            .iter()
            .map(|(n, b)| BonusMagiqueVue {
                nom_en: n.clone(),
                bonus: *b,
            })
            .collect(),
        objets_magiques: eq
            .objets_magiques
            .iter()
            .map(|o| ObjetMagiqueVue {
                nom: o.nom.clone(),
                effets: o
                    .effets
                    .iter()
                    .map(|x| format!("{:+} {} ({})", x.valeur, x.cible.0, x.type_bonus.nom()))
                    .collect(),
            })
            .collect(),
        armure: eq.armure.as_deref().map(|n| objet_vue(r, n, 1)),
        bouclier: eq.bouclier.as_deref().map(|n| objet_vue(r, n, 1)),
        armes: v
            .armes
            .iter()
            .map(|a| ArmeVue {
                nom_en: a.arme_en.clone(),
                nom: a.nom.clone(),
                maitrise: a.maitrise,
                attaque: valeur(&a.attaque),
                degats: degats_texte(a),
                critique: a.critique.clone(),
                portee: a.portee.clone(),
                type_degats: a.type_degats.clone(),
            })
            .collect(),
        objets: eq
            .objets
            .iter()
            .map(|o| objet_vue(r, &o.nom_en, o.quantite))
            .collect(),
        poids_total: be.poids_total,
        poids_inconnu: be.poids_inconnu.clone(),
        charge: be.charge.nom().to_string(),
        limites: be.limites,
        dex_max: be.dex_max.as_ref().map(|(m, s)| format!("{m:+} ({s})")),
        malus_d_armure: be.malus_d_armure.0,
        source_malus: be.malus_d_armure.1.clone(),
        echec_sorts_pct: be.echec_sorts_pct,
        non_maitrises: be.non_maitrises.clone(),
    };

    let etats = f
        .etats
        .iter()
        .filter_map(|id| r.etat(id))
        .map(|s| EtatVue {
            id: s.id.clone(),
            nom: s.nom.clone(),
            texte: s.texte.clone(),
            note: s.note.clone(),
        })
        .collect();
    let temporaires = f
        .temporaires
        .iter()
        .map(|tmp| TemporaireVue {
            nom: tmp.nom.clone(),
            rounds_restants: tmp.rounds_restants,
            effets: tmp
                .effets
                .iter()
                .map(|x| format!("{:+} {} ({})", x.valeur, x.cible.0, x.type_bonus.nom()))
                .collect(),
        })
        .collect();

    let niveau = v.niveau_de_personnage;
    Ok(FicheVue {
        identite: IdentiteVue {
            nom: nom_pj,
            race: r.race(&f.race).map(|x| x.nom.clone()).unwrap_or_default(),
            classes: classes_texte(r, f),
            lignage,
            niveau,
            xp: e.xp,
            xp_suivant: None,
            niveau_atteignable: e.niveau_atteignable,
            niveaux_en_attente: e.niveaux_en_attente(),
            prestige,
            prestige_calcule,
            compte,
        },
        caracteristiques,
        pv: valeur(&v.pv_max),
        ca: valeur(&v.ca),
        ca_contact: valeur(&v.ca_contact),
        ca_depourvu: valeur(&v.ca_pris_au_depourvu),
        initiative: valeur(&v.initiative),
        vigueur: valeur(&v.vigueur),
        reflexes: valeur(&v.reflexes),
        volonte: valeur(&v.volonte),
        bba: valeur(&v.bba),
        attaques_iteratives: v.bba_iteratif.clone(),
        bmo: valeur(&v.bmo),
        dmd: valeur(&v.dmd),
        vitesse: v.vitesse,
        competences,
        rangs_par_niveau: v
            .rangs
            .iter()
            .map(|b| (b.niveau, b.places, b.disponibles))
            .collect(),
        dons,
        dons_par_niveau: f.niveaux.iter().map(|n| n.dons.clone()).collect(),
        style_de_combat: f.style_de_combat.clone(),
        choix_classes: choix_classes(r, f),
        classes_par_niveau: f
            .niveaux
            .iter()
            .map(|n| {
                r.classe(&n.classe)
                    .map(|c| c.nom_en.to_lowercase())
                    .unwrap_or_default()
            })
            .collect(),
        sorts_connus: f.sorts_connus.clone(),
        traits,
        capacites,
        divin,
        pouvoirs,
        formes,
        pacte: f.pacte.clone(),
        compagnon,
        preparation,
        incantateurs,
        equipement,
        etats,
        temporaires,
        notes_etats: v.notes_etats.clone(),
        remplacements_de_sorts: remplacements,
        problemes,
        historique,
    })
}

/// Complète l'identité avec les XP du prochain niveau selon la vitesse de progression.
pub fn avec_xp_suivant(mut fiche: FicheVue, vitesse: fiches::progression::Vitesse) -> FicheVue {
    let r = Referentiel::partage();
    fiche.identite.xp_suivant = xp_requis(r, vitesse, fiche.identite.niveau + 1);
    fiche
}

fn degats_texte(a: &fiches::fiche::AttaqueArme) -> String {
    match (&a.des, a.bonus_degats.total) {
        (Some(d), 0) => d.clone(),
        (Some(d), b) => format!("{d}{b:+}"),
        (None, _) => String::new(),
    }
}

fn nom_carac(c: Carac) -> &'static str {
    match c {
        Carac::For => "Force",
        Carac::Dex => "Dextérité",
        Carac::Con => "Constitution",
        Carac::Int => "Intelligence",
        Carac::Sag => "Sagesse",
        Carac::Cha => "Charisme",
    }
}

/// Fiche d'un PJ de la campagne, ou `None` s'il n'en a pas.
pub fn fiche_du_pj(etat: &Etat, id: IdPersonnage) -> Option<Result<FicheVue, String>> {
    let e = etat.fiche_de(id)?;
    let j = etat.fiches.get(&id)?;
    let p = etat.personnages.get(&id)?;
    let prestige = etat.prestige_effectif(id);
    let calcule = p.prestige.is_none();
    let historique = j.historique();
    let vitesse = etat.regles_personnages.vitesse;
    Some(
        fiche_complete(
            &e,
            historique,
            (p.nom.clone(), p.compte.into(), prestige, calcule),
            Vec::new(),
        )
        .map(|f| avec_xp_suivant(f, vitesse)),
    )
}

// ------------------------------------------------------------------ catalogue de création

vue! {
    pub struct RaceVue {
        pub id: String,
        pub nom: String,
        pub taille: String,
        pub vitesse: u32,
        /// Ajustements fixes : (caractéristique, valeur).
        pub ajustements: Vec<(String, i32)>,
        pub choix_libre: Option<i32>,
        pub bonus_don: u32,
        pub rang_supplementaire: u32,
        pub don_impose: Option<String>,
        pub deux_classes_de_predilection: bool,
        pub traits: Vec<TexteVue>,
    }
}
vue! {
    pub struct LignageVue {
        pub id: String,
        pub nom: String,
        pub competence_de_classe: String,
        pub arcane: String,
        /// Sous-type de Connaissances à choisir (lignage arcanique).
        pub choix_connaissance: bool,
        pub sorts: Vec<(String, u32)>,
        pub dons: Vec<String>,
    }
}
vue! {
    pub struct ClasseVue {
        pub id: String,
        pub nom: String,
        /// Clé anglaise minuscule (« fighter »), celle que citent les conditions de don.
        pub cle: String,
        pub de_vie: u32,
        pub rangs_par_niveau: i32,
        pub incantation: bool,
        pub caracteristique_incantation: Option<String>,
        pub lignages: Vec<LignageVue>,
        /// Compétences de classe (clés de compétence).
        pub competences_de_classe: Vec<String>,
        pub don_bonus_niveau_1: Option<String>,
        /// Sorts connus (niveau de sort 0 à 9) à chaque niveau de classe.
        pub sorts_connus: Vec<Vec<Option<u32>>>,
        /// Le lanceur choisit et prépare ses sorts chaque jour (prêtre, druide).
        pub prepare: bool,
        /// Nombre de domaines à choisir avec un dieu (prêtre : 2).
        pub domaines_choisis: u32,
        pub canalisation: bool,
        /// Domaines que le pacte avec la nature peut donner (druide), vide sinon.
        pub pacte_domaines: Vec<String>,
        /// Capacités du niveau 1.
        pub capacites: Vec<TexteVue>,
        /// Armes et armures maniées (texte du livre).
        pub maitrises: String,
    }
}
vue! {
    pub struct DomaineCatalogue {
        pub id: String,
        pub nom: String,
        pub description: String,
        pub divinites: Vec<String>,
        /// Bien, Mal, Loi ou Chaos : domaine réservé à un alignement correspondant.
        pub alignement: Option<String>,
        pub pouvoirs: Vec<PouvoirDomaineVue>,
        pub sorts: Vec<SortDomaineVue>,
    }
}
vue! {
    pub struct AnimalCatalogue {
        pub id: String,
        pub nom: String,
        pub taille: String,
        pub vitesse: String,
        pub armure_naturelle: i32,
        pub attaques: String,
        pub caracteristiques: String,
        pub particularites: String,
        pub developpement: String,
        pub niveau_developpement: u32,
    }
}
vue! {
    pub struct DiviniteCatalogue {
        pub id: String,
        pub nom: String,
        pub alignement: String,
        pub theme: String,
        pub domaines: Vec<String>,
        pub arme_de_predilection: String,
    }
}
vue! {
    pub struct CompetenceCatalogue {
        pub id: String,
        pub nom: String,
        pub caracteristique: String,
        pub formation_requise: bool,
        pub malus_d_armure: bool,
        pub sous_types: bool,
    }
}
vue! {
    pub struct DonCatalogue {
        pub nom_en: String,
        pub nom: String,
        pub resume: Option<String>,
        pub prerequis: Option<String>,
        pub repetable: bool,
        pub a_un_choix: bool,
        /// Classes exigées (clés anglaises minuscules). Vide : le don est ouvert à toutes les classes.
        pub classes_requises: Vec<String>,
    }
}
vue! {
    pub struct SortCatalogue {
        pub nom_en: String,
        pub nom: String,
        pub ecole: String,
        pub niveau: u32,
        pub resume: String,
        pub source: Option<String>,
        pub note_livre: Option<String>,
        /// Descripteurs en minuscules (good, evil, lawful, chaotic, fire…).
        pub descripteurs: Vec<String>,
    }
}
vue! {
    pub struct EquipementCatalogue {
        pub nom_en: String,
        pub nom: String,
        pub genre: String,
        pub poids_lb: Option<f64>,
        pub source: Option<String>,
        pub note_livre: Option<String>,
        pub description: Option<String>,
    }
}
vue! {
    pub struct EtatCatalogue {
        pub id: String,
        pub nom: String,
    }
}
vue! {
    pub struct CatalogueCreation {
        pub races: Vec<RaceVue>,
        pub classes: Vec<ClasseVue>,
        pub competences: Vec<CompetenceCatalogue>,
        pub dons: Vec<DonCatalogue>,
        /// Sorts de la liste de chaque classe lanceuse de sorts (identifiant de classe), par niveau.
        /// Un personnage ne peut choisir que dans la liste de sa classe.
        pub sorts_par_classe: std::collections::BTreeMap<String, Vec<SortCatalogue>>,
        pub domaines: Vec<DomaineCatalogue>,
        pub divinites: Vec<DiviniteCatalogue>,
        /// Animaux que le druide peut choisir comme compagnon.
        pub animaux: Vec<AnimalCatalogue>,
        pub armures: Vec<EquipementCatalogue>,
        pub armes: Vec<EquipementCatalogue>,
        pub materiel: Vec<EquipementCatalogue>,
        pub etats: Vec<EtatCatalogue>,
        /// Coût d'achat de points par score (7 à 18).
        pub cout_achat: Vec<(i32, i32)>,
    }
}

/// Sorts de la liste d'une classe (clé anglaise en minuscules : « sorcerer », « cleric »…),
/// triés par niveau de sort puis par nom.
pub fn sorts_de_la_liste(r: &Referentiel, t: &Traductions, cle: &str) -> Vec<SortCatalogue> {
    let mut sorts: Vec<SortCatalogue> = r
        .sorts
        .iter()
        .filter_map(|s| {
            let niveau = s.niveau_pour(cle)?;
            let fr = t.sort(&s.nom_en);
            Some(SortCatalogue {
                nom_en: s.nom_en.clone(),
                nom: fr.map(|x| x.nom.clone()).unwrap_or(s.nom_en.clone()),
                ecole: fr.map(|x| x.ecole.clone()).unwrap_or_default(),
                niveau,
                resume: fr
                    .map(|x| x.description.chars().take(220).collect())
                    .unwrap_or_default(),
                source: t.source_vf("sorts", &s.nom_en),
                note_livre: t.note_livre("sorts", &s.nom_en),
                descripteurs: s.descripteurs.iter().map(|d| d.to_lowercase()).collect(),
            })
        })
        .collect();
    sorts.sort_by_key(|s| (s.niveau, s.nom.clone()));
    sorts
}

/// Nom d'une capacité de classe sans ses précisions : minuscules, sans « (2/jour) » ni « 3d6 ».
fn cle_capacite(cap: &str) -> String {
    let mut s = cap.to_lowercase();
    if let Some(i) = s.find(" (") {
        s.truncate(i);
    }
    // On retire les valeurs de fin (« +1 », « 2/jour », « 1d6 », « 6 m », « 1/-- »).
    let mut mots: Vec<&str> = s.split(' ').collect();
    while mots.len() > 1
        && mots.last().is_some_and(|m| {
            *m == "m"
                || m.chars()
                    .next()
                    .is_some_and(|c| c == '+' || c.is_ascii_digit())
        })
    {
        mots.pop();
    }
    mots.join(" ").trim().to_string()
}

pub fn catalogue_creation() -> CatalogueCreation {
    let r = Referentiel::partage();
    let t = Traductions::partage();
    let races = r
        .races
        .iter()
        .map(|x| RaceVue {
            id: x.id.clone(),
            nom: x.nom.clone(),
            taille: x.taille.clone(),
            vitesse: x.vitesse,
            ajustements: x.ajustements.clone(),
            choix_libre: x.choix_libre,
            bonus_don: x.bonus_don,
            rang_supplementaire: x.rang_supplementaire,
            don_impose: x.don_impose.clone(),
            deux_classes_de_predilection: x.deux_classes_de_predilection,
            traits: x
                .traits
                .iter()
                .map(|tr| match t.trait_racial(&x.id, &tr.nom_en) {
                    Some((n, c)) => TexteVue {
                        nom: n.to_string(),
                        texte: c.to_string(),
                    },
                    None => TexteVue {
                        nom: tr.nom_en.clone(),
                        texte: tr.texte_en.clone(),
                    },
                })
                .collect(),
        })
        .collect();
    let classes = r
        .classes
        .iter()
        .map(|c| ClasseVue {
            id: c.id.clone(),
            nom: c.nom.clone(),
            cle: c.nom_en.to_lowercase(),
            de_vie: c.de_vie,
            rangs_par_niveau: c.rangs_par_niveau,
            incantation: c.incantation.is_some(),
            caracteristique_incantation: c.incantation.as_ref().map(|i| i.caracteristique.clone()),
            lignages: c
                .lignages
                .iter()
                .map(|l| {
                    let fr = t.lignage(&l.id);
                    LignageVue {
                        id: l.id.clone(),
                        nom: fr.map(|x| x.nom.to_string()).unwrap_or(l.nom_en.clone()),
                        competence_de_classe: fr
                            .map(|x| x.competence_de_classe.to_string())
                            .unwrap_or(l.competence_de_classe_en.clone()),
                        arcane: fr
                            .map(|x| x.arcane.to_string())
                            .unwrap_or(l.arcane_en.clone()),
                        choix_connaissance: l.competence_de_classe_en.contains("(any one)"),
                        sorts: l
                            .sorts_bonus
                            .iter()
                            .map(|s| {
                                let def = r.sort_approx(&s.nom_en);
                                let nom = def
                                    .and_then(|d| t.sort(&d.nom_en))
                                    .map(|x| x.nom.clone())
                                    .unwrap_or(s.nom_en.clone());
                                (nom, s.niveau_classe)
                            })
                            .collect(),
                        dons: l
                            .dons_bonus_en
                            .iter()
                            .map(|d| t.don(d).map(|x| x.nom.clone()).unwrap_or(d.clone()))
                            .collect(),
                    }
                })
                .collect(),
            competences_de_classe: c.competences_de_classe_en.clone(),
            don_bonus_niveau_1: c.don_bonus_niveau_1.clone(),
            sorts_connus: c
                .progression
                .iter()
                .map(|l| l.sorts_connus.clone())
                .collect(),
            prepare: c.incantation.as_ref().is_some_and(|i| i.prepare()),
            domaines_choisis: c.domaines_choisis,
            canalisation: c.canalisation,
            pacte_domaines: c.pacte_domaines.clone(),
            capacites: c
                .progression
                .first()
                .map(|l| {
                    t.capacites_de_niveau(&l.special_en)
                        .into_iter()
                        .map(|texte| TexteVue {
                            nom: "Niveau 1".into(),
                            texte,
                        })
                        .collect()
                })
                .unwrap_or_default(),
            maitrises: r
                .equipement
                .maitrises
                .classes
                .get(&c.id)
                .map(|m| {
                    // Armes nommées : noms français du catalogue quand ils existent.
                    let armes: Vec<String> = m
                        .armes
                        .iter()
                        .cloned()
                        .chain(m.armes_nommees.iter().map(|n| {
                            r.equipement
                                .arme(n)
                                .map(|a| a.nom.clone())
                                .unwrap_or_else(|| n.clone())
                        }))
                        .collect();
                    let armures: Vec<&str> = m.armures.iter().map(|a| a.nom()).collect();
                    format!(
                        "armes : {} ; armures : {}{}{}",
                        if armes.is_empty() {
                            "aucune".to_string()
                        } else {
                            armes.join(", ")
                        },
                        if armures.is_empty() {
                            "aucune".to_string()
                        } else {
                            armures.join(", ")
                        },
                        if m.boucliers { " ; boucliers" } else { "" },
                        if m.armures_permises.is_some() {
                            " (sans métal : cuir, peau ou matelassé ; boucliers en bois)"
                        } else {
                            ""
                        }
                    )
                })
                .unwrap_or_default(),
        })
        .collect();
    // Connaissances : une entrée par sous-type (Craft, Profession et Perform restent à saisir).
    const CONNAISSANCES: [&str; 10] = [
        "arcana",
        "dungeoneering",
        "engineering",
        "geography",
        "history",
        "local",
        "nature",
        "nobility",
        "planes",
        "religion",
    ];
    let mut competences: Vec<CompetenceCatalogue> = Vec::new();
    for c in &r.competences {
        if c.id == "knowledge" {
            for s in CONNAISSANCES {
                let cle = format!("knowledge:{s}");
                competences.push(CompetenceCatalogue {
                    nom: t.competence_avec_sous_type(&cle).unwrap_or(cle.clone()),
                    id: cle,
                    caracteristique: c.caracteristique.clone(),
                    formation_requise: c.formation_requise,
                    malus_d_armure: c.malus_d_armure,
                    sous_types: false,
                });
            }
        } else {
            competences.push(CompetenceCatalogue {
                id: c.id.clone(),
                nom: t.competence(&c.id).unwrap_or(&c.nom_en).to_string(),
                caracteristique: c.caracteristique.clone(),
                formation_requise: c.formation_requise,
                malus_d_armure: c.malus_d_armure,
                sous_types: c.sous_types,
            });
        }
    }
    let dons = r
        .dons
        .iter()
        .map(|d| {
            let fr = t.don(&d.nom_en);
            DonCatalogue {
                nom_en: d.nom_en.clone(),
                nom: fr.map(|x| x.nom.clone()).unwrap_or(d.nom_en.clone()),
                resume: fr.and_then(|x| x.resume.clone()),
                prerequis: fr.and_then(|x| x.prerequis.clone()),
                repetable: d.repetable(),
                a_un_choix: d.a_un_choix(),
                classes_requises: d.classes_requises().into_iter().map(String::from).collect(),
            }
        })
        .collect();
    let sorts_par_classe = r
        .classes
        .iter()
        .filter(|c| c.incantation.is_some())
        .map(|c| {
            (
                c.id.clone(),
                sorts_de_la_liste(r, t, &c.nom_en.to_lowercase()),
            )
        })
        .collect();
    let domaines = r
        .domaines
        .iter()
        .map(|d| DomaineCatalogue {
            id: d.id.clone(),
            nom: d.nom.clone(),
            description: d.description.clone(),
            divinites: d.divinites.clone(),
            alignement: d.alignement.clone(),
            pouvoirs: d
                .pouvoirs
                .iter()
                .map(|p| PouvoirDomaineVue {
                    cle: String::new(),
                    nom: p.nom.clone(),
                    genre: p.genre.clone(),
                    niveau: p.niveau,
                    actif: false,
                    utilisations: None,
                    texte: p.texte.clone(),
                })
                .collect(),
            sorts: d
                .sorts_de_domaine
                .iter()
                .map(|s| SortDomaineVue {
                    niveau: s.niveau,
                    nom: s.nom_fr.clone(),
                    nom_en: s.nom_en.clone(),
                    note: s.note.clone(),
                    accessible: false,
                })
                .collect(),
        })
        .collect();
    let divinites = r
        .divinites
        .iter()
        .map(|d| DiviniteCatalogue {
            id: d.id.clone(),
            nom: d.nom.clone(),
            alignement: d.alignement.clone(),
            theme: d.theme.clone(),
            domaines: d.domaines.clone(),
            arme_de_predilection: d.arme_de_predilection.clone(),
        })
        .collect();
    let animaux = r
        .compagnons
        .animaux
        .iter()
        .map(|a| {
            let c = &a.caracteristiques;
            let d = &a.developpement;
            AnimalCatalogue {
                id: a.id.clone(),
                nom: a.nom.clone(),
                taille: a.taille.clone(),
                vitesse: a.vitesse.clone(),
                armure_naturelle: a.armure_naturelle,
                attaques: a.attaques.clone(),
                caracteristiques: format!(
                    "For {}, Dex {}, Con {}, Int {}, Sag {}, Cha {}",
                    c.force, c.dex, c.con, c.int, c.sag, c.cha
                ),
                particularites: a.autres.clone(),
                developpement: format!(
                    "{}{}{}",
                    d.taille
                        .as_ref()
                        .map(|t| format!("taille {t} ; "))
                        .unwrap_or_default(),
                    d.caracteristiques
                        .iter()
                        .map(|(k, v)| format!("{} {:+}", k, v))
                        .collect::<Vec<_>>()
                        .join(", "),
                    if d.autres.is_empty() {
                        String::new()
                    } else {
                        format!(" ; {}", d.autres)
                    }
                ),
                niveau_developpement: d.niveau,
            }
        })
        .collect();
    let eq = &r.equipement;
    CatalogueCreation {
        races,
        classes,
        competences,
        dons,
        sorts_par_classe,
        domaines,
        divinites,
        animaux,
        armures: eq
            .armures
            .iter()
            .map(|a| EquipementCatalogue {
                nom_en: a.nom_en.clone(),
                nom: a.nom.clone(),
                genre: a.genre.nom().to_string(),
                poids_lb: a.poids_lb,
                source: t.source_vf("armures", &a.nom_en),
                note_livre: t.note_livre("armures", &a.nom_en),
                description: t.description_equipement("armures", &a.nom_en),
            })
            .collect(),
        armes: eq
            .armes
            .iter()
            .map(|a| EquipementCatalogue {
                nom_en: a.nom_en.clone(),
                nom: a.nom.clone(),
                genre: if a.est_maniable() {
                    "arme"
                } else {
                    "munition ou engin"
                }
                .to_string(),
                poids_lb: a.poids_lb,
                source: t.source_vf("armes", &a.nom_en),
                note_livre: t.note_livre("armes", &a.nom_en),
                description: t.description_equipement("armes", &a.nom_en),
            })
            .collect(),
        materiel: eq
            .materiel
            .iter()
            .map(|a| EquipementCatalogue {
                nom_en: a.nom_en.clone(),
                nom: a.nom.clone(),
                genre: a.categorie_en.clone().unwrap_or_default(),
                poids_lb: a.poids_lb,
                source: t.source_vf("materiel", &a.nom_en),
                note_livre: t.note_livre("materiel", &a.nom_en),
                description: t.description_equipement("materiel", &a.nom_en),
            })
            .collect(),
        etats: r
            .etats
            .iter()
            .map(|e| EtatCatalogue {
                id: e.id.clone(),
                nom: e.nom.clone(),
            })
            .collect(),
        cout_achat: (7..=18)
            .filter_map(|s| fiches::generation::cout_achat(s).map(|c| (s, c)))
            .collect(),
    }
}

/// Aperçu d'une fiche en cours de création : valeurs, problèmes à corriger.
pub fn apercu_fiche(f: &Fiche) -> Result<FicheVue, String> {
    let e = EtatPersonnage {
        fiche: f.clone(),
        xp: 0,
        jalon: 0,
        niveau_atteignable: f.niveau_de_personnage().max(1),
        sorts_depenses: Default::default(),
        prepares: Default::default(),
        prepares_utilises: Default::default(),
        pouvoirs_utilises: Default::default(),
    };
    fiche_complete(
        &e,
        Vec::new(),
        (
            f.nom.clone(),
            Somme {
                pc: 0,
                texte: String::new(),
            },
            None,
            false,
        ),
        Vec::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::evenements::Evenement;
    use campagne::Campagne;
    use domaine::Montant;
    use fiches::journal::EvenementFiche;

    #[test]
    fn la_fiche_complete_et_la_liste_sont_calculees_par_le_coeur() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Essai");
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
        assert!(fiche_du_pj(c.etat(&r), pj).is_none());
        c.executer(
            Evenement::Fiche {
                pj,
                evenement: EvenementFiche::Creation(Box::new(fiches::exemple::elfe_ensorceleuse())),
            },
            &r,
        )
        .unwrap();
        let etat = c.etat(&r).clone();
        let liste = liste_personnages(&etat);
        assert_eq!(liste.len(), 1);
        let resume = liste[0].fiche.as_ref().unwrap();
        assert_eq!(resume.niveau, 1);
        assert_eq!(resume.classes, "Ensorceleur 1");
        assert_eq!(liste[0].prestige, Some(4));
        assert!(liste[0].prestige_calcule);

        let f = fiche_du_pj(&etat, pj).unwrap().unwrap();
        assert_eq!(f.identite.race, "Elfe");
        assert_eq!(f.ca.total, 13);
    }

    #[test]
    fn chaque_classe_ne_propose_que_les_sorts_de_sa_liste() {
        let c = catalogue_creation();
        let r = Referentiel::partage();
        assert!(!c.sorts_par_classe.is_empty());
        for classe in r.classes.iter().filter(|c| c.incantation.is_some()) {
            let cle = classe.nom_en.to_lowercase();
            let liste = &c.sorts_par_classe[&classe.id];
            assert!(!liste.is_empty(), "{}", classe.id);
            for s in liste {
                let def = r.sort(&s.nom_en).unwrap();
                assert_eq!(
                    def.niveau_pour(&cle),
                    Some(s.niveau),
                    "{} / {}",
                    classe.id,
                    s.nom_en
                );
            }
        }
        // un sort de prêtre n'est pas proposé à l'ensorceleur
        let ens = &c.sorts_par_classe["ensorceleur"];
        assert!(ens.iter().any(|s| s.nom_en == "Magic Missile"));
        assert!(!ens.iter().any(|s| s.nom_en == "Cure Light Wounds"));
    }

    #[test]
    fn le_catalogue_de_creation_est_complet() {
        let c = catalogue_creation();
        assert_eq!(c.races.len(), 7);
        assert_eq!(c.competences.len(), 35);
        assert!(c
            .competences
            .iter()
            .any(|x| x.nom == "Connaissances (mystères)"));
        assert_eq!(c.dons.len(), 176);
        assert_eq!(c.sorts_par_classe["ensorceleur"].len(), 390);
        assert_eq!(c.classes[0].lignages.len(), 10);
        assert_eq!(c.cout_achat.len(), 12);
        assert!(c.etats.len() == 34 && c.armures.len() == 21);
        let apercu = apercu_fiche(&fiches::exemple::elfe_ensorceleuse()).unwrap();
        assert!(apercu.problemes.is_empty(), "{:?}", apercu.problemes);
    }
}

// ------------------------------------------------------------------ montée de niveau

vue! {
    pub struct DonDeLignageVue {
        /// Valeur à enregistrer dans la fiche (nom anglais, avec la précision éventuelle).
        pub nom_en: String,
        pub nom: String,
    }
}

vue! {
    pub struct ApercuMonteeVue {
        pub niveau_de_personnage: u32,
        pub classe: String,
        pub niveau_de_classe: u32,
        pub de_vie: u32,
        /// Dé de vie imposé par la méthode de la campagne ; `None` : le joueur lance le dé.
        pub de_vie_propose: Option<u32>,
        pub bba: (i32, i32),
        pub vigueur: (i32, i32),
        pub reflexes: (i32, i32),
        pub volonte: (i32, i32),
        pub dons_a_choisir: u32,
        /// Parmi ces dons, nombre à prendre dans la liste du lignage.
        pub dons_de_lignage: u32,
        /// Dons permis pour le don de lignage (vide si aucun n'est à prendre).
        pub dons_de_lignage_possibles: Vec<DonDeLignageVue>,
        pub augmentation: bool,
        pub predilection_possible: bool,
        pub capacites: Vec<String>,
        pub sorts_connus_gagnes: Vec<(u32, u32)>,
        pub echange_de_sort: bool,
        /// Rangs de compétence à placer, sans le bonus de classe de prédilection (+1 si « rang »).
        pub rangs_a_placer: u32,
        pub niveau_atteint: bool,
        pub niveau_atteignable: u32,
    }
}

vue! {
    pub struct EssaiMontee {
        pub erreurs: Vec<String>,
        pub apres: Option<FicheVue>,
    }
}

/// Dons de la liste du lignage, que la fiche n'a pas déjà pris, avec leur nom français.
fn dons_du_lignage(
    t: &Traductions,
    classe: &fiches::donnees::Classe,
    lignage: Option<&str>,
) -> Vec<DonDeLignageVue> {
    let Some(l) = lignage.and_then(|id| classe.lignages.iter().find(|l| l.id == id)) else {
        return Vec::new();
    };
    l.dons_bonus_en
        .iter()
        .map(|d| {
            let base = fiches::catalogue::nom_de_base(d);
            let nom_base = t.don(base).map(|x| x.nom.clone());
            let nom = match (nom_base, d.find(" (")) {
                (Some(n), Some(i)) => format!("{n}{}", &d[i..]),
                (Some(n), None) => n,
                (None, _) => d.clone(),
            };
            DonDeLignageVue {
                nom_en: d.clone(),
                nom,
            }
        })
        .collect()
}

/// Ce que l'assistant de montée montre pour passer un niveau dans `classe`.
pub fn apercu_montee(etat: &Etat, pj: IdPersonnage, classe: &str) -> Option<ApercuMonteeVue> {
    let r = Referentiel::partage();
    let t = Traductions::partage();
    let e = etat.fiche_de(pj)?;
    let a = fiches::progression::apercu(r, &e.fiche, classe)?;
    let regles = etat.regles_personnages;
    let v = e.fiche.calculer(r).ok()?;
    let c = r.classe(classe)?;
    let race = r
        .race(&e.fiche.race)
        .map(|x| x.rang_supplementaire)
        .unwrap_or(0);
    let mod_int = v.modificateurs[&Carac::Int];
    Some(ApercuMonteeVue {
        niveau_de_personnage: a.niveau_de_personnage,
        classe: a.classe,
        niveau_de_classe: a.niveau_de_classe,
        de_vie: a.de_vie,
        de_vie_propose: fiches::progression::de_de_vie(regles.methode_pv, a.de_vie, None).ok(),
        bba: a.bba,
        vigueur: a.vigueur,
        reflexes: a.reflexes,
        volonte: a.volonte,
        dons_a_choisir: a.dons_a_choisir,
        dons_de_lignage: a.dons_de_lignage,
        dons_de_lignage_possibles: if a.dons_de_lignage > 0 {
            dons_du_lignage(t, c, e.fiche.lignage.as_deref())
        } else {
            Vec::new()
        },
        augmentation: a.augmentation,
        predilection_possible: a.predilection_possible && regles.classe_de_predilection,
        capacites: t.capacites_de_niveau(&a.capacites_en),
        sorts_connus_gagnes: a.sorts_connus_gagnes,
        echange_de_sort: a.echange_de_sort,
        rangs_a_placer: ((c.rangs_par_niveau + mod_int).max(1) as u32) + race,
        niveau_atteint: a.niveau_de_personnage <= e.niveau_atteignable,
        niveau_atteignable: e.niveau_atteignable,
    })
}

/// Essaie une montée de niveau sans l'enregistrer : erreurs, ou fiche obtenue.
pub fn essai_montee(
    etat: &Etat,
    pj: IdPersonnage,
    montee: &fiches::progression::Montee,
) -> Option<EssaiMontee> {
    let r = Referentiel::partage();
    let e = etat.fiche_de(pj)?;
    let regles = etat.regles_personnages.fiche();
    Some(
        match fiches::progression::appliquer(r, &regles, &e.fiche, montee, e.niveau_atteignable) {
            Err(erreurs) => EssaiMontee {
                erreurs: erreurs.iter().map(|x| x.to_string()).collect(),
                apres: None,
            },
            Ok(f) => {
                let apres = apercu_fiche(&f).ok();
                EssaiMontee {
                    erreurs: Vec::new(),
                    apres,
                }
            }
        },
    )
}

#[cfg(test)]
mod tests_pretre {
    use super::*;
    use fiches::fiche::{BonusPredilection, NiveauPris};
    use std::collections::BTreeMap;

    fn pretre(niveau: u32) -> Fiche {
        Fiche {
            nom: "Test".into(),
            race: "humain".into(),
            bonus_racial_libre: Some(Carac::Sag),
            scores_base: [
                (Carac::For, 12),
                (Carac::Dex, 10),
                (Carac::Con, 14),
                (Carac::Int, 10),
                (Carac::Sag, 16),
                (Carac::Cha, 12),
            ]
            .into_iter()
            .collect(),
            dommages: BTreeMap::new(),
            drain: BTreeMap::new(),
            augmentations: vec![],
            niveaux: (0..niveau)
                .map(|i| NiveauPris {
                    classe: "pretre".into(),
                    de_vie_obtenu: if i == 0 { 8 } else { 5 },
                    rangs: BTreeMap::new(),
                    predilection: Some(BonusPredilection::PointDeVie),
                    dons: vec![],
                })
                .collect(),
            classes_de_predilection: vec!["pretre".into()],
            lignage: None,
            connaissance_de_lignage: None,
            style_de_combat: None,
            effets: vec![],
            sorts_connus: vec![],
            equipement: Default::default(),
            etats: vec![],
            temporaires: vec![],
            divinite: Some("iomedae".into()),
            alignement: Some("LB".into()),
            domaines: vec!["bien".into(), "soleil".into()],
            energie: None,
            pacte: None,
            compagnon: None,
            graces: vec![],
            talents_roublard: vec![],
            ennemis_jures: vec![],
            augmentations_ennemis: vec![],
            pouvoirs_rage: vec![],
            en_rage: false,
            rounds_de_rage: 0,
            fatigue_rage: None,
        }
    }

    #[test]
    fn la_fiche_d_un_pretre_montre_foi_domaines_et_preparation() {
        let f = apercu_fiche(&pretre(5)).unwrap();
        let divin = f.divin.expect("foi");
        assert_eq!(divin.divinite.as_deref(), Some("Iomédae"));
        assert_eq!(divin.arme_de_predilection.as_deref(), Some("épée longue"));
        assert_eq!(divin.domaines.len(), 2);
        assert_eq!(divin.domaines[0].nom, "Bien");
        assert!(divin.domaines[0].pouvoirs[0].actif);
        let c = divin.canalisation.expect("canalisation");
        assert_eq!((c.positive, c.des), (Some(true), 3));
        assert!(!divin.energie_a_choisir);
        let prep = f.preparation.expect("préparation");
        assert_eq!(prep.classe_id, "pretre");
        // Oraisons : 4 ; niveau 1 : 3 + 1 de Sagesse ; un emplacement de domaine.
        let cap = |n: u32| prep.capacites.iter().find(|c| c.niveau == n).unwrap();
        assert_eq!(cap(0).normaux, 4);
        assert_eq!((cap(1).normaux, cap(1).domaine), (4, 1));
        assert!(prep.prepares.is_empty());
        assert_eq!(prep.conversion, Some(true));
        assert!(prep.interdits.contains(&"evil".to_string()));
        // Seuls les sorts de domaine des niveaux accessibles (1 à 3 au niveau 5) sont proposés.
        assert!(prep.sorts_de_domaine.iter().all(|s| s.niveau <= 3));
        assert!(!prep.sorts_de_domaine.is_empty());
    }

    fn druide(niveau: u32) -> Fiche {
        let mut f = pretre(niveau);
        for n in &mut f.niveaux {
            n.classe = "druide".into();
        }
        f.classes_de_predilection = vec!["druide".into()];
        f.divinite = None;
        f.alignement = None;
        f.domaines = vec![];
        f
    }

    #[test]
    fn la_fiche_d_un_druide_montre_formes_pouvoirs_et_compagnon() {
        let mut f = druide(8);
        f.pacte = Some("compagnon".into());
        f.compagnon = Some(fiches::compagnon::Compagnon {
            animal: "loup".into(),
            nom: "Croc".into(),
            ignore_developpement: false,
            augmentations: vec![Carac::Sag],
        });
        let v = apercu_fiche(&f).unwrap();
        assert_eq!(v.pacte.as_deref(), Some("compagnon"));
        let c = v.compagnon.expect("compagnon");
        assert_eq!(
            (c.animal.as_str(), c.nom.as_str(), c.dv),
            ("Loup", "Croc", 7)
        );
        assert!(c
            .capacites
            .iter()
            .any(|t| t.nom == "lien" && !t.texte.is_empty()));
        assert_eq!(c.augmentations, vec!["sag".to_string()]);
        // Forme animale : 3 utilisations au niveau 8 ; formes permises jusqu'au niveau 8 seulement.
        let formes = v.formes.expect("formes");
        assert_eq!(formes.duree_heures, 8);
        let p = v
            .pouvoirs
            .iter()
            .find(|p| p.cle == "forme_animale")
            .unwrap();
        assert_eq!((p.max, p.restants), (Some(3), Some(3)));
        assert!(formes.formes.iter().any(|x| x.niveau == 8 && x.disponible));
        assert!(formes
            .formes
            .iter()
            .any(|x| x.niveau == 10 && !x.disponible));
        assert!(formes.en_cours.is_none());
        // Pas de domaine ni de préparation de domaine avec le compagnon.
        assert!(v.divin.is_none());
        // Avec le pacte du domaine : la foi montre le domaine, sans dieu.
        let mut g = druide(5);
        g.pacte = Some("domaine".into());
        g.domaines = vec!["feu".into()];
        let vg = apercu_fiche(&g).unwrap();
        let divin = vg.divin.expect("domaine du pacte");
        assert_eq!(divin.domaines[0].nom, "Feu");
        assert!(divin.divinite.is_none() && divin.canalisation.is_none());
        assert!(vg.compagnon.is_none());
    }

    #[test]
    fn le_catalogue_propose_pretre_dieux_domaines_et_listes_de_sorts() {
        let c = catalogue_creation();
        let classe = c.classes.iter().find(|x| x.id == "pretre").expect("prêtre");
        assert!(classe.prepare && classe.canalisation);
        assert_eq!(classe.domaines_choisis, 2);
        assert!(!classe.capacites.is_empty());
        assert!(classe.maitrises.contains("simple"));
        assert_eq!(c.domaines.len(), 33);
        assert_eq!(c.divinites.len(), 20);
        let liste = &c.sorts_par_classe["pretre"];
        assert!(liste
            .iter()
            .any(|s| s.nom_en == "Cure Light Wounds" && s.niveau == 1));
        assert!(c.sorts_par_classe.contains_key("ensorceleur"));
    }
}

#[cfg(all(test, feature = "serde"))]
mod export_apercu {
    use super::*;

    /// Écrit des données d'exemple pour l'aperçu de l'interface (variable GP_EXPORT_DIR).
    #[test]
    fn exporter_les_donnees_d_apercu() {
        let Some(dossier) = std::env::var_os("GP_EXPORT_DIR") else {
            return;
        };
        let d = std::path::Path::new(&dossier);
        // GP_EXPORT_CLASSE=pretre : aperçu de l'interface avec un prêtre au lieu de l'ensorceleuse.
        let exemple = match std::env::var("GP_EXPORT_CLASSE").as_deref() {
            Ok("pretre") => fiches::exemple::pretre_d_iomedae(),
            Ok("druide") => fiches::exemple::druide_de_la_foret(),
            _ => fiches::exemple::elfe_ensorceleuse(),
        };
        let fiche = apercu_fiche(&exemple).unwrap();
        let ecrire = |nom: &str, v: &dyn erased::Serialize| {
            std::fs::write(d.join(nom), v.vers_json()).unwrap();
        };
        ecrire("fiche.json", &fiche);
        ecrire("catalogue.json", &catalogue_creation());
        mod erased {
            pub trait Serialize {
                fn vers_json(&self) -> String;
            }
            impl<T: serde::Serialize> Serialize for T {
                fn vers_json(&self) -> String {
                    serde_json::to_string(self).unwrap()
                }
            }
        }
    }
}
