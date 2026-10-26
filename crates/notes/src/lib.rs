//! Prise de notes pendant la partie, sans interface.
//!
//! Trois genres de notes, au même modèle :
//! - la **feuille de session** : ce qu'on note pendant la séance ;
//! - le **journal de campagne** : des entrées classées par rubrique (événement, PNJ, lieu, quête,
//!   indice, promesse ou dette), consultables par rubrique ;
//! - les **notes de MJ** : ce que le MJ a dit ou consigné à la table.
//!
//! Chaque note a un texte en Markdown, des étiquettes, une épingle, et deux horodatages : l'horloge de
//! campagne (jour et heure du mode aventure) et l'heure réelle. Les liens internes s'écrivent
//! `[[genre:cible|libellé]]` (par exemple `[[objet:12|Épée longue]]`). Les rubriques sont des
//! données (`data/notes/rubriques.json`) : le moteur n'en connaît aucune.

use std::collections::BTreeMap;
use std::fmt;

pub type Id = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Genre {
    /// Feuille de session.
    Session,
    /// Journal de campagne structuré.
    Journal,
    /// Notes de MJ.
    Mj,
}

impl Genre {
    pub fn identifiant(self) -> &'static str {
        match self {
            Genre::Session => "session",
            Genre::Journal => "journal",
            Genre::Mj => "mj",
        }
    }

    pub fn nom(self) -> &'static str {
        match self {
            Genre::Session => "Feuille de session",
            Genre::Journal => "Journal de campagne",
            Genre::Mj => "Notes de MJ",
        }
    }
}

/// Quand la note a été prise.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Quand {
    /// Jour de la campagne.
    pub jour: u32,
    /// Heure de l'horloge de campagne (« 14:30 ») quand le mode aventure est en cours.
    pub heure: Option<String>,
    /// Heure réelle, en secondes depuis le 1er janvier 1970 (UTC).
    pub reel: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Note {
    pub id: Id,
    pub genre: Genre,
    /// Rubrique du journal de campagne (identifiant défini en données).
    pub rubrique: Option<String>,
    pub titre: String,
    /// Markdown.
    pub texte: String,
    pub etiquettes: Vec<String>,
    pub epingle: bool,
    pub quand: Quand,
}

/// Un lien interne trouvé dans le texte : `[[genre:cible|libellé]]`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Mention {
    pub genre: String,
    pub cible: String,
    pub libelle: String,
}

impl Note {
    pub fn mentions(&self) -> Vec<Mention> {
        mentions(&self.texte)
    }
}

/// Les liens internes d'un texte. Un lien mal formé reste du texte.
pub fn mentions(texte: &str) -> Vec<Mention> {
    let mut trouvees = Vec::new();
    let mut reste = texte;
    while let Some(debut) = reste.find("[[") {
        let apres = &reste[debut + 2..];
        let Some(fin) = apres.find("]]") else { break };
        let interieur = &apres[..fin];
        if let Some((genre, suite)) = interieur.split_once(':') {
            let (cible, libelle) = suite.split_once('|').unwrap_or((suite, suite));
            if !genre.trim().is_empty() && !cible.trim().is_empty() {
                trouvees.push(Mention {
                    genre: genre.trim().to_string(),
                    cible: cible.trim().to_string(),
                    libelle: libelle.trim().to_string(),
                });
            }
        }
        reste = &apres[fin + 2..];
    }
    trouvees
}

/// Texte comparable pour la recherche : minuscules, sans accents.
pub fn texte_simple(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            '’' => '\'',
            c => c,
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Erreur {
    NoteInconnue(Id),
    /// Une note a au moins un titre ou un texte.
    NoteVide,
    /// Une entrée du journal de campagne a une rubrique.
    RubriqueManquante,
}

impl fmt::Display for Erreur {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Erreur::NoteInconnue(id) => write!(f, "note inconnue : {id}"),
            Erreur::NoteVide => write!(f, "la note est vide : donnez-lui un titre ou un texte"),
            Erreur::RubriqueManquante => {
                write!(
                    f,
                    "une entrée du journal de campagne a besoin d'une rubrique"
                )
            }
        }
    }
}

impl std::error::Error for Erreur {}

/// Opération sur les notes, enregistrée dans le journal de campagne.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Commande {
    Creer {
        genre: Genre,
        rubrique: Option<String>,
        titre: String,
        texte: String,
        etiquettes: Vec<String>,
        quand: Quand,
    },
    Modifier {
        id: Id,
        rubrique: Option<String>,
        titre: String,
        texte: String,
        etiquettes: Vec<String>,
    },
    Epingler {
        id: Id,
        epingle: bool,
    },
    Supprimer {
        id: Id,
    },
}

