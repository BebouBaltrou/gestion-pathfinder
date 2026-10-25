//! Localités prêtes à l'emploi (villes de Varisie), à l'image des bâtiments et organisations
//! préfaits. Données : `data/core/localites_varisie.json`, tirées des pages du wiki
//! Pathfinder-FR. La catégorie officielle est toujours calculée ici d'après la population
//! (tableau du Guide du maître), jamais lue dans le wiki.

use crate::localites::SaisieLocalite;
use regles::Referentiel;
use std::sync::OnceLock;

const JSON: &str = include_str!("../../../data/core/localites_varisie.json");

/// Une localité prête à l'emploi, avec le formulaire de création déjà rempli.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ModeleLocalite {
    pub id: String,
    pub nom: String,
    /// « Korvosa — grande ville · 18 486 hab. »
    pub libelle: String,
    /// `true` : valeurs du wiki sans doute ; `false` : « à confirmer ».
    pub officiel: bool,
    /// Pourquoi la localité est « à confirmer ».
    pub remarque: Option<String>,
    /// Page du wiki d'où viennent les valeurs.
    pub source: String,
    /// Formulaire prérempli (catégorie officielle proposée si la population est connue).
    pub saisie: SaisieLocalite,
}

#[derive(Debug, Clone)]
struct Brut {
    id: String,
    nom: String,
    genre: String,
    population: Option<u32>,
    alignement: Option<String>,
    type_wiki: Option<String>,
    valeur_base: Option<String>,
    limite_achat: Option<String>,
    habitants: Option<String>,
    rattachement: Option<String>,
    description: String,
    page: String,
    officiel: bool,
    remarque: Option<String>,
}

fn texte(v: &serde_json::Value, cle: &str) -> Option<String> {
    v.get(cle).and_then(|x| x.as_str()).map(str::to_string)
}

fn brutes() -> &'static [Brut] {
    static T: OnceLock<Vec<Brut>> = OnceLock::new();
    T.get_or_init(|| {
        let racine: serde_json::Value = serde_json::from_str(JSON)
            .expect("data/core/localites_varisie.json est un JSON valide");
        racine["localites"]
            .as_array()
            .expect("« localites » est une liste")
            .iter()
            .map(|v| Brut {
                id: texte(v, "id").expect("id"),
                nom: texte(v, "nom").expect("nom"),
                genre: texte(v, "genre").unwrap_or_default(),
                population: v
                    .get("population")
                    .and_then(|x| x.as_u64())
                    .map(|p| p as u32),
                alignement: texte(v, "alignement"),
                type_wiki: texte(v, "type_wiki"),
                valeur_base: texte(v, "valeur_base"),
                limite_achat: texte(v, "limite_achat"),
                habitants: texte(v, "habitants"),
                rattachement: texte(v, "rattachement"),
                description: texte(v, "description").unwrap_or_default(),
                page: texte(v, "page").expect("page"),
                officiel: texte(v, "statut").as_deref() == Some("officiel"),
                remarque: texte(v, "remarque"),
            })
            .collect()
    })
}

/// Nombre entier avec espaces fines : 18486 → « 18 486 ».
fn groupe_milliers(n: u32) -> String {
    let s = n.to_string();
    let mut sortie = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            sortie.push(' ');
        }
        sortie.push(c);
    }
    sortie
}

fn notes(b: &Brut) -> String {
    let mut lignes = vec![b.description.clone()];
    if let Some(r) = &b.rattachement {
        lignes.push(format!("Statut : {r}."));
    }
    if let Some(t) = &b.type_wiki {
        lignes.push(format!("Type (wiki) : {t}."));
    }
    if let Some(a) = &b.alignement {
        lignes.push(format!("Alignement : {a}."));
    }
    if let Some(h) = &b.habitants {
        lignes.push(format!("Habitants : {h}."));
    }
    if let Some(v) = &b.valeur_base {
        lignes.push(format!("Valeur de base (wiki) : {v}."));
    }
    if let Some(l) = &b.limite_achat {
        lignes.push(format!("Limite d'achat (wiki) : {l}."));
    }
    lignes.push(format!(
        "Source : wiki Pathfinder-FR, page « {} » (Golarion).",
        b.page
    ));
    if let Some(r) = &b.remarque {
        lignes.push(format!("À confirmer : {r}"));
    }
    lignes.join("\n")
}

