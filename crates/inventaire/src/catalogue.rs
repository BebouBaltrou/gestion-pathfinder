//! Import du catalogue depuis des fichiers texte : un fichier par tableau du livre, avec sa
//! source. Les données restent en dehors du code (`data/catalogue/`).
//!
//! Format, une ligne par objet, cellules séparées par `|` :
//! - `#source=...` : tableau et page d'origine, recopiés dans le champ `source` de chaque modèle ;
//! - `#type=...` : type d'objet par défaut ;
//! - `#prefixe=...` : préfixe des identifiants (`prefixe:nom-de-l-objet`) ;
//! - `#colonnes=nom|prix|poids|...` : intitulé des cellules. `nom`, `prix` et `poids` sont lus ;
//!   les autres deviennent des propriétés (une cellule vide ou « — » est omise) ;
//! - `@cle=valeur` : propriété ajoutée à toutes les lignes suivantes (catégorie, groupe…).
//!   La clé `type_objet` change le type des lignes suivantes ;
//! - toute autre ligne commençant par `#` est un commentaire.
//!
//! Prix et poids : « — » dans la colonne prix ou « spécial » = non précisé ; « — » dans la
//! colonne poids = négligeable (zéro) ; « spécial » = non précisé.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use domaine::Montant;

use crate::{Modele, Poids};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErreurCatalogue {
    pub ligne: usize,
    pub message: String,
}

impl fmt::Display for ErreurCatalogue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ligne {} : {}", self.ligne, self.message)
    }
}

impl std::error::Error for ErreurCatalogue {}

/// Lit un prix : « 2 po », « 5 pa », « 1 pc », « 1 000 po », « +50 po ». `None` = non précisé.
pub fn lire_valeur(texte: &str) -> Result<Option<Montant>, String> {
    let t = texte.trim().trim_start_matches('+').trim();
    if t.is_empty() || t == "—" || t.eq_ignore_ascii_case("spécial") {
        return Ok(None);
    }
    let (unite, facteur) = [("pc", 1), ("pa", 10), ("po", 100), ("pp", 1000)]
        .into_iter()
        .find(|(u, _)| t.ends_with(u))
        .ok_or_else(|| format!("prix illisible : « {texte} »"))?;
    let nombre: String = t[..t.len() - unite.len()]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    let n: i64 = nombre
        .parse()
        .map_err(|_| format!("prix illisible : « {texte} »"))?;
    Ok(Some(Montant::pc(n * facteur)))
}

/// Lit un poids : « 500 g », « 1,5 kg », « +2,5 kg ». « — » = négligeable (zéro),
/// « spécial » ou vide = non précisé.
pub fn lire_poids(texte: &str) -> Result<Option<Poids>, String> {
    let t = texte.trim().trim_start_matches('+').trim();
    if t == "—" {
        return Ok(Some(Poids::ZERO));
    }
    if t.is_empty() || t.eq_ignore_ascii_case("spécial") {
        return Ok(None);
    }
    let erreur = || format!("poids illisible : « {texte} »");
    let (nombre, facteur) = if let Some(n) = t.strip_suffix("kg") {
        (n.trim(), 1000)
    } else if let Some(n) = t.strip_suffix('g') {
        (n.trim(), 1)
    } else {
        return Err(erreur());
    };
    let (entier, fraction) = nombre.split_once(',').unwrap_or((nombre, ""));
    let entier: i64 = entier.parse().map_err(|_| erreur())?;
    let mut grammes = entier * facteur;
    if !fraction.is_empty() {
        // Au plus trois décimales en kg (1 g de précision), aucune en g.
        let chiffres = if facteur == 1000 { 3 } else { 0 };
        if fraction.len() > chiffres || !fraction.chars().all(|c| c.is_ascii_digit()) {
            return Err(erreur());
        }
        let mut complet = fraction.to_string();
        while complet.len() < chiffres {
            complet.push('0');
        }
        grammes += complet.parse::<i64>().map_err(|_| erreur())?;
    }
    Ok(Some(Poids(grammes)))
}

