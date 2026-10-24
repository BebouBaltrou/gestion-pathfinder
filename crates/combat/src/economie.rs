//! Économie d'actions d'un tour (Core Rulebook p. 181-182).
//!
//! - Un tour normal : une action simple et une action de mouvement, ou une action complexe.
//! - On peut toujours remplacer une action simple par une action de mouvement.
//! - Une action rapide par tour ; des actions libres sans limite fixe (limite laissée au MJ).
//! - Un pas de 1,5 m si on ne se déplace pas réellement ce tour-là.
//! - Round de surprise ou personnage chancelant : une seule action simple OU de mouvement, pas
//!   d'action complexe (lecture à valider pour les actions rapides et immédiates).
//! - Action immédiate : sur son tour elle compte comme l'action rapide du tour ; hors de son
//!   tour, elle prend l'action rapide du prochain tour (lecture à valider).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum TypeAction {
    Simple,
    Mouvement,
    Complexe,
    Rapide,
    Immediate,
    Libre,
    /// Ne coûte aucune action (pas de 1,5 m, retarder).
    Aucune,
}

impl TypeAction {
    pub fn nom(self) -> &'static str {
        match self {
            TypeAction::Simple => "action simple",
            TypeAction::Mouvement => "action de mouvement",
            TypeAction::Complexe => "action complexe",
            TypeAction::Rapide => "action rapide",
            TypeAction::Immediate => "action immédiate",
            TypeAction::Libre => "action libre",
            TypeAction::Aucune => "aucune action",
        }
    }
}

/// Ce qui a été dépensé dans le tour du personnage.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EconomieTour {
    pub simple: bool,
    pub mouvement: bool,
    pub rapide: bool,
    /// Un pas de 1,5 m a été fait.
    pub pas: bool,
    /// Un déplacement réel a été fait.
    pub deplace: bool,
}

/// Ce que la dépense a changé, à noter dans le journal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depense {
    Normale,
    /// Une action simple a servi de mouvement.
    ConvertiEnMouvement,
}

/// Ce qu'il reste à dépenser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reste {
    pub simple: bool,
    pub mouvement: bool,
    pub complexe: bool,
    pub rapide: bool,
    pub pas: bool,
}

impl EconomieTour {
    /// `limite` : round de surprise ou personnage chancelant (une seule action simple ou de mouvement).
    pub fn reste(&self, limite: bool) -> Reste {
        let aucune_prise = !self.simple && !self.mouvement;
        let une_prise = aucune_prise || !limite;
        Reste {
            simple: !self.simple && (!limite || aucune_prise),
            mouvement: (!self.mouvement || !self.simple) && une_prise,
            complexe: !limite && aucune_prise,
            rapide: !self.rapide,
            pas: !self.pas && !self.deplace,
        }
    }

    /// Dépense une action. `deplacement` : l'action déplace réellement le personnage.
    pub fn depenser(
        &mut self,
        t: TypeAction,
        deplacement: bool,
        limite: bool,
    ) -> Result<Depense, String> {
        if deplacement && self.pas {
            return Err(
                "un pas de 1,5 m a déjà été fait : pas d'autre déplacement ce tour-ci".into(),
            );
        }
        let r = self.reste(limite);
        let mut depense = Depense::Normale;
        match t {
            TypeAction::Libre | TypeAction::Aucune => {}
            TypeAction::Simple => {
                if !r.simple {
                    return Err(refus("action simple", limite));
                }
                self.simple = true;
                if limite {
                    self.mouvement = true; // une seule action : l'autre est perdue
                }
            }
            TypeAction::Mouvement => {
                if !r.mouvement {
                    return Err(refus("action de mouvement", limite));
                }
                if !self.mouvement {
                    self.mouvement = true;
                } else {
                    self.simple = true;
                    depense = Depense::ConvertiEnMouvement;
                }
                if limite {
                    self.simple = true;
                    self.mouvement = true;
                }
            }
            TypeAction::Complexe => {
                if !r.complexe {
                    return Err(if limite {
                        "pas d'action complexe dans un round de surprise ou en étant chancelant"
                            .into()
                    } else {
                        "une action complexe demande l'action simple et l'action de mouvement du tour".into()
                    });
                }
                self.simple = true;
                self.mouvement = true;
            }
            TypeAction::Rapide | TypeAction::Immediate => {
                if !r.rapide {
                    return Err("l'action rapide du tour est déjà utilisée".into());
                }
                self.rapide = true;
            }
        }
        if t == TypeAction::Aucune && deplacement {
            // le pas de 1,5 m : aucun autre déplacement ce tour-ci
            if self.deplace {
                return Err("un déplacement a déjà eu lieu : pas de pas de 1,5 m".into());
            }
            self.pas = true;
        } else if deplacement {
            self.deplace = true;
        }
        Ok(depense)
    }
}

