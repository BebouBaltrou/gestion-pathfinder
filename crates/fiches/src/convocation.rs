//! Convocation d'alliés naturels (Manuel des joueurs, sorts « Summon Nature's Ally » I à IX) :
//! liste des créatures convocables par niveau, nombre de créatures selon le niveau du sort et
//! remplacement d'un sort préparé par le druide.
//!
//! Les données sont dans `data/core/convocation_allies.json` (statistiques du Bestiaire, saisies
//! par niveau de créature). Le texte du sort : « 1 créature du niveau du sort, 1d3 créatures du
//! niveau inférieur, ou 1d4+1 créatures de deux niveaux inférieurs, toutes du même type ».

use serde_json::Value;
use std::collections::BTreeMap;

/// Ligne « libellé : valeur » d'un bloc de statistiques, telle que saisie.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ligne {
    pub libelle: String,
    pub valeur: String,
}

/// Une créature convocable, avec les nombres utiles à la feuille de combat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Creature {
    pub nom: String,
    /// Facteur de puissance (« 1/2 », « 5 »…).
    pub fp: String,
    /// Niveau de créature (1 à 9) : la liste est rangée par ce niveau.
    pub niveau: u32,
    /// Type de créature (« Animal », « Extérieur (Air, élémentaire…) »…), tel que saisi.
    pub type_creature: String,
    /// Taille (« P », « M », « G »…), lue dans la ligne de type ; vide si absente.
    pub taille: String,
    /// Page du Bestiaire, telle que saisie.
    pub source: String,
    /// Points de vie moyens ou jetés, tels que saisis (premier nombre de « pv »).
    pub pv: i32,
    /// Classe d'armure (valeur normale).
    pub ca: i32,
    /// CA contact et CA pris au dépourvu (saisies dans la ligne CA ; à défaut, la CA normale).
    pub ca_contact: i32,
    pub ca_depourvu: i32,
    /// Modificateur d'initiative (premier nombre de « Init »).
    pub init: i32,
    /// Jets de sauvegarde : « Réf », « Vig », « Vol ».
    pub sauvegardes: BTreeMap<String, i32>,
    /// Caractéristiques : « For », « Dex », « Con », « Int », « Sag », « Cha ».
    pub caracteristiques: BTreeMap<String, i32>,
    /// Texte des attaques (corps à corps et distance), tel que saisi.
    pub attaques: String,
    /// Vitesse de déplacement (« VD »), telle que saisie.
    pub vitesse: String,
    /// Vrai si le bloc contient PV, CA, Init et les trois jets (certaines fiches n'en ont pas).
    pub complete: bool,
    /// Toutes les lignes du bloc, dans l'ordre (affichage complet de la fiche).
    pub lignes: Vec<Ligne>,
}

/// Nombre de créatures convoquées pour un sort donné.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Nombre {
    /// Une seule créature, du niveau du sort.
    Une,
    /// 1d3 créatures, d'un niveau de moins que le sort.
    D3,
    /// 1d4+1 créatures, de deux niveaux de moins que le sort.
    D4Plus1,
}

impl Nombre {
    /// Plage de valeurs possibles (dé inclus), pour valider le résultat du jet.
    pub fn intervalle(self) -> (u32, u32) {
        match self {
            Nombre::Une => (1, 1),
            Nombre::D3 => (1, 3),
            Nombre::D4Plus1 => (2, 5),
        }
    }

    /// Vrai si `n` est un résultat possible de ce nombre.
    pub fn valide(self, n: u32) -> bool {
        let (min, max) = self.intervalle();
        (min..=max).contains(&n)
    }
}

/// Nombre de créatures pour un sort de `niveau_sort` qui convoque une créature de
/// `niveau_creature` (`None` si le niveau de créature ne peut pas être convoqué par ce sort).
pub fn nombre_pour(niveau_sort: u32, niveau_creature: u32) -> Option<Nombre> {
    if niveau_creature == niveau_sort {
        Some(Nombre::Une)
    } else if niveau_creature + 1 == niveau_sort {
        Some(Nombre::D3)
    } else if niveau_creature + 2 == niveau_sort {
        Some(Nombre::D4Plus1)
    } else {
        None
    }
}

/// Niveau du sort « Summon Nature's Ally N » d'après son nom anglais, si c'en est un.
pub fn niveau_de_convocation(nom_en: &str) -> Option<u32> {
    nom_en
        .strip_prefix("Summon Nature's Ally ")
        .and_then(|n| n.trim().parse().ok())
}

/// Liste de convocation complète (toutes les créatures de tous les niveaux).
#[derive(Debug, Clone, Default)]
pub struct Convocations {
    pub creatures: Vec<Creature>,
}

/// Une option proposée au joueur : une créature convocable et le nombre qui va avec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionConvocation {
    pub creature: usize,
    pub nombre: Nombre,
}

