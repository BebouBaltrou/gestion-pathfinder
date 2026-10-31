//! Contrôle de cohérence : on reconstruit un personnage dans le moteur de règles à partir de
//! ce que la feuille dit, puis on compare chaque valeur. Aucune règle n'est écrite ici :
//! tout ce qui est « attendu » vient du moteur (`fiches`).

use crate::feuille::{entier, Feuille};
use fiches::{Carac, Effet, Fiche, Referentiel, TypeBonus};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Gravite {
    /// La feuille et les règles sont d'accord.
    Accord,
    /// Les règles ne permettent pas de trancher : à confirmer avec le joueur.
    AValider,
    /// Les règles donnent une autre valeur.
    Ecart,
}

/// Une ligne du rapport : ce que dit la feuille, ce que disent les règles, et pourquoi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ligne {
    pub rubrique: &'static str,
    pub libelle: String,
    /// Nom du champ du PDF (pour le retrouver sur la feuille).
    pub champ: String,
    pub feuille: String,
    pub regles: String,
    pub explication: String,
    pub gravite: Gravite,
    /// Identifiant stable de la ligne (champ du PDF + libellé), pour y attacher un choix.
    pub cle: String,
    /// Si la ligne est numérique et modifiable : ce qu'ajoute « Garder » (feuille − règles).
    pub garde: Option<Garde>,
}

/// Valeur de la feuille que le joueur choisit de garder : elle devient un effet « personnalisé ».
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Garde {
    pub cible: String,
    pub delta: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Rapport {
    pub lignes: Vec<Ligne>,
    /// Ce que le moteur ne peut pas vérifier (classe, livre, texte libre…).
    pub non_verifie: Vec<String>,
}

impl Rapport {
    pub fn nombre(&self, g: Gravite) -> usize {
        self.lignes.iter().filter(|l| l.gravite == g).count()
    }
}

/// Identifiant de classe du moteur d'après le nom écrit sur la feuille (classes prises en charge).
fn classe_id(nom: &str) -> Option<&'static str> {
    match crate::dons::normaliser(nom).as_str() {
        "ensorceleur" => Some("ensorceleur"),
        "pretre" => Some("pretre"),
        "druide" => Some("druide"),
        _ => None,
    }
}

/// Niveau lu dans « Ensorceleur 11 » : nom de la classe et niveau.
fn niveau_lu(texte: &str) -> Option<(String, u32)> {
    let mut mots: Vec<&str> = texte.split_whitespace().collect();
    let n: u32 = mots.pop()?.parse().ok()?;
    Some((mots.join(" ").to_lowercase(), n))
}

fn fmt(v: Option<i32>) -> String {
    v.map(|x| x.to_string()).unwrap_or_else(|| "—".into())
}

