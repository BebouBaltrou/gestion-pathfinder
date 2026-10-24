//! Consommables, objets à charges et ressources à utilisations quotidiennes.
//!
//! Règles du Core relevées sur Archives of Nethys (voir `docs/POINTS_A_CONFIRMER_OBJETS.md`) :
//! potion (p. 477), parchemin (p. 490), bâton (p. 491), baguette (p. 496).

use crate::etat::ErreurAventure;

pub type IdObjet = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum GenreObjet {
    Potion,
    Parchemin,
    /// 50 charges au plus, un sort de niveau 4 ou moins (p. 496).
    Baguette,
    /// 10 charges au plus, un sort de tout niveau (p. 491).
    Baton,
    /// Objet à charges ou à utilisations quotidiennes sans sort du catalogue.
    Charges,
    /// Flèches, carreaux, billes… : on décompte les tirs.
    Munition,
    Autre,
}

impl GenreObjet {
    pub fn nom(self) -> &'static str {
        match self {
            GenreObjet::Potion => "potion",
            GenreObjet::Parchemin => "parchemin",
            GenreObjet::Baguette => "baguette",
            GenreObjet::Baton => "bâton",
            GenreObjet::Charges => "objet à charges",
            GenreObjet::Munition => "munition",
            GenreObjet::Autre => "objet",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Charges {
    pub restantes: u32,
    pub max: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Consommable {
    pub nom: String,
    pub genre: GenreObjet,
    /// Sort contenu (nom anglais du catalogue).
    #[cfg_attr(feature = "serde", serde(default))]
    pub sort: Option<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub niveau_du_sort: u32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub niveau_de_lanceur: u32,
    /// Parchemin : sort profane (vrai) ou divin (faux).
    #[cfg_attr(feature = "serde", serde(default))]
    pub arcanique: bool,
    /// Nombre d'exemplaires (potions, parchemins, munitions) ; 1 pour les autres objets.
    pub quantite: u32,
    /// Charges (baguette, bâton, objet à charges) ; ne se recharge pas au repos.
    #[cfg_attr(feature = "serde", serde(default))]
    pub charges: Option<Charges>,
    /// Utilisations par jour, rendues par une nuit de repos complète.
    #[cfg_attr(feature = "serde", serde(default))]
    pub par_jour: Option<Charges>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub notes: String,
}

impl Consommable {
    /// Vérifie ce que le Core impose à la création de l'objet.
    pub fn verifier(&self) -> Result<(), ErreurAventure> {
        let ko = |m: &str| Err(ErreurAventure::Invalide(m.to_string()));
        if self.nom.trim().is_empty() {
            return ko("objet sans nom");
        }
        if self.quantite == 0 {
            return ko("quantité à zéro");
        }
        match self.genre {
            GenreObjet::Potion
            | GenreObjet::Parchemin
            | GenreObjet::Baguette
            | GenreObjet::Baton
                if self.sort.is_none() =>
            {
                return ko("ce genre d'objet contient un sort");
            }
            _ => {}
        }
        match self.genre {
            GenreObjet::Potion if self.niveau_du_sort > 3 => {
                return ko("une potion contient un sort de niveau 3 ou moins (Core p. 477)")
            }
            GenreObjet::Baguette if self.niveau_du_sort > 4 => {
                return ko("une baguette contient un sort de niveau 4 ou moins (Core p. 496)")
            }
            GenreObjet::Baton if self.niveau_de_lanceur < 8 => {
                return ko("le niveau de lanceur minimum d'un bâton est 8 (Core p. 491)")
            }
            _ => {}
        }
        let plafond = match self.genre {
            GenreObjet::Baguette => Some(50),
            GenreObjet::Baton => Some(10),
            _ => None,
        };
        if let (Some(p), Some(c)) = (plafond, self.charges) {
            if c.max > p {
                return ko(&format!(
                    "un(e) {} a {p} charges au plus (Core)",
                    self.genre.nom()
                ));
            }
        }
        if matches!(self.genre, GenreObjet::Baguette | GenreObjet::Baton) && self.charges.is_none()
        {
            return ko("une baguette ou un bâton a des charges");
        }
        for c in [self.charges, self.par_jour].into_iter().flatten() {
            if c.restantes > c.max {
                return ko("plus de charges restantes que le maximum");
            }
        }
        Ok(())
    }

    /// Dépense l'objet : un exemplaire, des charges, ou une utilisation quotidienne.
    pub fn depenser(&mut self, charges: u32) -> Result<(), ErreurAventure> {
        let ko = |m: String| Err(ErreurAventure::Invalide(m));
        if let Some(j) = &mut self.par_jour {
            if j.restantes == 0 {
                return ko(format!("{} : plus d'utilisation aujourd'hui", self.nom));
            }
            j.restantes -= 1;
        }
        match self.genre {
            GenreObjet::Baguette | GenreObjet::Baton | GenreObjet::Charges => {
                if let Some(c) = &mut self.charges {
                    let cout = charges.max(1);
                    if c.restantes < cout {
                        return ko(format!(
                            "{} : {} charge(s) demandée(s), {} restante(s)",
                            self.nom, cout, c.restantes
                        ));
                    }
                    c.restantes -= cout;
                }
            }
            GenreObjet::Potion | GenreObjet::Parchemin | GenreObjet::Munition => {
                if self.quantite < charges.max(1) {
                    return ko(format!("{} : plus assez d'exemplaires", self.nom));
                }
                self.quantite -= charges.max(1);
            }
            GenreObjet::Autre => {}
        }
        Ok(())
    }

    /// Plus rien à utiliser : exemplaires épuisés (les objets à charges restent, vides).
    pub fn epuise(&self) -> bool {
        matches!(
            self.genre,
            GenreObjet::Potion | GenreObjet::Parchemin | GenreObjet::Munition
        ) && self.quantite == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum UniteRessource {
    Utilisations,
    Rounds,
    Minutes,
}

/// Une ressource à usages quotidiens (pouvoir de lignage, objet…), rendue par une nuit complète de repos.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ressource {
    pub unite: UniteRessource,
    pub max: u32,
    pub utilisees: u32,
    /// D'où elle vient (« lignage aberrant, Acidic Ray (niveau 1) »).
    pub source: String,
}

impl Ressource {
    pub fn restantes(&self) -> u32 {
        self.max.saturating_sub(self.utilisees)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baguette(max: u32) -> Consommable {
        Consommable {
            nom: "Baguette de projectiles magiques".into(),
            genre: GenreObjet::Baguette,
            sort: Some("Magic Missile".into()),
            niveau_du_sort: 1,
            niveau_de_lanceur: 1,
            arcanique: true,
            quantite: 1,
            charges: Some(Charges {
                restantes: max,
                max,
            }),
            par_jour: None,
            notes: String::new(),
        }
    }

    #[test]
    fn limites_du_core() {
        assert!(baguette(50).verifier().is_ok());
        assert!(baguette(51).verifier().is_err());
        let mut b = baguette(10);
        b.niveau_du_sort = 5;
        assert!(b.verifier().is_err());
        let mut s = baguette(10);
        s.genre = GenreObjet::Baton;
        s.niveau_du_sort = 6;
        assert!(s.verifier().is_err()); // niveau de lanceur 1 < 8
        s.niveau_de_lanceur = 8;
        assert!(s.verifier().is_ok());
        s.charges = Some(Charges {
            restantes: 11,
            max: 11,
        });
        assert!(s.verifier().is_err());
        let mut p = baguette(1);
        p.genre = GenreObjet::Potion;
        p.charges = None;
        p.niveau_du_sort = 4;
        assert!(p.verifier().is_err());
    }

    #[test]
    fn depenser_charges_et_exemplaires() {
        let mut b = baguette(2);
        b.depenser(1).unwrap();
        b.depenser(1).unwrap();
        assert!(b.depenser(1).is_err());
        let mut parchemin = Consommable {
            genre: GenreObjet::Parchemin,
            charges: None,
            quantite: 2,
            ..baguette(1)
        };
        parchemin.depenser(1).unwrap();
        assert!(!parchemin.epuise());
        parchemin.depenser(1).unwrap();
        assert!(parchemin.epuise());
        assert!(parchemin.depenser(1).is_err());
    }

    #[test]
    fn utilisations_quotidiennes() {
        let mut o = Consommable {
            genre: GenreObjet::Charges,
            sort: None,
            charges: None,
            par_jour: Some(Charges {
                restantes: 1,
                max: 1,
            }),
            ..baguette(1)
        };
        o.depenser(1).unwrap();
        assert!(o.depenser(1).is_err());
    }
}
