//! Modèles personnels de bâtiments et d'organisations : enregistrés à partir d'un bâtiment ou
//! d'une organisation existants, réutilisables dans toutes les campagnes. Données
//! personnalisées, rangées à part des modèles officiels (fichier `modeles_personnels.json`).

use crate::evenements::Evenement;
use crate::modele::*;
use domaine::fiches::{ElementModele, Modele};
use domaine::{CoutCapitaux, Source};
use regles::Referentiel;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Préfixe des identifiants de modèles personnels dans les listes de choix.
pub const PREFIXE: &str = "perso:";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtagePerso {
    pub nom: String,
    pub mode: ModeChantier,
    /// Salles du référentiel (identifiants), dans l'ordre.
    pub salles: Vec<String>,
    /// Extensions des salles (GC p. 108), repérées par le rang de la salle dans l'étage.
    #[serde(default)]
    pub extensions: Vec<ExtensionPerso>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionPerso {
    /// Rang de la salle dans `salles`.
    pub salle: usize,
    pub reference: String,
    pub cout: CoutCapitaux,
    pub prix_po: u32,
    pub duree: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatimentPerso {
    pub nom: String,
    pub etages: Vec<EtagePerso>,
    pub etages_successifs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganisationPerso {
    pub nom: String,
    /// Équipes du référentiel (identifiants).
    pub equipes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelesPerso {
    #[serde(default)]
    pub batiments: Vec<BatimentPerso>,
    #[serde(default)]
    pub organisations: Vec<OrganisationPerso>,
}

fn fichier(dossier_campagnes: &Path) -> PathBuf {
    dossier_campagnes
        .parent()
        .unwrap_or(dossier_campagnes)
        .join("modeles_personnels.json")
}

/// Résumé « 2 × Bar, Cuisine » d'une liste d'identifiants.
fn resume(ids: &[String], nom: impl Fn(&str) -> String) -> String {
    let mut v: Vec<(String, u32)> = Vec::new();
    for id in ids {
        let n = nom(id);
        match v.iter_mut().find(|(x, _)| *x == n) {
            Some((_, q)) => *q += 1,
            None => v.push((n, 1)),
        }
    }
    v.into_iter()
        .map(|(n, q)| if q > 1 { format!("{q} × {n}") } else { n })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Fiche de modèle synthétisée (coût et prix additionnés), pour réutiliser l'écran de création.
fn synthese(
    id: String,
    nom: &str,
    ids: &[String],
    fiche: impl Fn(&str) -> Option<(CoutCapitaux, u32)>,
    imprimee: String,
) -> Modele {
    let (cout, prix) = ids
        .iter()
        .filter_map(|i| fiche(i))
        .fold((CoutCapitaux::default(), 0), |(c, p), (d, q)| {
            (c + d, p + q)
        });
    Modele {
        id,
        nom: nom.to_string(),
        source: Source {
            livre: "Personnel".into(),
            page: None,
            section: "Modèle personnel".into(),
        },
        cout,
        prix_po: prix,
        composition: ids
            .iter()
            .map(|i| ElementModele::Unite {
                id: i.clone(),
                quantite: 1,
            })
            .collect(),
        composition_imprimee: imprimee,
        point_a_confirmer: None,
    }
}

impl BatimentPerso {
    pub fn depuis(etat: &crate::etat::Etat, b: &Batiment, nom: &str) -> Self {
        BatimentPerso {
            nom: nom.to_string(),
            etages_successifs: b.etages_successifs,
            etages: b
                .etages
                .iter()
                .map(|e| {
                    let salles: Vec<&SalleBatie> = etat.salles_etage(e.id).collect();
                    EtagePerso {
                        nom: e.nom.clone(),
                        mode: e.mode,
                        salles: salles.iter().map(|s| s.reference.clone()).collect(),
                        extensions: salles
                            .iter()
                            .enumerate()
                            .flat_map(|(i, s)| {
                                s.extensions.iter().map(move |x| ExtensionPerso {
                                    salle: i,
                                    reference: x.reference.clone(),
                                    cout: x.cout,
                                    prix_po: x.prix_po,
                                    duree: x.duree,
                                })
                            })
                            .collect(),
                    }
                })
                .collect(),
        }
    }

    fn toutes(&self) -> Vec<String> {
        self.etages.iter().flat_map(|e| e.salles.clone()).collect()
    }

    pub fn en_modele(&self, r: &Referentiel) -> Modele {
        let nom_salle = |id: &str| r.salle(id).map(|u| u.nom.clone()).unwrap_or(id.into());
        let imprimee = self
            .etages
            .iter()
            .map(|e| format!("{} : {}", e.nom, resume(&e.salles, nom_salle)))
            .collect::<Vec<_>>()
            .join(" ; ");
        let mut m = synthese(
            format!("{PREFIXE}{}", self.nom),
            &self.nom,
            &self.toutes(),
            |i| {
                r.salle(i)
                    .map(|u| (u.cout.unwrap_or_default(), u.prix_po.unwrap_or(0)))
            },
            imprimee,
        );
        // les extensions comptent dans le coût et le prix du modèle
        let nb = self
            .etages
            .iter()
            .map(|e| e.extensions.len())
            .sum::<usize>();
        for x in self.etages.iter().flat_map(|e| &e.extensions) {
            m.cout = m.cout + x.cout;
            m.prix_po += x.prix_po;
        }
        if nb > 0 {
            m.composition_imprimee
                .push_str(&format!(" ; {nb} extension(s) de salle"));
        }
        m
    }

    /// Événements de création : étages et salles à construire, comme dans le modèle.
    pub fn evenements(
        &self,
        batiment: IdBatiment,
        localite: IdLocalite,
        nom: &str,
        premier_etage: IdEtage,
        premiere_salle: IdSalle,
    ) -> Vec<Evenement> {
        let mut evs = vec![Evenement::BatimentCree {
            id: batiment,
            localite,
            nom: nom.to_string(),
            classement: self.nom.clone(),
            notes: String::new(),
            etages_successifs: self.etages_successifs,
            modele: None,
        }];
        let (mut e, mut s) = (premier_etage.0, premiere_salle.0);
        for etage in &self.etages {
            evs.push(Evenement::EtageAjoute {
                batiment,
                id: IdEtage(e),
                nom: etage.nom.clone(),
                mode: etage.mode,
            });
            let debut = s;
            for reference in &etage.salles {
                evs.push(Evenement::SalleAjoutee {
                    id: IdSalle(s),
                    batiment,
                    etage: IdEtage(e),
                    reference: reference.clone(),
                    nom: None,
                    deja_construite: false,
                });
                s += 1;
            }
            for x in etage
                .extensions
                .iter()
                .filter(|x| x.salle < etage.salles.len())
            {
                evs.push(Evenement::ExtensionAjoutee {
                    salle: IdSalle(debut + x.salle as u32),
                    reference: x.reference.clone(),
                    deja_construite: false,
                    cout: Some(x.cout),
                    prix_po: Some(x.prix_po),
                    duree: Some(x.duree),
                });
            }
            e += 1;
        }
        evs
    }
}

impl OrganisationPerso {
    pub fn depuis(etat: &crate::etat::Etat, o: &Organisation, nom: &str) -> Self {
        OrganisationPerso {
            nom: nom.to_string(),
            equipes: etat
                .equipes_organisation(o.id)
                .map(|e| e.reference.clone())
                .collect(),
        }
    }

    pub fn en_modele(&self, r: &Referentiel) -> Modele {
        let nom_eq = |id: &str| r.equipe(id).map(|u| u.nom.clone()).unwrap_or(id.into());
        synthese(
            format!("{PREFIXE}{}", self.nom),
            &self.nom,
            &self.equipes,
            |i| {
                r.equipe(i)
                    .map(|u| (u.cout.unwrap_or_default(), u.prix_po.unwrap_or(0)))
            },
            resume(&self.equipes, nom_eq),
        )
    }
}

impl ModelesPerso {
    pub fn charger(dossier_campagnes: &Path) -> Self {
        std::fs::read_to_string(fichier(dossier_campagnes))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn enregistrer(&self, dossier_campagnes: &Path) -> Result<(), String> {
        let f = fichier(dossier_campagnes);
        if let Some(p) = f.parent() {
            std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
        }
        let t = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&f, t).map_err(|e| e.to_string())
    }

    pub fn batiment(&self, id: &str) -> Option<&BatimentPerso> {
        let nom = id.strip_prefix(PREFIXE)?;
        self.batiments.iter().find(|m| m.nom == nom)
    }

    pub fn organisation(&self, id: &str) -> Option<&OrganisationPerso> {
        let nom = id.strip_prefix(PREFIXE)?;
        self.organisations.iter().find(|m| m.nom == nom)
    }

    /// Ajoute ou remplace (même nom).
    pub fn ajouter_batiment(&mut self, m: BatimentPerso) {
        self.batiments.retain(|x| x.nom != m.nom);
        self.batiments.push(m);
        self.batiments.sort_by(|a, b| a.nom.cmp(&b.nom));
    }

    pub fn ajouter_organisation(&mut self, m: OrganisationPerso) {
        self.organisations.retain(|x| x.nom != m.nom);
        self.organisations.push(m);
        self.organisations.sort_by(|a, b| a.nom.cmp(&b.nom));
    }
}