pub fn controler(f: &Feuille, r: &Referentiel) -> Rapport {
    let mut rap = Rapport::default();
    let mut ajoute = |rubrique: &'static str,
                      libelle: &str,
                      champ: &str,
                      feuille: String,
                      regles: String,
                      explication: String,
                      gravite: Gravite| {
        rap.lignes.push(Ligne {
            rubrique,
            libelle: libelle.to_string(),
            champ: champ.to_string(),
            feuille,
            regles,
            explication,
            gravite,
            cle: String::new(),
            garde: None,
        });
    };

    // ── Identité : classe et niveau ────────────────────────────────────────────
    let Some((classe, niveau)) = niveau_lu(&f.classes_et_niveaux) else {
        rap.non_verifie.push(format!(
            "Classe et niveau illisibles : « {} ».",
            f.classes_et_niveaux
        ));
        return rap;
    };
    let Some(classe_id) = classe_id(&classe) else {
        rap.non_verifie.push(format!(
            "Classe « {classe} » : pas encore prise en charge par le moteur (ensorceleur, prêtre et \
             druide seulement). Les valeurs de la feuille sont lues mais ne peuvent pas être vérifiées."
        ));
        return rap;
    };
    let dv = r.classe(classe_id).map(|c| c.de_vie as i32).unwrap_or(6);
    if !f.classe_et_niveau_bis.is_empty() && f.classe_et_niveau_bis != f.classes_et_niveaux {
        ajoute(
            "Identité",
            "Classe et niveau (deuxième case)",
            "ChampG.1",
            f.classe_et_niveau_bis.clone(),
            f.classes_et_niveaux.clone(),
            "La feuille donne deux niveaux différents. Le niveau de la case principale est utilisé \
             pour le reste du contrôle."
                .into(),
            Gravite::Ecart,
        );
    }

    // ── Personnage dans le moteur ──────────────────────────────────────────────
    let fiche = match fiche_depuis_feuille(f, niveau, r) {
        Ok(fiche) => fiche,
        Err(m) => {
            rap.non_verifie.push(m);
            return rap;
        }
    };

    // ── Caractéristiques ───────────────────────────────────────────────────────
    let valeurs_a = fiche.calculer(r);
    let Ok(v) = valeurs_a else {
        rap.non_verifie.push(format!(
            "Le moteur n'a pas pu reconstruire le personnage : {:?}",
            valeurs_a.err()
        ));
        return rap;
    };
    for cl in &f.caracs {
        let carac = carac_de(cl.id);
        let moteur = v.scores[&carac].total;
        let calc_feuille = cl
            .base
            .zip(Some(cl.alteration.unwrap_or(0)))
            .map(|(b, a)| b + a);
        if let (Some(val), Some(calc)) = (cl.val, calc_feuille) {
            if val != calc {
                ajoute(
                    "Caractéristiques",
                    &format!("{} (total)", cl.nom),
                    &format!("Val{}", index_carac(cl.id)),
                    val.to_string(),
                    calc.to_string(),
                    format!(
                        "Base {} + altération {} = {calc}, mais la feuille écrit {val}.",
                        fmt(cl.base),
                        cl.alteration.unwrap_or(0)
                    ),
                    Gravite::Ecart,
                );
            } else {
                ajoute(
                    "Caractéristiques",
                    &format!("{} (total)", cl.nom),
                    &format!("Val{}", index_carac(cl.id)),
                    val.to_string(),
                    moteur.to_string(),
                    v.scores[&carac].explication(),
                    if moteur == val {
                        Gravite::Accord
                    } else {
                        Gravite::AValider
                    },
                );
            }
        }
        let mod_regles = fiches::modificateur(cl.val.unwrap_or(moteur));
        if let Some(m) = cl.modificateur {
            ajoute(
                "Caractéristiques",
                &format!("Modificateur de {}", cl.nom),
                &format!("Mod{}", index_carac(cl.id)),
                format!("{m:+}"),
                format!("{mod_regles:+}"),
                format!("(score − 10) ÷ 2, arrondi à l'inférieur : {mod_regles:+}."),
                if m == mod_regles {
                    Gravite::Accord
                } else {
                    Gravite::Ecart
                },
            );
        }
    }

    // ── Jets de sauvegarde ─────────────────────────────────────────────────────
    let jets = [
        (
            "Réflexes",
            "JdS1",
            f.reflexes,
            f.base_reflexes,
            v.reflexes.total,
        ),
        (
            "Vigueur",
            "JdS2",
            f.vigueur,
            f.base_vigueur,
            v.vigueur.total,
        ),
        (
            "Volonté",
            "JdS3",
            f.volonte,
            f.base_volonte,
            v.volonte.total,
        ),
    ];
    for (nom, champ, total, base, moteur) in jets {
        let Some(t) = total else { continue };
        ajoute(
            "Jets de sauvegarde",
            nom,
            champ,
            t.to_string(),
            moteur.to_string(),
            match base {
                Some(b) => format!(
                    "La feuille note une base de {b}. Au niveau {niveau}, le moteur donne : {}",
                    match nom {
                        "Réflexes" => v.reflexes.explication(),
                        "Vigueur" => v.vigueur.explication(),
                        _ => v.volonte.explication(),
                    }
                ),
                None => String::new(),
            },
            bonus_total(t, moteur),
        );
    }

    // ── Bonus de base à l'attaque, initiative ──────────────────────────────────
    if !f.bba_texte.is_empty() {
        let attendu = if v.bba_iteratif.len() > 1 {
            v.bba_iteratif
                .iter()
                .map(|b| format!("{b:+}"))
                .collect::<Vec<_>>()
                .join("/")
        } else {
            format!("{:+}", v.bba.total)
        };
        let lu = f.bba_texte.replace('+', "");
        let attendu_sans = attendu.replace('+', "");
        ajoute(
            "Combat",
            "Bonus de base à l'attaque",
            "Combat.0.0",
            f.bba_texte.clone(),
            attendu,
            format!("Ensorceleur niveau {niveau} : {}", v.bba.explication()),
            if lu == attendu_sans {
                Gravite::Accord
            } else {
                Gravite::Ecart
            },
        );
    }
    if let Some(i) = f.initiative {
        ajoute(
            "Combat",
            "Initiative",
            "Init",
            format!("{i:+}"),
            format!("{:+}", v.initiative.total),
            v.initiative.explication(),
            if i == v.initiative.total {
                Gravite::Accord
            } else {
                Gravite::Ecart
            },
        );
    }

    // ── Points de vie ──────────────────────────────────────────────────────────
    if let Some(pv) = f.pv_max {
        let con = v.modificateurs[&Carac::Con];
        let n = niveau as i32;
        // Niveau 1 au maximum, puis 1 au dé de vie par niveau, plus CON à chaque niveau.
        let mini = dv + (n - 1) + n * con;
        let maxi = dv + (n - 1) * dv + n * con;
        // Le moteur ajoute lui-même les bonus de classe de prédilection ou de dons (Robustesse) :
        // on les compte comme marge haute.
        let marge = n + 3;
        let ok = pv >= mini.max(n) && pv <= maxi + marge;
        ajoute(
            "Points de vie",
            "Points de vie maximum",
            "MaxPV",
            pv.to_string(),
            format!("entre {mini} et {}", maxi + marge),
            format!(
                "d{dv} de la classe (niveau 1 au maximum), niveau {niveau}, CON {con:+} par niveau ; \
                 jusqu'à {marge} de plus avec la classe de prédilection et Robustesse. La feuille ne \
                 garde pas les jets de dé : le moteur ne peut donner qu'un intervalle."
            ),
            if ok { Gravite::Accord } else { Gravite::Ecart },
        );
    }

    // ── Classe d'armure (arithmétique de la feuille) ───────────────────────────
    if let (Some(ca), Some(contact), Some(depourvu)) = (f.ca, f.ca_contact, f.ca_depourvu) {
        let dex = v.modificateurs[&Carac::Dex];
        ajoute(
            "Classe d'armure",
            "CA (valeur de la feuille)",
            "CombatCA.0",
            ca.to_string(),
            v.ca.total.to_string(),
            format!(
                "Calcul du moteur avec les objets magiques reconnus de la feuille : {}. Une CA plus \
                 basse que celle des règles est un écart ; plus haute, un objet ou un don à valider.",
                v.ca.explication()
            ),
            bonus_total(ca, v.ca.total),
        );
        let attendu_depourvu = ca - dex.max(0);
        ajoute(
            "Classe d'armure",
            "CA prise au dépourvu",
            "CombatCA.2",
            depourvu.to_string(),
            attendu_depourvu.to_string(),
            "CA moins le bonus de DEX (les bonus d'esquive sont aussi perdus ; non détectables ici)."
                .into(),
            if depourvu == attendu_depourvu {
                Gravite::Accord
            } else {
                Gravite::AValider
            },
        );
        let _ = contact;
    }

    // ── Compétences ────────────────────────────────────────────────────────────
    let rangs_dispo: i32 = v.rangs.iter().map(|b| b.disponibles as i32).sum();
    let rangs_places: i32 = f.competences.iter().filter_map(|c| c.rangs).sum();
    ajoute(
        "Compétences",
        "Rangs placés au total",
        "Comprg.*",
        rangs_places.to_string(),
        format!("au plus {rangs_dispo}"),
        format!(
            "Rangs disponibles aux niveaux 1 à {niveau} selon le moteur (classe, INT, race humaine, \
             classe de prédilection si elle est choisie en rangs) : {rangs_dispo}."
        ),
        if rangs_places <= rangs_dispo {
            Gravite::Accord
        } else {
            Gravite::Ecart
        },
    );
    for c in &f.competences {
        let Some(rangs) = c.rangs else { continue };
        if rangs as u32 > niveau {
            ajoute(
                "Compétences",
                &format!("{} — rangs", c.libelle),
                &c.champ,
                rangs.to_string(),
                format!("au plus {niveau}"),
                "Le rang maximum d'une compétence est égal au niveau du personnage.".into(),
                Gravite::Ecart,
            );
        }
        if let (Some(total), Some(d)) = (c.total, v.competences.get(&c.cle)) {
            let moteur = d.total;
            let classe_feuille = c.classe.unwrap_or(0) > 0;
            let classe_moteur = d.termes.iter().any(|t| t.libelle == "compétence de classe");
            if classe_feuille != classe_moteur {
                ajoute(
                    "Compétences",
                    &format!("{} — compétence de classe", c.libelle),
                    &c.champ,
                    if classe_feuille { "oui (+3)" } else { "non" }.into(),
                    if classe_moteur { "oui (+3)" } else { "non" }.into(),
                    "Liste des compétences de classe de l'ensorceleur d'après le Manuel de base."
                        .into(),
                    Gravite::Ecart,
                );
            }
            ajoute(
                "Compétences",
                &c.libelle,
                &c.champ,
                format!("{total:+}"),
                format!("{moteur:+}"),
                d.explication(),
                bonus_total(total, moteur),
            );
        } else if c.total.is_some() {
            rap.non_verifie.push(format!(
                "{} : compétence non reconnue par le moteur.",
                c.libelle
            ));
        }
    }

    // Les sorts de la feuille sont volontairement ignorés (décision du joueur).
    // ── Dons ───────────────────────────────────────────────────────────────────
    if !f.notes_dons.is_empty() {
        let dons = crate::dons::rapprocher(&f.notes_dons);
        let attendus: u32 = fiche
            .valider(r)
            .iter()
            .filter_map(|p| match p {
                fiches::Probleme::DonsAuNiveau { attendus, .. } => Some(*attendus),
                _ => None,
            })
            .sum();
        let mut reconnus = 0;
        for d in &dons {
            let (regles, explication, gravite) = match (&d.nom_en, d.par_alias) {
                (Some(en), false) => (
                    en.clone(),
                    "Reconnu dans le catalogue.".to_string(),
                    Gravite::Accord,
                ),
                (Some(en), true) => (
                    en.clone(),
                    "Nom français de la feuille rapproché du don du Manuel de base : à confirmer."
                        .to_string(),
                    Gravite::AValider,
                ),
                (None, _) => (
                    "—".to_string(),
                    "Ce don n'est pas dans le catalogue du Manuel de base : il sera gardé comme \
                     don personnalisé si tu le souhaites."
                        .to_string(),
                    Gravite::AValider,
                ),
            };
            if d.nom_en.is_some() {
                reconnus += 1;
            }
            rap.lignes.push(Ligne {
                rubrique: "Dons",
                libelle: d.texte.clone(),
                champ: "Notes.2.3".into(),
                feuille: d.texte.clone(),
                regles,
                explication,
                gravite,
                cle: String::new(),
                garde: None,
            });
        }
        rap.lignes.push(Ligne {
            rubrique: "Dons",
            libelle: "Nombre de dons".into(),
            champ: "Notes.2.3".into(),
            feuille: dons.len().to_string(),
            regles: attendus.to_string(),
            explication: format!(
                "Au niveau {niveau}, le moteur attend {attendus} dons choisis (niveau, humain, \
                 lignage), plus le don gratuit « Écarter les composantes ». {reconnus} reconnus sur la feuille."
            ),
            gravite: if dons.len() as u32 == attendus {
                Gravite::Accord
            } else {
                Gravite::Ecart
            },
            cle: String::new(),
            garde: None,
        });
    }
    // ── Objets magiques ────────────────────────────────────────────────────────
    for (champ, texte) in &f.objets {
        let (regles, explication, gravite, libelle) =
            match r.objets_magiques.objet_par_nom_francais(texte) {
                Some(m) => {
                    let effets: Vec<String> = m
                        .effets
                        .iter()
                        .map(|e| {
                            format!(
                                "{:+} {} sur {}",
                                e.valeur,
                                e.type_bonus.nom(),
                                cible_lisible(&e.cible.0)
                            )
                        })
                        .collect();
                    let mut explication = format!(
                        "{} ({}), {} po.{}",
                        m.nom,
                        m.source,
                        m.prix_po,
                        if effets.is_empty() {
                            " Aucun effet chiffré.".to_string()
                        } else {
                            format!(" Effets : {}.", effets.join(", "))
                        }
                    );
                    let mut gravite = Gravite::Accord;
                    if let Some(p) = prix_ecrit(texte) {
                        if p != m.prix_po {
                            gravite = Gravite::AValider;
                            explication.push_str(&format!(
                                " La feuille note {p} po, le livre donne {} po.",
                                m.prix_po
                            ));
                        }
                    }
                    (m.nom_en.clone(), explication, gravite, m.nom.clone())
                }
                None => (
                    "—".to_string(),
                    "Objet absent du catalogue des objets à bonus chiffré : gardé comme note \
                     personnalisée, ses effets ne sont pas appliqués."
                        .to_string(),
                    Gravite::AValider,
                    texte.chars().take(40).collect(),
                ),
            };
        rap.lignes.push(Ligne {
            rubrique: "Objets magiques",
            libelle,
            champ: champ.clone(),
            feuille: texte.clone(),
            regles,
            explication,
            gravite,
            cle: String::new(),
            garde: None,
        });
    }
    rap.non_verifie.push(
        "Baguettes, parchemins et objets sans bonus chiffré (ou absents du catalogue) restent des \
         notes : leurs effets ne sont pas appliqués aux calculs."
            .into(),
    );

    // Identifiants stables et valeurs « gardables ».
    for l in &mut rap.lignes {
        l.cle = format!("{}|{}", l.champ, l.libelle);
        l.garde = garde_pour(f, l);
    }
    // Tri : écarts d'abord.
    rap.lignes.sort_by_key(|l| std::cmp::Reverse(l.gravite));
    rap
}