impl Convocations {
    /// Lit `data/core/convocation_allies.json`.
    pub fn depuis_json(json: &str) -> Result<Self, String> {
        let v: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        let niveaux = v["niveaux"].as_array().ok_or("« niveaux » manquant")?;
        let mut creatures = Vec::new();
        for niveau in niveaux {
            let n = niveau["niveau"].as_u64().ok_or("niveau sans numéro")? as u32;
            for c in niveau["creatures"].as_array().into_iter().flatten() {
                creatures.push(creature_depuis(c, n));
            }
        }
        Ok(Convocations { creatures })
    }

    /// Options possibles pour un sort de `niveau_sort` : toutes les créatures d'un niveau
    /// convocable, avec le nombre qui va avec.
    pub fn options_pour(&self, niveau_sort: u32) -> Vec<OptionConvocation> {
        self.creatures
            .iter()
            .enumerate()
            .filter(|(_, c)| c.complete)
            .filter_map(|(i, c)| {
                nombre_pour(niveau_sort, c.niveau).map(|nombre| OptionConvocation {
                    creature: i,
                    nombre,
                })
            })
            .collect()
    }
}

fn texte(v: &Value) -> String {
    v.as_str().unwrap_or("").trim().to_string()
}

/// Premier entier signé d'un texte (« +2 ; Sens… » donne 2 ; « 5 (1d8+1) » donne 5).
pub fn premier_entier(s: &str) -> Option<i32> {
    let octets = s.as_bytes();
    let debut = octets
        .iter()
        .position(|b| b.is_ascii_digit() || *b == b'+' || *b == b'-')?;
    let signe = usize::from(octets[debut] == b'+' || octets[debut] == b'-');
    let fin = octets[debut + signe..]
        .iter()
        .position(|b| !b.is_ascii_digit())
        .map_or(octets.len(), |k| debut + signe + k);
    if fin == debut + signe {
        return None;
    }
    s[debut..fin].trim_start_matches('+').parse().ok()
}

/// Premier entier qui suit `mot` dans `s` (« contact 13 » → 13).
fn nombre_apres(s: &str, mot: &str) -> Option<i32> {
    s.find(mot)
        .and_then(|i| premier_entier(&s[i + mot.len()..]))
}

/// Paires « nom valeur » d'une ligne (« Réf +4, Vig +3, Vol +2 » ; « 10, Dex 15 » avec `libelle`
/// pour la première valeur nue).
fn paires(libelle: &str, valeur: &str) -> Vec<(String, i32)> {
    valeur
        .split(',')
        .filter_map(|morceau| {
            let morceau = morceau.trim();
            match morceau.rsplit_once(' ') {
                Some((nom, nb)) => premier_entier(nb).map(|n| (nom.trim().to_string(), n)),
                None => premier_entier(morceau).map(|n| (libelle.to_string(), n)),
            }
        })
        .collect()
}

