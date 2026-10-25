//! Statistiques d'un sort à un niveau de lanceur donné : dégâts ou soins, portée, durée.
//!
//! Tout est lu dans le texte du Core (`data/core/sorts.json`) ; rien n'est inventé. Quand le texte
//! n'est pas une formule reconnue, la valeur est absente (ou marquée « à confirmer ») et le joueur
//! se reporte à la description.

use fiches::catalogue::Sort;

/// Une ligne de dégâts ou de soins du sort, calculée au niveau de lanceur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LigneDegats {
    /// « Dégâts de feu », « Soins », « Dégâts » (type non précisé).
    pub libelle: String,
    /// Valeur au niveau de lanceur : « 5d6 », « 1d8+5 ».
    pub formule: String,
    /// Règle du livre : « 1d6 par niveau de lanceur (maximum 10d6) ».
    pub regle: String,
    /// Lecture non évidente (ex. progression par paliers de niveaux) : à valider.
    pub a_confirmer: bool,
}

/// Statistiques calculées d'un sort.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StatsSort {
    pub degats: Vec<LigneDegats>,
    /// Nombre de projectiles ou de rayons (Magic Missile, Scorching Ray).
    pub nombre: Option<String>,
    /// Portée au niveau de lanceur : « 180 m ».
    pub portee: Option<String>,
}

fn type_fr(t: &str) -> Option<&'static str> {
    Some(match t.trim() {
        "fire" => "feu",
        "cold" => "froid",
        "acid" => "acide",
        "electricity" => "électricité",
        "sonic" => "son",
        "force" => "force",
        "negative energy" => "énergie négative",
        "positive energy" => "énergie positive",
        "nonlethal" => "non-létaux",
        _ => return None,
    })
}

/// Dés « NdF » (+ bonus fixe « +B ») commençant à `i` ; renvoie (n, faces, bonus, fin).
fn lire_des(t: &str, i: usize) -> Option<(u32, u32, i32, usize)> {
    let b = t.as_bytes();
    let mut j = i;
    while j < b.len() && b[j].is_ascii_digit() {
        j += 1;
    }
    if j == i || j >= b.len() || b[j] != b'd' {
        return None;
    }
    let n: u32 = t[i..j].parse().ok()?;
    let mut k = j + 1;
    while k < b.len() && b[k].is_ascii_digit() {
        k += 1;
    }
    if k == j + 1 {
        return None;
    }
    let faces: u32 = t[j + 1..k].parse().ok()?;
    let mut bonus = 0;
    if k < b.len() && b[k] == b'+' {
        let mut m = k + 1;
        while m < b.len() && b[m].is_ascii_digit() {
            m += 1;
        }
        if m > k + 1 {
            bonus = t[k + 1..m].parse().ok()?;
            k = m;
        }
    }
    Some((n, faces, bonus, k))
}

fn formule(n: u32, faces: u32, bonus: i32) -> String {
    if bonus > 0 {
        format!("{n}d{faces}+{bonus}")
    } else {
        format!("{n}d{faces}")
    }
}

/// « (maximum 10d6) » ou « (maximum +5) » dans les 40 caractères qui suivent.
fn maximum(suite: &str) -> Option<&str> {
    let debut = suite.get(..suite.len().min(60))?;
    let p = debut.find("(maximum ")? + "(maximum ".len();
    let fin = debut[p..].find(')')? + p;
    Some(&debut[p..fin])
}

