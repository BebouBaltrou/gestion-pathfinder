pub mod batiments;
pub mod campagnes;
pub mod gestionnaires;
pub mod historique;
pub mod localites;
pub mod organisations;
pub mod parametres;
pub mod personnages;
pub mod points;
pub mod recettes;
pub mod referentiel;
pub mod tableau;
pub mod ventes;

use campagne::modele::CategorieLocalite;
use regles::Referentiel;

/// « Petite bourgade (15/jour) » ou « Avant-poste (perso., 3/jour) ».
pub fn libelle_categorie(c: &CategorieLocalite, r: &Referentiel) -> String {
    match c {
        CategorieLocalite::Officielle { id } => match r.communaute(id) {
            Some(x) => format!("{} — limite {}/jour", x.nom, x.limite_depense_jour),
            None => id.clone(),
        },
        CategorieLocalite::Personnalisee {
            nom,
            limite_depense_jour,
        } => {
            format!("{nom} (personnalisée) — limite {limite_depense_jour}/jour")
        }
    }
}