fn refus(quoi: &str, limite: bool) -> String {
    if limite {
        format!("{quoi} impossible : une seule action simple ou de mouvement par round (surprise ou chancelant)")
    } else {
        format!("{quoi} déjà dépensée ce tour-ci")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_puis_mouvement() {
        let mut e = EconomieTour::default();
        e.depenser(TypeAction::Simple, false, false).unwrap();
        assert!(e.depenser(TypeAction::Simple, false, false).is_err());
        e.depenser(TypeAction::Mouvement, true, false).unwrap();
        assert!(e.depenser(TypeAction::Mouvement, false, false).is_err());
        assert!(e.depenser(TypeAction::Complexe, false, false).is_err());
    }

    #[test]
    fn deux_mouvements_en_remplacant_l_action_simple() {
        let mut e = EconomieTour::default();
        e.depenser(TypeAction::Mouvement, true, false).unwrap();
        assert_eq!(
            e.depenser(TypeAction::Mouvement, true, false).unwrap(),
            Depense::ConvertiEnMouvement
        );
        assert!(e.depenser(TypeAction::Simple, false, false).is_err());
    }

    #[test]
    fn action_complexe_prend_tout() {
        let mut e = EconomieTour::default();
        e.depenser(TypeAction::Complexe, false, false).unwrap();
        assert!(e.depenser(TypeAction::Simple, false, false).is_err());
        assert!(e.depenser(TypeAction::Mouvement, false, false).is_err());
        e.depenser(TypeAction::Rapide, false, false).unwrap(); // l'action rapide reste possible
        e.depenser(TypeAction::Libre, false, false).unwrap();
        let mut e = EconomieTour::default();
        e.depenser(TypeAction::Simple, false, false).unwrap();
        assert!(e.depenser(TypeAction::Complexe, false, false).is_err());
    }

    #[test]
    fn une_action_rapide_par_tour() {
        let mut e = EconomieTour::default();
        e.depenser(TypeAction::Rapide, false, false).unwrap();
        assert!(e.depenser(TypeAction::Rapide, false, false).is_err());
        assert!(e.depenser(TypeAction::Immediate, false, false).is_err());
    }

    #[test]
    fn pas_de_1m5_et_deplacement_s_excluent() {
        let mut e = EconomieTour::default();
        e.depenser(TypeAction::Aucune, true, false).unwrap(); // le pas
        assert!(e.depenser(TypeAction::Aucune, true, false).is_err()); // un seul
        assert!(e.depenser(TypeAction::Mouvement, true, false).is_err());
        e.depenser(TypeAction::Complexe, false, false).unwrap(); // pas autour d'une attaque à outrance
        let mut e = EconomieTour::default();
        e.depenser(TypeAction::Mouvement, true, false).unwrap();
        assert!(e.depenser(TypeAction::Aucune, true, false).is_err());
        assert!(!e.reste(false).pas);
    }

    #[test]
    fn surprise_ou_chancelant_une_seule_action() {
        let mut e = EconomieTour::default();
        assert!(!e.reste(true).complexe);
        e.depenser(TypeAction::Simple, false, true).unwrap();
        assert!(e.depenser(TypeAction::Mouvement, false, true).is_err());
        assert!(e.depenser(TypeAction::Simple, false, true).is_err());
        e.depenser(TypeAction::Libre, false, true).unwrap();
        let mut e = EconomieTour::default();
        e.depenser(TypeAction::Mouvement, true, true).unwrap();
        assert!(e.depenser(TypeAction::Simple, false, true).is_err());
        assert!(e.depenser(TypeAction::Complexe, false, true).is_err());
    }
}
