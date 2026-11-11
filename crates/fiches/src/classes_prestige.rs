//! Classes de prestige (Manuel des joueurs, chapitre 11) : conditions d'entrée.
//!
//! Les conditions viennent de `data/core/classes_prestige.json`. Elles sont évaluées avec le même
//! moteur que les prérequis de dons : une condition non remplie bloque la prise de niveau, une
//! condition qu'on ne sait pas vérifier (texte) est signalée « à vérifier ».

use crate::catalogue::Prerequis;
use crate::donnees::{Classe, Referentiel};
use crate::fiche::{Fiche, Valeurs};
use crate::prerequis::{evaluer_un, Contexte, Statut};
use serde::Deserialize;

/// Une classe de prestige (même forme que les autres classes, progression comprise) et ses
/// conditions d'entrée.
#[derive(Debug, Clone, Deserialize)]
pub struct ClassePrestige {
    #[serde(flatten)]
    pub classe: Classe,
    pub conditions: Vec<Prerequis>,
    /// Conditions du livre qu'on ne vérifie pas dans l'application (rappel seulement).
    #[serde(default)]
    pub conditions_speciales: Vec<String>,
}

#[derive(Deserialize)]
struct Fichier {
    classes: Vec<ClassePrestige>,
}

/// Lit le fichier `classes_prestige.json`.
pub fn depuis_json(json: &str) -> Result<Vec<ClassePrestige>, serde_json::Error> {
    Ok(serde_json::from_str::<Fichier>(json)?.classes)
}

/// Évalue chaque condition d'entrée de la classe pour la fiche. `valeurs` : valeurs calculées de
/// la fiche avant la prise du niveau.
pub fn verifier(
    r: &Referentiel,
    f: &Fiche,
    valeurs: &Valeurs,
    classe: &ClassePrestige,
) -> Vec<(String, Statut)> {
    let dons: Vec<String> = f
        .dons_automatiques(r)
        .into_iter()
        .chain(f.dons_choisis())
        .collect();
    let ctx = Contexte {
        referentiel: r,
        fiche: f,
        valeurs,
        dons: &dons,
    };
    classe
        .conditions
        .iter()
        .map(|p| evaluer_un(&ctx, p))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_dix_classes_de_prestige_se_chargent() {
        let json = include_str!("../../../data/core/classes_prestige.json");
        let classes = depuis_json(json).expect("JSON valide");
        assert_eq!(classes.len(), 10);
        assert!(classes.iter().all(|c| !c.conditions.is_empty()));
        assert!(classes.iter().all(|c| c.classe.progression.len() == 10));
        let archer = classes
            .iter()
            .find(|c| c.classe.id == "archer_mage")
            .unwrap();
        assert!(matches!(archer.conditions[0], Prerequis::Bba { min: 6 }));
    }

    #[test]
    fn archer_mage_refuse_sans_bba_suffisant() {
        use crate::progression::{appliquer, ErreurMontee, Montee, ReglesFiche};
        let r = Referentiel::partage();
        let f = crate::fiche::tests::elfe_ensorceleuse();
        let m = Montee {
            classe: "archer_mage".into(),
            de_vie: 1,
            rangs: Default::default(),
            predilection: None,
            dons: vec![],
            augmentation: None,
            sorts_appris: vec![],
            echange: None,
        };
        let err = appliquer(r, &ReglesFiche::default(), &f, &m, 20).unwrap_err();
        assert!(matches!(
            &err[0],
            ErreurMontee::ConditionsNonRemplies { manquants, .. }
                if manquants.iter().any(|t| t.contains("bonus de base"))
        ));
    }
}