/// Une ligne numérique dont le moteur sait recevoir un ajustement : on garde l'écart feuille − règles.
fn garde_pour(f: &Feuille, l: &Ligne) -> Option<Garde> {
    if l.gravite == Gravite::Accord {
        return None;
    }
    let (a, b) = (entier(&l.feuille)?, entier(&l.regles)?);
    if a == b {
        return None;
    }
    let cible = match (l.rubrique, l.champ.as_str()) {
        ("Caractéristiques", c) if c.starts_with("Val") => {
            let n: usize = c[3..].parse().ok()?;
            carac_de(f.caracs.get(n - 1)?.id).cible().0
        }
        ("Jets de sauvegarde", "JdS1") => "jds.ref".into(),
        ("Jets de sauvegarde", "JdS2") => "jds.vig".into(),
        ("Jets de sauvegarde", "JdS3") => "jds.vol".into(),
        ("Combat", "Init") => "init".into(),
        ("Classe d'armure", "CombatCA.0") => "ca".into(),
        ("Compétences", champ) => {
            let c = f.competences.iter().find(|c| c.champ == champ)?;
            if l.libelle != c.libelle {
                return None;
            }
            format!("comp.{}", c.cle)
        }
        _ => return None,
    };
    Some(Garde {
        cible,
        delta: a - b,
    })
}

