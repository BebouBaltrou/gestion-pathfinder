//! Montures et barde, d'après le Manuel des joueurs (VF) :
//! table 7-9 « Montures et véhicules » (p. 173-174) et table 6-8 « Barde » (p. 163).
//!
//! Les plages entre parenthèses de la table 7-9 sont les bornes des charges intermédiaire et
//! lourde de la monture (confirmé par Antoine) : vitesse réduite dans la plage. Au-delà de la
//! borne haute, la table ne dit rien : on renvoie `None`.

/// Barde de monture qui ralentit (table 6-8 du Manuel). Une barde légère ne ralentit pas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Barde {
    Intermediaire,
    Lourde,
}

/// Genre d'une barde : celui de l'armure dont elle est tirée (Manuel p. 162 : « il existe presque
/// autant de bardes que d'armures », table 6-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenreBarde {
    Legere,
    Intermediaire,
    Lourde,
}

impl GenreBarde {
    /// La barde qui ralentit la monture, s'il y en a une.
    pub fn ralentissante(self) -> Option<Barde> {
        match self {
            GenreBarde::Legere => None,
            GenreBarde::Intermediaire => Some(Barde::Intermediaire),
            GenreBarde::Lourde => Some(Barde::Lourde),
        }
    }
}

/// Monture de la table 7-9 : vitesse normale et vitesse réduite (par heure et par jour, km).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Monture {
    pub nom: &'static str,
    /// Créature de taille G (cheval) ou M (poney, chien de selle) : prix et poids de sa barde.
    pub grande: bool,
    /// Vitesse de base, en mètres par round : les colonnes 12, 15 et 18 m de la table 6-8
    /// (lecture à valider : poney et chien, cheval lourd, cheval léger).
    pub vitesse_base_m: u32,
    pub vitesse_km_h: f64,
    pub vitesse_km_jour: f64,
    /// Bornes (kg, incluses) des charges intermédiaire et lourde : vitesse réduite dans la plage.
    pub plage_reduite_kg: (f64, f64),
    pub vitesse_reduite_km_h: f64,
    pub vitesse_reduite_km_jour: f64,
}

/// Vitesse d'une monture selon sa charge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VitesseMonture {
    pub km_h: f64,
    pub km_jour: f64,
    pub reduite: bool,
}

pub const MONTURES: &[Monture] = &[
    Monture {
        nom: "Cheval léger",
        grande: true,
        vitesse_base_m: 18,
        vitesse_km_h: 7.5,
        vitesse_km_jour: 60.0,
        plage_reduite_kg: (87.5, 262.5),
        vitesse_reduite_km_h: 5.25,
        vitesse_reduite_km_jour: 42.0,
    },
    Monture {
        nom: "Cheval lourd",
        grande: true,
        vitesse_base_m: 15,
        vitesse_km_h: 7.5,
        vitesse_km_jour: 60.0,
        plage_reduite_kg: (114.5, 345.0),
        vitesse_reduite_km_h: 5.25,
        vitesse_reduite_km_jour: 42.0,
    },
    Monture {
        nom: "Poney",
        grande: false,
        vitesse_base_m: 12,
        vitesse_km_h: 6.0,
        vitesse_km_jour: 48.0,
        plage_reduite_kg: (75.5, 225.0),
        vitesse_reduite_km_h: 4.5,
        vitesse_reduite_km_jour: 36.0,
    },
    Monture {
        nom: "Chien de selle",
        grande: false,
        vitesse_base_m: 12,
        vitesse_km_h: 6.0,
        vitesse_km_jour: 48.0,
        plage_reduite_kg: (50.5, 150.0),
        vitesse_reduite_km_h: 4.5,
        vitesse_reduite_km_jour: 36.0,
    },
];

/// Vitesse de déplacement d'une monture selon la charge portée (kg).
/// `None` : charge au-delà de la plage de la table, pas de valeur dans le Manuel.
pub fn vitesse_monture(monture: &Monture, charge_kg: f64) -> Option<VitesseMonture> {
    let (basse, haute) = monture.plage_reduite_kg;
    if charge_kg > haute {
        None
    } else if charge_kg >= basse {
        Some(VitesseMonture {
            km_h: monture.vitesse_reduite_km_h,
            km_jour: monture.vitesse_reduite_km_jour,
            reduite: true,
        })
    } else {
        Some(VitesseMonture {
            km_h: monture.vitesse_km_h,
            km_jour: monture.vitesse_km_jour,
            reduite: false,
        })
    }
}

