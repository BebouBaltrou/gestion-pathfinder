//! Fabrication d'objets magiques.
//!
//! Sources : Manuel des joueurs, chapitre 15, p. 554-555 (DD, fournitures, temps, conditions,
//! objets maudits) ; Guide de campagne p. 85 (une journée d'intermède = 8 h de création) ;
//! Guide de campagne table 2-1 p. 78 (valeur de la Magie et des Marchandises en po).
//! Montants en pièces d'argent (pa) : 1 po = 10 pa.

/// Heures de création par journée d'intermède (GC p. 85).
pub const HEURES_PAR_JOUR: u32 = 8;
/// Valeur d'un point de Magie en po (GC table 2-1, p. 78).
pub const MAGIE_PO: i64 = 100;
/// Valeur d'un point de Marchandises en po (GC table 2-1, p. 78).
pub const MARCHANDISES_PO: i64 = 20;

/// DD du test de fin de fabrication : 5 + niveau de lanceur de sorts de l'objet,
/// + 5 par condition de création non remplie (MJ p. 554). Le don Création d'objet est
/// obligatoire et ne compte pas ici.
pub fn dd(niveau_lanceur: u32, conditions_non_remplies: u32) -> i32 {
    5 + niveau_lanceur as i32 + 5 * conditions_non_remplies as i32
}

/// Heures de travail nécessaires (MJ p. 554).
/// - Potion ou parchemin de 250 po ou moins : 2 h.
/// - Sinon : 8 h par tranche de 1 000 po, arrondi à la tranche supérieure, minimum 8 h.
/// - Avec +5 au DD (accélération) : 4 h par tranche de 1 000 po, arrondi à la tranche supérieure.
pub fn heures_requises(prix_po: i64, potion_ou_parchemin: bool, acceleree: bool) -> u32 {
    if potion_ou_parchemin && prix_po <= 250 {
        return 2;
    }
    let tranches = ((prix_po.max(1) + 999) / 1000) as u32;
    if acceleree {
        4 * tranches
    } else {
        (8 * tranches).max(8)
    }
}

/// Jours d'intermède nécessaires : heures arrondies à la journée de 8 h.
pub fn jours_requis(heures: u32) -> u32 {
    heures.div_ceil(HEURES_PAR_JOUR)
}

/// Fournitures à payer au début, en pa : moitié du prix de base (MJ p. 554).
pub fn fournitures_pa(prix_base_po: i64) -> i64 {
    prix_base_po * 10 / 2
}

/// Paiement des fournitures avec les trois ressources, en pa.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Paiement {
    pub po: i64,
    pub marchandises: u32,
    pub magie: u32,
}

impl Paiement {
    pub fn valeur_pa(self) -> i64 {
        self.po * 10 + self.marchandises as i64 * MARCHANDISES_PO * 10 + self.magie as i64 * MAGIE_PO * 10
    }
}

/// Issue du test final de fabrication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Issue {
    /// Résultat ≥ DD : l'objet fonctionne.
    Reussi,
    /// Échec de 1 à 4 sous le DD : temps et composantes perdus.
    Echec,
    /// Échec d'au moins 5 sous le DD : temps et composantes perdus, et l'objet est maudit.
    Maudit,
}

/// Résultat du test final : `resultat` = d20 + bonus de compétence, saisi par l'utilisateur.
pub fn issue_test(resultat: i32, dd: i32) -> Issue {
    if resultat >= dd {
        Issue::Reussi
    } else if dd - resultat >= 5 {
        Issue::Maudit
    } else {
        Issue::Echec
    }
}

/// Avancement d'une fabrication en heures, jour par jour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progression {
    pub heures_requises: u32,
    pub heures_faites: u32,
}

impl Progression {
    pub fn nouvelle(heures_requises: u32) -> Self {
        Self { heures_requises, heures_faites: 0 }
    }

    /// Ajoute une journée d'intermède (8 h), sans dépasser le total.
    pub fn avancer_jour(&mut self) {
        self.heures_faites = (self.heures_faites + HEURES_PAR_JOUR).min(self.heures_requises);
    }

    /// Vrai quand toutes les heures sont faites : il reste alors le test final.
    pub fn terminee(&self) -> bool {
        self.heures_faites >= self.heures_requises
    }

    pub fn jours_restants(&self) -> u32 {
        jours_requis(self.heures_requises - self.heures_faites)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dd_base_et_conditions() {
        assert_eq!(dd(5, 0), 10);
        assert_eq!(dd(5, 2), 20);
    }

    #[test]
    fn heures_par_tranche_de_1000_po() {
        assert_eq!(heures_requises(1000, false, false), 8);
        assert_eq!(heures_requises(1001, false, false), 16);
        assert_eq!(heures_requises(100, false, false), 8);
        assert_eq!(heures_requises(3000, false, true), 12);
    }

    #[test]
    fn potion_ou_parchemin_leger() {
        assert_eq!(heures_requises(250, true, false), 2);
        assert_eq!(heures_requises(300, true, false), 8);
    }

    #[test]
    fn jours_arrondis_a_huit_heures() {
        assert_eq!(jours_requis(8), 1);
        assert_eq!(jours_requis(16), 2);
        assert_eq!(jours_requis(12), 2);
    }

    #[test]
    fn fournitures_moitie_du_prix() {
        assert_eq!(fournitures_pa(1500), 7500);
    }

    #[test]
    fn paiement_mixte_en_pa() {
        let p = Paiement { po: 50, marchandises: 2, magie: 1 };
        assert_eq!(p.valeur_pa(), 500 + 400 + 1000);
    }

    #[test]
    fn issue_reussite_echec_maudit() {
        assert_eq!(issue_test(10, 10), Issue::Reussi);
        assert_eq!(issue_test(6, 10), Issue::Echec);
        assert_eq!(issue_test(5, 10), Issue::Maudit);
    }

    #[test]
    fn progression_jour_par_jour() {
        let mut p = Progression::nouvelle(16);
        assert_eq!(p.jours_restants(), 2);
        p.avancer_jour();
        assert!(!p.terminee());
        assert_eq!(p.jours_restants(), 1);
        p.avancer_jour();
        assert!(p.terminee());
        p.avancer_jour();
        assert_eq!(p.heures_faites, 16);
    }
}