/// Ce que le contrôle propose : la fiche reconstruite et le rapport d'écarts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposition {
    pub fiche: Fiche,
    pub niveau: u32,
    pub rapport: Rapport,
}

/// Prépare l'import d'une feuille déjà lue (classe prise en charge : ensorceleur).
pub fn proposer(f: &Feuille, r: &Referentiel) -> Result<Proposition, String> {
    let (classe, niveau) = niveau_lu(&f.classes_et_niveaux).ok_or_else(|| {
        format!(
            "Classe et niveau illisibles : « {} ».",
            f.classes_et_niveaux
        )
    })?;
    if classe_id(&classe).is_none() {
        return Err(format!(
            "Classe « {classe} » : pas encore prise en charge (ensorceleur, prêtre et druide seulement)."
        ));
    }
    Ok(Proposition {
        fiche: fiche_depuis_feuille(f, niveau, r)?,
        niveau,
        rapport: controler(f, r),
    })
}

/// Applique les valeurs que le joueur a choisi de garder : chacune devient un effet
/// « personnalisé » (jamais présenté comme officiel).
pub fn appliquer_gardes(mut fiche: Fiche, rapport: &Rapport, gardes: &[String]) -> Fiche {
    for l in &rapport.lignes {
        if let (true, Some(g)) = (gardes.contains(&l.cle), &l.garde) {
            fiche.effets.push(Effet {
                cible: fiches::Cible::new(&g.cible),
                type_bonus: TypeBonus::SansType,
                valeur: g.delta,
                source: format!(
                    "{} (valeur de la feuille PDF gardée : personnalisé)",
                    l.libelle
                ),
            });
        }
    }
    fiche
}

