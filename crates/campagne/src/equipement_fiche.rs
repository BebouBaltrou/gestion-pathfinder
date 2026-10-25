//! Lien entre l'équipement porté de l'inventaire et la fiche de personnage.
//!
//! Pour un personnage qui a un inventaire, l'équipement de sa fiche (armure, bouclier, armes
//! portées, objets magiques) reflète ce qu'il a équipé dans l'inventaire : une seule vérité. La
//! fiche garde sa saisie manuelle pour les personnages sans inventaire.
//!
//! [`Campagne::executer_inventaire`] fait la commande d'inventaire et les événements de fiche qui
//! l'accompagnent dans un seul bloc : annuler défait les deux, et si le moteur de règles de la fiche
//! refuse l'équipement, rien n'est équipé. Les bonus eux-mêmes sont calculés par le moteur d'effets
//! de la fiche, jamais ici.

use crate::etat::{Consequence, Erreur, Etat};
use crate::evenements::Evenement;
use crate::modele::IdPersonnage;
use crate::Campagne;
use fiches::equipement::{Equipement, ObjetMagique};
use fiches::journal::{EmplacementProtection, EvenementFiche};
use fiches::Referentiel as ReferentielFiches;
use inventaire::{Commande, Id, Registre};
use regles::Referentiel;

/// Type de propriétaire de l'inventaire d'un personnage.
pub const PROPRIETAIRE_PERSONNAGE: &str = "personnage";

/// Ce que l'inventaire fait porter à un personnage.
#[derive(Debug, Default, PartialEq, Eq)]
struct Cible {
    armure: Option<String>,
    bouclier: Option<String>,
    armes: Vec<String>,
    magiques: Vec<ObjetMagique>,
}

fn cible(registre: &Registre, racine: Id, fiches: &ReferentielFiches) -> Cible {
    let mut cible = Cible::default();
    for equipe in registre.equipement_de(racine) {
        let Ok(exemplaire) = registre.exemplaire(equipe.objet) else {
            continue;
        };
        let Some(modele) = exemplaire.modele.as_ref().and_then(|m| registre.modele(m)) else {
            continue;
        };
        let Some(nom_en) = modele.proprietes.get("nom_en") else {
            continue;
        };
        match modele.type_objet.as_str() {
            "armure" => cible.armure = Some(nom_en.clone()),
            "bouclier" => cible.bouclier = Some(nom_en.clone()),
            "arme" => cible.armes.push(nom_en.clone()),
            "objet-magique" => {
                if let Some(i) = fiches.objets_magiques.objet(nom_en) {
                    cible.magiques.push(ObjetMagique {
                        nom: i.nom,
                        effets: i.effets,
                    });
                }
            }
            _ => {}
        }
    }
    cible
}

/// Événements de fiche qui amènent l'équipement actuel à la cible : les retraits d'abord.
fn ecarts(pj: IdPersonnage, actuel: &Equipement, cible: &Cible) -> Vec<Evenement> {
    let fiche = |evenement| Evenement::Fiche { pj, evenement };
    let mut retraits = Vec::new();
    let mut ajouts = Vec::new();
    for (emplacement, avant, apres) in [
        (EmplacementProtection::Armure, &actuel.armure, &cible.armure),
        (
            EmplacementProtection::Bouclier,
            &actuel.bouclier,
            &cible.bouclier,
        ),
    ] {
        if avant != apres {
            let ev = fiche(EvenementFiche::Equiper {
                emplacement,
                objet: apres.clone(),
            });
            if apres.is_none() {
                retraits.push(ev);
            } else {
                ajouts.push(ev);
            }
        }
    }
    let mut voulues = cible.armes.clone();
    for arme in &actuel.armes {
        match voulues.iter().position(|a| a == arme) {
            Some(i) => {
                voulues.remove(i);
            }
            None => retraits.push(fiche(EvenementFiche::ArmeRetiree(arme.clone()))),
        }
    }
    ajouts.extend(
        voulues
            .into_iter()
            .map(|a| fiche(EvenementFiche::ArmeAjoutee(a))),
    );
    for objet in &actuel.objets_magiques {
        let remplace = cible.magiques.iter().any(|m| m.nom == objet.nom);
        if !remplace {
            retraits.push(fiche(EvenementFiche::ObjetMagiqueRetire(objet.nom.clone())));
        }
    }
    for objet in &cible.magiques {
        if !actuel.objets_magiques.contains(objet) {
            ajouts.push(fiche(EvenementFiche::ObjetMagiqueAjoute(objet.clone())));
        }
    }
    retraits.extend(ajouts);
    retraits
}

/// Événements de fiche à ajouter pour que chaque personnage porte ce que dit son inventaire.
fn synchronisation(etat: &Etat) -> Vec<Evenement> {
    let fiches = ReferentielFiches::partage();
    let mut evenements = Vec::new();
    for pj in etat.personnages.keys() {
        let Some(racine) = etat
            .inventaire
            .racine_du_proprietaire(PROPRIETAIRE_PERSONNAGE, &pj.to_string())
        else {
            continue;
        };
        let Some(sa_fiche) = etat.fiche_de(*pj) else {
            continue;
        };
        let cible = cible(&etat.inventaire, racine, fiches);
        evenements.extend(ecarts(*pj, &sa_fiche.fiche.equipement, &cible));
    }
    evenements
}

