//! Vendre, abandonner (GC p. 84) et réparer (GC p. 92) : éléments communs aux pages
//! Bâtiments et Organisations.

use campagne::etat::Etat;
use campagne::evenements::Evenement;
use campagne::modele::{Actif, IdBatiment, IdEquipe, IdOrganisation, IdSalle};
use domaine::{CoutCapitaux, Montant};
use regles::Referentiel;

/// Bien visé par une vente ou un abandon, tel que l'interface le désigne.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cible {
    /// « batiment », « salle », « organisation » ou « equipe ».
    pub genre: GenreCible,
    pub id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum GenreCible {
    Batiment,
    Salle,
    Organisation,
    Equipe,
}

impl From<Cible> for Actif {
    fn from(c: Cible) -> Self {
        match c.genre {
            GenreCible::Batiment => Actif::Batiment(IdBatiment(c.id)),
            GenreCible::Salle => Actif::Salle(IdSalle(c.id)),
            GenreCible::Organisation => Actif::Organisation(IdOrganisation(c.id)),
            GenreCible::Equipe => Actif::Equipe(IdEquipe(c.id)),
        }
    }
}

impl From<Actif> for Cible {
    fn from(a: Actif) -> Self {
        let (genre, id) = match a {
            Actif::Batiment(x) => (GenreCible::Batiment, x.0),
            Actif::Salle(x) => (GenreCible::Salle, x.0),
            Actif::Organisation(x) => (GenreCible::Organisation, x.0),
            Actif::Equipe(x) => (GenreCible::Equipe, x.0),
        };
        Cible { genre, id }
    }
}

/// Ce que l'interface montre pour vendre ou abandonner un bien.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct EtatVente {
    pub cible: Cible,
    pub libelle: String,
    /// Vente en cours : « acheteur dans 4 j — produit : 120 po ».
    pub en_cours: Option<String>,
    /// « Prix d'achat des parties construites ou recrutées : 240 po (coût …) ».
    pub valeur: String,
    /// Produit proposé (moitié du prix d'achat), à préremplir.
    pub produit: String,
    /// Influence disponible dans la localité (1d6 de moins par point dépensé).
    pub influence_dispo: u32,
}

pub fn etat_vente(etat: &Etat, r: &Referentiel, a: Actif) -> EtatVente {
    let libelle = etat.libelle_actif(a, r);
    let en_cours = etat.en_vente(a).map(|v| {
        let cap = if v.capitaux.unites() > 0 {
            format!(" + {}", v.capitaux)
        } else {
            String::new()
        };
        format!(
            "acheteur dans {} j — produit : {}{cap}",
            v.jour_fin.saturating_sub(etat.jour),
            v.po
        )
    });
    let (prix, cout) = etat.valeur_actif(a, r);
    let (produit, _) = etat.produit_vente(a, r);
    EtatVente {
        cible: a.into(),
        libelle,
        en_cours,
        valeur: format!(
            "Prix d'achat des parties construites ou recrutées : {prix} (coût {cout}). \
             Produit de la vente : la moitié du prix d'achat, en po (GC p. 84)."
        ),
        produit: produit.to_string(),
        influence_dispo: etat
            .localite_actif(a)
            .ok()
            .and_then(|l| etat.localites.get(&l))
            .map(|l| l.capitaux.influence)
            .unwrap_or(0),
    }
}

/// Coûts de réparation d'une salle brisée : (po, capitaux si possible).
pub fn reparation(etat: &Etat, r: &Referentiel, salle: IdSalle) -> (String, Option<String>) {
    let (po, cap) = etat.cout_reparation(salle, r);
    (po.to_string(), (cap.unites() > 0).then(|| cap.to_string()))
}

/// Mise en vente : `jours` = 3d6 moins 1d6 par point d'Influence, lancés par le joueur.
pub fn mise_en_vente(
    etat: &Etat,
    r: &Referentiel,
    cible: Cible,
    jours: &str,
    influence: u32,
    po: &str,
) -> Result<(Evenement, String), String> {
    let a: Actif = cible.into();
    let jours = jours
        .trim()
        .parse()
        .map_err(|_| "Indiquez le nombre de jours pour trouver l'acheteur (3d6)".to_string())?;
    let po = Montant::depuis_texte(po)
        .filter(|m| !m.est_negatif())
        .ok_or("Produit illisible : écrivez par exemple 120 ou 4 po 7 pa")?;
    Ok((
        Evenement::VenteLancee {
            cible: a,
            jours,
            influence,
            po,
            capitaux: CoutCapitaux::default(),
        },
        format!("Mise en vente : {}", etat.libelle_actif(a, r)),
    ))
}
