//! Pages Notes : feuille de session, journal de campagne, notes de MJ, et la chronologie qui les
//! réunit aux journaux automatiques (intermède, inventaire, aventure) sans les mélanger à la source.

use campagne::evenements::Evenement;
use campagne::Campagne;
use notes::{Critere, Genre, Note, Quand};
use regles::Referentiel;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct RubriqueVue {
    pub id: String,
    pub nom: String,
}

/// Les rubriques du journal de campagne (`data/notes/rubriques.json`).
pub fn rubriques() -> Vec<RubriqueVue> {
    #[derive(serde::Deserialize)]
    struct Fichier {
        rubriques: Vec<Rub>,
    }
    #[derive(serde::Deserialize)]
    struct Rub {
        id: String,
        nom: String,
    }
    let f: Fichier = serde_json::from_str(include_str!("../../../data/notes/rubriques.json"))
        .expect("rubriques.json valide");
    f.rubriques
        .into_iter()
        .map(|r| RubriqueVue {
            id: r.id,
            nom: r.nom,
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct HorlogeVue {
    pub jour: u32,
    pub date: String,
    /// Heure de l'horloge de campagne, quand le mode aventure est en cours.
    pub heure: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PageNotes {
    pub rubriques: Vec<RubriqueVue>,
    pub etiquettes: Vec<String>,
    pub horloge: HorlogeVue,
    pub nombre: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct MentionVue {
    pub genre: String,
    pub cible: String,
    pub libelle: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct NoteVue {
    pub id: u64,
    /// « session », « journal » ou « mj ».
    pub genre: String,
    pub genre_nom: String,
    pub rubrique: Option<String>,
    pub rubrique_nom: Option<String>,
    pub titre: String,
    pub texte: String,
    pub etiquettes: Vec<String>,
    pub epingle: bool,
    pub jour: u32,
    pub date: String,
    pub heure: Option<String>,
    /// Heure réelle, secondes depuis 1970 (UTC).
    pub reel: i64,
    pub mentions: Vec<MentionVue>,
}

/// Jour et heure de la campagne, avec l'heure réelle donnée : de quoi horodater une nouvelle note.
pub fn quand_courant(c: &mut Campagne, r: &Referentiel, reel: i64) -> Quand {
    let etat = c.etat(r);
    let heure = etat.aventure.as_ref().map(|a| {
        format!(
            "{:02}:{:02}",
            a.horloge.maintenant.heure(),
            a.horloge.maintenant.minute()
        )
    });
    Quand {
        jour: etat.jour,
        heure,
        reel,
    }
}

pub fn page_notes(c: &mut Campagne, r: &Referentiel) -> PageNotes {
    let quand = quand_courant(c, r, 0);
    let (etiquettes, nombre) = {
        let etat = c.etat(r);
        (etat.notes.etiquettes(), etat.notes.nombre())
    };
    PageNotes {
        rubriques: rubriques(),
        etiquettes,
        horloge: HorlogeVue {
            jour: quand.jour,
            date: c.calendrier.libelle(quand.jour),
            heure: quand.heure,
        },
        nombre,
    }
}

fn note_vue(n: &Note, c: &Campagne, noms: &[RubriqueVue]) -> NoteVue {
    NoteVue {
        id: n.id,
        genre: n.genre.identifiant().to_string(),
        genre_nom: n.genre.nom().to_string(),
        rubrique: n.rubrique.clone(),
        rubrique_nom: n.rubrique.as_ref().map(|id| {
            noms.iter()
                .find(|r| &r.id == id)
                .map_or_else(|| id.clone(), |r| r.nom.clone())
        }),
        titre: n.titre.clone(),
        texte: n.texte.clone(),
        etiquettes: n.etiquettes.clone(),
        epingle: n.epingle,
        jour: n.quand.jour,
        date: c.calendrier.libelle(n.quand.jour),
        heure: n.quand.heure.clone(),
        reel: n.quand.reel,
        mentions: n
            .mentions()
            .into_iter()
            .map(|m| MentionVue {
                genre: m.genre,
                cible: m.cible,
                libelle: m.libelle,
            })
            .collect(),
    }
}

/// Les notes qui répondent au critère : épinglées d'abord, puis de la plus récente à la plus ancienne.
pub fn liste_notes(c: &mut Campagne, r: &Referentiel, critere: &Critere) -> Vec<NoteVue> {
    let noms = rubriques();
    let trouvees: Vec<Note> = c
        .etat(r)
        .notes
        .rechercher(critere)
        .into_iter()
        .cloned()
        .collect();
    trouvees.iter().map(|n| note_vue(n, c, &noms)).collect()
}

/// Export Markdown des notes qui répondent au critère.
pub fn exporter_notes(c: &mut Campagne, r: &Referentiel, critere: &Critere) -> String {
    let cal = c.calendrier.clone();
    c.etat(r)
        .notes
        .exporter_markdown(critere, &|jour| cal.libelle(jour))
}

/// D'où vient une ligne de la chronologie. Les journaux restent distincts à la source.
pub const SOURCES: [(&str, &str); 6] = [
    ("note-session", "Feuille de session"),
    ("note-journal", "Journal de campagne"),
    ("note-mj", "Notes de MJ"),
    ("intermede", "Intermède"),
    ("inventaire", "Inventaire"),
    ("aventure", "Aventure"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct LigneChrono {
    pub source: String,
    pub source_nom: String,
    pub jour: u32,
    pub date: String,
    pub heure: Option<String>,
    pub texte: String,
    /// La note correspondante, pour l'ouvrir.
    pub note: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase", default))]
pub struct FiltreChrono {
    /// Sources à montrer ; vide = toutes.
    pub sources: Vec<String>,
    pub texte: Option<String>,
}

fn source_nom(id: &str) -> String {
    SOURCES
        .iter()
        .find(|(s, _)| *s == id)
        .map_or_else(|| id.to_string(), |(_, nom)| (*nom).to_string())
}

fn est_inventaire(e: &Evenement) -> bool {
    match e {
        Evenement::Inventaire(_) => true,
        Evenement::Groupe { evenements, .. } => evenements.iter().any(est_inventaire),
        _ => false,
    }
}

/// La chronologie unifiée, de la plus récente à la plus ancienne : notes, journal d'intermède,
/// actions d'inventaire et journal d'aventure.
pub fn chronologie(c: &mut Campagne, r: &Referentiel, filtre: &FiltreChrono) -> Vec<LigneChrono> {
    // (jour, minute du jour, rang) : le rang garde l'ordre d'écriture à l'intérieur d'une source.
    let mut lignes: Vec<((u32, u32, u64), LigneChrono)> = Vec::new();
    let cal = c.calendrier.clone();
    let minutes = |h: &Option<String>| {
        h.as_deref()
            .and_then(|h| h.split_once(':'))
            .and_then(|(a, b)| Some(a.parse::<u32>().ok()? * 60 + b.parse::<u32>().ok()?))
            .unwrap_or(0)
    };

    let toutes: Vec<Note> = c.etat(r).notes.notes().cloned().collect();
    for n in &toutes {
        let source = match n.genre {
            Genre::Session => "note-session",
            Genre::Journal => "note-journal",
            Genre::Mj => "note-mj",
        };
        let titre = if n.titre.is_empty() {
            n.texte.lines().next().unwrap_or("").to_string()
        } else {
            n.titre.clone()
        };
        lignes.push((
            (n.quand.jour, minutes(&n.quand.heure), n.id),
            LigneChrono {
                source: source.to_string(),
                source_nom: source_nom(source),
                jour: n.quand.jour,
                date: cal.libelle(n.quand.jour),
                heure: n.quand.heure.clone(),
                texte: titre,
                note: Some(n.id),
            },
        ));
    }

    let actives: Vec<_> = c.historique().cloned().collect();
    for e in &actives {
        if matches!(e.evenement, Evenement::Notes(_)) {
            continue; // déjà montrée comme note
        }
        let source = if est_inventaire(&e.evenement) {
            "inventaire"
        } else {
            "intermede"
        };
        lignes.push((
            (e.jour, 0, e.numero),
            LigneChrono {
                source: source.to_string(),
                source_nom: source_nom(source),
                jour: e.jour,
                date: cal.libelle(e.jour),
                heure: None,
                texte: c.decrire(&e.evenement, r),
                note: None,
            },
        ));
    }

    let entrees: Vec<(u32, u32, String)> = c
        .etat(r)
        .aventure
        .as_ref()
        .map(|a| {
            a.journal
                .iter()
                .map(|e| {
                    (
                        u32::try_from(e.instant.jour()).unwrap_or(0),
                        e.instant.heure() * 60 + e.instant.minute(),
                        e.texte.clone(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    for (rang, (jour, minute, texte)) in entrees.into_iter().enumerate() {
        lignes.push((
            (jour, minute, rang as u64),
            LigneChrono {
                source: "aventure".to_string(),
                source_nom: source_nom("aventure"),
                jour,
                date: cal.libelle(jour),
                heure: Some(format!("{:02}:{:02}", minute / 60, minute % 60)),
                texte,
                note: None,
            },
        ));
    }

    let mots: Vec<String> = filtre
        .texte
        .as_deref()
        .map(notes::texte_simple)
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_string)
        .collect();
    lignes.sort_by(|a, b| b.0.cmp(&a.0));
    lignes
        .into_iter()
        .map(|(_, l)| l)
        .filter(|l| filtre.sources.is_empty() || filtre.sources.contains(&l.source))
        .filter(|l| {
            let foin = notes::texte_simple(&l.texte);
            mots.iter().all(|m| foin.contains(m))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use notes::Commande;

    fn creer(
        genre: Genre,
        rubrique: Option<&str>,
        titre: &str,
        texte: &str,
        jour: u32,
    ) -> Evenement {
        Evenement::Notes(Commande::Creer {
            genre,
            rubrique: rubrique.map(String::from),
            titre: titre.into(),
            texte: texte.into(),
            etiquettes: vec!["Ameiko".into()],
            quand: Quand {
                jour,
                heure: Some("14:30".into()),
                reel: 1_000 + i64::from(jour),
            },
        })
    }

    fn campagne() -> (Campagne, Referentiel) {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        c.executer(Evenement::JoursAvances { nombre: 2 }, &r)
            .unwrap();
        c.executer(
            creer(Genre::Session, None, "Séance 1", "Le dragon dort", 3),
            &r,
        )
        .unwrap();
        c.executer(
            creer(
                Genre::Journal,
                Some("pnj"),
                "Ameiko",
                "Aubergiste de Sandpoint",
                3,
            ),
            &r,
        )
        .unwrap();
        c.executer(
            creer(Genre::Mj, None, "Le MJ a dit", "Le pont est piégé", 3),
            &r,
        )
        .unwrap();
        (c, r)
    }

    #[test]
    fn les_rubriques_viennent_des_donnees() {
        let r = rubriques();
        assert_eq!(r.len(), 6);
        assert!(r.iter().any(|x| x.id == "promesse-dette"));
    }

    #[test]
    fn liste_et_recherche() {
        let (mut c, r) = campagne();
        let toutes = liste_notes(&mut c, &r, &Critere::default());
        assert_eq!(toutes.len(), 3);
        let pnj = liste_notes(
            &mut c,
            &r,
            &Critere {
                genre: Some(Genre::Journal),
                rubrique: Some("pnj".into()),
                ..Default::default()
            },
        );
        assert_eq!(pnj.len(), 1);
        assert_eq!(pnj[0].rubrique_nom.as_deref(), Some("PNJ"));
        assert_eq!(pnj[0].genre_nom, "Journal de campagne");
        let piege = liste_notes(
            &mut c,
            &r,
            &Critere {
                texte: Some("PIÉGÉ".into()),
                ..Default::default()
            },
        );
        assert_eq!(piege.len(), 1);
        assert_eq!(piege[0].genre, "mj");
        let page = page_notes(&mut c, &r);
        assert_eq!(page.nombre, 3);
        assert_eq!(page.etiquettes, vec!["Ameiko"]);
    }

    #[test]
    fn chronologie_unifiee_et_filtres() {
        let (mut c, r) = campagne();
        let tout = chronologie(&mut c, &r, &FiltreChrono::default());
        // Journal d'intermède (jours avancés) + trois notes ; les événements « Notes » ne sont pas répétés.
        assert_eq!(tout.len(), 4, "{tout:#?}");
        let sources: Vec<&str> = tout.iter().map(|l| l.source.as_str()).collect();
        assert!(sources.contains(&"intermede"));
        assert!(sources.contains(&"note-mj"));
        // La plus récente d'abord : les notes du jour 3 avant l'avance de deux jours.
        assert_eq!(tout.last().unwrap().source, "intermede");
        let seulement_mj = chronologie(
            &mut c,
            &r,
            &FiltreChrono {
                sources: vec!["note-mj".into()],
                texte: None,
            },
        );
        assert_eq!(seulement_mj.len(), 1);
        // La chronologie cherche dans le titre de la note, pas dans son texte.
        let texte = chronologie(
            &mut c,
            &r,
            &FiltreChrono {
                sources: vec![],
                texte: Some("séance".into()),
            },
        );
        assert_eq!(texte.len(), 1);
    }

    #[test]
    fn export_des_notes() {
        let (mut c, r) = campagne();
        let md = exporter_notes(&mut c, &r, &Critere::default());
        assert!(md.contains("## Ameiko"));
        assert!(md.contains("Notes de MJ"));
    }

    #[test]
    fn horodatage_courant() {
        let (mut c, r) = campagne();
        let q = quand_courant(&mut c, &r, 42);
        assert_eq!(q.reel, 42);
        assert_eq!(q.heure, None); // pas de mode aventure : pas d'horloge de campagne
    }
}