/// Ce qu'on cherche : tous les critères donnés doivent être vrais.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase", default))]
pub struct Critere {
    pub genre: Option<Genre>,
    pub rubrique: Option<String>,
    pub etiquette: Option<String>,
    /// Plein texte : tous les mots, sans accents ni casse, dans le titre, le texte ou les étiquettes.
    pub texte: Option<String>,
    pub epingles_seulement: bool,
}

/// L'ensemble des notes d'une campagne.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Registre {
    notes: BTreeMap<Id, Note>,
    dernier_id: Id,
}

fn nettoyer(etiquettes: &[String]) -> Vec<String> {
    let mut propres: Vec<String> = Vec::new();
    for e in etiquettes {
        let e = e.trim().trim_start_matches('#').trim();
        if !e.is_empty() && !propres.iter().any(|p| texte_simple(p) == texte_simple(e)) {
            propres.push(e.to_string());
        }
    }
    propres
}

fn verifier(
    genre: Genre,
    rubrique: &Option<String>,
    titre: &str,
    texte: &str,
) -> Result<(), Erreur> {
    if titre.trim().is_empty() && texte.trim().is_empty() {
        return Err(Erreur::NoteVide);
    }
    if genre == Genre::Journal && rubrique.as_deref().is_none_or(|r| r.trim().is_empty()) {
        return Err(Erreur::RubriqueManquante);
    }
    Ok(())
}

impl Registre {
    pub fn note(&self, id: Id) -> Option<&Note> {
        self.notes.get(&id)
    }

