//! Objets magiques à bonus chiffré (Manuel des joueurs, chapitre 15) : catalogue sourcé et
//! effets typés. Seuls les objets dont l'effet est un bonus d'un type précis sont décrits ;
//! les autres ne sont pas appliqués aux calculs.

use crate::effet::{Effet, TypeBonus};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Variante {
    pub bonus: i32,
    pub prix_po: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EffetObjet {
    pub cible: String,
    #[serde(rename = "type")]
    pub type_bonus: TypeBonus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObjetMagique {
    pub nom_en: String,
    pub nom: String,
    pub emplacement: Option<String>,
    pub page: u32,
    /// Objet sans niveau de bonus dans son nom (cape elfique…).
    #[serde(default)]
    pub nom_fixe: bool,
    pub variantes: Vec<Variante>,
    pub effets: Vec<EffetObjet>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Deserialize)]
pub struct FichierObjetsMagiques {
    pub livre: String,
    pub objets: Vec<ObjetMagique>,
}

/// Un objet magique précis (avec son niveau de bonus), prêt à être porté.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjetMagiqueInstance {
    /// Nom anglais complet, tel que rangé dans l'équipement (« Cloak of Resistance (+2) »).
    pub nom_en: String,
    /// Nom français (« Cape de résistance (+2) »).
    pub nom: String,
    pub emplacement: Option<String>,
    pub bonus: i32,
    pub prix_po: i64,
    pub source: String,
    pub notes: String,
    pub effets: Vec<Effet>,
}

fn nom_complet(base: &str, bonus: i32, fixe: bool) -> String {
    if fixe {
        base.to_string()
    } else {
        format!("{base} (+{bonus})")
    }
}

impl ObjetMagique {
    fn instancier(&self, v: &Variante, livre: &str) -> ObjetMagiqueInstance {
        let nom = nom_complet(&self.nom, v.bonus, self.nom_fixe);
        ObjetMagiqueInstance {
            nom_en: nom_complet(&self.nom_en, v.bonus, self.nom_fixe),
            emplacement: self.emplacement.clone(),
            bonus: v.bonus,
            prix_po: v.prix_po,
            source: format!("{livre}, p. {}", self.page),
            notes: self.notes.clone(),
            effets: self
                .effets
                .iter()
                .map(|e| Effet::new(&e.cible, e.type_bonus, v.bonus, &nom))
                .collect(),
            nom,
        }
    }
}

/// Catalogue des objets magiques, tous niveaux de bonus confondus.
#[derive(Debug, Clone, Default)]
pub struct CatalogueMagique {
    livre: String,
    objets: Vec<ObjetMagique>,
}

impl CatalogueMagique {
    pub fn depuis_json(json: &str) -> Result<Self, serde_json::Error> {
        let f: FichierObjetsMagiques = serde_json::from_str(json)?;
        Ok(CatalogueMagique {
            livre: f.livre,
            objets: f.objets,
        })
    }

    /// Cherche un objet par son nom anglais complet (« Ring of Protection (+2) »).
    pub fn objet(&self, nom_en: &str) -> Option<ObjetMagiqueInstance> {
        self.toutes_les_instances()
            .find(|i| i.nom_en.eq_ignore_ascii_case(nom_en))
    }

    /// Cherche un objet par son nom français tel qu'écrit sur une feuille, avec ou sans niveau
    /// de bonus (« Cape de résistance +2 », « Bracelets d'armure (+2) »).
    pub fn objet_par_nom_francais(&self, texte: &str) -> Option<ObjetMagiqueInstance> {
        let t = normaliser(texte);
        // Le plus long nom d'abord, pour ne pas confondre « Bandeau d'intelligence » et un préfixe.
        let mut candidats: Vec<&ObjetMagique> = self
            .objets
            .iter()
            .filter(|o| {
                t == normaliser(&o.nom) || t.starts_with(&format!("{} ", normaliser(&o.nom)))
            })
            .collect();
        candidats.sort_by_key(|o| std::cmp::Reverse(o.nom.len()));
        let o = candidats.first()?;
        if o.nom_fixe {
            return o.variantes.first().map(|v| o.instancier(v, &self.livre));
        }
        let reste = &t[normaliser(&o.nom).len()..];
        let bonus: i32 = chercher_bonus(reste)?;
        o.variantes
            .iter()
            .find(|v| v.bonus == bonus)
            .map(|v| o.instancier(v, &self.livre))
    }