impl Campagne {
    /// Exécute une commande d'inventaire et aligne, dans le même bloc annulable, la fiche de chaque
    /// personnage sur son équipement porté. Tout ou rien.
    pub fn executer_inventaire(
        &mut self,
        commande: Commande,
        r: &Referentiel,
    ) -> Result<Vec<Consequence>, Erreur> {
        let evenement = Evenement::Inventaire(commande);
        let mut apres = self.etat(r).clone();
        apres.appliquer(&evenement, r)?;
        let mut evenements = vec![evenement];
        evenements.extend(synchronisation(&apres));
        let evenement = if evenements.len() == 1 {
            evenements.remove(0)
        } else {
            Evenement::Groupe {
                libelle: "Équipement porté".to_string(),
                evenements,
            }
        };
        self.executer(evenement, r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue_inventaire::{emplacements_par_defaut, modeles, modeles_magiques};
    use domaine::Montant;
    use fiches::traductions::Traductions;
    use inventaire::{Conteneur, Modele, Proprietaire};

    /// Campagne avec un personnage (fiche d'exemple) et son inventaire garni de quatre objets :
    /// armure de cuir (2), targe (3), épée longue (4), cape de résistance +2 (5).
    fn campagne() -> (Campagne, Referentiel, IdPersonnage) {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let pj = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id: pj,
                nom: "Lyra".into(),
                compte_initial: Montant::po(100),
                prestige: None,
            },
            &r,
        )
        .unwrap();
        c.executer(
            Evenement::Fiche {
                pj,
                evenement: EvenementFiche::Creation(Box::new(fiches::exemple::elfe_ensorceleuse())),
            },
            &r,
        )
        .unwrap();
        let fiches = ReferentielFiches::partage();
        let regles = emplacements_par_defaut();
        let tous = modeles(&fiches.equipement, Traductions::partage());
        let magiques = modeles_magiques(&fiches.objets_magiques, &regles);
        let trouver =
            |liste: &[Modele], nom: &str| liste.iter().find(|m| m.nom == nom).unwrap().clone();
        let voulus = [
            trouver(&tous, "Armure de cuir"),
            trouver(&tous, "Targe"),
            trouver(&tous, "Épée longue"),
            trouver(&magiques, "Cape de résistance (+2)"),
        ];
        let mut commandes = vec![Commande::DefinirEmplacements(regles)];
        commandes.extend(voulus.iter().cloned().map(Commande::AjouterModele));
        commandes.push(Commande::CreerInventaire {
            proprietaire: Proprietaire {
                type_id: PROPRIETAIRE_PERSONNAGE.into(),
                ref_id: pj.to_string(),
            },
            conteneur: Conteneur {
                capacite: None,
                types_autorises: None,
                contenu_compte: true,
            },
        });
        commandes.extend(voulus.iter().map(|m| Commande::Ajouter {
            modele: m.id.clone(),
            quantite: 1,
            dans: 1,
        }));
        for commande in commandes {
            c.executer_inventaire(commande, &r).unwrap();
        }
        (c, r, pj)
    }

    fn equipement(c: &mut Campagne, r: &Referentiel, pj: IdPersonnage) -> Equipement {
        c.etat(r).fiche_de(pj).unwrap().fiche.equipement
    }

    #[test]
    fn equiper_met_la_fiche_a_jour_et_annuler_defait_les_deux() {
        let (mut c, r, pj) = campagne();
        assert_eq!(equipement(&mut c, &r, pj).armure, None);
        let equiper = |id, emplacement: &str| Commande::Equiper {
            id,
            emplacement: emplacement.into(),
        };
        c.executer_inventaire(equiper(2, "corps"), &r).unwrap();
        c.executer_inventaire(equiper(3, "main_non_directrice"), &r)
            .unwrap();
        c.executer_inventaire(equiper(4, "main_directrice"), &r)
            .unwrap();
        c.executer_inventaire(equiper(5, "epaules"), &r).unwrap();
        let eq = equipement(&mut c, &r, pj);
        assert_eq!(eq.armure.as_deref(), Some("Leather"));
        assert_eq!(eq.bouclier.as_deref(), Some("Buckler"));
        assert_eq!(eq.armes, vec!["Longsword"]);
        assert_eq!(eq.objets_magiques.len(), 1);
        assert_eq!(eq.objets_magiques[0].nom, "Cape de résistance (+2)");
        assert!(!eq.objets_magiques[0].effets.is_empty());

        // Déséquiper la cape la retire de la fiche ; annuler la remet.
        c.executer_inventaire(Commande::Deequiper { id: 5 }, &r)
            .unwrap();
        assert!(equipement(&mut c, &r, pj).objets_magiques.is_empty());
        c.annuler();
        assert_eq!(equipement(&mut c, &r, pj).objets_magiques.len(), 1);
        // Annuler l'équipement de la cape : ni l'inventaire ni la fiche ne la portent.
        c.annuler();
        assert!(equipement(&mut c, &r, pj).objets_magiques.is_empty());
        assert_eq!(c.etat(&r).inventaire.equipement_de(1).len(), 3);
        c.retablir();
        assert_eq!(equipement(&mut c, &r, pj).objets_magiques.len(), 1);
    }

    #[test]
    fn un_equipement_impossible_ne_change_rien() {
        let (mut c, r, pj) = campagne();
        c.executer_inventaire(
            Commande::Equiper {
                id: 2,
                emplacement: "corps".into(),
            },
            &r,
        )
        .unwrap();
        let longueur = c.journal.len();
        // La targe va dans la main non directrice, pas sur le corps.
        let refus = c.executer_inventaire(
            Commande::Equiper {
                id: 3,
                emplacement: "corps".into(),
            },
            &r,
        );
        assert!(refus.is_err());
        assert_eq!(c.journal.len(), longueur);
        assert_eq!(equipement(&mut c, &r, pj).bouclier, None);
    }
}
