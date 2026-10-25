//! Modèles d'objets de l'inventaire, construits à partir du catalogue d'équipement de
//! `crates/fiches` (`data/core/`) : une seule source pour les armes, armures et le matériel.
//!
//! L'identifiant d'un modèle vient du nom anglais (`equipement:longsword`), pour rester stable
//! quand un nom français est corrigé.

use domaine::Montant;
use fiches::equipement::{CatalogueEquipement, Cout, GenreProtection};
use inventaire::{Modele, Poids};
use std::collections::BTreeMap;

fn identifiant(nom_en: &str) -> String {
    let mut slug = String::new();
    for c in nom_en.chars() {
        if c.is_alphanumeric() {
            slug.extend(c.to_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    format!("equipement:{}", slug.trim_matches('-'))
}

/// Prix du catalogue (pp, po, pa, pc) en pièces de cuivre.
fn montant(cout: &Option<Cout>) -> Option<Montant> {
    let c = cout.as_ref()?;
    let facteur = match c.unite.as_str() {
        "cp" => 1.0,
        "sp" => 10.0,
        "gp" => 100.0,
        "pp" => 1000.0,
        _ => return None,
    };
    Some(Montant::pc((c.valeur * facteur).round() as i64))
}

/// Poids du catalogue (livres) en grammes : 1 lb = 500 g, comme dans la version française.
fn poids(lb: Option<f64>) -> Option<Poids> {
    lb.map(|l| Poids((l * 500.0).round() as i64))
}

fn proprietes(valeurs: &[(&str, Option<String>)]) -> BTreeMap<String, String> {
    valeurs
        .iter()
        .filter_map(|(cle, v)| v.clone().map(|v| ((*cle).to_string(), v)))
        .collect()
}

/// Tous les modèles du catalogue d'équipement : armes, armures et boucliers, matériel.
pub fn modeles(catalogue: &CatalogueEquipement) -> Vec<Modele> {
    let mut modeles = Vec::new();
    for a in &catalogue.armes {
        let munition = a.categorie_en.as_deref() == Some("Ammunition");
        modeles.push(Modele {
            id: identifiant(&a.nom_en),
            nom: a.nom.clone(),
            type_objet: if munition { "munition" } else { "arme" }.to_string(),
            valeur: montant(&a.cout),
            poids: poids(a.poids_lb),
            source: format!("{}, p. {}", a.source.livre, a.source.page.unwrap_or(0)),
            proprietes: proprietes(&[
                ("nom_en", Some(a.nom_en.clone())),
                ("categorie", a.categorie_en.clone()),
                ("maniement", a.maniement_en.clone()),
                ("degats", a.degats_en.clone()),
                ("critique", a.critique_en.clone()),
                ("portee", a.portee_en.clone()),
                ("type_degats", a.type_en.clone()),
                ("special", a.special_en.clone()),
                ("groupes", a.groupes_en.clone()),
            ]),
        });
    }
    for a in &catalogue.armures {
        let type_objet = match a.genre {
            GenreProtection::Bouclier => "bouclier",
            GenreProtection::Accessoire => "supplement-armure",
            _ => "armure",
        };
        modeles.push(Modele {
            id: identifiant(&a.nom_en),
            nom: a.nom.clone(),
            type_objet: type_objet.to_string(),
            valeur: montant(&a.cout),
            poids: poids(a.poids_lb),
            source: format!("{}, p. {}", a.source.livre, a.source.page.unwrap_or(0)),
            proprietes: proprietes(&[
                ("nom_en", Some(a.nom_en.clone())),
                ("categorie", Some(a.genre.nom().to_string())),
                ("bonus_ca", a.bonus_armure.map(|v| v.to_string())),
                ("dex_max", a.dex_max.map(|v| v.to_string())),
                ("malus_armure", a.malus_armure.map(|v| v.to_string())),
                ("echec_sorts_pct", a.echec_sorts_pct.map(|v| v.to_string())),
            ]),
        });
    }
    for o in &catalogue.materiel {
        modeles.push(Modele {
            id: identifiant(&o.nom_en),
            nom: o.nom.clone(),
            type_objet: "materiel".to_string(),
            valeur: montant(&o.cout),
            poids: poids(o.poids_lb),
            source: format!("{}, p. {}", o.source.livre, o.source.page.unwrap_or(0)),
            proprietes: proprietes(&[
                ("nom_en", Some(o.nom_en.clone())),
                ("categorie", o.categorie_en.clone()),
            ]),
        });
    }
    modeles
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalogue() -> CatalogueEquipement {
        CatalogueEquipement::depuis_json(
            include_str!("../../../data/core/armures.json"),
            include_str!("../../../data/core/armes.json"),
            include_str!("../../../data/core/materiel.json"),
            include_str!("../../../data/core/charge.json"),
            include_str!("../../../data/core/maitrises.json"),
        )
        .unwrap()
    }

    fn trouver<'a>(modeles: &'a [Modele], nom: &str) -> &'a Modele {
        modeles
            .iter()
            .find(|m| m.nom == nom)
            .unwrap_or_else(|| panic!("« {nom} » absent"))
    }

    #[test]
    fn tout_le_catalogue_devient_des_modeles_uniques() {
        let c = catalogue();
        let m = modeles(&c);
        assert_eq!(m.len(), c.armes.len() + c.armures.len() + c.materiel.len());
        let mut ids: Vec<&str> = m.iter().map(|x| x.id.as_str()).collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n, "identifiants en double");
    }

    #[test]
    fn valeurs_et_poids_du_manuel_des_joueurs() {
        let m = modeles(&catalogue());
        let dague = trouver(&m, "Dague");
        assert_eq!(dague.valeur, Some(Montant::po(2)));
        assert_eq!(dague.poids, Some(Poids(500)));
        assert_eq!(dague.type_objet, "arme");
        // Les noms officiels du Manuel des joueurs priment sur la traduction automatique.
        assert_eq!(trouver(&m, "Morgenstern").type_objet, "arme");
        let harnois = trouver(&m, "Harnois");
        assert_eq!(harnois.valeur, Some(Montant::po(1500)));
        assert_eq!(harnois.poids, Some(Poids(25_000)));
        assert_eq!(harnois.proprietes["bonus_ca"], "9");
        assert_eq!(trouver(&m, "Targe").type_objet, "bouclier");
        assert_eq!(trouver(&m, "Bougie").valeur, Some(Montant::pc(1)));
        assert_eq!(trouver(&m, "Flèches (20)").type_objet, "munition");
    }
}
