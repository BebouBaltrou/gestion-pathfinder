//! Calcul de fabrication d'objets magiques : ce fichier relie l'interface au cœur
//! (`regles::fabrication`). Aucune règle ici.

use regles::fabrication::{self as f, Issue, Paiement, Progression};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct EntreeFabrication {
    pub prix_po: i64,
    pub niveau_lanceur: u32,
    pub conditions_manquantes: u32,
    pub potion_ou_parchemin: bool,
    pub acceleree: bool,
    pub po_payes: i64,
    pub marchandises: u32,
    pub magie: u32,
    /// Jours d'intermède déjà passés sur cet objet.
    pub jours_faits: u32,
    /// Résultat du d20 + bonus, saisi une fois la fabrication terminée.
    pub jet: Option<i32>,
}

#[derive(Serialize)]
pub struct SortieFabrication {
    pub dd: i32,
    pub heures_requises: u32,
    pub jours_requis: u32,
    pub jours_restants: u32,
    pub terminee: bool,
    /// Fournitures à payer, en pa (1 po = 10 pa).
    pub fournitures_pa: i64,
    pub paiement_pa: i64,
    pub suffisant: bool,
    /// Issue du test final, si le jet est saisi.
    pub issue: Option<&'static str>,
}

#[tauri::command]
pub fn fabrication_calculer(entree: EntreeFabrication) -> SortieFabrication {
    let dd = f::dd(entree.niveau_lanceur, entree.conditions_manquantes);
    let heures = f::heures_requises(entree.prix_po, entree.potion_ou_parchemin, entree.acceleree);
    let mut progression = Progression::nouvelle(heures);
    for _ in 0..entree.jours_faits {
        progression.avancer_jour();
    }
    let fournitures = f::fournitures_pa(entree.prix_po);
    let paiement = Paiement {
        po: entree.po_payes,
        marchandises: entree.marchandises,
        magie: entree.magie,
    }
    .valeur_pa();
    let issue = entree.jet.filter(|_| progression.terminee()).map(|jet| match f::issue_test(jet, dd) {
        Issue::Reussi => "reussi",
        Issue::Echec => "echec",
        Issue::Maudit => "maudit",
    });
    SortieFabrication {
        dd,
        heures_requises: heures,
        jours_requis: f::jours_requis(heures),
        jours_restants: progression.jours_restants(),
        terminee: progression.terminee(),
        fournitures_pa: fournitures,
        paiement_pa: paiement,
        suffisant: paiement >= fournitures,
        issue,
    }
}