fn carac_de(id: &str) -> Carac {
    match id {
        "for" => Carac::For,
        "dex" => Carac::Dex,
        "con" => Carac::Con,
        "int" => Carac::Int,
        "wis" => Carac::Sag,
        _ => Carac::Cha,
    }
}

fn index_carac(id: &str) -> usize {
    ["for", "dex", "con", "int", "wis", "cha"]
        .iter()
        .position(|x| *x == id)
        .unwrap_or(0)
        + 1
}

/// Reconstruit une fiche du moteur à partir de la feuille. Hypothèses (signalées au joueur) :
/// la colonne « Base » inclut déjà l'ajustement racial ; le +2 libre de l'humain est en CHA ;
/// la colonne « Altération » vient d'objets (bonus d'altération) ; tous les rangs sont posés
/// au niveau 1 (la feuille ne dit pas à quel niveau ils ont été pris).
pub fn fiche_depuis_feuille(f: &Feuille, niveau: u32, r: &Referentiel) -> Result<Fiche, String> {
    let (nom_classe, _) = niveau_lu(&f.classes_et_niveaux).ok_or_else(|| {
        format!(
            "Classe et niveau illisibles : « {} ».",
            f.classes_et_niveaux
        )
    })?;
    let classe_id = classe_id(&nom_classe)
        .ok_or_else(|| format!("Classe « {nom_classe} » non prise en charge."))?;
    let classe = r
        .classe(classe_id)
        .ok_or_else(|| format!("Classe « {classe_id} » absente du catalogue."))?;
    // Race : celle dont le nom commence la case « Race » (« Humain (chélaxien), M »).
    let race_lue = crate::dons::normaliser(&f.race);
    let race = r
        .races
        .iter()
        .find(|x| {
            let n = crate::dons::normaliser(&x.nom);
            race_lue == n || race_lue.starts_with(&format!("{n} "))
        })
        .ok_or_else(|| format!("Race « {} » non reconnue par le catalogue.", f.race))?;
    // Le bonus libre (humain, demi-elfe, demi-orque) va à la caractéristique d'incantation :
    // hypothèse signalée au joueur.
    let carac_libre = classe
        .incantation
        .as_ref()
        .map(|i| carac_incantation(&i.caracteristique))
        .unwrap_or(Carac::Cha);
    let mut scores: BTreeMap<Carac, i32> = BTreeMap::new();
    let mut effets: Vec<Effet> = Vec::new();
    // Objets magiques reconnus : ils portent leurs propres bonus (le catalogue du livre).
    let objets = objets_reconnus(f, r);
    for cl in &f.caracs {
        let c = carac_de(cl.id);
        let mut base = cl.base.or(cl.val).unwrap_or(10);
        // La colonne « Base » inclut déjà la race : on retire les ajustements raciaux que le
        // moteur remet lui-même.
        for (id, v) in &race.ajustements {
            if carac_incantation(id) == c {
                base -= v;
            }
        }
        if c == carac_libre {
            base -= race.choix_libre.unwrap_or(0);
        }
        scores.insert(c, base);
        // La colonne « Altération » de la feuille : on n'ajoute que ce que les objets reconnus
        // n'expliquent pas déjà.
        let par_objets = objets
            .iter()
            .flat_map(|(_, o)| o.effets.iter())
            .filter(|e| e.cible == c.cible() && e.type_bonus == TypeBonus::Alteration)
            .map(|e| e.valeur)
            .max()
            .unwrap_or(0);
        if let Some(a) = cl.alteration.filter(|a| *a > par_objets) {
            effets.push(Effet {
                cible: c.cible(),
                type_bonus: TypeBonus::Alteration,
                valeur: a,
                source: "objet (colonne Altération de la feuille)".into(),
            });
        }
    }
    let mut rangs: BTreeMap<String, u32> = BTreeMap::new();
    for c in &f.competences {
        if let Some(rg) = c.rangs.filter(|r| *r > 0) {
            rangs.insert(c.cle.clone(), rg as u32);
        }
    }
    // Dés de vie : la feuille ne donne que le total de PV, pas les jets. On répartit ce total
    // le plus régulièrement possible (niveau 1 au maximum) ; le résultat est signalé au joueur.
    let con = fiches::modificateur(
        f.caracs
            .iter()
            .find(|c| c.id == "con")
            .and_then(|c| c.val.or(c.base))
            .unwrap_or(10),
    );
    let n = niveau as i32;
    let dv = classe.de_vie as i32;
    let reste = (f.pv_max.unwrap_or(0) - n * con - dv).clamp(n - 1, dv * (n - 1).max(1));
    let (socle, supplement) = if n > 1 {
        (reste / (n - 1), reste % (n - 1))
    } else {
        (0, 0)
    };
    let de_vie = |i: u32| -> u32 {
        if i == 0 {
            dv as u32
        } else if (i as i32) <= supplement {
            (socle + 1) as u32
        } else {
            socle.max(1) as u32
        }
    };
    let mut niveaux: Vec<fiches::fiche::NiveauPris> = (0..niveau)
        .map(|i| fiches::fiche::NiveauPris {
            classe: classe_id.into(),
            de_vie_obtenu: de_vie(i),
            rangs: BTreeMap::new(),
            predilection: Some(fiches::fiche::BonusPredilection::Rang),
            dons: vec![],
        })
        .collect();
    let mut fiche = Fiche {
        nom: f.nom.clone(),
        race: race.id.clone(),
        bonus_racial_libre: race.choix_libre.map(|_| carac_libre),
        scores_base: scores,
        dommages: BTreeMap::new(),
        drain: BTreeMap::new(),
        augmentations: vec![],
        niveaux: std::mem::take(&mut niveaux),
        classes_de_predilection: vec![classe_id.into()],
        lignage: if classe.lignages.is_empty() {
            None
        } else {
            Some(lignage_lu(f, classe, r)?)
        },
        connaissance_de_lignage: None,
        style_de_combat: None,
        effets,
        sorts_connus: vec![],
        equipement: {
            let mut eq = fiches::equipement::Equipement::default();
            for (_, o) in &objets {
                eq.ajouter_objet(&o.nom_en, 1);
            }
            eq
        },
        etats: vec![],
        temporaires: vec![],
        divinite: None,
        alignement: None,
        domaines: vec![],
        energie: None,
        pacte: None,
        compagnon: None,
        graces: vec![],
        talents_roublard: vec![],
        ennemis_jures: vec![],
        augmentations_ennemis: vec![],
        environnements: vec![],
        augmentations_environnements: vec![],
        pacte_du_chasseur: None,
        groupes_d_armes: vec![],
        arme_de_maitrise: None,
        pacte_divin: None,
        formes_de_representation: vec![],
        pouvoirs_rage: vec![],
        en_rage: false,
        rounds_de_rage: 0,
        fatigue_rage: None,
    };
    // Rangs : la feuille ne dit pas à quel niveau ils ont été pris. On les répartit niveau par
    // niveau dans ce que le moteur permet (rangs disponibles, rang maximum = niveau), en
    // alternant entre les compétences ; ce qui ne rentre pas est signalé par le rapport.
    let disponibles: Vec<u32> = fiche
        .calculer(r)
        .map(|v| v.rangs.iter().map(|b| b.disponibles).collect())
        .unwrap_or_default();
    let mut restants: Vec<(String, u32)> = rangs.into_iter().collect();
    restants.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    let mut cumul: BTreeMap<String, u32> = BTreeMap::new();
    for (i, budget) in disponibles.iter().enumerate() {
        let niveau_i = i as u32 + 1;
        let mut budget = *budget;
        while budget > 0 {
            let mut place = false;
            for (cle, reste) in restants.iter_mut() {
                let deja = cumul.entry(cle.clone()).or_insert(0);
                if budget > 0 && *reste > 0 && *deja < niveau_i {
                    *deja += 1;
                    *reste -= 1;
                    budget -= 1;
                    *fiche.niveaux[i].rangs.entry(cle.clone()).or_insert(0) += 1;
                    place = true;
                }
            }
            if !place {
                break;
            }
        }
    }
    // Dons reconnus : posés dans les emplacements de don que le moteur prévoit, dans l'ordre.
    let mut dons: std::collections::VecDeque<String> = crate::dons::rapprocher(&f.notes_dons)
        .into_iter()
        .filter_map(|d| {
            let en = d.nom_en?;
            Some(match d.precision.as_deref().and_then(ecole_en) {
                Some(e) => format!("{en} ({e})"),
                None => en,
            })
        })
        .collect();
    let emplacements: Vec<(u32, u32)> = fiche
        .valider(r)
        .iter()
        .filter_map(|p| match p {
            fiches::Probleme::DonsAuNiveau {
                niveau, attendus, ..
            } => Some((*niveau, *attendus)),
            _ => None,
        })
        .collect();
    for (nv, attendus) in emplacements {
        for _ in 0..attendus {
            if let Some(d) = dons.pop_front() {
                fiche.niveaux[nv as usize - 1].dons.push(d);
            }
        }
    }
    Ok(fiche)
}

