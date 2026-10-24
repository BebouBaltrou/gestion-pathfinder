//! Niveau de personnage des employés de chaque équipe (Guide de campagne p. 108-110, « Employés »).
//!
//! Le livre donne le niveau « généralement » ou « typiquement » de l'employé dans la description de
//! chaque équipe ; il sert à la limite de recrutement par le Prestige (GC p. 93 : un personnage ne
//! peut recruter que des équipes dont les employés ont un niveau inférieur ou égal à son Prestige).
//! Relevé d'après le texte français des descriptions ; les cas douteux portent une note.

pub struct NiveauEquipe {
    pub equipe: &'static str,
    pub niveau: u32,
    /// Lecture à valider, le cas échéant.
    pub note: Option<&'static str>,
}

const fn n(equipe: &'static str, niveau: u32) -> NiveauEquipe {
    NiveauEquipe {
        equipe,
        niveau,
        note: None,
    }
}

const NIVEAUX: &[NiveauEquipe] = &[
    n("acolyte", 1),
    n("apprenti-mage", 1),
    n("archers", 1),
    n("archers-d-elite", 3),
    n("archers-montes", 3),
    NiveauEquipe {
        equipe: "artisans",
        niveau: 4,
        note: Some("le texte donne « un charpentier typique est un expert de niveau 4 » ; on applique ce niveau à toute l'équipe"),
    },
    NiveauEquipe {
        equipe: "bureaucrates",
        niveau: 3,
        note: Some("le texte donne « expert de niveau 1, 2 ou 3 » ; on retient le niveau le plus haut (3)"),
    },
    n("cambrioleurs", 3),
    n("cavaliers", 3),
    n("conducteur-d-attelage", 1),
    n("cure", 3),
    n("gardes", 1),
    n("gardes-d-elite", 3),
    n("laquais", 1),
    n("mage", 3),
    NiveauEquipe {
        equipe: "marins",
        niveau: 2,
        note: Some("le texte donne « un marin typique est un expert de niveau 2 »"),
    },
    n("sage", 3),
    n("soldats", 1),
    n("soldats-d-elite", 3),
    n("tire-laine", 1),
    n("travailleurs", 1),
];

/// Niveau des employés de l'équipe ; `None` si le livre n'en donne pas (fraudeurs).
pub fn niveau_des_employes(equipe: &str) -> Option<&'static NiveauEquipe> {
    NIVEAUX.iter().find(|x| x.equipe == equipe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toutes_les_equipes_du_referentiel_sont_traitees_sauf_celles_sans_niveau() {
        let r = crate::referentiel_officiel();
        for e in &r.equipes {
            let connu = niveau_des_employes(&e.id).is_some();
            assert_eq!(connu, e.id != "fraudeurs", "équipe {}", e.id);
        }
    }

    #[test]
    fn niveaux_du_livre() {
        assert_eq!(niveau_des_employes("acolyte").unwrap().niveau, 1);
        assert_eq!(niveau_des_employes("archers-d-elite").unwrap().niveau, 3);
    }
}