/// Dégâts et soins du texte du sort au niveau de lanceur `cl`.
pub fn degats_du_sort(description_en: &str, cl: u32) -> Vec<LigneDegats> {
    let t = description_en;
    let mut out: Vec<LigneDegats> = Vec::new();
    let b = t.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let debut_de_mot = i == 0 || !b[i - 1].is_ascii_alphanumeric();
        if !(b[i].is_ascii_digit() && debut_de_mot) {
            i += 1;
            continue;
        }
        let Some((n, faces, bonus, fin)) = lire_des(t, i) else {
            i += 1;
            continue;
        };
        let debut = i;
        i = fin.max(i + 1);
        let suite = &t[fin..];
        let Some(apres) = suite
            .strip_prefix(" points of ")
            .or_else(|| suite.strip_prefix(" point of "))
        else {
            continue;
        };
        // « fire damage », « damage » (soins : « cures 1d8 points of damage »)
        let Some(pos) = apres.find("damage") else {
            continue;
        };
        if pos > 40 {
            continue;
        }
        let type_en = apres[..pos].trim();
        let avant = t[..debut].trim_end();
        let soin = type_en.is_empty() && (avant.ends_with("cures") || avant.ends_with("heals"));
        let libelle = if soin {
            "Soins".to_string()
        } else if type_en.is_empty() {
            "Dégâts".to_string()
        } else if let Some(fr) = type_fr(type_en) {
            format!("Dégâts de {fr}")
        } else {
            // type non reconnu : on ne devine pas
            continue;
        };
        let reste = &apres[pos + "damage".len()..];
        let par_niveau = reste.trim_start().starts_with("per caster level");
        let par_deux = reste.trim_start().starts_with("per two caster levels");
        let (formule_txt, regle, a_confirmer) = if par_niveau {
            let plafond = maximum(reste).and_then(|m| lire_des(m, 0)).map(|d| d.0);
            let total = plafond.map_or(n * cl, |p| (n * cl).min(p));
            let regle = match plafond {
                Some(p) => format!("{n}d{faces} par niveau de lanceur (maximum {p}d{faces})"),
                None => format!("{n}d{faces} par niveau de lanceur"),
            };
            (formule(total.max(1), faces, bonus), regle, false)
        } else if par_deux {
            // lecture non évidente : on montre la base et on demande de valider
            (
                formule(n, faces, bonus),
                format!("{n}d{faces} de base, augmente tous les deux niveaux de lanceur"),
                true,
            )
        } else {
            // soins : « + 1 point per caster level (maximum +5) »
            let suivant = reste.trim_start();
            if soin && suivant.starts_with("+ 1 point per caster level") {
                let plafond: i32 = maximum(suivant)
                    .and_then(|m| m.trim_start_matches('+').parse().ok())
                    .unwrap_or(i32::MAX);
                let bonus_total = (cl as i32).min(plafond);
                (
                    formule(n, faces, bonus_total),
                    format!("{n}d{faces} + 1 par niveau de lanceur (maximum +{plafond})"),
                    false,
                )
            } else {
                (formule(n, faces, bonus), formule(n, faces, bonus), false)
            }
        };
        if out
            .iter()
            .any(|l| l.libelle == libelle && l.formule == formule_txt)
        {
            continue;
        }
        out.push(LigneDegats {
            libelle,
            formule: formule_txt,
            regle,
            a_confirmer,
        });
        if out.len() >= 4 {
            break;
        }
    }
    out
}

/// Portée en mètres d'après le texte du Core (1 pied = 0,3 m, comme le livre en français :
/// « moyenne (30 m + 3 m/niveau) »). `None` si le texte n'est pas une formule connue.
pub fn portee_calculee(portee_en: &str, cl: u32) -> Option<String> {
    let t = portee_en.trim().to_lowercase();
    let pieds: u32 = if t.starts_with("close") {
        25 + 5 * (cl / 2)
    } else if t.starts_with("medium") {
        100 + 10 * cl
    } else if t.starts_with("long") {
        400 + 40 * cl
    } else {
        let n: String = t.chars().take_while(char::is_ascii_digit).collect();
        let suite = t[n.len()..].trim_start();
        if n.is_empty()
            || !suite.starts_with("ft.")
            || suite.contains("see text")
            || suite.contains('/')
        {
            return None;
        }
        n.parse().ok()?
    };
    // dixièmes de mètre : 1 pied = 0,3 m
    let dixiemes = pieds * 3;
    Some(if dixiemes.is_multiple_of(10) {
        format!("{} m", dixiemes / 10)
    } else {
        format!("{},{} m", dixiemes / 10, dixiemes % 10)
    })
}

/// Nombre de projectiles ou de rayons, lu dans le texte du sort (deux sorts du Core).
pub fn nombre_d_elements(nom_en: &str, cl: u32) -> Option<String> {
    match nom_en {
        // « un projectile de plus tous les deux niveaux au-delà du 1er … maximum de cinq au 9e »
        "Magic Missile" => Some(format!(
            "{} projectile(s)",
            (1 + cl.saturating_sub(1) / 2).min(5)
        )),
        // « un rayon, plus un rayon de plus tous les quatre niveaux au-delà du 3e (maximum 3 au 11e) »
        "Scorching Ray" => Some(format!(
            "{} rayon(s)",
            (1 + cl.saturating_sub(3) / 4).min(3)
        )),
        _ => None,
    }
}

/// Statistiques du sort au niveau de lanceur `cl`.
pub fn statistiques(sort: &Sort, cl: u32) -> StatsSort {
    StatsSort {
        degats: degats_du_sort(&sort.description_en, cl),
        nombre: nombre_d_elements(&sort.nom_en, cl),
        portee: sort
            .portee_en
            .as_deref()
            .and_then(|p| portee_calculee(p, cl)),
    }
}