/// École de magie en anglais minuscule, comme dans les choix du catalogue (« Spell Focus (evocation) »).
fn ecole_en(fr: &str) -> Option<&'static str> {
    Some(match fr.trim().to_lowercase().replace('é', "e").as_str() {
        "abjuration" => "abjuration",
        "divination" => "divination",
        "enchantement" => "enchantment",
        "evocation" => "evocation",
        "illusion" => "illusion",
        "invocation" => "conjuration",
        "necromancie" => "necromancy",
        "transmutation" => "transmutation",
        _ => return None,
    })
}

/// Une valeur de la feuille plus haute que celle du moteur peut venir d'un objet ou d'un don
/// que le moteur ne connaît pas encore (à valider) ; plus basse, rien ne l'explique (écart).
fn bonus_total(feuille: i32, moteur: i32) -> Gravite {
    match feuille.cmp(&moteur) {
        std::cmp::Ordering::Equal => Gravite::Accord,
        std::cmp::Ordering::Greater => Gravite::AValider,
        std::cmp::Ordering::Less => Gravite::Ecart,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lit_les_nombres_de_la_feuille() {
        assert_eq!(entier("+13"), Some(13));
        assert_eq!(entier("-2"), Some(-2));
        assert_eq!(entier("6/1"), Some(6));
        assert_eq!(entier(""), None);
        assert_eq!(entier("undefined"), None);
    }

    #[test]
    fn lit_classe_et_niveau() {
        assert_eq!(
            niveau_lu("Ensorceleur 11"),
            Some(("ensorceleur".into(), 11))
        );
        assert_eq!(niveau_lu("Rôdeur"), None);
    }

    #[test]
    fn plus_haut_se_valide_plus_bas_est_un_ecart() {
        assert_eq!(bonus_total(8, 8), Gravite::Accord);
        assert_eq!(bonus_total(9, 8), Gravite::AValider);
        assert_eq!(bonus_total(7, 8), Gravite::Ecart);
    }

    #[test]
    fn une_autre_classe_n_est_pas_verifiee() {
        let f = Feuille {
            classes_et_niveaux: "Rôdeur 3".into(),
            ..Default::default()
        };
        let rap = controler(&f, Referentiel::partage());
        assert!(rap.lignes.is_empty());
        assert!(rap.non_verifie[0].contains("pas encore prise en charge"));
    }
}

#[cfg(test)]
mod tests_import {
    use super::*;
    use crate::feuille::lire_feuille;
    use crate::lecteur::Champs;

    fn champs_ensorceleur_3() -> Champs {
        let mut c = Champs::new();
        let mut mets = |k: &str, v: &str| {
            c.insert(k.to_string(), v.to_string());
        };
        mets("Nom", "Test");
        mets("Race", "Humain (test), M");
        mets("Notes.1.1.1.2", "Résistance infernale");
        mets("Classes&Niveaux", "Ensorceleur 3");
        // FOR 10, DEX 10, CON 12, INT 10, SAG 10, CHA 16 (14 + 2 humain)
        for (n, v) in [(1, 10), (2, 10), (3, 12), (4, 10), (5, 10), (6, 16)] {
            mets(&format!("Base{n}"), &v.to_string());
            mets(&format!("Val{n}"), &v.to_string());
            mets(&format!("Mod{n}"), &format!("{:+}", (v - 10) / 2));
        }
        // Vigueur au niveau 3 : 1 (table) + 1 (CON) = 2 ; la feuille écrit 4.
        mets("JdS2", "4");
        mets("MaxPV", "17");
        c
    }

    #[test]
    fn garder_une_valeur_ajoute_un_effet_personnalise() {
        let r = Referentiel::partage();
        let feuille = lire_feuille(&champs_ensorceleur_3());
        let p = proposer(&feuille, r).unwrap();
        assert_eq!(p.niveau, 3);
        let sans = p.fiche.calculer(r).unwrap();
        assert_eq!(sans.vigueur.total, 2);
        let ligne = p
            .rapport
            .lignes
            .iter()
            .find(|l| l.champ == "JdS2")
            .expect("ligne Vigueur");
        assert_eq!(ligne.garde.as_ref().map(|g| g.delta), Some(2));

        // Corriger : rien ne change. Garder : la fiche donne la valeur de la feuille, avec une
        // source marquée « personnalisé ».
        let corrigee = appliquer_gardes(p.fiche.clone(), &p.rapport, &[]);
        assert_eq!(corrigee.calculer(r).unwrap().vigueur.total, 2);
        let gardee = appliquer_gardes(p.fiche, &p.rapport, std::slice::from_ref(&ligne.cle));
        let v = gardee.calculer(r).unwrap();
        assert_eq!(v.vigueur.total, 4);
        assert!(v.vigueur.explication().contains("personnalisé"));
    }

    #[test]
    fn les_points_de_vie_de_la_feuille_sont_retrouves() {
        let r = Referentiel::partage();
        let p = proposer(&lire_feuille(&champs_ensorceleur_3()), r).unwrap();
        assert_eq!(p.fiche.calculer(r).unwrap().pv_max.total, 17);
    }

    /// Feuille minimale d'un lanceur de sorts divins : SAG 16 de base (humain : +2 libre → 18).
    fn champs_divin(classe: &str, niveau: u32, vigueur: i32, pv: i32) -> Champs {
        let mut c = Champs::new();
        let mut mets = |k: &str, v: &str| {
            c.insert(k.to_string(), v.to_string());
        };
        mets("Nom", "Divin");
        mets("Race", "Humain, M");
        mets("Classes&Niveaux", &format!("{classe} {niveau}"));
        // FOR 12, DEX 10, CON 14, INT 10, SAG 18 (16 + 2), CHA 12.
        for (n, v) in [(1, 12), (2, 10), (3, 14), (4, 10), (5, 18), (6, 12)] {
            mets(&format!("Base{n}"), &v.to_string());
            mets(&format!("Val{n}"), &v.to_string());
        }
        mets("JdS2", &vigueur.to_string());
        mets("MaxPV", &pv.to_string());
        c
    }

    #[test]
    fn un_pretre_est_importe_avec_les_regles_de_sa_classe() {
        let r = Referentiel::partage();
        // Prêtre niveau 3 : Vigueur 3 (table) + CON +2 = 5 ; PV entre 8+2+2 et 8+16+... : on donne 30.
        let p = proposer(&lire_feuille(&champs_divin("Prêtre", 3, 5, 30)), r).unwrap();
        assert_eq!(p.niveau, 3);
        assert_eq!(p.fiche.niveaux[0].classe, "pretre");
        let v = p.fiche.calculer(r).unwrap();
        assert_eq!(v.vigueur.total, 5);
        assert_eq!(v.pv_max.total, 30);
        // Le bonus libre de l'humain est allé à la Sagesse (caractéristique d'incantation).
        assert_eq!(v.scores[&Carac::Sag].total, 18);
        assert!(p
            .rapport
            .lignes
            .iter()
            .any(|l| l.champ == "JdS2" && l.gravite == Gravite::Accord));
    }

    #[test]
    fn un_druide_est_importe_et_le_lignage_n_est_demande_qu_a_l_ensorceleur() {
        let r = Referentiel::partage();
        let p = proposer(&lire_feuille(&champs_divin("Druide", 2, 5, 20)), r).unwrap();
        assert_eq!(p.fiche.niveaux[0].classe, "druide");
        assert!(p.fiche.lignage.is_none());
        // Un ensorceleur sans lignage écrit sur la feuille : refus clair, rien n'est deviné.
        let e = proposer(&lire_feuille(&champs_divin("Ensorceleur", 2, 5, 20)), r).unwrap_err();
        assert!(e.contains("lignage"), "{e}");
        // Classe ou race inconnue : refus clair.
        assert!(proposer(&lire_feuille(&champs_divin("Barde", 2, 5, 20)), r).is_err());
    }
}

/// Objets de la feuille reconnus par le catalogue : (champ, objet).
fn objets_reconnus(
    f: &Feuille,
    r: &Referentiel,
) -> Vec<(String, fiches::objets_magiques::ObjetMagiqueInstance)> {
    f.objets
        .iter()
        .filter_map(|(champ, t)| {
            r.objets_magiques
                .objet_par_nom_francais(t)
                .map(|o| (champ.clone(), o))
        })
        .collect()
}

/// « 16k po », « 2,5k po » ou « 8 000 po » dans le texte d'un objet : le prix noté par le joueur.
fn prix_ecrit(texte: &str) -> Option<i64> {
    let t = texte.to_lowercase();
    let i = t.find(" po").or_else(|| t.find("po"))?;
    let avant = t[..i].trim_end();
    let (nombre, mille) = match avant.strip_suffix('k') {
        Some(n) => (n, true),
        None => (avant, false),
    };
    let chiffres: String = nombre
        .chars()
        .rev()
        .take_while(|c| c.is_ascii_digit() || *c == ',' || *c == '.' || *c == ' ')
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let v: f64 = chiffres
        .trim()
        .replace(' ', "")
        .replace(',', ".")
        .parse()
        .ok()?;
    Some((v * if mille { 1000.0 } else { 1.0 }).round() as i64)
}

fn cible_lisible(cible: &str) -> String {
    match cible {
        "ca" => "la CA".into(),
        "jds.vig" => "Vigueur".into(),
        "jds.ref" => "Réflexes".into(),
        "jds.vol" => "Volonté".into(),
        c => match c.split_once('.') {
            Some(("carac", x)) => x.to_uppercase(),
            Some(("comp", x)) => x.replace('_', " "),
            _ => c.to_string(),
        },
    }
}

/// Caractéristique désignée par un identifiant du catalogue (`sag`, `wis`, `cha`…).
fn carac_incantation(id: &str) -> Carac {
    match id.to_lowercase().as_str() {
        "for" | "str" => Carac::For,
        "dex" => Carac::Dex,
        "con" => Carac::Con,
        "int" => Carac::Int,
        "sag" | "wis" => Carac::Sag,
        _ => Carac::Cha,
    }
}

/// Lignage d'ensorceleur reconnu dans le texte libre de la feuille (« Résistance infernale »…).
/// La feuille ne le dit pas dans une case dédiée : on n'en choisit un que s'il est le seul cité.
fn lignage_lu(
    f: &Feuille,
    classe: &fiches::donnees::Classe,
    _r: &Referentiel,
) -> Result<String, String> {
    let texte = crate::dons::normaliser(&f.texte_libre);
    let t = fiches::traductions::Traductions::partage();
    let trouves: Vec<&str> = classe
        .lignages
        .iter()
        .filter(|l| {
            let nom = t.lignage(&l.id).map(|x| x.nom).unwrap_or(&l.nom_en);
            let nom = crate::dons::normaliser(nom);
            // « infernal » se retrouve dans « résistance infernale ».
            nom.len() >= 4 && texte.contains(&nom)
        })
        .map(|l| l.id.as_str())
        .collect();
    match trouves.as_slice() {
        [un] => Ok((*un).to_string()),
        [] => Err("Le lignage de l'ensorceleur n'est pas écrit sur la feuille : à choisir.".into()),
        plusieurs => Err(format!(
            "Plusieurs lignages sont cités sur la feuille ({}) : à choisir.",
            plusieurs.join(", ")
        )),
    }
}