/// Vitesse de base (mètres par round) d'une créature en barde, table 6-8 du Manuel.
/// Seules les vitesses 12, 15 et 18 m sont dans la table ; les autres renvoient `None`.
pub fn vitesse_avec_barde(vitesse_base_m: u32, barde: Option<Barde>) -> Option<f64> {
    let sans = f64::from(vitesse_base_m);
    match barde {
        None => Some(sans),
        Some(_) => match vitesse_base_m {
            12 => Some(9.0),
            15 => Some(10.5),
            18 => Some(12.0),
            _ => None,
        },
    }
}

/// Catégorie de charge portée par une monture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategorieCharge {
    Legere,
    Intermediaire,
    Lourde,
    /// Au-delà de la charge lourde : la table ne dit rien ; le dépassement reste permis.
    Depassement,
}

/// Seuils de charge d'une monture, en kg : jusqu'à (légère, intermédiaire, lourde).
///
/// La table 7-9 donne le début de la charge intermédiaire (borne basse) et la fin de la charge
/// lourde (borne haute) : c'est la charge des quadrupèdes. Lecture à valider : la limite entre
/// intermédiaire et lourde suit les proportions de la table 7-4 (un tiers, deux tiers, tout).
pub fn seuils_charge_monture(m: &Monture) -> (f64, f64, f64) {
    let (basse, haute) = m.plage_reduite_kg;
    (basse, haute * 2.0 / 3.0, haute)
}

/// Catégorie de la charge (kg) portée par une monture.
pub fn categorie_charge_monture(m: &Monture, charge_kg: f64) -> CategorieCharge {
    let (legere, intermediaire, lourde) = seuils_charge_monture(m);
    if charge_kg <= legere {
        CategorieCharge::Legere
    } else if charge_kg <= intermediaire {
        CategorieCharge::Intermediaire
    } else if charge_kg <= lourde {
        CategorieCharge::Lourde
    } else {
        CategorieCharge::Depassement
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cheval_leger() -> &'static Monture {
        &MONTURES[0]
    }

    #[test]
    fn cheval_leger_sans_charge_lourde_garde_sa_vitesse() {
        let v = vitesse_monture(cheval_leger(), 50.0).unwrap();
        assert_eq!((v.km_h, v.km_jour, v.reduite), (7.5, 60.0, false));
    }

    #[test]
    fn cheval_leger_charge_moyenne_vitesse_reduite() {
        let v = vitesse_monture(cheval_leger(), 100.0).unwrap();
        assert_eq!((v.km_h, v.km_jour, v.reduite), (5.25, 42.0, true));
    }

    #[test]
    fn au_dela_de_la_plage_pas_de_valeur() {
        assert_eq!(vitesse_monture(cheval_leger(), 300.0), None);
    }

    #[test]
    fn poney_bornes_incluses() {
        let poney = &MONTURES[2];
        assert!(vitesse_monture(poney, 75.5).unwrap().reduite);
        assert!(vitesse_monture(poney, 225.0).unwrap().reduite);
    }

    #[test]
    fn barde_table_6_8() {
        assert_eq!(vitesse_avec_barde(12, None), Some(12.0));
        assert_eq!(
            vitesse_avec_barde(15, Some(Barde::Intermediaire)),
            Some(10.5)
        );
        assert_eq!(vitesse_avec_barde(18, Some(Barde::Lourde)), Some(12.0));
        assert_eq!(vitesse_avec_barde(9, Some(Barde::Lourde)), None);
    }
}

#[cfg(test)]
mod tests_charge {
    use super::*;

    #[test]
    fn seuils_du_cheval_leger_d_apres_la_table_7_9() {
        let (legere, intermediaire, lourde) = seuils_charge_monture(&MONTURES[0]);
        assert_eq!((legere, intermediaire, lourde), (87.5, 175.0, 262.5));
    }

    #[test]
    fn categories_et_depassement_sans_blocage() {
        let cheval = &MONTURES[0];
        assert_eq!(
            categorie_charge_monture(cheval, 50.0),
            CategorieCharge::Legere
        );
        assert_eq!(
            categorie_charge_monture(cheval, 100.0),
            CategorieCharge::Intermediaire
        );
        assert_eq!(
            categorie_charge_monture(cheval, 200.0),
            CategorieCharge::Lourde
        );
        assert_eq!(
            categorie_charge_monture(cheval, 300.0),
            CategorieCharge::Depassement
        );
    }

    #[test]
    fn la_vitesse_baisse_des_la_charge_intermediaire() {
        let poney = &MONTURES[2];
        assert!(!vitesse_monture(poney, 50.0).unwrap().reduite);
        assert!(vitesse_monture(poney, 100.0).unwrap().reduite);
    }
}