    pub fn notes(&self) -> impl Iterator<Item = &Note> + '_ {
        self.notes.values()
    }

    pub fn nombre(&self) -> usize {
        self.notes.len()
    }

    /// Toutes les étiquettes utilisées, sans doublon, dans l'ordre alphabétique.
    pub fn etiquettes(&self) -> Vec<String> {
        let mut toutes: Vec<String> = Vec::new();
        for n in self.notes.values() {
            for e in &n.etiquettes {
                if !toutes.iter().any(|t| texte_simple(t) == texte_simple(e)) {
                    toutes.push(e.clone());
                }
            }
        }
        toutes.sort_by_key(|e| texte_simple(e));
        toutes
    }

    /// Applique une commande. Les identifiants sont attribués dans l'ordre : rejouer le même
    /// journal donne le même registre.
    pub fn appliquer(&mut self, commande: &Commande) -> Result<Option<Id>, Erreur> {
        match commande {
            Commande::Creer {
                genre,
                rubrique,
                titre,
                texte,
                etiquettes,
                quand,
            } => {
                verifier(*genre, rubrique, titre, texte)?;
                self.dernier_id += 1;
                let id = self.dernier_id;
                self.notes.insert(
                    id,
                    Note {
                        id,
                        genre: *genre,
                        rubrique: rubrique.clone().filter(|r| !r.trim().is_empty()),
                        titre: titre.trim().to_string(),
                        texte: texte.clone(),
                        etiquettes: nettoyer(etiquettes),
                        epingle: false,
                        quand: quand.clone(),
                    },
                );
                Ok(Some(id))
            }
            Commande::Modifier {
                id,
                rubrique,
                titre,
                texte,
                etiquettes,
            } => {
                let genre = self.notes.get(id).ok_or(Erreur::NoteInconnue(*id))?.genre;
                verifier(genre, rubrique, titre, texte)?;
                let n = self.notes.get_mut(id).ok_or(Erreur::NoteInconnue(*id))?;
                n.rubrique = rubrique.clone().filter(|r| !r.trim().is_empty());
                n.titre = titre.trim().to_string();
                n.texte = texte.clone();
                n.etiquettes = nettoyer(etiquettes);
                Ok(None)
            }
            Commande::Epingler { id, epingle } => {
                self.notes
                    .get_mut(id)
                    .ok_or(Erreur::NoteInconnue(*id))?
                    .epingle = *epingle;
                Ok(None)
            }
            Commande::Supprimer { id } => self
                .notes
                .remove(id)
                .map(|_| None)
                .ok_or(Erreur::NoteInconnue(*id)),
        }
    }

    /// Les notes qui répondent au critère : épinglées d'abord, puis de la plus récente à la plus
    /// ancienne (jour de campagne, heure, heure réelle).
    pub fn rechercher(&self, critere: &Critere) -> Vec<&Note> {
        let mots: Vec<String> = critere
            .texte
            .as_deref()
            .map(texte_simple)
            .unwrap_or_default()
            .split_whitespace()
            .map(str::to_string)
            .collect();
        let etiquette = critere.etiquette.as_deref().map(texte_simple);
        let mut trouvees: Vec<&Note> = self
            .notes
            .values()
            .filter(|n| critere.genre.is_none_or(|g| n.genre == g))
            .filter(|n| {
                critere
                    .rubrique
                    .as_ref()
                    .is_none_or(|r| n.rubrique.as_ref() == Some(r))
            })
            .filter(|n| !critere.epingles_seulement || n.epingle)
            .filter(|n| {
                etiquette
                    .as_ref()
                    .is_none_or(|e| n.etiquettes.iter().any(|x| &texte_simple(x) == e))
            })
            .filter(|n| {
                let foin = texte_simple(&format!(
                    "{} {} {}",
                    n.titre,
                    n.texte,
                    n.etiquettes.join(" ")
                ));
                mots.iter().all(|m| foin.contains(m))
            })
            .collect();
        trouvees.sort_by(|a, b| {
            b.epingle
                .cmp(&a.epingle)
                .then_with(|| b.quand.cmp(&a.quand))
                .then_with(|| b.id.cmp(&a.id))
        });
        trouvees
    }

    /// Export Markdown des notes qui répondent au critère, de la plus ancienne à la plus récente.
    /// `date` donne le libellé d'un jour de campagne.
    pub fn exporter_markdown(&self, critere: &Critere, date: &dyn Fn(u32) -> String) -> String {
        let mut notes = self.rechercher(critere);
        notes.sort_by(|a, b| a.quand.cmp(&b.quand).then_with(|| a.id.cmp(&b.id)));
        let mut sortie = String::from("# Notes de la campagne\n");
        for n in notes {
            let titre = if n.titre.is_empty() {
                "(sans titre)"
            } else {
                &n.titre
            };
            sortie.push_str(&format!("\n## {titre}\n\n"));
            let mut meta = vec![n.genre.nom().to_string()];
            if let Some(r) = &n.rubrique {
                meta.push(r.clone());
            }
            meta.push(date(n.quand.jour));
            if let Some(h) = &n.quand.heure {
                meta.push(h.clone());
            }
            if n.epingle {
                meta.push("épinglée".to_string());
            }
            sortie.push_str(&format!("*{}*\n", meta.join(" · ")));
            if !n.etiquettes.is_empty() {
                let tags: Vec<String> = n.etiquettes.iter().map(|e| format!("#{e}")).collect();
                sortie.push_str(&format!("\n{}\n", tags.join(" ")));
            }
            if !n.texte.trim().is_empty() {
                sortie.push_str(&format!("\n{}\n", n.texte.trim()));
            }
        }
        sortie
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quand(jour: u32, reel: i64) -> Quand {
        Quand {
            jour,
            heure: Some("14:30".into()),
            reel,
        }
    }

    fn creer(
        genre: Genre,
        rubrique: Option<&str>,
        titre: &str,
        texte: &str,
        jour: u32,
    ) -> Commande {
        Commande::Creer {
            genre,
            rubrique: rubrique.map(String::from),
            titre: titre.into(),
            texte: texte.into(),
            etiquettes: vec!["Clef".into(), "#clef".into(), "  ".into()],
            quand: quand(jour, i64::from(jour) * 1000),
        }
    }

    #[test]
    fn creer_modifier_epingler_supprimer() {
        let mut r = Registre::default();
        let id = r
            .appliquer(&creer(
                Genre::Session,
                None,
                "Auberge",
                "On a rencontré **Ameiko**.",
                3,
            ))
            .unwrap()
            .unwrap();
        assert_eq!(r.note(id).unwrap().etiquettes, vec!["Clef"]);
        r.appliquer(&Commande::Epingler { id, epingle: true })
            .unwrap();
        assert!(r.note(id).unwrap().epingle);
        r.appliquer(&Commande::Modifier {
            id,
            rubrique: None,
            titre: "L'auberge".into(),
            texte: "Autre texte".into(),
            etiquettes: vec![],
        })
        .unwrap();
        assert_eq!(r.note(id).unwrap().titre, "L'auberge");
        r.appliquer(&Commande::Supprimer { id }).unwrap();
        assert_eq!(
            r.appliquer(&Commande::Supprimer { id }),
            Err(Erreur::NoteInconnue(id))
        );
    }

    #[test]
    fn validations() {
        let mut r = Registre::default();
        assert_eq!(
            r.appliquer(&creer(Genre::Session, None, " ", "  ", 1)),
            Err(Erreur::NoteVide)
        );
        assert_eq!(
            r.appliquer(&creer(Genre::Journal, None, "PNJ", "x", 1)),
            Err(Erreur::RubriqueManquante)
        );
        assert!(r
            .appliquer(&creer(Genre::Journal, Some("pnj"), "PNJ", "x", 1))
            .is_ok());
    }

    #[test]
    fn recherche_par_genre_rubrique_etiquette_et_texte() {
        let mut r = Registre::default();
        r.appliquer(&creer(
            Genre::Session,
            None,
            "Séance 1",
            "Le dragon dort",
            1,
        ))
        .unwrap();
        r.appliquer(&creer(
            Genre::Journal,
            Some("pnj"),
            "Ameiko",
            "Aubergiste de Sandpoint",
            2,
        ))
        .unwrap();
        r.appliquer(&creer(
            Genre::Journal,
            Some("quete"),
            "Sauver Ameiko",
            "Gobelins",
            3,
        ))
        .unwrap();
        r.appliquer(&creer(
            Genre::Mj,
            None,
            "Le MJ a dit",
            "Le dragon est rouge",
            4,
        ))
        .unwrap();
        let cherche = |r: &Registre, c: Critere| r.rechercher(&c).len();
        assert_eq!(cherche(&r, Critere::default()), 4);
        assert_eq!(
            cherche(
                &r,
                Critere {
                    genre: Some(Genre::Journal),
                    ..Default::default()
                }
            ),
            2
        );
        assert_eq!(
            cherche(
                &r,
                Critere {
                    rubrique: Some("pnj".into()),
                    ..Default::default()
                }
            ),
            1
        );
        assert_eq!(
            cherche(
                &r,
                Critere {
                    texte: Some("DRAGON rouge".into()),
                    ..Default::default()
                }
            ),
            1
        );
        assert_eq!(
            cherche(
                &r,
                Critere {
                    texte: Some("aubergiste".into()),
                    ..Default::default()
                }
            ),
            1
        );
        assert_eq!(
            cherche(
                &r,
                Critere {
                    texte: Some("séance".into()),
                    ..Default::default()
                }
            ),
            1
        );
        assert_eq!(
            cherche(
                &r,
                Critere {
                    etiquette: Some("CLEF".into()),
                    ..Default::default()
                }
            ),
            4
        );
        // Les plus récentes d'abord ; une note épinglée passe devant.
        assert_eq!(r.rechercher(&Critere::default())[0].titre, "Le MJ a dit");
        r.appliquer(&Commande::Epingler {
            id: 1,
            epingle: true,
        })
        .unwrap();
        assert_eq!(r.rechercher(&Critere::default())[0].titre, "Séance 1");
        assert_eq!(
            cherche(
                &r,
                Critere {
                    epingles_seulement: true,
                    ..Default::default()
                }
            ),
            1
        );
    }

    #[test]
    fn liens_internes() {
        let m = mentions(
            "Voir [[objet:12|Épée longue]], [[note:3]] et [[ ]] ou [[mal]] puis [[lieu:L-1|Sandpoint]].",
        );
        assert_eq!(
            m,
            vec![
                Mention {
                    genre: "objet".into(),
                    cible: "12".into(),
                    libelle: "Épée longue".into()
                },
                Mention {
                    genre: "note".into(),
                    cible: "3".into(),
                    libelle: "3".into()
                },
                Mention {
                    genre: "lieu".into(),
                    cible: "L-1".into(),
                    libelle: "Sandpoint".into()
                },
            ]
        );
    }

    #[test]
    fn export_markdown() {
        let mut r = Registre::default();
        r.appliquer(&creer(
            Genre::Journal,
            Some("pnj"),
            "Ameiko",
            "Aubergiste",
            2,
        ))
        .unwrap();
        r.appliquer(&creer(Genre::Session, None, "Séance 1", "Début", 1))
            .unwrap();
        let md = r.exporter_markdown(&Critere::default(), &|j| format!("jour {j}"));
        assert!(md.starts_with("# Notes de la campagne"));
        assert!(md.find("Séance 1").unwrap() < md.find("Ameiko").unwrap());
        assert!(md.contains("*Journal de campagne · pnj · jour 2 · 14:30*"));
        assert!(md.contains("#Clef"));
    }

    #[test]
    fn rejouer_donne_le_meme_registre() {
        let commandes = [
            creer(Genre::Session, None, "A", "a", 1),
            creer(Genre::Mj, None, "B", "b", 2),
            Commande::Epingler {
                id: 2,
                epingle: true,
            },
        ];
        let rejoue = || {
            let mut r = Registre::default();
            for c in &commandes {
                r.appliquer(c).unwrap();
            }
            r
        };
        assert_eq!(rejoue(), rejoue());
    }
}
