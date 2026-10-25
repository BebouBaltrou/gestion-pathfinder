//! Modèles d'affichage des personnages : liste, fiche complète, catalogue de création.
//! Tout est calculé par le cœur (`fiches`) ; l'interface n'a rien à recalculer.

use crate::Somme;
use campagne::etat::Etat;
use campagne::modele::IdPersonnage;
use fiches::fiche::{Fiche, Probleme};
use fiches::journal::EtatPersonnage;
use fiches::progression::xp_requis;
use fiches::traductions::Traductions;
use fiches::{Carac, Detail, Referentiel};

macro_rules! vue {
    ($(#[$m:meta])* $v:vis struct $n:ident { $($(#[$fm:meta])* $fv:vis $f:ident : $t:ty),* $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize))]
        #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
        $v struct $n { $($(#[$fm])* $fv $f : $t),* }
    };
}

vue! {
    /// Un nombre et le détail de son calcul.
    pub struct ValeurVue {
        pub total: i32,
        pub explication: String,
        pub termes: Vec<TermeVue>,
    }
}
vue! {
    pub struct TermeVue {
        pub libelle: String,
        pub valeur: i32,
        pub compte: bool,
    }
}

fn valeur(d: &Detail) -> ValeurVue {
    ValeurVue {
        total: d.total,
        explication: d.explication(),
        termes: d
            .termes
            .iter()
            .map(|t| TermeVue {
                libelle: t.libelle.clone(),
                valeur: t.valeur,
                compte: t.compte,
            })
            .collect(),
    }
}

vue! {
    pub struct ResumeFiche {
        pub race: String,
        pub classes: String,
        pub niveau: u32,
        pub pv: i32,
        pub ca: i32,
        pub xp: u32,
        pub niveau_atteignable: u32,
        pub niveaux_en_attente: u32,
    }
}
vue! {
    pub struct LignePersonnage {
        pub id: u32,
        pub nom: String,
        pub compte: Somme,
        pub lieu: Option<String>,
        /// Prestige saisi, ou niveau + modificateur de Charisme de la fiche.
        pub prestige: Option<i32>,
        pub prestige_calcule: bool,
        /// Nom du maître, si ce personnage est l'acolyte d'un autre (don Prestige).
        pub acolyte_de: Option<String>,
        pub fiche: Option<ResumeFiche>,
    }
}

fn classes_texte(r: &Referentiel, f: &Fiche) -> String {
    let mut vues: Vec<&str> = Vec::new();
    for n in &f.niveaux {
        if !vues.contains(&n.classe.as_str()) {
            vues.push(&n.classe);
        }
    }
    vues.iter()
        .map(|id| {
            let nom = r
                .classe(id)
                .map(|c| c.nom.clone())
                .unwrap_or(id.to_string());
            format!("{nom} {}", f.niveau_de_classe(id))
        })
        .collect::<Vec<_>>()
        .join(" / ")
}

fn resume(r: &Referentiel, e: &EtatPersonnage) -> Option<ResumeFiche> {
    let v = e.fiche.calculer(r).ok()?;
    Some(ResumeFiche {
        race: r
            .race(&e.fiche.race)
            .map(|x| x.nom.clone())
            .unwrap_or_default(),
        classes: classes_texte(r, &e.fiche),
        niveau: v.niveau_de_personnage,
        pv: v.pv_max.total,
        ca: v.ca.total,
        xp: e.xp,
        niveau_atteignable: e.niveau_atteignable,
        niveaux_en_attente: e.niveaux_en_attente(),
    })
}

/// Liste des PJ avec le résumé de leur fiche.
pub fn liste_personnages(etat: &Etat) -> Vec<LignePersonnage> {
    let r = Referentiel::partage();
    etat.personnages
        .values()
        .map(|p| {
            let e = etat.fiche_de(p.id);
            LignePersonnage {
                id: p.id.0,
                nom: p.nom.clone(),
                compte: p.compte.into(),
                lieu: p
                    .localite
                    .and_then(|l| etat.localites.get(&l))
                    .map(|l| l.nom.clone()),
                prestige: etat.prestige_effectif(p.id),
                prestige_calcule: p.prestige.is_none() && e.is_some(),
                acolyte_de: etat
                    .acolytes
                    .get(&p.id)
                    .and_then(|m| etat.personnages.get(m))
                    .map(|m| m.nom.clone()),
                fiche: e.as_ref().and_then(|e| resume(r, e)),
            }
        })
        .collect()
}

// ------------------------------------------------------------------ fiche complète

vue! {
    pub struct CaracVue {
        pub id: String,
        pub nom: String,
        pub score: ValeurVue,
        pub modificateur: i32,
    }
}
vue! {
    pub struct CompetenceVue {
        pub cle: String,
        pub nom: String,
        pub valeur: ValeurVue,
        pub rangs: u32,
        pub de_classe: bool,
        pub formation_requise: bool,
        pub caracteristique: String,
    }
}
vue! {
    pub struct DonVue {
        pub nom_en: String,
        pub nom: String,
        pub resume: Option<String>,
        pub avantage: Option<String>,
        pub source: Option<String>,
        pub note_livre: Option<String>,
        /// Prérequis non remplis : le don est inutilisable (Core p. 111).
        pub inutilisable: Option<String>,
    }
}
vue! {
    pub struct TexteVue {
        pub nom: String,
        pub texte: String,
    }
}
vue! {
    pub struct EmplacementVue {
        pub niveau_de_sort: u32,
        pub base: Option<u32>,
        pub bonus: u32,
        pub par_jour: Option<u32>,
        pub restants: Option<u32>,
        pub connus: u32,
        pub dd: i32,
    }
}
vue! {
    pub struct RemplacementSortVue {
        pub ancien_en: String,
        pub ancien: String,
        pub niveau_de_sort: u32,
        pub propositions: Vec<SortProposeVue>,
    }
}
vue! {
    pub struct SortProposeVue {
        pub nom_en: String,
        pub nom: String,
    }
}
vue! {
    pub struct SortVue {
        pub nom_en: String,
        pub nom: String,
        pub niveau: u32,
        pub ecole: String,
        /// Obtenu automatiquement par le lignage.
        pub de_lignage: bool,
        /// Statistiques du sort au niveau de lanceur du personnage (affichées au clic).
        pub stats: Option<StatsSortFicheVue>,
    }
}
vue! {
    pub struct StatsSortFicheVue {
        pub niveau_de_lanceur: u32,
        pub temps_d_incantation: Option<String>,
        pub composantes: Option<String>,
        pub portee: Option<String>,
        pub portee_calculee: Option<String>,
        pub cible: Option<String>,
        pub duree: Option<String>,
        pub duree_calculee: Option<String>,
        pub jet_de_sauvegarde: Option<String>,
        /// DD du jet de sauvegarde pour ce niveau de sort.
        pub dd: Option<i32>,
        pub resistance_a_la_magie: Option<String>,
        pub nombre: Option<String>,
        pub degats: Vec<crate::aventure::DegatsSortVue>,
        pub description: String,
        pub source: Option<String>,
        pub note_livre: Option<String>,
    }
}
vue! {
    pub struct IncantateurVue {
        pub classe: String,
        pub niveau_de_lanceur: u32,
        pub emplacements: Vec<EmplacementVue>,
        pub sorts: Vec<SortVue>,
    }
}
vue! {
    pub struct ArmeVue {
        pub nom_en: String,
        pub nom: String,
        pub maitrise: bool,
        pub attaque: ValeurVue,
        pub degats: String,
        pub critique: Option<String>,
        pub portee: Option<String>,
        pub type_degats: Option<String>,
    }
}
vue! {
    pub struct ObjetVue {
        pub nom_en: String,
        pub nom: String,
        pub quantite: u32,
    }
}
vue! {
    pub struct EquipementVue {
        pub armure: Option<ObjetVue>,
        pub bouclier: Option<ObjetVue>,
        pub armes: Vec<ArmeVue>,
        pub objets: Vec<ObjetVue>,
        pub poids_total: f64,
        pub poids_inconnu: Vec<String>,
        pub charge: String,
        /// Limites légère / intermédiaire / lourde de la Force actuelle, en livres.
        pub limites: Option<(f64, f64, f64)>,
        pub dex_max: Option<String>,
        pub malus_d_armure: i32,
        pub source_malus: String,
        pub echec_sorts_pct: u32,
        pub non_maitrises: Vec<String>,
    }
}
vue! {
    pub struct EtatVue {
        pub id: String,
        pub nom: String,
        pub texte: String,
        pub note: Option<String>,
    }
}
vue! {
    pub struct TemporaireVue {
        pub nom: String,
        pub rounds_restants: Option<u32>,
        pub effets: Vec<String>,
    }
}
vue! {
    pub struct IdentiteVue {
        pub nom: String,
        pub race: String,
        pub classes: String,
        pub lignage: Option<String>,
        pub niveau: u32,
        pub xp: u32,
        /// XP du prochain niveau (absent aux jalons ou au niveau 20).
        pub xp_suivant: Option<u32>,
        pub niveau_atteignable: u32,
        pub niveaux_en_attente: u32,
        pub prestige: Option<i32>,
        pub prestige_calcule: bool,
        pub compte: Somme,
    }
}
vue! {
    pub struct FicheVue {
        pub identite: IdentiteVue,
        pub caracteristiques: Vec<CaracVue>,
        pub pv: ValeurVue,
        pub ca: ValeurVue,
        pub ca_contact: ValeurVue,
        pub ca_depourvu: ValeurVue,
        pub initiative: ValeurVue,
        pub vigueur: ValeurVue,
        pub reflexes: ValeurVue,
        pub volonte: ValeurVue,
        pub bba: ValeurVue,
        pub attaques_iteratives: Vec<i32>,
        pub bmo: ValeurVue,
        pub dmd: ValeurVue,
        pub vitesse: u32,
        pub competences: Vec<CompetenceVue>,
        pub rangs_par_niveau: Vec<(u32, u32, u32)>,
        pub dons: Vec<DonVue>,
        pub traits: Vec<TexteVue>,
        pub capacites: Vec<TexteVue>,
        pub incantateurs: Vec<IncantateurVue>,
        pub equipement: EquipementVue,
        pub etats: Vec<EtatVue>,
        pub temporaires: Vec<TemporaireVue>,
        pub notes_etats: Vec<String>,
        /// Ce qui rend la fiche invalide ou incomplète.
        pub problemes: Vec<String>,
        /// Sorts connus à remplacer (doublon ou sort déjà accordé par le lignage), avec propositions.
        pub remplacements_de_sorts: Vec<RemplacementSortVue>,
        pub historique: Vec<String>,
    }
}

fn nom_objet(r: &Referentiel, nom_en: &str) -> String {
    let c = &r.equipement;
    c.armure(nom_en)
        .map(|a| a.nom.clone())
        .or_else(|| c.arme(nom_en).map(|a| a.nom.clone()))
        .or_else(|| c.objet(nom_en).map(|a| a.nom.clone()))
        .unwrap_or_else(|| nom_en.to_string())
}

fn objet_vue(r: &Referentiel, nom_en: &str, quantite: u32) -> ObjetVue {
    ObjetVue {
        nom_en: nom_en.to_string(),
        nom: nom_objet(r, nom_en),
        quantite,
    }
}

/// Construit la fiche complète d'un personnage à partir de son état.
pub fn fiche_complete(
    e: &EtatPersonnage,
    historique: Vec<String>,
    identite: (String, Somme, Option<i32>, bool),
    problemes_supplementaires: Vec<String>,
) -> Result<FicheVue, String> {
    let r = Referentiel::partage();
    let t = Traductions::partage();
    let f = &e.fiche;
    let v = f.calculer(r).map_err(|p| p.to_string())?;
    let mut problemes: Vec<String> = f.valider(r).iter().map(Probleme::to_string).collect();
    problemes.extend(problemes_supplementaires);
    let (nom_pj, compte, prestige, prestige_calcule) = identite;
    let nom_sort = |nom_en: &str| {
        t.sort(nom_en)
            .map(|s| s.nom.clone())
            .unwrap_or_else(|| nom_en.to_string())
    };
    let remplacements: Vec<RemplacementSortVue> = f
        .remplacements_de_sorts(r)
        .into_iter()
        .map(|x| RemplacementSortVue {
            ancien: nom_sort(&x.ancien),
            ancien_en: x.ancien,
            niveau_de_sort: x.niveau_de_sort,
            propositions: x
                .propositions
                .into_iter()
                .map(|p| SortProposeVue {
                    nom: nom_sort(&p),
                    nom_en: p,
                })
                .collect(),
        })
        .collect();

    let classe_principale = f.niveaux.first().and_then(|n| r.classe(&n.classe));
    let lignage = f.lignage.as_ref().and_then(|id| {
        t.lignage(id)
            .map(|l| l.nom.to_string())
            .or_else(|| Some(id.clone()))
    });

    let caracteristiques = Carac::TOUTES
        .into_iter()
        .map(|c| CaracVue {
            id: c.abrege().to_lowercase(),
            nom: nom_carac(c).to_string(),
            score: valeur(&v.scores[&c]),
            modificateur: v.modificateurs[&c],
        })
        .collect();

    // Compétences : toutes, avec leur détail.
    let de_classe_ids: Vec<String> = classe_principale
        .map(|c| c.competences_de_classe_en.clone())
        .unwrap_or_default();
    let mut competences: Vec<CompetenceVue> = v
        .competences
        .iter()
        .map(|(cle, d)| {
            let base = cle.split(':').next().unwrap_or(cle);
            let def = r.competence(base);
            let rangs: u32 = f.niveaux.iter().filter_map(|n| n.rangs.get(cle)).sum();
            let nom_en = def.map(|x| x.nom_en.as_str()).unwrap_or(base);
            CompetenceVue {
                cle: cle.clone(),
                nom: t.competence_avec_sous_type(cle).unwrap_or(cle.clone()),
                valeur: valeur(d),
                rangs,
                de_classe: de_classe_ids
                    .iter()
                    .any(|c| c.to_lowercase().starts_with(&nom_en.to_lowercase())),
                formation_requise: def.is_some_and(|x| x.formation_requise),
                caracteristique: def.map(|x| x.caracteristique.clone()).unwrap_or_default(),
            }
        })
        .collect();
    competences.sort_by_key(|c| c.nom.to_lowercase());

    // Dons : automatiques puis choisis, avec prérequis non remplis.
    let invalides = fiches::prerequis::dons_invalides(r, f, &v);
    let mut dons = Vec::new();
    for d in f.dons_automatiques(r).into_iter().chain(f.dons_choisis()) {
        let fr = t.don(&d);
        let nom = match d.split_once(" (") {
            Some((_, choix)) => format!("{} ({}", fr.map(|x| x.nom.as_str()).unwrap_or(&d), choix),
            None => fr.map(|x| x.nom.clone()).unwrap_or(d.clone()),
        };
        let inutilisable = invalides.iter().find(|(n, _)| *n == d).map(|(_, ver)| {
            ver.details
                .iter()
                .filter_map(|(txt, s)| match s {
                    fiches::prerequis::Statut::Manquant(m) => Some(format!("{txt} : {m}")),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join(" ; ")
        });
        dons.push(DonVue {
            nom_en: d.clone(),
            nom,
            resume: fr.and_then(|x| x.resume.clone()),
            avantage: fr.and_then(|x| x.avantage.clone()),
            source: t.source_vf("dons", &d),
            note_livre: t.note_livre("dons", &d),
            inutilisable,
        });
    }

    // Traits raciaux et capacités de classe.
    let traits = r
        .race(&f.race)
        .map(|race| {
            race.traits
                .iter()
                .map(|tr| match t.trait_racial(&f.race, &tr.nom_en) {
                    Some((n, x)) => TexteVue {
                        nom: n.to_string(),
                        texte: x.to_string(),
                    },
                    None => TexteVue {
                        nom: tr.nom_en.clone(),
                        texte: tr.texte_en.clone(),
                    },
                })
                .collect()
        })
        .unwrap_or_default();
    let mut capacites = Vec::new();
    if let Some(c) = classe_principale {
        let nc = f.niveau_de_classe(&c.id) as usize;
        for l in c.progression.iter().take(nc) {
            for cap in t.capacites_de_niveau(&l.special_en) {
                capacites.push(TexteVue {
                    nom: format!("Niveau {}", l.niveau),
                    texte: cap,
                });
            }
        }
    }

    // Sorts.
    let restants = e.emplacements_restants(r);
    let mut incantateurs = Vec::new();
    for inc in &v.incantateurs {
        let classe = r.classe(&inc.classe);
        let cle = classe.map(|c| c.nom_en.to_lowercase()).unwrap_or_default();
        let mut sorts: Vec<SortVue> = Vec::new();
        let mut ajouter = |nom_en: &str, de_lignage: bool| {
            let def = r.sort(nom_en).or_else(|| r.sort_approx(nom_en));
            let fr = def.and_then(|d| t.sort(&d.nom_en));
            sorts.push(SortVue {
                nom_en: def.map(|d| d.nom_en.clone()).unwrap_or(nom_en.to_string()),
                nom: fr.map(|s| s.nom.clone()).unwrap_or(nom_en.to_string()),
                niveau: def.and_then(|d| d.niveau_pour(&cle)).unwrap_or(0),
                ecole: fr.map(|s| s.ecole.clone()).unwrap_or_default(),
                de_lignage,
                stats: def.map(|d| {
                    let cl = inc.niveau_de_lanceur;
                    let st = combat::stats_sorts::statistiques(d, cl);
                    let niveau = d.niveau_pour(&cle).unwrap_or(0);
                    StatsSortFicheVue {
                        niveau_de_lanceur: cl,
                        temps_d_incantation: fr.and_then(|s| s.temps_d_incantation.clone()),
                        composantes: fr.and_then(|s| s.composantes.clone()),
                        portee: fr.and_then(|s| s.portee.clone()),
                        portee_calculee: st.portee,
                        cible: fr.and_then(|s| s.cible.clone()),
                        duree: fr.and_then(|s| s.duree.clone()),
                        duree_calculee: d
                            .duree_en
                            .as_deref()
                            .and_then(|x| combat::sorts::duree_suggeree(x, cl))
                            .map(|x| x.texte()),
                        jet_de_sauvegarde: fr.and_then(|s| s.jet_de_sauvegarde.clone()),
                        dd: inc
                            .emplacements
                            .iter()
                            .find(|e| e.niveau_de_sort == niveau)
                            .map(|e| e.dd),
                        resistance_a_la_magie: fr.and_then(|s| s.resistance_a_la_magie.clone()),
                        nombre: st.nombre,
                        degats: st
                            .degats
                            .iter()
                            .map(|x| crate::aventure::DegatsSortVue {
                                libelle: x.libelle.clone(),
                                formule: x.formule.clone(),
                                regle: x.regle.clone(),
                                a_confirmer: x.a_confirmer,
                            })
                            .collect(),
                        description: fr.map(|s| s.description.clone()).unwrap_or_default(),
                        source: t.source_vf("sorts", &d.nom_en),
                        note_livre: t.note_livre("sorts", &d.nom_en),
                    }
                }),
            });
        };
        for s in &f.sorts_connus {
            ajouter(s, false);
        }
        for (s, _) in &inc.sorts_de_lignage {
            ajouter(s, true);
        }
        sorts.sort_by_key(|s| (s.niveau, s.nom.clone()));
        incantateurs.push(IncantateurVue {
            classe: classe.map(|c| c.nom.clone()).unwrap_or(inc.classe.clone()),
            niveau_de_lanceur: inc.niveau_de_lanceur,
            emplacements: inc
                .emplacements
                .iter()
                .map(|em| EmplacementVue {
                    niveau_de_sort: em.niveau_de_sort,
                    base: em.base,
                    bonus: em.bonus,
                    par_jour: em.par_jour,
                    restants: restants
                        .iter()
                        .find(|(c, n, _, _)| *c == inc.classe && *n == em.niveau_de_sort)
                        .map(|(_, _, rest, _)| *rest),
                    connus: em.connus,
                    dd: em.dd,
                })
                .collect(),
            sorts,
        });
    }

    let eq = &f.equipement;
    let be = &v.equipement;
    let equipement = EquipementVue {
        armure: eq.armure.as_deref().map(|n| objet_vue(r, n, 1)),
        bouclier: eq.bouclier.as_deref().map(|n| objet_vue(r, n, 1)),
        armes: v
            .armes
            .iter()
            .map(|a| ArmeVue {
                nom_en: a.arme_en.clone(),
                nom: a.nom.clone(),
                maitrise: a.maitrise,
                attaque: valeur(&a.attaque),
                degats: degats_texte(a),
                critique: a.critique.clone(),
                portee: a.portee.clone(),
                type_degats: a.type_degats.clone(),
            })
            .collect(),
        objets: eq
            .objets
            .iter()
            .map(|o| objet_vue(r, &o.nom_en, o.quantite))
            .collect(),
        poids_total: be.poids_total,
        poids_inconnu: be.poids_inconnu.clone(),
        charge: be.charge.nom().to_string(),
        limites: be.limites,
        dex_max: be.dex_max.as_ref().map(|(m, s)| format!("{m:+} ({s})")),
        malus_d_armure: be.malus_d_armure.0,
        source_malus: be.malus_d_armure.1.clone(),
        echec_sorts_pct: be.echec_sorts_pct,
        non_maitrises: be.non_maitrises.clone(),
    };

    let etats = f
        .etats
        .iter()
        .filter_map(|id| r.etat(id))
        .map(|s| EtatVue {
            id: s.id.clone(),
            nom: s.nom.clone(),
            texte: s.texte.clone(),
            note: s.note.clone(),
        })
        .collect();
    let temporaires = f
        .temporaires
        .iter()
        .map(|tmp| TemporaireVue {
            nom: tmp.nom.clone(),
            rounds_restants: tmp.rounds_restants,
            effets: tmp
                .effets
                .iter()
                .map(|x| format!("{:+} {} ({})", x.valeur, x.cible.0, x.type_bonus.nom()))
                .collect(),
        })
        .collect();

    let niveau = v.niveau_de_personnage;
    Ok(FicheVue {
        identite: IdentiteVue {
            nom: nom_pj,
            race: r.race(&f.race).map(|x| x.nom.clone()).unwrap_or_default(),
            classes: classes_texte(r, f),
            lignage,
            niveau,
            xp: e.xp,
            xp_suivant: None,
            niveau_atteignable: e.niveau_atteignable,
            niveaux_en_attente: e.niveaux_en_attente(),
            prestige,
            prestige_calcule,
            compte,
        },
        caracteristiques,
        pv: valeur(&v.pv_max),
        ca: valeur(&v.ca),
        ca_contact: valeur(&v.ca_contact),
        ca_depourvu: valeur(&v.ca_pris_au_depourvu),
        initiative: valeur(&v.initiative),
        vigueur: valeur(&v.vigueur),
        reflexes: valeur(&v.reflexes),
        volonte: valeur(&v.volonte),
        bba: valeur(&v.bba),
        attaques_iteratives: v.bba_iteratif.clone(),
        bmo: valeur(&v.bmo),
        dmd: valeur(&v.dmd),
        vitesse: v.vitesse,
        competences,
        rangs_par_niveau: v
            .rangs
            .iter()
            .map(|b| (b.niveau, b.places, b.disponibles))
            .collect(),
        dons,
        traits,
        capacites,
        incantateurs,
        equipement,
        etats,
        temporaires,
        notes_etats: v.notes_etats.clone(),
        remplacements_de_sorts: remplacements,
        problemes,
        historique,
    })
}

/// Complète l'identité avec les XP du prochain niveau selon la vitesse de progression.
pub fn avec_xp_suivant(mut fiche: FicheVue, vitesse: fiches::progression::Vitesse) -> FicheVue {
    let r = Referentiel::partage();
    fiche.identite.xp_suivant = xp_requis(r, vitesse, fiche.identite.niveau + 1);
    fiche
}

fn degats_texte(a: &fiches::fiche::AttaqueArme) -> String {
    match (&a.des, a.bonus_degats.total) {
        (Some(d), 0) => d.clone(),
        (Some(d), b) => format!("{d}{b:+}"),
        (None, _) => String::new(),
    }
}

fn nom_carac(c: Carac) -> &'static str {
    match c {
        Carac::For => "Force",
        Carac::Dex => "Dextérité",
        Carac::Con => "Constitution",
        Carac::Int => "Intelligence",
        Carac::Sag => "Sagesse",
        Carac::Cha => "Charisme",
    }
}

/// Fiche d'un PJ de la campagne, ou `None` s'il n'en a pas.
pub fn fiche_du_pj(etat: &Etat, id: IdPersonnage) -> Option<Result<FicheVue, String>> {
    let e = etat.fiche_de(id)?;
    let j = etat.fiches.get(&id)?;
    let p = etat.personnages.get(&id)?;
    let prestige = etat.prestige_effectif(id);
    let calcule = p.prestige.is_none();
    let historique = j.historique();
    let vitesse = etat.regles_personnages.vitesse;
    Some(
        fiche_complete(
            &e,
            historique,
            (p.nom.clone(), p.compte.into(), prestige, calcule),
            Vec::new(),
        )
        .map(|f| avec_xp_suivant(f, vitesse)),
    )
}

// ------------------------------------------------------------------ catalogue de création

vue! {
    pub struct RaceVue {
        pub id: String,
        pub nom: String,
        pub taille: String,
        pub vitesse: u32,
        /// Ajustements fixes : (caractéristique, valeur).
        pub ajustements: Vec<(String, i32)>,
        pub choix_libre: Option<i32>,
        pub bonus_don: u32,
        pub rang_supplementaire: u32,
        pub don_impose: Option<String>,
        pub deux_classes_de_predilection: bool,
        pub traits: Vec<TexteVue>,
    }
}
vue! {
    pub struct LignageVue {
        pub id: String,
        pub nom: String,
        pub competence_de_classe: String,
        pub arcane: String,
        /// Sous-type de Connaissances à choisir (lignage arcanique).
        pub choix_connaissance: bool,
        pub sorts: Vec<(String, u32)>,
        pub dons: Vec<String>,
    }
}
vue! {
    pub struct ClasseVue {
        pub id: String,
        pub nom: String,
        pub de_vie: u32,
        pub rangs_par_niveau: i32,
        pub incantation: bool,
        pub caracteristique_incantation: Option<String>,
        pub lignages: Vec<LignageVue>,
        /// Compétences de classe (clés de compétence).
        pub competences_de_classe: Vec<String>,
        pub don_bonus_niveau_1: Option<String>,
        /// Sorts connus (niveau de sort 0 à 9) à chaque niveau de classe.
        pub sorts_connus: Vec<Vec<Option<u32>>>,
    }
}
vue! {
    pub struct CompetenceCatalogue {
        pub id: String,
        pub nom: String,
        pub caracteristique: String,
        pub formation_requise: bool,
        pub malus_d_armure: bool,
        pub sous_types: bool,
    }
}
vue! {
    pub struct DonCatalogue {
        pub nom_en: String,
        pub nom: String,
        pub resume: Option<String>,
        pub prerequis: Option<String>,
        pub repetable: bool,
        pub a_un_choix: bool,
    }
}
vue! {
    pub struct SortCatalogue {
        pub nom_en: String,
        pub nom: String,
        pub ecole: String,
        pub niveau: u32,
        pub resume: String,
        pub source: Option<String>,
        pub note_livre: Option<String>,
    }
}
vue! {
    pub struct EquipementCatalogue {
        pub nom_en: String,
        pub nom: String,
        pub genre: String,
        pub poids_lb: Option<f64>,
        pub source: Option<String>,
        pub note_livre: Option<String>,
        pub description: Option<String>,
    }
}
vue! {
    pub struct EtatCatalogue {
        pub id: String,
        pub nom: String,
    }
}
vue! {
    pub struct CatalogueCreation {
        pub races: Vec<RaceVue>,
        pub classes: Vec<ClasseVue>,
        pub competences: Vec<CompetenceCatalogue>,
        pub dons: Vec<DonCatalogue>,
        /// Sorts de la liste de chaque classe lanceuse de sorts (identifiant de classe), par niveau.
        /// Un personnage ne peut choisir que dans la liste de sa classe.
        pub sorts_par_classe: std::collections::BTreeMap<String, Vec<SortCatalogue>>,
        pub armures: Vec<EquipementCatalogue>,
        pub armes: Vec<EquipementCatalogue>,
        pub materiel: Vec<EquipementCatalogue>,
        pub etats: Vec<EtatCatalogue>,
        /// Coût d'achat de points par score (7 à 18).
        pub cout_achat: Vec<(i32, i32)>,
    }
}

/// Sorts de la liste d'une classe (clé anglaise en minuscules : « sorcerer », « cleric »…),
/// triés par niveau de sort puis par nom.
pub fn sorts_de_la_liste(r: &Referentiel, t: &Traductions, cle: &str) -> Vec<SortCatalogue> {
    let mut sorts: Vec<SortCatalogue> = r
        .sorts
        .iter()
        .filter_map(|s| {
            let niveau = s.niveau_pour(cle)?;
            let fr = t.sort(&s.nom_en);
            Some(SortCatalogue {
                nom_en: s.nom_en.clone(),
                nom: fr.map(|x| x.nom.clone()).unwrap_or(s.nom_en.clone()),
                ecole: fr.map(|x| x.ecole.clone()).unwrap_or_default(),
                niveau,
                resume: fr
                    .map(|x| x.description.chars().take(220).collect())
                    .unwrap_or_default(),
                source: t.source_vf("sorts", &s.nom_en),
                note_livre: t.note_livre("sorts", &s.nom_en),
            })
        })
        .collect();
    sorts.sort_by_key(|s| (s.niveau, s.nom.clone()));
    sorts
}

pub fn catalogue_creation() -> CatalogueCreation {
    let r = Referentiel::partage();
    let t = Traductions::partage();
    let races = r
        .races
        .iter()
        .map(|x| RaceVue {
            id: x.id.clone(),
            nom: x.nom.clone(),
            taille: x.taille.clone(),
            vitesse: x.vitesse,
            ajustements: x.ajustements.clone(),
            choix_libre: x.choix_libre,
            bonus_don: x.bonus_don,
            rang_supplementaire: x.rang_supplementaire,
            don_impose: x.don_impose.clone(),
            deux_classes_de_predilection: x.deux_classes_de_predilection,
            traits: x
                .traits
                .iter()
                .map(|tr| match t.trait_racial(&x.id, &tr.nom_en) {
                    Some((n, c)) => TexteVue {
                        nom: n.to_string(),
                        texte: c.to_string(),
                    },
                    None => TexteVue {
                        nom: tr.nom_en.clone(),
                        texte: tr.texte_en.clone(),
                    },
                })
                .collect(),
        })
        .collect();
    let classes = r
        .classes
        .iter()
        .map(|c| ClasseVue {
            id: c.id.clone(),
            nom: c.nom.clone(),
            de_vie: c.de_vie,
            rangs_par_niveau: c.rangs_par_niveau,
            incantation: c.incantation.is_some(),
            caracteristique_incantation: c.incantation.as_ref().map(|i| i.caracteristique.clone()),
            lignages: c
                .lignages
                .iter()
                .map(|l| {
                    let fr = t.lignage(&l.id);
                    LignageVue {
                        id: l.id.clone(),
                        nom: fr.map(|x| x.nom.to_string()).unwrap_or(l.nom_en.clone()),
                        competence_de_classe: fr
                            .map(|x| x.competence_de_classe.to_string())
                            .unwrap_or(l.competence_de_classe_en.clone()),
                        arcane: fr
                            .map(|x| x.arcane.to_string())
                            .unwrap_or(l.arcane_en.clone()),
                        choix_connaissance: l.competence_de_classe_en.contains("(any one)"),
                        sorts: l
                            .sorts_bonus
                            .iter()
                            .map(|s| {
                                let def = r.sort_approx(&s.nom_en);
                                let nom = def
                                    .and_then(|d| t.sort(&d.nom_en))
                                    .map(|x| x.nom.clone())
                                    .unwrap_or(s.nom_en.clone());
                                (nom, s.niveau_classe)
                            })
                            .collect(),
                        dons: l
                            .dons_bonus_en
                            .iter()
                            .map(|d| t.don(d).map(|x| x.nom.clone()).unwrap_or(d.clone()))
                            .collect(),
                    }
                })
                .collect(),
            competences_de_classe: c.competences_de_classe_en.clone(),
            don_bonus_niveau_1: c.don_bonus_niveau_1.clone(),
            sorts_connus: c
                .progression
                .iter()
                .map(|l| l.sorts_connus.clone())
                .collect(),
        })
        .collect();
    // Connaissances : une entrée par sous-type (Craft, Profession et Perform restent à saisir).
    const CONNAISSANCES: [&str; 10] = [
        "arcana",
        "dungeoneering",
        "engineering",
        "geography",
        "history",
        "local",
        "nature",
        "nobility",
        "planes",
        "religion",
    ];
    let mut competences: Vec<CompetenceCatalogue> = Vec::new();
    for c in &r.competences {
        if c.id == "knowledge" {
            for s in CONNAISSANCES {
                let cle = format!("knowledge:{s}");
                competences.push(CompetenceCatalogue {
                    nom: t.competence_avec_sous_type(&cle).unwrap_or(cle.clone()),
                    id: cle,
                    caracteristique: c.caracteristique.clone(),
                    formation_requise: c.formation_requise,
                    malus_d_armure: c.malus_d_armure,
                    sous_types: false,
                });
            }
        } else {
            competences.push(CompetenceCatalogue {
                id: c.id.clone(),
                nom: t.competence(&c.id).unwrap_or(&c.nom_en).to_string(),
                caracteristique: c.caracteristique.clone(),
                formation_requise: c.formation_requise,
                malus_d_armure: c.malus_d_armure,
                sous_types: c.sous_types,
            });
        }
    }
    let dons = r
        .dons
        .iter()
        .map(|d| {
            let fr = t.don(&d.nom_en);
            DonCatalogue {
                nom_en: d.nom_en.clone(),
                nom: fr.map(|x| x.nom.clone()).unwrap_or(d.nom_en.clone()),
                resume: fr.and_then(|x| x.resume.clone()),
                prerequis: fr.and_then(|x| x.prerequis.clone()),
                repetable: d.repetable(),
                a_un_choix: d.a_un_choix(),
            }
        })
        .collect();
    let sorts_par_classe = r
        .classes
        .iter()
        .filter(|c| c.incantation.is_some())
        .map(|c| {
            (
                c.id.clone(),
                sorts_de_la_liste(r, t, &c.nom_en.to_lowercase()),
            )
        })
        .collect();
    let eq = &r.equipement;
    CatalogueCreation {
        races,
        classes,
        competences,
        dons,
        sorts_par_classe,
        armures: eq
            .armures
            .iter()
            .map(|a| EquipementCatalogue {
                nom_en: a.nom_en.clone(),
                nom: a.nom.clone(),
                genre: a.genre.nom().to_string(),
                poids_lb: a.poids_lb,
                source: t.source_vf("armures", &a.nom_en),
                note_livre: t.note_livre("armures", &a.nom_en),
                description: t.description_equipement("armures", &a.nom_en),
            })
            .collect(),
        armes: eq
            .armes
            .iter()
            .map(|a| EquipementCatalogue {
                nom_en: a.nom_en.clone(),
                nom: a.nom.clone(),
                genre: if a.est_maniable() {
                    "arme"
                } else {
                    "munition ou engin"
                }
                .to_string(),
                poids_lb: a.poids_lb,
                source: t.source_vf("armes", &a.nom_en),
                note_livre: t.note_livre("armes", &a.nom_en),
                description: t.description_equipement("armes", &a.nom_en),
            })
            .collect(),
        materiel: eq
            .materiel
            .iter()
            .map(|a| EquipementCatalogue {
                nom_en: a.nom_en.clone(),
                nom: a.nom.clone(),
                genre: a.categorie_en.clone().unwrap_or_default(),
                poids_lb: a.poids_lb,
                source: t.source_vf("materiel", &a.nom_en),
                note_livre: t.note_livre("materiel", &a.nom_en),
                description: t.description_equipement("materiel", &a.nom_en),
            })
            .collect(),
        etats: r
            .etats
            .iter()
            .map(|e| EtatCatalogue {
                id: e.id.clone(),
                nom: e.nom.clone(),
            })
            .collect(),
        cout_achat: (7..=18)
            .filter_map(|s| fiches::generation::cout_achat(s).map(|c| (s, c)))
            .collect(),
    }
}

/// Aperçu d'une fiche en cours de création : valeurs, problèmes à corriger.
pub fn apercu_fiche(f: &Fiche) -> Result<FicheVue, String> {
    let e = EtatPersonnage {
        fiche: f.clone(),
        xp: 0,
        jalon: 0,
        niveau_atteignable: f.niveau_de_personnage().max(1),
        sorts_depenses: Default::default(),
    };
    fiche_complete(
        &e,
        Vec::new(),
        (
            f.nom.clone(),
            Somme {
                pa: 0,
                texte: String::new(),
            },
            None,
            false,
        ),
        Vec::new(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::evenements::Evenement;
    use campagne::Campagne;
    use domaine::Montant;
    use fiches::journal::EvenementFiche;

    #[test]
    fn la_fiche_complete_et_la_liste_sont_calculees_par_le_coeur() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Essai");
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
        assert!(fiche_du_pj(c.etat(&r), pj).is_none());
        c.executer(
            Evenement::Fiche {
                pj,
                evenement: EvenementFiche::Creation(Box::new(fiches::exemple::elfe_ensorceleuse())),
            },
            &r,
        )
        .unwrap();
        let etat = c.etat(&r).clone();
        let liste = liste_personnages(&etat);
        assert_eq!(liste.len(), 1);
        let resume = liste[0].fiche.as_ref().unwrap();
        assert_eq!(resume.niveau, 1);
        assert_eq!(resume.classes, "Ensorceleur 1");
        assert_eq!(liste[0].prestige, Some(4));
        assert!(liste[0].prestige_calcule);

        let f = fiche_du_pj(&etat, pj).unwrap().unwrap();
        assert_eq!(f.identite.race, "Elfe");
        assert_eq!(f.ca.total, 13);
    }

    #[test]
    fn chaque_classe_ne_propose_que_les_sorts_de_sa_liste() {
        let c = catalogue_creation();
        let r = Referentiel::partage();
        assert!(!c.sorts_par_classe.is_empty());
        for classe in r.classes.iter().filter(|c| c.incantation.is_some()) {
            let cle = classe.nom_en.to_lowercase();
            let liste = &c.sorts_par_classe[&classe.id];
            assert!(!liste.is_empty(), "{}", classe.id);
            for s in liste {
                let def = r.sort(&s.nom_en).unwrap();
                assert_eq!(
                    def.niveau_pour(&cle),
                    Some(s.niveau),
                    "{} / {}",
                    classe.id,
                    s.nom_en
                );
            }
        }
        // un sort de prêtre n'est pas proposé à l'ensorceleur
        let ens = &c.sorts_par_classe["ensorceleur"];
        assert!(ens.iter().any(|s| s.nom_en == "Magic Missile"));
        assert!(!ens.iter().any(|s| s.nom_en == "Cure Light Wounds"));
    }

    #[test]
    fn le_catalogue_de_creation_est_complet() {
        let c = catalogue_creation();
        assert_eq!(c.races.len(), 7);
        assert_eq!(c.competences.len(), 35);
        assert!(c
            .competences
            .iter()
            .any(|x| x.nom == "Connaissances (mystères)"));
        assert_eq!(c.dons.len(), 176);
        assert_eq!(c.sorts_par_classe["ensorceleur"].len(), 390);
        assert_eq!(c.classes[0].lignages.len(), 10);
        assert_eq!(c.cout_achat.len(), 12);
        assert!(c.etats.len() == 34 && c.armures.len() == 21);
        let apercu = apercu_fiche(&fiches::exemple::elfe_ensorceleuse()).unwrap();
        assert!(apercu.problemes.is_empty(), "{:?}", apercu.problemes);
    }
}

// ------------------------------------------------------------------ montée de niveau

vue! {
    pub struct DonDeLignageVue {
        /// Valeur à enregistrer dans la fiche (nom anglais, avec la précision éventuelle).
        pub nom_en: String,
        pub nom: String,
    }
}

vue! {
    pub struct ApercuMonteeVue {
        pub niveau_de_personnage: u32,
        pub classe: String,
        pub niveau_de_classe: u32,
        pub de_vie: u32,
        /// Dé de vie imposé par la méthode de la campagne ; `None` : le joueur lance le dé.
        pub de_vie_propose: Option<u32>,
        pub bba: (i32, i32),
        pub vigueur: (i32, i32),
        pub reflexes: (i32, i32),
        pub volonte: (i32, i32),
        pub dons_a_choisir: u32,
        /// Parmi ces dons, nombre à prendre dans la liste du lignage.
        pub dons_de_lignage: u32,
        /// Dons permis pour le don de lignage (vide si aucun n'est à prendre).
        pub dons_de_lignage_possibles: Vec<DonDeLignageVue>,
        pub augmentation: bool,
        pub predilection_possible: bool,
        pub capacites: Vec<String>,
        pub sorts_connus_gagnes: Vec<(u32, u32)>,
        pub echange_de_sort: bool,
        /// Rangs de compétence à placer, sans le bonus de classe de prédilection (+1 si « rang »).
        pub rangs_a_placer: u32,
        pub niveau_atteint: bool,
        pub niveau_atteignable: u32,
    }
}

vue! {
    pub struct EssaiMontee {
        pub erreurs: Vec<String>,
        pub apres: Option<FicheVue>,
    }
}

/// Dons de la liste du lignage, que la fiche n'a pas déjà pris, avec leur nom français.
fn dons_du_lignage(
    t: &Traductions,
    classe: &fiches::donnees::Classe,
    lignage: Option<&str>,
) -> Vec<DonDeLignageVue> {
    let Some(l) = lignage.and_then(|id| classe.lignages.iter().find(|l| l.id == id)) else {
        return Vec::new();
    };
    l.dons_bonus_en
        .iter()
        .map(|d| {
            let base = fiches::catalogue::nom_de_base(d);
            let nom_base = t.don(base).map(|x| x.nom.clone());
            let nom = match (nom_base, d.find(" (")) {
                (Some(n), Some(i)) => format!("{n}{}", &d[i..]),
                (Some(n), None) => n,
                (None, _) => d.clone(),
            };
            DonDeLignageVue {
                nom_en: d.clone(),
                nom,
            }
        })
        .collect()
}

/// Ce que l'assistant de montée montre pour passer un niveau dans `classe`.
pub fn apercu_montee(etat: &Etat, pj: IdPersonnage, classe: &str) -> Option<ApercuMonteeVue> {
    let r = Referentiel::partage();
    let t = Traductions::partage();
    let e = etat.fiche_de(pj)?;
    let a = fiches::progression::apercu(r, &e.fiche, classe)?;
    let regles = etat.regles_personnages;
    let v = e.fiche.calculer(r).ok()?;
    let c = r.classe(classe)?;
    let race = r
        .race(&e.fiche.race)
        .map(|x| x.rang_supplementaire)
        .unwrap_or(0);
    let mod_int = v.modificateurs[&Carac::Int];
    Some(ApercuMonteeVue {
        niveau_de_personnage: a.niveau_de_personnage,
        classe: a.classe,
        niveau_de_classe: a.niveau_de_classe,
        de_vie: a.de_vie,
        de_vie_propose: fiches::progression::de_de_vie(regles.methode_pv, a.de_vie, None).ok(),
        bba: a.bba,
        vigueur: a.vigueur,
        reflexes: a.reflexes,
        volonte: a.volonte,
        dons_a_choisir: a.dons_a_choisir,
        dons_de_lignage: a.dons_de_lignage,
        dons_de_lignage_possibles: if a.dons_de_lignage > 0 {
            dons_du_lignage(t, c, e.fiche.lignage.as_deref())
        } else {
            Vec::new()
        },
        augmentation: a.augmentation,
        predilection_possible: a.predilection_possible && regles.classe_de_predilection,
        capacites: t.capacites_de_niveau(&a.capacites_en),
        sorts_connus_gagnes: a.sorts_connus_gagnes,
        echange_de_sort: a.echange_de_sort,
        rangs_a_placer: ((c.rangs_par_niveau + mod_int).max(1) as u32) + race,
        niveau_atteint: a.niveau_de_personnage <= e.niveau_atteignable,
        niveau_atteignable: e.niveau_atteignable,
    })
}

/// Essaie une montée de niveau sans l'enregistrer : erreurs, ou fiche obtenue.
pub fn essai_montee(
    etat: &Etat,
    pj: IdPersonnage,
    montee: &fiches::progression::Montee,
) -> Option<EssaiMontee> {
    let r = Referentiel::partage();
    let e = etat.fiche_de(pj)?;
    let regles = etat.regles_personnages.fiche();
    Some(
        match fiches::progression::appliquer(r, &regles, &e.fiche, montee, e.niveau_atteignable) {
            Err(erreurs) => EssaiMontee {
                erreurs: erreurs.iter().map(|x| x.to_string()).collect(),
                apres: None,
            },
            Ok(f) => {
                let apres = apercu_fiche(&f).ok();
                EssaiMontee {
                    erreurs: Vec::new(),
                    apres,
                }
            }
        },
    )
}

#[cfg(all(test, feature = "serde"))]
mod export_apercu {
    use super::*;

    /// Écrit des données d'exemple pour l'aperçu de l'interface (variable GP_EXPORT_DIR).
    #[test]
    fn exporter_les_donnees_d_apercu() {
        let Some(dossier) = std::env::var_os("GP_EXPORT_DIR") else {
            return;
        };
        let d = std::path::Path::new(&dossier);
        let fiche = apercu_fiche(&fiches::exemple::elfe_ensorceleuse()).unwrap();
        let ecrire = |nom: &str, v: &dyn erased::Serialize| {
            std::fs::write(d.join(nom), v.vers_json()).unwrap();
        };
        ecrire("fiche.json", &fiche);
        ecrire("catalogue.json", &catalogue_creation());
        mod erased {
            pub trait Serialize {
                fn vers_json(&self) -> String;
            }
            impl<T: serde::Serialize> Serialize for T {
                fn vers_json(&self) -> String {
                    serde_json::to_string(self).unwrap()
                }
            }
        }
    }
}