fn identifiant(nom: &str) -> String {
    let mut slug = String::new();
    for c in nom.chars() {
        if c.is_alphanumeric() {
            slug.extend(c.to_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_string()
}

/// Charge un fichier de catalogue.
pub fn charger(texte: &str) -> Result<Vec<Modele>, ErreurCatalogue> {
    let mut source = String::new();
    let mut type_defaut = String::new();
    let mut prefixe = String::new();
    let mut colonnes: Vec<String> = Vec::new();
    let mut contexte: BTreeMap<String, String> = BTreeMap::new();
    let mut deja: HashMap<String, u32> = HashMap::new();
    let mut modeles = Vec::new();

    for (index, brut) in texte.lines().enumerate() {
        let ligne = index + 1;
        let erreur = |message: String| ErreurCatalogue { ligne, message };
        let l = brut.trim();
        if l.is_empty() {
            continue;
        }
        if let Some(v) = l.strip_prefix("#source=") {
            source = v.trim().to_string();
        } else if let Some(v) = l.strip_prefix("#type=") {
            type_defaut = v.trim().to_string();
        } else if let Some(v) = l.strip_prefix("#prefixe=") {
            prefixe = v.trim().to_string();
        } else if let Some(v) = l.strip_prefix("#colonnes=") {
            colonnes = v.split('|').map(|c| c.trim().to_string()).collect();
        } else if l.starts_with('#') {
            continue;
        } else if let Some(v) = l.strip_prefix('@') {
            let (cle, valeur) = v
                .split_once('=')
                .ok_or_else(|| erreur("directive « @cle=valeur » attendue".into()))?;
            contexte.insert(cle.trim().to_string(), valeur.trim().to_string());
        } else {
            if source.is_empty() || prefixe.is_empty() || colonnes.is_empty() {
                return Err(erreur(
                    "#source, #prefixe et #colonnes doivent précéder les lignes".into(),
                ));
            }
            let cellules: Vec<&str> = l.split('|').map(str::trim).collect();
            if cellules.len() != colonnes.len() {
                return Err(erreur(format!(
                    "{} cellules au lieu de {}",
                    cellules.len(),
                    colonnes.len()
                )));
            }
            let mut nom = "";
            let mut valeur = None;
            let mut poids = None;
            let mut proprietes = contexte.clone();
            for (colonne, cellule) in colonnes.iter().zip(&cellules) {
                match colonne.as_str() {
                    "nom" => nom = cellule,
                    "prix" => valeur = lire_valeur(cellule).map_err(erreur)?,
                    "poids" => poids = lire_poids(cellule).map_err(erreur)?,
                    _ if !cellule.is_empty() && *cellule != "—" => {
                        proprietes.insert(colonne.clone(), (*cellule).to_string());
                    }
                    _ => {}
                }
            }
            if nom.is_empty() {
                return Err(erreur("nom vide".into()));
            }
            let type_objet = proprietes
                .remove("type_objet")
                .unwrap_or_else(|| type_defaut.clone());
            let racine = format!("{prefixe}:{}", identifiant(nom));
            let n = deja.entry(racine.clone()).or_insert(0);
            *n += 1;
            let id = if *n == 1 {
                racine
            } else {
                format!("{racine}-{n}")
            };
            modeles.push(Modele {
                id,
                nom: nom.to_string(),
                type_objet,
                valeur,
                poids,
                source: source.clone(),
                proprietes,
            });
        }
    }
    Ok(modeles)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXEMPLE: &str = "\
#source=Exemple, Table 1
#type=materiel
#prefixe=exemple
#colonnes=nom|prix|poids|note
@section=Matériel
Bougie|1 pc|—|
Corde en chanvre, 15 m|1 po|5 kg|Une note
Bougie|1 pc|—|
Houx et gui|—|—|
Armure spéciale|spécial|spécial|
";

    #[test]
    fn prix_et_poids() {
        assert_eq!(lire_valeur("2 po"), Ok(Some(Montant::pc(200))));
        assert_eq!(lire_valeur("5 pa"), Ok(Some(Montant::pc(50))));
        assert_eq!(lire_valeur("1 pc"), Ok(Some(Montant::pc(1))));
        assert_eq!(lire_valeur("1 500 po"), Ok(Some(Montant::pc(150_000))));
        assert_eq!(lire_valeur("+50 po"), Ok(Some(Montant::pc(5000))));
        assert_eq!(lire_valeur("—"), Ok(None));
        assert_eq!(lire_valeur("spécial"), Ok(None));
        assert!(lire_valeur("beaucoup").is_err());
        assert_eq!(lire_poids("500 g"), Ok(Some(Poids(500))));
        assert_eq!(lire_poids("1,5 kg"), Ok(Some(Poids(1500))));
        assert_eq!(lire_poids("2,25 kg"), Ok(Some(Poids(2250))));
        assert_eq!(lire_poids("+2,5 kg"), Ok(Some(Poids(2500))));
        assert_eq!(lire_poids("—"), Ok(Some(Poids::ZERO)));
        assert_eq!(lire_poids("spécial"), Ok(None));
        assert!(lire_poids("1,5 tonnes").is_err());
    }

    #[test]
    fn ligne_mal_formee_donne_son_numero() {
        let texte = "#source=s\n#prefixe=p\n#colonnes=nom|prix|poids\nA|1 po\n";
        let e = charger(texte).unwrap_err();
        assert_eq!(e.ligne, 4);
    }

    #[test]
    fn chargement_d_un_exemple() {
        let m = charger(EXEMPLE).unwrap();
        assert_eq!(m.len(), 5);
        let corde = &m[1];
        assert_eq!(corde.id, "exemple:corde-en-chanvre-15-m");
        assert_eq!(corde.valeur, Some(Montant::po(1)));
        assert_eq!(corde.poids, Some(Poids(5000)));
        assert_eq!(corde.source, "Exemple, Table 1");
        assert_eq!(corde.proprietes["section"], "Matériel");
        assert_eq!(corde.proprietes["note"], "Une note");
        // Deux lignes du même nom : identifiants distincts.
        assert_eq!(m[0].id, "exemple:bougie");
        assert_eq!(m[2].id, "exemple:bougie-2");
        // « — » en prix = non précisé ; « — » en poids = zéro ; « spécial » = non précisé.
        assert_eq!(m[3].valeur, None);
        assert_eq!(m[3].poids, Some(Poids::ZERO));
        assert_eq!(m[4].poids, None);
    }
}
