//! Montures et barde, d'après le Manuel des joueurs (VF) :
//! table 7-9 « Montures et véhicules » (p. 173-174) et table 6-8 « Barde » (p. 163).
//!
//! Lecture à valider : les plages de charge de la table 7-9 sont lues comme les bornes où la
//! vitesse est réduite. Au-delà de la plage haute, la table ne dit rien : on renvoie `None`.

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
    /// Plage de charge (kg, bornes incluses) où la vitesse est réduite. Lecture à valider.
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
