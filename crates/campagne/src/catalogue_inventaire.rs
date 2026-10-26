//! Modèles d'objets de l'inventaire, construits à partir du catalogue d'équipement de
//! `crates/fiches` (`data/core/`) : une seule source pour les armes, armures et le matériel.
//!
//! L'identifiant d'un modèle vient du nom anglais (`equipement:longsword`), pour rester stable
//! quand un nom français est corrigé.

use domaine::Montant;
use fiches::equipement::{CatalogueEquipement, Cout, GenreProtection};
use fiches::objets_magiques::CatalogueMagique;
use fiches::traductions::Traductions;
use inventaire::{Modele, Poids};
use std::collections::BTreeMap;

fn identifiant(nom_en: &str) -> String {
    let mut slug = String::new();
    for c in nom_en.chars() {
        if c.is_alphanumeric() {
            slug.extend(c.to_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    format!("equipement:{}", slug.trim_matches('-'))
}

/// Prix du catalogue (pp, po, pa, pc) en pièces de cuivre.
fn montant(cout: &Option<Cout>) -> Option<Montant> {
    let c = cout.as_ref()?;
    let facteur = match c.unite.as_str() {
        "cp" => 1.0,
        "sp" => 10.0,
        "gp" => 100.0,
        "pp" => 1000.0,
        _ => return None,
    };
    Some(Montant::pc((c.valeur * facteur).round() as i64))
}

/// Poids du catalogue (livres) en grammes : 1 lb = 500 g, comme dans la version française.
fn poids(lb: Option<f64>) -> Option<Poids> {
    lb.map(|l| Poids((l * 500.0).round() as i64))
}

fn proprietes(valeurs: &[(&str, Option<String>)]) -> BTreeMap<String, String> {
    valeurs
        .iter()
        .filter_map(|(cle, v)| v.clone().map(|v| ((*cle).to_string(), v)))
        .collect()
}

/// Tous les modèles du catalogue d'équipement : armes, armures et boucliers, matériel. La note « erreur du
/// livre » de `notes_livre.json`, quand il y en a une, est dans la propriété `note_livre`.
pub fn modeles(catalogue: &CatalogueEquipement, traductions: &Traductions) -> Vec<Modele> {
    let mut modeles = Vec::new();
    for a in &catalogue.armes {
        let munition = a.categorie_en.as_deref() == Some("Ammunition");
        modeles.push(Modele {
            id: identifiant(&a.nom_en),
            nom: a.nom.clone(),
            type_objet: if munition { "munition" } else { "arme" }.to_string(),
            valeur: montant(&a.cout),
            poids: poids(a.poids_lb),
            source: format!("{}, p. {}", a.source.livre, a.source.page.unwrap_or(0)),
            proprietes: proprietes(&[
                ("nom_en", Some(a.nom_en.clone())),
                ("categorie", a.categorie_en.clone()),
                ("maniement", a.maniement_en.clone()),
                ("degats", a.degats_en.clone()),
                ("critique", a.critique_en.clone()),
                ("portee", a.portee_en.clone()),
                ("type_degats", a.type_en.clone()),
                ("special", a.special_en.clone()),
                ("groupes", a.groupes_en.clone()),
                ("mains", a.est_a_deux_mains().then(|| "2".to_string())),
                ("note_livre", traductions.note_livre("armes", &a.nom_en)),
            ]),
        });
    }
    for a in &catalogue.armures {
        let type_objet = match a.genre {
            GenreProtection::Bouclier => "bouclier",
            GenreProtection::Accessoire => "supplement-armure",
            _ => "armure",
        };
        modeles.push(Modele {
            id: identifiant(&a.nom_en),
            nom: a.nom.clone(),
            type_objet: type_objet.to_string(),
            valeur: montant(&a.cout),
            poids: poids(a.poids_lb),
            source: format!("{}, p. {}", a.source.livre, a.source.page.unwrap_or(0)),
            proprietes: proprietes(&[
                ("nom_en", Some(a.nom_en.clone())),
                ("categorie", Some(a.genre.nom().to_string())),
                ("bonus_ca", a.bonus_armure.map(|v| v.to_string())),
                ("dex_max", a.dex_max.map(|v| v.to_string())),
                ("malus_armure", a.malus_armure.map(|v| v.to_string())),
                ("echec_sorts_pct", a.echec_sorts_pct.map(|v| v.to_string())),
                ("note_livre", traductions.note_livre("armures", &a.nom_en)),
            ]),
        });
    }
    for o in &catalogue.materiel {
        modeles.push(Modele {
            id: identifiant(&o.nom_en),
            nom: o.nom.clone(),
            type_objet: "materiel".to_string(),
            valeur: montant(&o.cout),
            poids: poids(o.poids_lb),
            source: format!("{}, p. {}", o.source.livre, o.source.page.unwrap_or(0)),
            proprietes: proprietes(&[
                ("nom_en", Some(o.nom_en.clone())),
                ("categorie", o.categorie_en.clone()),
                ("note_livre", traductions.note_livre("materiel", &o.nom_en)),
            ]),
        });
    }
    modeles
}

/// Fichiers du catalogue des objets magiques (`data/core/objets_magiques/`) qui décrivent de
/// vrais objets. Les propriétés spéciales, les améliorations et les tarifs ne sont pas des objets.
const FICHIERS_OBJETS_MAGIQUES: [&str; 6] = [
    include_str!("../../../data/core/objets_magiques/anneaux.txt"),
    include_str!("../../../data/core/objets_magiques/sceptres.txt"),
    include_str!("../../../data/core/objets_magiques/batons.txt"),
    include_str!("../../../data/core/objets_magiques/armures_boucliers_specifiques.txt"),
    include_str!("../../../data/core/objets_magiques/armes_specifiques.txt"),
    include_str!("../../../data/core/objets_magiques/objets_merveilleux.txt"),
];

/// Nom comparable : sans parenthèses ni préfixe « Anneau de », apostrophes unifiées.
fn nom_comparable(nom: &str) -> String {
    let t = nom.to_lowercase().replace(['(', ')'], "").replace('’', "'");
    let t = t.strip_prefix("anneau de ").unwrap_or(&t);
    let t = t.strip_prefix("anneau d'").unwrap_or(t);
    t.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Les objets magiques : ceux de `data/core/objets_magiques.json` (avec leurs effets chiffrés, leur
/// emplacement et leur page), complétés par les tables du chapitre 15 (nom et prix de vente). Un
/// objet présent dans les deux vient de `objets_magiques.json`.
pub fn modeles_magiques(
    catalogue: &CatalogueMagique,
    regles: &inventaire::equipement::ReglesEmplacements,
) -> Vec<Modele> {
    let mut modeles = Vec::new();
    let mut connus = std::collections::BTreeSet::new();
    for i in catalogue.toutes_les_instances() {
        connus.insert(nom_comparable(&i.nom));
        let mut proprietes = proprietes(&[
            ("nom_en", Some(i.nom_en.clone())),
            ("bonus", Some(i.bonus.to_string())),
            ("notes", (!i.notes.is_empty()).then(|| i.notes.clone())),
        ]);
        if let Some(slot) = i
            .emplacement
            .as_deref()
            .and_then(|e| regles.noms_du_livre.get(e))
        {
            proprietes.insert("emplacement".to_string(), slot.clone());
        }
        modeles.push(Modele {
            id: identifiant(&i.nom_en).replacen("equipement:", "objet-magique:", 1),
            nom: i.nom.clone(),
            type_objet: "objet-magique".to_string(),
            valeur: Some(Montant::po(i.prix_po)),
            poids: None,
            source: i.source.clone(),
            proprietes,
        });
    }
    for fichier in FICHIERS_OBJETS_MAGIQUES {
        for m in inventaire::catalogue::charger(fichier).expect("catalogue des objets magiques") {
            if !connus.contains(&nom_comparable(&m.nom)) {
                modeles.push(m);
            }
        }
    }
    modeles
}

/// Emplacements du corps par défaut (`data/equipement/emplacements.json`).
#[cfg(feature = "serde")]
pub fn emplacements_par_defaut() -> inventaire::equipement::ReglesEmplacements {
    serde_json::from_str(include_str!("../../../data/equipement/emplacements.json"))
        .expect("emplacements.json valide")
}

/// Tout le catalogue des objets (équipement et objets magiques), construit une seule fois.
#[cfg(feature = "serde")]
pub fn catalogue_complet() -> &'static [Modele] {
    static CATALOGUE: std::sync::OnceLock<Vec<Modele>> = std::sync::OnceLock::new();
    CATALOGUE.get_or_init(|| {
        let fiches = fiches::Referentiel::partage();
        let mut tous = modeles(&fiches.equipement, Traductions::partage());
        tous.extend(modeles_magiques(
            &fiches.objets_magiques,
            &emplacements_par_defaut(),
        ));
        tous
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalogue() -> CatalogueEquipement {
        CatalogueEquipement::depuis_json(
            include_str!("../../../data/core/armures.json"),
            include_str!("../../../data/core/armes.json"),
            include_str!("../../../data/core/materiel.json"),
            include_str!("../../../data/core/charge.json"),
            include_str!("../../../data/core/maitrises.json"),
        )
        .unwrap()
    }

    fn trouver<'a>(modeles: &'a [Modele], nom: &str) -> &'a Modele {
        modeles
            .iter()
            .find(|m| m.nom == nom)
            .unwrap_or_else(|| panic!("« {nom} » absent"))
    }

    #[test]
    fn tout_le_catalogue_devient_des_modeles_uniques() {
        let c = catalogue();
        let m = modeles(&c, Traductions::partage());
        assert_eq!(m.len(), c.armes.len() + c.armures.len() + c.materiel.len());
        let mut ids: Vec<&str> = m.iter().map(|x| x.id.as_str()).collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n, "identifiants en double");
    }

    #[test]
    fn valeurs_et_poids_du_manuel_des_joueurs() {
        let m = modeles(&catalogue(), Traductions::partage());
        let dague = trouver(&m, "Dague");
        assert_eq!(dague.valeur, Some(Montant::po(2)));
        assert_eq!(dague.poids, Some(Poids(500)));
        assert_eq!(dague.type_objet, "arme");
        // Les noms officiels du Manuel des joueurs priment sur la traduction automatique.
        assert_eq!(trouver(&m, "Morgenstern").type_objet, "arme");
        let harnois = trouver(&m, "Harnois");
        assert_eq!(harnois.valeur, Some(Montant::po(1500)));
        assert_eq!(harnois.poids, Some(Poids(25_000)));
        assert_eq!(harnois.proprietes["bonus_ca"], "9");
        assert_eq!(trouver(&m, "Targe").type_objet, "bouclier");
        assert_eq!(trouver(&m, "Bougie").valeur, Some(Montant::pc(1)));
        assert_eq!(trouver(&m, "Flèches (20)").type_objet, "munition");
    }

    #[test]
    fn les_notes_erreur_du_livre_suivent_l_objet() {
        let m = modeles(&catalogue(), Traductions::partage());
        let hallebarde = trouver(&m, "Hallebarde");
        assert!(hallebarde.proprietes["note_livre"].contains("Halllebarde"));
        assert!(trouver(&m, "Flasque (vide)").proprietes["note_livre"].contains("0,75 kg"));
        assert!(!trouver(&m, "Dague").proprietes.contains_key("note_livre"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn equiper_des_objets_du_catalogue() {
        use inventaire::{Conteneur, Erreur, Proprietaire, Registre};
        let tous = modeles(&catalogue(), Traductions::partage());
        let mut r = Registre::default();
        for m in &tous {
            r.ajouter_modele(m.clone());
        }
        r.definir_emplacements(emplacements_par_defaut()).unwrap();
        let racine = r.creer_inventaire(
            Proprietaire {
                type_id: "personnage".into(),
                ref_id: "1".into(),
            },
            Conteneur {
                capacite: None,
                types_autorises: None,
                contenu_compte: true,
            },
        );
        let mut ajouter = |nom: &str| {
            let id = trouver(&tous, nom).id.clone();
            r.ajouter(&id, 1, racine).unwrap()
        };
        let armure = ajouter("Armure de cuir");
        let targe = ajouter("Targe");
        let deux_mains = ajouter("Épée à deux mains");
        let epee_longue = ajouter("Épée longue");

        r.equiper(armure, "corps").unwrap();
        // L'épée à deux mains occupe les deux mains tenues : la targe ne passe pas.
        r.equiper(deux_mains, "main_directrice").unwrap();
        assert_eq!(
            r.equiper(targe, "main_non_directrice"),
            Err(Erreur::EmplacementOccupe("main_non_directrice".into()))
        );
        r.deequiper(deux_mains).unwrap();
        r.equiper(epee_longue, "main_directrice").unwrap();
        r.equiper(targe, "main_non_directrice").unwrap();
        assert_eq!(r.equipement_de(racine).len(), 3);
        // Le poids ne change pas quand on équipe : tout reste dans l'inventaire.
        assert_eq!(
            r.poids_total(racine),
            inventaire::Poids(7_500 + 2_500 + 4_000 + 2_000)
        );
    }

    fn magiques() -> Vec<Modele> {
        let catalogue =
            CatalogueMagique::depuis_json(include_str!("../../../data/core/objets_magiques.json"))
                .unwrap();
        let regles: inventaire::equipement::ReglesEmplacements =
            serde_json::from_str(include_str!("../../../data/equipement/emplacements.json"))
                .unwrap();
        modeles_magiques(&catalogue, &regles)
    }

    #[test]
    fn les_objets_magiques_ont_des_identifiants_uniques() {
        let m = magiques();
        assert!(m.len() > 400, "{} objets magiques", m.len());
        let mut ids: Vec<&str> = m.iter().map(|x| x.id.as_str()).collect();
        ids.sort_unstable();
        let n = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), n, "identifiants en double");
    }

    #[test]
    fn les_objets_a_bonus_gardent_leurs_effets_leur_emplacement_et_leur_prix() {
        let m = magiques();
        let cape = trouver(&m, "Cape de résistance (+2)");
        assert_eq!(cape.type_objet, "objet-magique");
        assert_eq!(cape.valeur, Some(Montant::po(4_000)));
        assert_eq!(cape.proprietes["emplacement"], "epaules");
        assert_eq!(cape.proprietes["nom_en"], "Cloak of Resistance (+2)");
        // Un seul anneau de protection +1 : celui de `objets_magiques.json`.
        let anneaux: Vec<_> = m
            .iter()
            .filter(|x| x.nom.to_lowercase().contains("protection") && x.nom.contains("+1"))
            .collect();
        assert_eq!(anneaux.len(), 1, "{anneaux:?}");
        assert_eq!(anneaux[0].proprietes["emplacement"], "anneau");
    }

    #[test]
    fn les_tables_completent_le_catalogue() {
        let m = magiques();
        assert_eq!(
            trouver(&m, "Feu d’étoiles").valeur,
            Some(Montant::po(50_000))
        );
        assert_eq!(
            trouver(&m, "Épée de justice").valeur,
            Some(Montant::po(120_630))
        );
        assert_eq!(trouver(&m, "Forêt profonde").type_objet, "baton");
        let pierre = trouver(&m, "Pierre ioun (rhombe bleu pâle)");
        assert_eq!(
            pierre.proprietes["effet"],
            "Bonus d’altération de +2 en Force"
        );
        assert_eq!(pierre.proprietes["puissance"], "intermédiaire");
        let collier = trouver(&m, "Collier à boules de feu (1er modèle)");
        assert_eq!(collier.poids, Some(inventaire::Poids(500)));
        assert!(collier.source.contains("Table 15-18"));
    }

    /// Les prix des tables et ceux de `objets_magiques.json` doivent s'accorder.
    #[test]
    fn les_prix_des_tables_et_de_objets_magiques_json_s_accordent() {
        let catalogue =
            CatalogueMagique::depuis_json(include_str!("../../../data/core/objets_magiques.json"))
                .unwrap();
        let connus: std::collections::BTreeMap<String, i64> = catalogue
            .toutes_les_instances()
            .map(|i| (nom_comparable(&i.nom), i.prix_po))
            .collect();
        let mut ecarts = Vec::new();
        for fichier in FICHIERS_OBJETS_MAGIQUES {
            for m in inventaire::catalogue::charger(fichier).unwrap() {
                if let Some(prix_po) = connus.get(&nom_comparable(&m.nom)) {
                    // Écart du livre : la table 15-19 donne 16 000 po au « Bandeau de supériorité mentale
                    // (+4) », la table 15-20 et `objets_magiques.json` 64 000 po (16 000 po vaut le +2).
                    let ecart_connu = m.nom == "Bandeau de supériorité mentale (+4)"
                        && m.valeur == Some(Montant::po(16_000));
                    if !ecart_connu && m.valeur != Some(Montant::po(*prix_po)) {
                        ecarts.push(format!(
                            "{} : table {:?}, json {prix_po} po",
                            m.nom, m.valeur
                        ));
                    }
                }
            }
        }
        assert!(ecarts.is_empty(), "{ecarts:#?}");
    }

    #[test]
    fn les_objets_merveilleux_se_rangent_sans_emplacement_sans_limite() {
        use inventaire::{Conteneur, Proprietaire, Registre};
        let regles = emplacements_par_defaut();
        let m = magiques();
        let mut r = Registre::default();
        r.definir_emplacements(regles).unwrap();
        let racine = r.creer_inventaire(
            Proprietaire {
                type_id: "personnage".into(),
                ref_id: "1".into(),
            },
            Conteneur {
                capacite: None,
                types_autorises: None,
                contenu_compte: true,
            },
        );
        for nom in ["Bille de force", "Trou portable", "Cube de force"] {
            let modele = trouver(&m, nom).clone();
            r.ajouter_modele(modele.clone());
            let id = r.ajouter(&modele.id, 1, racine).unwrap();
            r.equiper(id, "sans_emplacement").unwrap();
        }
        assert_eq!(r.equipement_de(racine).len(), 3);
    }
}
