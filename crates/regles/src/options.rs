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
    /// Obsolète, conservé pour relire les anciennes sauvegardes : ce réglage est désormais
    /// journalisé (événement `ReglageSalaireGestionnaire` de la campagne, PC-06).
    pub salaire_gestionnaire_unique: bool,
    /// Officiel (GC p. 94) : une équipe sans bâtiment pour travailler voit ses recettes divisées
    /// par 2. Désactivé = personnalisé, l'équipe garde son bonus entier.
    pub moitie_equipe_sans_batiment: bool,
    /// Personnalisé : délocalisation d'une organisation entière avec un seul jet de 1d6 pour la
    /// préparation, au lieu d'un jet par groupe d'employés (GC p. 95).
    pub delocalisation_jet_unique: bool,
    /// Règle officielle (GC p. 94) : les capitaux produits par les tests des bâtiments et
    /// organisations se paient au Coût produit. Désactivé par défaut (décision de l'utilisateur
    /// du 23/10/2026 : gratuits) ; ne concerne pas le travail du PJ, payant par défaut.
    pub capitaux_produits_payants: bool,
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
            delocalisation_jet_unique: false,
            capitaux_produits_payants: false,
            afficher_avertissements: true,
        }
    }
}
