//! Points d'expérience d'un combat : on additionne les XP des créatures vaincues (d'après leur FP,
//! saisi à la main) et on divise par le nombre de personnages (« méthode exacte », Core p. 397-399),
//! ou bien on saisit directement le total.

use serde::Deserialize;
use std::sync::OnceLock;

const JSON: &str = include_str!("../../../data/combat/experience.json");

#[derive(Deserialize)]
struct Fichier {
    xp: Vec<(String, u32)>,
}

fn table() -> &'static [(String, u32)] {
    static T: OnceLock<Vec<(String, u32)>> = OnceLock::new();
    T.get_or_init(|| {
        serde_json::from_str::<Fichier>(JSON)
            .expect("data/combat/experience.json est valide")
            .xp
    })
}

/// Facteurs de puissance acceptés, dans l'ordre du tableau (« 1/2 », « 3 »…).
pub fn facteurs_de_puissance() -> Vec<String> {
    table().iter().map(|(fp, _)| fp.clone()).collect()
}

/// XP d'une créature de ce FP (colonne « Total XP » du tableau 12-2).
pub fn xp_du_fp(fp: &str) -> Option<u32> {
    let f = fp.trim().replace(',', ".");
    table().iter().find(|(n, _)| *n == f).map(|(_, x)| *x)
}

/// XP total d'une liste de FP ; erreur si l'un d'eux n'est pas dans le tableau.
pub fn xp_total(fps: &[String]) -> Result<u32, String> {
    fps.iter().try_fold(0u32, |acc, fp| {
        xp_du_fp(fp).map(|x| acc.saturating_add(x)).ok_or_else(|| {
            format!("FP inconnu : « {fp} » (valeurs du Core : 1/8, 1/6, 1/4, 1/3, 1/2, 1 à 25)")
        })
    })
}

/// XP de chaque personnage : le total divisé par le nombre de personnages, arrondi vers le bas
/// (le livre ne précise pas l'arrondi : lecture à valider).
pub fn par_personnage(total: u32, personnages: u32) -> u32 {
    total.checked_div(personnages).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_tableau_suit_le_modele_du_livre() {
        let t = table();
        assert_eq!(t.len(), 30);
        assert_eq!(xp_du_fp("1"), Some(400));
        assert_eq!(xp_du_fp("1/2"), Some(200));
        assert_eq!(xp_du_fp("5"), Some(1600));
        assert_eq!(xp_du_fp("0"), None);
        // l'XP d'un FP est le double de celui de deux FP plus bas (à partir du FP 3)
        for n in 3..=25u32 {
            let a = xp_du_fp(&n.to_string()).unwrap();
            let b = xp_du_fp(&(n - 2).to_string()).unwrap();
            assert_eq!(a, b * 2, "FP {n}");
        }
    }

    #[test]
    fn totaux_et_partage() {
        let fps = ["1".to_string(), "1/2".to_string(), "1/2".to_string()];
        assert_eq!(xp_total(&fps), Ok(800));
        assert!(xp_total(&["99".to_string()]).is_err());
        assert_eq!(par_personnage(800, 3), 266);
        assert_eq!(par_personnage(800, 4), 200);
        assert_eq!(par_personnage(800, 0), 0);
    }
}
