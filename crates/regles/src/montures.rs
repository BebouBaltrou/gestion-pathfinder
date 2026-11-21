//! Montures et barde, d'après le Manuel des joueurs (VF) :
//! table 7-9 « Montures et véhicules » (p. 173-174) et table 6-8 « Barde » (p. 163).
//!
//! Les plages entre parenthèses de la table 7-9 sont les bornes des charges intermédiaire et
//! lourde de la monture (confirmé par Antoine) : vitesse réduite dans la plage. Au-delà de la
//! borne haute, la table ne dit rien : on renvoie `None`.

/// Barde de monture (armure), table 6-8 du Manuel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Barde {
    Intermediaire,
    Lourde,
}

/// Monture de la table 7-9 : vitesse normale et vitesse réduite (par heure et par jour, km).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Monture {
    pub nom: &'static str,
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
        vitesse_km_h: 7.5,
        vitesse_km_jour: 60.0,
        plage_reduite_kg: (87.5, 262.5),
        vitesse_reduite_km_h: 5.25,
        vitesse_reduite_km_jour: 42.0,
    },
    Monture {
        nom: "Cheval lourd",
        vitesse_km_h: 7.5,
        vitesse_km_jour: 60.0,
        plage_reduite_kg: (114.5, 345.0),
        vitesse_reduite_km_h: 5.25,
        vitesse_reduite_km_jour: 42.0,
    },
    Monture {
        nom: "Poney",
        vitesse_km_h: 6.0,
        vitesse_km_jour: 48.0,
        plage_reduite_kg: (75.5, 225.0),
        vitesse_reduite_km_h: 4.5,
        vitesse_reduite_km_jour: 36.0,
    },
    Monture {
        nom: "Chien de selle",
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

/// Seuils de charge (kg) par Force, table « Charge transportable » p. 171 du Manuel (VF).
/// Entrée : (Force, charge légère max, charge intermédiaire max, charge lourde max).
/// Pour une Force au-delà de 29 : on prend la Force moins 10 et on multiplie par 4, à chaque +10.
/// Lecture à valider pour les montures : la table est celle des personnages ; le Manuel ne donne
/// pas leur coefficient, et leur Force n'est pas encore connue (à renseigner).
const SEUILS_CHARGE: &[(u8, f64, f64, f64)] = &[
    (1, 1.5, 3.0, 5.0),
    (2, 3.0, 6.5, 10.0),
    (3, 5.0, 10.0, 15.0),
    (4, 6.5, 13.0, 20.0),
    (5, 8.0, 16.5, 25.0),
    (6, 10.0, 20.0, 30.0),
    (7, 11.5, 23.0, 35.0),
    (8, 13.0, 26.5, 40.0),
    (9, 15.0, 30.0, 45.0),
    (10, 16.5, 33.0, 50.0),
    (11, 19.0, 38.0, 57.5),
    (12, 21.5, 43.0, 65.0),
    (13, 25.0, 50.0, 75.0),
    (14, 29.0, 58.0, 87.5),
    (15, 33.0, 66.5, 100.0),
    (16, 38.0, 76.5, 115.0),
    (17, 43.0, 86.5, 130.0),
    (18, 50.0, 100.0, 150.0),
    (19, 58.0, 116.5, 175.0),
    (20, 66.5, 133.0, 200.0),
    (21, 76.5, 153.0, 230.0),
    (22, 86.5, 173.0, 260.0),
    (23, 100.0, 200.0, 300.0),
    (24, 116.5, 233.0, 350.0),
    (25, 133.0, 266.5, 400.0),
    (26, 153.0, 306.5, 460.0),
    (27, 173.0, 346.5, 520.0),
    (28, 200.0, 400.0, 600.0),
    (29, 233.0, 466.5, 700.0),
];

/// Catégorie de charge portée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategorieCharge {
    Legere,
    Intermediaire,
    Lourde,
    /// Au-delà de la charge lourde : dépassement, seulement si la règle le permet (option).
    Depassement,
}

/// Seuils (légère, intermédiaire, lourde) pour une Force donnée. `None` si la Force est inconnue.
pub fn seuils_charge(force: Option<u8>) -> Option<(f64, f64, f64)> {
    let mut f = i32::from(force?);
    if !(1..=60).contains(&f) {
        return None;
    }
    let mut multiplicateur = 1.0;
    while f > 29 {
        f -= 10;
        multiplicateur *= 4.0;
    }
    let ligne = SEUILS_CHARGE.iter().find(|(n, ..)| i32::from(*n) == f)?;
    Some((
        ligne.1 * multiplicateur,
        ligne.2 * multiplicateur,
        ligne.3 * multiplicateur,
    ))
}

/// Catégorie de la charge pour une Force et un poids (kg).
/// Avec `depassement_autorise`, une charge au-delà de la lourde est affichée comme dépassement
/// (sans bloquer). Sans l'option, la même charge renvoie `None`.
pub fn categorie_charge(
    force: Option<u8>,
    charge_kg: f64,
    depassement_autorise: bool,
) -> Option<CategorieCharge> {
    let (legere, inter, lourde) = seuils_charge(force)?;
    if charge_kg <= legere {
        Some(CategorieCharge::Legere)
    } else if charge_kg <= inter {
        Some(CategorieCharge::Intermediaire)
    } else if charge_kg <= lourde {
        Some(CategorieCharge::Lourde)
    } else if depassement_autorise {
        Some(CategorieCharge::Depassement)
    } else {
        None
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
        assert_eq!(vitesse_avec_barde(15, Some(Barde::Intermediaire)), Some(10.5));
        assert_eq!(vitesse_avec_barde(18, Some(Barde::Lourde)), Some(12.0));
        assert_eq!(vitesse_avec_barde(9, Some(Barde::Lourde)), None);
    }
}

#[cfg(test)]
mod tests_charge {
    use super::*;

    #[test]
    fn seuils_force_1_et_20() {
        assert_eq!(seuils_charge(Some(1)), Some((1.5, 3.0, 5.0)));
        assert_eq!(seuils_charge(Some(20)), Some((66.5, 133.0, 200.0)));
    }

    #[test]
    fn force_au_dela_de_29_multiplie_par_4_par_10() {
        // Force 30 = Force 20 × 4
        assert_eq!(seuils_charge(Some(30)), Some((266.0, 532.0, 800.0)));
    }

    #[test]
    fn force_inconnue_ou_hors_borne() {
        assert_eq!(seuils_charge(None), None);
        assert_eq!(seuils_charge(Some(0)), None);
    }

    #[test]
    fn categories_et_depassement() {
        assert_eq!(categorie_charge(Some(20), 50.0, false), Some(CategorieCharge::Legere));
        assert_eq!(categorie_charge(Some(20), 100.0, false), Some(CategorieCharge::Intermediaire));
        assert_eq!(categorie_charge(Some(20), 150.0, false), Some(CategorieCharge::Lourde));
        assert_eq!(categorie_charge(Some(20), 250.0, false), None);
        assert_eq!(categorie_charge(Some(20), 250.0, true), Some(CategorieCharge::Depassement));
    }
}
