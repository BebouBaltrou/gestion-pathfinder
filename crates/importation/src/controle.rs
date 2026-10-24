//! Contrôle de cohérence : on reconstruit un personnage dans le moteur de règles à partir de
//! ce que la feuille dit, puis on compare chaque valeur. Aucune règle n'est écrite ici :
//! tout ce qui est « attendu » vient du moteur (`fiches`).

use crate::feuille::Feuille;
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

/// Niveau d'ensorceleur lu dans « Ensorceleur 11 » (seule classe prise en charge pour l'instant).
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
    if classe != "ensorceleur" {
        rap.non_verifie.push(format!(
            "Classe « {classe} » : pas encore prise en charge par le moteur (Ensorceleur seulement). \
             Les valeurs de la feuille sont lues mais ne peuvent pas être vérifiées."
        ));
        return rap;
    }
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
    let fiche = fiche_depuis_feuille(f, niveau);

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
        // Niveau 1 au maximum (6), puis 1 à 6 par niveau, plus CON à chaque niveau.
        let mini = 6 + (n - 1) + n * con;
        let maxi = 6 + (n - 1) * 6 + n * con;
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
                "d6 d'ensorceleur (niveau 1 au maximum), niveau {niveau}, CON {con:+} par niveau ; \
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
                "Le moteur ne connaît pas les objets magiques écrits en texte libre (bracelets, anneau, \
                 amulette…) ; sans eux : {}. À valider avec la liste d'objets.",
                v.ca.explication()
            ),
            Gravite::AValider,
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
    // ── Dons, objets ───────────────────────────────────────────────────────────
        rap.non_verifie.push(
            "Dons écrits en texte libre : rapprochement avec le catalogue à l'étape suivante."
                .into(),
        );
    }
    rap.non_verifie.push(
        "Objets magiques (bandeau, ceinturon, bracelets, anneau, amulette, baguettes, parchemins) \
         écrits en texte libre : conservés comme notes ; leurs effets ne sont pas encore appliqués \
         aux calculs."
            .into(),
    );

    // Tri : écarts d'abord.
    rap.lignes.sort_by_key(|l| std::cmp::Reverse(l.gravite));
    rap
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
pub fn fiche_depuis_feuille(f: &Feuille, niveau: u32) -> Fiche {
    let race = "humain";
    let mut scores: BTreeMap<Carac, i32> = BTreeMap::new();
    let mut effets: Vec<Effet> = Vec::new();
    for cl in &f.caracs {
        let c = carac_de(cl.id);
        let mut base = cl.base.or(cl.val).unwrap_or(10);
        if c == Carac::Cha {
            base -= 2; // l'humain remet +2 racial
        }
        scores.insert(c, base);
        if let Some(a) = cl.alteration.filter(|a| *a != 0) {
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
    let niveaux = (0..niveau)
        .map(|i| fiches::fiche::NiveauPris {
            classe: "ensorceleur".into(),
            de_vie_obtenu: if i == 0 { 6 } else { 4 },
            rangs: if i == 0 {
                rangs.clone()
            } else {
                BTreeMap::new()
            },
            predilection: Some(fiches::fiche::BonusPredilection::Rang),
            dons: vec![],
        })
        .collect();
    Fiche {
        nom: f.nom.clone(),
        race: race.into(),
        bonus_racial_libre: Some(Carac::Cha),
        scores_base: scores,
        dommages: BTreeMap::new(),
        drain: BTreeMap::new(),
        augmentations: vec![],
        niveaux,
        classes_de_predilection: vec!["ensorceleur".into()],
        lignage: Some("infernal".into()),
        connaissance_de_lignage: None,
        effets,
        sorts_connus: vec![],
        equipement: Default::default(),
        etats: vec![],
        temporaires: vec![],
    }
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
    use crate::feuille::entier;

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