fn modele(b: &Brut, r: &Referentiel) -> ModeleLocalite {
    let categorie = b
        .population
        .and_then(|p| r.categorie_pour_population(p))
        .map(|c| c.id.clone())
        .unwrap_or_default();
    let libelle = match b.population {
        Some(p) => format!("{} — {} · {} hab.", b.nom, b.genre, groupe_milliers(p)),
        None => format!("{} — {} · population inconnue", b.nom, b.genre),
    };
    ModeleLocalite {
        id: b.id.clone(),
        nom: b.nom.clone(),
        libelle,
        officiel: b.officiel,
        remarque: b.remarque.clone(),
        source: format!("Wiki Pathfinder-FR, Golarion.{}", b.page),
        saisie: SaisieLocalite {
            nom: b.nom.clone(),
            genre: b.genre.clone(),
            population: b.population.map(|p| p.to_string()).unwrap_or_default(),
            officielle: true,
            categorie_officielle: categorie,
            notes: notes(b),
            ..SaisieLocalite::default()
        },
    }
}

/// Toutes les localités prêtes à l'emploi, dans l'ordre du fichier (les mieux connues d'abord).
pub fn modeles_localites(r: &Referentiel) -> Vec<ModeleLocalite> {
    brutes().iter().map(|b| modele(b, r)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fichier_valide_et_complet() {
        let r = regles::referentiel_officiel();
        let m = modeles_localites(&r);
        assert_eq!(m.len(), 23);
        let mut ids: Vec<_> = m.iter().map(|x| x.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 23, "identifiants uniques");
        assert!(m.iter().all(|x| !x.saisie.nom.is_empty()));
    }

    #[test]
    fn categorie_calculee_d_apres_la_population() {
        let r = regles::referentiel_officiel();
        let m = modeles_localites(&r);
        let trouve = |id: &str| m.iter().find(|x| x.id == id).unwrap();
        // 18 486 hab. → grande ville (10 001 à 25 000) ; 1 240 → petite bourgade (201 à 2 000).
        assert_eq!(
            trouve("korvosa").saisie.categorie_officielle,
            "grande-ville"
        );
        assert_eq!(trouve("korvosa").saisie.population, "18486");
        assert_eq!(
            trouve("pointesable").saisie.categorie_officielle,
            "petite-bourgade"
        );
    }

    #[test]
    fn sans_population_la_categorie_reste_a_choisir_et_a_confirmer() {
        let r = regles::referentiel_officiel();
        for x in modeles_localites(&r)
            .iter()
            .filter(|x| x.saisie.population.is_empty())
        {
            assert_eq!(x.saisie.categorie_officielle, "", "{}", x.nom);
            assert!(!x.officiel && x.remarque.is_some(), "{}", x.nom);
        }
    }

    #[test]
    fn incoherences_du_wiki_signalees() {
        let r = regles::referentiel_officiel();
        let m = modeles_localites(&r);
        for id in ["pointesable", "bac-de-la-tortue"] {
            let x = m.iter().find(|x| x.id == id).unwrap();
            assert!(!x.officiel);
            assert!(x.saisie.notes.contains("À confirmer"));
        }
        assert!(m.iter().find(|x| x.id == "magnimar").unwrap().officiel);
    }

    #[test]
    fn libelle_et_notes() {
        let r = regles::referentiel_officiel();
        let m = modeles_localites(&r);
        let k = m.iter().find(|x| x.id == "korvosa").unwrap();
        assert_eq!(k.libelle, "Korvosa — grande ville · 18 486 hab.");
        assert!(k.saisie.notes.contains("Alignement : LN."));
        assert!(k.saisie.notes.contains("Source : wiki Pathfinder-FR"));
    }
}