/// Une phrase qui résume ce que fait le sort (pour le journal, après le lancement).
pub fn resume(sort: &Sort, cl: u32, dd: i32) -> String {
    let fr = fiches::traductions::Traductions::partage().sort(&sort.nom_en);
    let st = statistiques(sort, cl);
    let mut morceaux: Vec<String> = Vec::new();
    if let Some(n) = &st.nombre {
        morceaux.push(n.clone());
    }
    for l in &st.degats {
        morceaux.push(format!("{} {}", l.libelle, l.formule));
    }
    let portee = st
        .portee
        .clone()
        .or_else(|| fr.and_then(|s| s.portee.clone()));
    if let Some(p) = portee {
        morceaux.push(format!("portée {p}"));
    }
    if let Some(d) = fr.and_then(|s| s.duree.clone()) {
        morceaux.push(format!("durée {d}"));
    }
    if let Some(j) = fr.and_then(|s| s.jet_de_sauvegarde.clone()) {
        if j.to_lowercase().starts_with("aucun") || j == "non" || j.is_empty() {
            morceaux.push("pas de jet de sauvegarde".to_string());
        } else {
            morceaux.push(format!("jet de sauvegarde {j} (DD {dd})"));
        }
    }
    format!("{}.", morceaux.join(" ; "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fiches::Referentiel;

    fn stats(nom: &str, cl: u32) -> StatsSort {
        let r = Referentiel::core();
        statistiques(r.sort(nom).unwrap(), cl)
    }

    #[test]
    fn boule_de_feu_monte_avec_le_niveau_puis_plafonne() {
        assert_eq!(stats("Fireball", 5).degats[0].formule, "5d6");
        assert_eq!(stats("Fireball", 5).degats[0].libelle, "Dégâts de feu");
        assert_eq!(stats("Fireball", 14).degats[0].formule, "10d6");
        assert_eq!(
            stats("Fireball", 5).degats[0].regle,
            "1d6 par niveau de lanceur (maximum 10d6)"
        );
    }

    #[test]
    fn mains_brulantes_plafonne_a_5d4() {
        assert_eq!(stats("Burning Hands", 3).degats[0].formule, "3d4");
        assert_eq!(stats("Burning Hands", 9).degats[0].formule, "5d4");
    }

    #[test]
    fn degats_fixes_et_bonus_fixe() {
        assert_eq!(stats("Magic Missile", 1).degats[0].formule, "1d4+1");
        assert_eq!(stats("Scorching Ray", 7).degats[0].formule, "4d6");
    }

    #[test]
    fn soins_avec_bonus_par_niveau() {
        let s = stats("Cure Light Wounds", 3);
        assert_eq!(s.degats[0].libelle, "Soins");
        assert_eq!(s.degats[0].formule, "1d8+3");
        assert_eq!(stats("Cure Light Wounds", 9).degats[0].formule, "1d8+5");
    }

    #[test]
    fn projectiles_et_rayons() {
        assert_eq!(
            stats("Magic Missile", 1).nombre.as_deref(),
            Some("1 projectile(s)")
        );
        assert_eq!(
            stats("Magic Missile", 3).nombre.as_deref(),
            Some("2 projectile(s)")
        );
        assert_eq!(
            stats("Magic Missile", 9).nombre.as_deref(),
            Some("5 projectile(s)")
        );
        assert_eq!(
            stats("Magic Missile", 20).nombre.as_deref(),
            Some("5 projectile(s)")
        );
        assert_eq!(
            stats("Scorching Ray", 3).nombre.as_deref(),
            Some("1 rayon(s)")
        );
        assert_eq!(
            stats("Scorching Ray", 7).nombre.as_deref(),
            Some("2 rayon(s)")
        );
        assert_eq!(
            stats("Scorching Ray", 11).nombre.as_deref(),
            Some("3 rayon(s)")
        );
    }

    #[test]
    fn portees_comme_dans_le_livre_en_francais() {
        // « moyenne (30 m + 3 m/niveau) », « longue (120 m + 12 m/niveau) », « courte (7,5 m + 1,5 m/2 niveaux) »
        assert_eq!(stats("Magic Missile", 5).portee.as_deref(), Some("45 m"));
        assert_eq!(stats("Fireball", 5).portee.as_deref(), Some("180 m"));
        assert_eq!(
            portee_calculee("close (25 ft. + 5 ft./2 levels)", 4).as_deref(),
            Some("10,5 m")
        );
        assert_eq!(portee_calculee("120 ft.", 1).as_deref(), Some("36 m"));
        assert_eq!(portee_calculee("touch", 5), None);
        assert_eq!(portee_calculee("see text", 5), None);
    }

    #[test]
    fn aucun_sort_du_catalogue_ne_fait_planter_le_calcul() {
        let r = Referentiel::core();
        let mut avec_degats = 0;
        for s in &r.sorts {
            for cl in [1, 5, 20] {
                let st = statistiques(s, cl);
                for l in &st.degats {
                    assert!(!l.formule.is_empty(), "{}", s.nom_en);
                }
            }
            if !statistiques(s, 10).degats.is_empty() {
                avec_degats += 1;
            }
        }
        assert!(
            avec_degats > 60,
            "trop peu de sorts avec dégâts : {avec_degats}"
        );
    }
}
