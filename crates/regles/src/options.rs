//! Règles personnalisées décidées par l'utilisateur (document « Règles Ultimate Campaign »,
//! section « Règles personnalisées décidées »). Chacune est désactivable.

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct OptionsRegles {
    /// Personnalisé : une salle ou une équipe donne tout son bonus à un seul type de capital par
    /// jour. Désactivé = règle officielle (répartition libre entre types admis, p. 94).
    pub repartition_stricte: bool,
    /// Personnalisé : les salles d'un même étage peuvent être construites en parallèle.
    pub construction_parallele_autorisee: bool,
    /// Personnalisé : une salle en travaux d'amélioration continue de produire.
    pub production_pendant_amelioration: bool,
    /// Personnalisé : le salaire d'un gestionnaire affecté à plusieurs établissements
    /// n'est compté qu'une fois par jour (PC-06).
    pub salaire_gestionnaire_unique: bool,
    /// Officiel (GC p. 94) : une équipe sans bâtiment pour travailler voit ses recettes divisées
    /// par 2. Désactivé = personnalisé, l'équipe garde son bonus entier.
    pub moitie_equipe_sans_batiment: bool,
    /// Afficher les avertissements liés aux points à confirmer.
    pub afficher_avertissements: bool,
}

impl Default for OptionsRegles {
    fn default() -> Self {
        OptionsRegles {
            repartition_stricte: true,
            construction_parallele_autorisee: true,
            production_pendant_amelioration: false,
            salaire_gestionnaire_unique: true,
            moitie_equipe_sans_batiment: true,
            afficher_avertissements: true,
        }
    }
}