fn creature_depuis(c: &Value, niveau: u32) -> Creature {
    let mut lignes = Vec::new();
    let mut pv = None;
    let mut ca = None;
    let mut contact = None;
    let mut depourvu = None;
    let mut init = None;
    let mut sauvegardes = BTreeMap::new();
    let mut caracteristiques = BTreeMap::new();
    let mut attaques = Vec::new();
    let mut vitesse = String::new();
    for section in c["sections"].as_array().into_iter().flatten() {
        for l in section["lignes"].as_array().into_iter().flatten() {
            let libelle = texte(&l["label"]);
            let valeur = texte(&l["valeur"]);
            match libelle.as_str() {
                "pv" => pv = premier_entier(&valeur),
                "CA" => {
                    ca = premier_entier(&valeur);
                    contact = nombre_apres(&valeur, "contact");
                    depourvu = nombre_apres(&valeur, "dépourvu");
                }
                "Init" => init = premier_entier(&valeur),
                "Réf" => sauvegardes.extend(paires("Réf", &valeur)),
                "For" => caracteristiques.extend(paires("For", &valeur)),
                "VD" => vitesse = valeur.clone(),
                "Corps à corps" | "Distance" => attaques.push(format!("{libelle} : {valeur}")),
                _ => {}
            }
            // toutes les lignes, y compris celles sans libellé (taille, suites de capacités)
            lignes.push(Ligne { libelle, valeur });
        }
    }
    let complete = pv.is_some()
        && ca.is_some()
        && init.is_some()
        && ["Réf", "Vig", "Vol"]
            .iter()
            .all(|k| sauvegardes.contains_key(*k));
    let type_taille = c["sections"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|s| s["lignes"].as_array().into_iter().flatten())
        .map(|l| texte(&l["valeur"]))
        .find(|v| v.contains(" de taille "))
        .unwrap_or_default();
    let type_creature = type_taille
        .split(" de taille ")
        .next()
        .unwrap_or("")
        .to_string();
    let taille = type_taille
        .split(" de taille ")
        .nth(1)
        .and_then(|t| t.split(|ch: char| ch == ',' || ch.is_whitespace()).next())
        .unwrap_or("")
        .to_string();
    Creature {
        type_creature,
        taille,
        nom: texte(&c["nom"]),
        fp: texte(&c["fp"]),
        niveau,
        source: texte(&c["source"]),
        pv: pv.unwrap_or(0),
        ca: ca.unwrap_or(0),
        ca_contact: contact.or(ca).unwrap_or(0),
        ca_depourvu: depourvu.or(ca).unwrap_or(0),
        init: init.unwrap_or(0),
        sauvegardes,
        caracteristiques,
        attaques: attaques.join(" ; "),
        vitesse,
        complete,
        lignes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &str = include_str!("../../../data/core/convocation_allies.json");

    #[test]
    fn le_nombre_suit_le_texte_du_sort() {
        // Summon Nature's Ally III : 1 de niveau 3, 1d3 de niveau 2, 1d4+1 de niveau 1.
        assert_eq!(nombre_pour(3, 3), Some(Nombre::Une));
        assert_eq!(nombre_pour(3, 2), Some(Nombre::D3));
        assert_eq!(nombre_pour(3, 1), Some(Nombre::D4Plus1));
        assert_eq!(nombre_pour(3, 0), None);
        assert_eq!(nombre_pour(2, 1), Some(Nombre::D3));
        assert_eq!(nombre_pour(1, 1), Some(Nombre::Une));
        assert_eq!(nombre_pour(1, 2), None);
    }

    #[test]
    fn le_jet_est_valide_dans_sa_plage() {
        assert!(Nombre::Une.valide(1) && !Nombre::Une.valide(2));
        assert!(Nombre::D3.valide(3) && !Nombre::D3.valide(4));
        assert!(
            Nombre::D4Plus1.valide(2) && Nombre::D4Plus1.valide(5) && !Nombre::D4Plus1.valide(1)
        );
    }

    #[test]
    fn le_nom_du_sort_donne_son_niveau() {
        assert_eq!(niveau_de_convocation("Summon Nature's Ally 4"), Some(4));
        assert_eq!(niveau_de_convocation("Beast Shape II"), None);
    }

    #[test]
    fn les_nombres_sont_lus_dans_les_statistiques() {
        assert_eq!(premier_entier("5 (1d8+1)"), Some(5));
        assert_eq!(premier_entier("+2 ; Sens vision nocturne"), Some(2));
        assert_eq!(premier_entier("-1, Dex"), Some(-1));
        let c = Convocations::depuis_json(JSON).unwrap();
        let aigle = c.creatures.iter().find(|c| c.nom == "Aigle").unwrap();
        assert_eq!(aigle.niveau, 1);
        assert_eq!(aigle.pv, 5);
        assert_eq!(aigle.ca, 14);
        assert_eq!(aigle.init, 2);
        assert_eq!(aigle.sauvegardes["Réf"], 4);
        assert_eq!(aigle.sauvegardes["Vig"], 3);
        assert_eq!(aigle.sauvegardes["Vol"], 2);
        assert_eq!(aigle.caracteristiques["Dex"], 15);
        assert_eq!(aigle.caracteristiques["For"], 10);
        assert!(aigle.attaques.contains("Corps à corps"));
    }

    #[test]
    fn toutes_les_creatures_ont_des_statistiques_de_base() {
        let c = Convocations::depuis_json(JSON).unwrap();
        assert!(c.creatures.len() > 100);
        let incompletes: Vec<&str> = c
            .creatures
            .iter()
            .filter(|c| !c.complete)
            .map(|c| c.nom.as_str())
            .collect();
        // Dix méphites n'ont que leur nom et leur FP dans les données : à compléter.
        assert_eq!(incompletes.len(), 10);
        assert!(incompletes.iter().all(|n| n.starts_with("Méphite")));
    }

    #[test]
    fn le_type_et_la_taille_sont_lus() {
        let c = Convocations::depuis_json(JSON).unwrap();
        let aigle = c.creatures.iter().find(|c| c.nom == "Aigle").unwrap();
        assert_eq!(aigle.type_creature, "Animal");
        assert_eq!(aigle.taille, "P");
        assert!(c
            .creatures
            .iter()
            .any(|x| x.type_creature.starts_with("Extérieur (Air")));
    }

    #[test]
    fn les_options_suivent_le_niveau_du_sort() {
        let c = Convocations::depuis_json(JSON).unwrap();
        let o = c.options_pour(1);
        assert!(!o.is_empty());
        assert!(o
            .iter()
            .all(|x| c.creatures[x.creature].niveau == 1 && x.nombre == Nombre::Une));
        let o3 = c.options_pour(3);
        assert!(o3.iter().any(|x| x.nombre == Nombre::D4Plus1));
        assert!(o3.iter().any(|x| x.nombre == Nombre::D3));
    }

    #[test]
    fn toutes_les_capacites_de_l_elephant_sont_gardees() {
        let c = Convocations::depuis_json(JSON).unwrap();
        let e = c.creatures.iter().find(|x| x.nom == "Éléphant").unwrap();
        assert!(
            e.lignes
                .iter()
                .any(|l| l.libelle == "Attaques spéciales"
                    && l.valeur.contains("piétinement (2d8+15"))
        );
        assert!(e.lignes.iter().any(|l| l.valeur.contains("TG")));
    }
}