    pub fn toutes_les_instances(&self) -> impl Iterator<Item = ObjetMagiqueInstance> + '_ {
        self.objets
            .iter()
            .flat_map(|o| o.variantes.iter().map(|v| o.instancier(v, &self.livre)))
    }

    pub fn nombre(&self) -> usize {
        self.objets.len()
    }
}

/// Premier « +N » ou nombre isolé du texte (« +2 (force) » → 2).
fn chercher_bonus(texte: &str) -> Option<i32> {
    let mut chiffres = String::new();
    let mut vu_plus = false;
    for c in texte.chars() {
        if c == '+' {
            vu_plus = true;
        } else if c.is_ascii_digit() && (vu_plus || !chiffres.is_empty()) {
            chiffres.push(c);
        } else if !chiffres.is_empty() {
            break;
        } else if vu_plus && !c.is_whitespace() && c != '(' {
            vu_plus = false;
        }
    }
    chiffres.parse().ok()
}

/// Minuscules, sans accents ni ponctuation, pour comparer des noms écrits à la main.
pub fn normaliser(s: &str) -> String {
    let mut out = String::new();
    for c in s.to_lowercase().chars() {
        let c = match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            c => c,
        };
        if c.is_alphanumeric() || c == '+' || c == '(' {
            out.push(c);
        } else if c.is_whitespace() || c == '\'' || c == '’' || c == '-' || c == ')' {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Referentiel;

    fn cat() -> &'static CatalogueMagique {
        &Referentiel::partage().objets_magiques
    }

    #[test]
    fn un_bandeau_donne_son_bonus_d_alteration() {
        let o = cat().objet("Headband of Alluring Charisma (+4)").unwrap();
        assert_eq!(o.prix_po, 16000);
        assert_eq!(o.effets.len(), 1);
        assert_eq!(o.effets[0].valeur, 4);
        assert_eq!(o.effets[0].type_bonus, TypeBonus::Alteration);
        assert!(o.source.contains("507"));
    }

    #[test]
    fn les_noms_francais_de_la_feuille_sont_reconnus() {
        for (texte, attendu) in [
            (
                "Bandeau de belle allure +4 (charisme) 16k po",
                "Headband of Alluring Charisma (+4)",
            ),
            (
                "Ceinturon de force de géant +2 (force)",
                "Belt of Giant Strength (+2)",
            ),
            (
                "Anneau de protection +2 (armure) 8k po",
                "Ring of Protection (+2)",
            ),
            ("Bracelets d'armure +2 (armure)", "Bracers of Armor (+2)"),
            (
                "Amulette d'armure naturelle +2 (armure naturelle)",
                "Amulet of Natural Armor (+2)",
            ),
            (
                "Cape elfique +5 discrétion quand cape 2,5k po",
                "Cloak of Elvenkind",
            ),
        ] {
            assert_eq!(
                cat().objet_par_nom_francais(texte).map(|o| o.nom_en),
                Some(attendu.to_string()),
                "{texte}"
            );
        }
        // Bonus inexistant (pas de bandeau +3) ou objet inconnu : rien n'est deviné.
        assert!(cat()
            .objet_par_nom_francais("Bandeau de belle allure +3")
            .is_none());
        assert!(cat()
            .objet_par_nom_francais("Tunique serpent +2 dext")
            .is_none());
    }

    #[test]
    fn les_prix_suivent_la_formule_du_livre_pour_les_bonus_de_caracteristique() {
        // Bonus de caractéristique unique : 1 000 po × bonus².
        for (nom, bonus) in [("Belt of Giant Strength", 2), ("Belt of Giant Strength", 6)] {
            let o = cat().objet(&format!("{nom} (+{bonus})")).unwrap();
            assert_eq!(o.prix_po, 1000 * (bonus as i64) * (bonus as i64));
        }
    }
}
