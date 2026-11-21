//! Page Biens : véhicules (propriétaire : localité, organisation ou PJ ; équipes et gestionnaires
//! affectés, équipage) et montures (détenues par un PJ, un PNJ du don Prestige ou une équipe).
//! Les caractéristiques viennent du Manuel des joueurs ; les attributions sont personnalisées.

use crate::localites::Choix;
use crate::Somme;
use campagne::biens::{
    ameliorations_navires, capacites_vehicule, cout_amelioration, modeles_barde, modeles_vehicules,
    prix_monture, AmeliorationNavire, CoutAmelioration, DetenteurMonture, EffetsAmelioration,
    ModeleBarde, ModeleVehicule, ProprietaireBien, PROPRIETAIRE_MONTURE, PROPRIETAIRE_VEHICULE,
};
use campagne::etat::Etat;
use campagne::evenements::Evenement;
use campagne::modele::*;
use domaine::Montant;
use inventaire::{Commande, Conteneur, Proprietaire};
use regles::montures::{
    categorie_charge_monture, seuils_charge_monture, vitesse_avec_barde, CategorieCharge, Monture,
    MONTURES,
};
use regles::Referentiel;

/// Ce que désigne un choix de l'interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum GenreCible {
    Localite,
    Organisation,
    Personnage,
    Equipe,
    /// Partisan du don Prestige : `id` = numéro du maître × 1000 + rang du partisan.
    Partisan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cible {
    pub genre: GenreCible,
    pub id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ChoixCible {
    pub cible: Cible,
    pub libelle: String,
}

impl From<ProprietaireBien> for Cible {
    fn from(p: ProprietaireBien) -> Self {
        match p {
            ProprietaireBien::Localite(l) => Cible {
                genre: GenreCible::Localite,
                id: l.0,
            },
            ProprietaireBien::Organisation(o) => Cible {
                genre: GenreCible::Organisation,
                id: o.0,
            },
            ProprietaireBien::Personnage(p) => Cible {
                genre: GenreCible::Personnage,
                id: p.0,
            },
        }
    }
}

impl From<DetenteurMonture> for Cible {
    fn from(d: DetenteurMonture) -> Self {
        match d {
            DetenteurMonture::Personnage(p) => Cible {
                genre: GenreCible::Personnage,
                id: p.0,
            },
            DetenteurMonture::Equipe(e) => Cible {
                genre: GenreCible::Equipe,
                id: e.0,
            },
            DetenteurMonture::Partisan { maitre, index } => Cible {
                genre: GenreCible::Partisan,
                id: maitre.0 * 1000 + index as u32,
            },
        }
    }
}

fn proprietaire(c: &Cible) -> Result<ProprietaireBien, String> {
    match c.genre {
        GenreCible::Localite => Ok(ProprietaireBien::Localite(IdLocalite(c.id))),
        GenreCible::Organisation => Ok(ProprietaireBien::Organisation(IdOrganisation(c.id))),
        GenreCible::Personnage => Ok(ProprietaireBien::Personnage(IdPersonnage(c.id))),
        GenreCible::Equipe | GenreCible::Partisan => {
            Err("Un bien appartient à une localité, une organisation ou un PJ".into())
        }
    }
}

fn detenteur(c: &Cible) -> Result<DetenteurMonture, String> {
    match c.genre {
        GenreCible::Personnage => Ok(DetenteurMonture::Personnage(IdPersonnage(c.id))),
        GenreCible::Equipe => Ok(DetenteurMonture::Equipe(IdEquipe(c.id))),
        GenreCible::Partisan => Ok(DetenteurMonture::Partisan {
            maitre: IdPersonnage(c.id / 1000),
            index: (c.id % 1000) as usize,
        }),
        _ => Err("Une monture est confiée à un personnage, une équipe ou un partisan".into()),
    }
}

fn payeur(c: &Cible) -> Result<Payeur, String> {
    match c.genre {
        GenreCible::Localite => Ok(Payeur::Localite(IdLocalite(c.id))),
        GenreCible::Personnage => Ok(Payeur::Personnage(IdPersonnage(c.id))),
        _ => Err("Le payeur est une localité ou un PJ".into()),
    }
}

// ------------------------------------------------------------------ page

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PageBiens {
    pub modeles_vehicules: Vec<ModeleVue>,
    pub modeles_montures: Vec<ModeleVue>,
    pub proprietaires: Vec<ChoixCible>,
    pub detenteurs: Vec<ChoixCible>,
    pub payeurs: Vec<ChoixCible>,
    pub vehicules: Vec<FicheVehicule>,
    pub montures: Vec<FicheMonture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ModeleVue {
    pub id: String,
    pub nom: String,
    /// Caractéristiques du Manuel, en une ligne.
    pub resume: String,
    pub prix: Option<Somme>,
}

/// Jauge d'équipage : personnes recrutées (équipes recrutées) sur l'équipage du Manuel.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Jauge {
    pub recrutes: u32,
    /// Personnes des équipes dont le recrutement est en cours.
    pub en_cours: u32,
    pub requis_min: u32,
    pub requis_max: u32,
    pub complete: bool,
}

/// Poids porté (inventaire du bien) face à ce qu'il peut porter. Le dépassement est permis.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ChargeBien {
    pub poids_kg: f64,
    /// Montures : limites légère, intermédiaire et lourde ; véhicules : la cargaison maximale ;
    /// vide si le Manuel ne la donne pas.
    pub limites_kg: Vec<f64>,
    /// « légère », « intermédiaire », « lourde », « dans la limite », « dépassement »…
    pub categorie: String,
    pub depassement: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct AmelioreeVue {
    pub index: usize,
    pub nom: String,
    pub quantite: u32,
    pub prix: Somme,
    pub effets: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct AmeliorationPossibleVue {
    pub id: String,
    pub nom: String,
    pub prerequis: String,
    /// Le coût, en une ligne (« 15 % du prix de base », « 500 po par cube »…).
    pub cout: String,
    /// Le prix pour ce bateau, `None` s'il est à saisir.
    pub prix: Option<Somme>,
    pub cumulable: bool,
    pub a_la_construction: bool,
    pub unite: Option<String>,
    pub effets: String,
    pub texte: String,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BardeVue {
    pub nom: String,
    pub genre: String,
    pub bonus_ca: i32,
    pub dex_max: Option<i32>,
    pub malus_armure: i32,
    pub prix: Somme,
    pub poids_kg: Option<f64>,
    /// Vitesse avec la barde, quand elle ralentit.
    pub vitesse: Option<String>,
    /// Prix de revente : la moitié du prix payé (Manuel, « La revente du butin »).
    pub prix_revente: Somme,
}

/// Une barde rangée dans l'inventaire de la monture, qu'on peut rééquiper sans la repayer.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BardeRangeeVue {
    pub objet: u64,
    pub libelle: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BardePossibleVue {
    pub nom_en: String,
    pub libelle: String,
    pub prix: Option<Somme>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FicheVehicule {
    pub id: u32,
    pub nom: String,
    pub modele: String,
    pub proprietaire: ChoixCible,
    pub prix: Somme,
    pub details: Vec<String>,
    pub jauge: Option<Jauge>,
    pub equipes: Vec<ChoixCible>,
    pub equipes_possibles: Vec<ChoixCible>,
    pub gestionnaires: Vec<Choix>,
    pub gestionnaires_possibles: Vec<Choix>,
    /// Inventaire (cargaison) lié ; `None` : pas encore créé.
    pub inventaire: Option<u64>,
    pub charge: Option<ChargeBien>,
    pub passagers: u32,
    pub passagers_max: Option<u32>,
    pub ameliorations: Vec<AmelioreeVue>,
    pub ameliorations_possibles: Vec<AmeliorationPossibleVue>,
    /// Vitesse modifiée par les améliorations.
    pub vitesse_ajustee: Option<String>,
    pub bonus_navigation: i32,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FicheMonture {
    pub id: u32,
    pub nom: String,
    pub modele: String,
    pub proprietaire: ChoixCible,
    /// À qui elle est confiée ; `None` : à l'écurie du propriétaire.
    pub detenteur: Option<ChoixCible>,
    pub prix: Somme,
    pub details: Vec<String>,
    /// Inventaire (sacoches, charge) lié ; `None` : pas encore créé.
    pub inventaire: Option<u64>,
    pub charge: Option<ChargeBien>,
    pub barde: Option<BardeVue>,
    pub bardes_possibles: Vec<BardePossibleVue>,
    pub bardes_rangees: Vec<BardeRangeeVue>,
    pub notes: Vec<String>,
}

fn choix_personnage(etat: &Etat, id: IdPersonnage) -> ChoixCible {
    let nom = etat
        .personnages
        .get(&id)
        .map_or_else(|| id.to_string(), |p| p.nom.clone());
    let pnj = etat.acolytes.contains_key(&id);
    ChoixCible {
        cible: Cible {
            genre: GenreCible::Personnage,
            id: id.0,
        },
        libelle: if pnj {
            format!("{nom} (acolyte, don Prestige)")
        } else {
            nom
        },
    }
}

fn choix_partisan(etat: &Etat, maitre: IdPersonnage, index: usize) -> ChoixCible {
    let c: Cible = DetenteurMonture::Partisan { maitre, index }.into();
    let nom_maitre = etat
        .personnages
        .get(&maitre)
        .map_or_else(|| maitre.to_string(), |p| p.nom.clone());
    let libelle = etat
        .prestige_don
        .get(&maitre)
        .and_then(|d| d.partisans.get(index))
        .map_or_else(
            || "Partisan inconnu".to_string(),
            |p| {
                format!(
                    "Partisan : {} (niveau {}, de {nom_maitre})",
                    p.nom, p.niveau
                )
            },
        );
    ChoixCible { cible: c, libelle }
}

fn choix_equipe(etat: &Etat, r: &Referentiel, id: IdEquipe) -> ChoixCible {
    let libelle = etat.equipes.get(&id).map_or_else(
        || id.to_string(),
        |e| {
            let lieu = etat
                .localites
                .get(&e.localite)
                .map_or(String::new(), |l| format!(" ({})", l.nom));
            format!("Équipe : {}{lieu}", etat.nom_equipe(e, r))
        },
    );
    ChoixCible {
        cible: Cible {
            genre: GenreCible::Equipe,
            id: id.0,
        },
        libelle,
    }
}

fn choix_proprietaire(etat: &Etat, p: ProprietaireBien) -> ChoixCible {
    let libelle = match p {
        ProprietaireBien::Localite(l) => etat
            .localites
            .get(&l)
            .map_or(l.to_string(), |x| format!("Localité : {}", x.nom)),
        ProprietaireBien::Organisation(o) => etat
            .organisations
            .get(&o)
            .map_or(o.to_string(), |x| format!("Organisation : {}", x.nom)),
        ProprietaireBien::Personnage(pj) => {
            return choix_personnage(etat, pj);
        }
    };
    ChoixCible {
        cible: p.into(),
        libelle,
    }
}

/// Personnes d'une équipe (un membre de la suite compte pour une seule).
fn nombre_de_personnes(etat: &Etat, r: &Referentiel, e: &Equipe) -> u32 {
    etat.effectif_equipe(e, r)
}

fn details_modele(m: &ModeleVehicule) -> Vec<String> {
    let mut d = Vec::new();
    if let Some(x) = &m.equipage {
        d.push(format!("Équipage : {x}"));
    }
    if let Some(x) = &m.passagers {
        d.push(format!("Passagers : {x}"));
    }
    if let Some(x) = &m.cargaison {
        d.push(format!("Cargaison : {x}"));
    }
    if let Some(x) = &m.tracte_par {
        d.push(format!("Tracté par : {x}"));
    }
    if let (Some(h), Some(j)) = (m.vitesse_km_h, m.vitesse_km_jour) {
        d.push(format!("Vitesse : {h} km/h ({j} km par jour)"));
    }
    d
}

fn jauge(
    etat: &Etat,
    r: &Referentiel,
    equipes: &[IdEquipe],
    m: Option<&ModeleVehicule>,
) -> Option<Jauge> {
    let (min, max) = (m?.equipage_min?, m?.equipage_max?);
    let mut recrutes = 0;
    let mut en_cours = 0;
    for e in equipes.iter().filter_map(|id| etat.equipes.get(id)) {
        match e.statut {
            StatutEquipe::Recrutee => recrutes += nombre_de_personnes(etat, r, e),
            StatutEquipe::EnRecrutement(_) => en_cours += nombre_de_personnes(etat, r, e),
            _ => {}
        }
    }
    Some(Jauge {
        recrutes,
        en_cours,
        requis_min: min,
        requis_max: max,
        complete: recrutes >= min,
    })
}

fn kg(grammes: i64) -> f64 {
    grammes as f64 / 1000.0
}

fn texte_effets(e: &EffetsAmelioration) -> String {
    let mut t = Vec::new();
    let signe = |n: i32| {
        if n > 0 {
            format!("+{n}")
        } else {
            n.to_string()
        }
    };
    if e.cargaison_pct != 0 {
        t.push(format!("{} % de cargaison", signe(e.cargaison_pct)));
    }
    if e.cargaison_kg != 0 {
        t.push(format!(
            "{} kg de cargaison par unité",
            signe(e.cargaison_kg)
        ));
    }
    if e.passagers_pct != 0 {
        t.push(format!("{} % de passagers", signe(e.passagers_pct)));
    }
    if e.vitesse_pct != 0 {
        t.push(format!("{} % de vitesse", signe(e.vitesse_pct)));
    }
    if e.bonus_navigation != 0 {
        t.push(format!(
            "{} aux jets de navigation",
            signe(e.bonus_navigation)
        ));
    }
    t.join(" · ")
}

fn texte_cout(a: &AmeliorationNavire) -> String {
    match &a.cout {
        CoutAmelioration::Pourcent(p) => format!("{p} % du prix de base"),
        CoutAmelioration::Fixe(m) => m.to_string(),
        CoutAmelioration::Taille(_) => "selon la taille du bateau".into(),
        CoutAmelioration::ParUnite { prix, unite, .. } => format!("{prix} par {unite}"),
        CoutAmelioration::Libre { min, max } => format!("de {min} à {max} (à saisir)"),
    }
}

fn nom_genre(g: regles::montures::GenreBarde) -> &'static str {
    use regles::montures::GenreBarde::*;
    match g {
        Legere => "légère",
        Intermediaire => "intermédiaire",
        Lourde => "lourde",
    }
}

fn nombre(x: f64) -> String {
    format!("{x}").replace('.', ",")
}

fn charge_vehicule(etat: &Etat, v: &campagne::biens::Vehicule) -> Option<ChargeBien> {
    let racine = etat.inventaire_du_vehicule(v.id)?;
    let poids_kg = kg(etat.inventaire.poids_total(racine).0);
    let cap = capacites_vehicule(v).cargaison_kg;
    Some(ChargeBien {
        poids_kg,
        limites_kg: cap.into_iter().collect(),
        categorie: match cap {
            Some(c) if poids_kg > c => "dépassement".into(),
            Some(_) => "dans la limite".into(),
            None => "limite inconnue".into(),
        },
        depassement: cap.is_some_and(|c| poids_kg > c),
    })
}

/// Poids de la barde d'une monture, en kg.
fn poids_barde_kg(m: &campagne::biens::Monture, modele: Option<&Monture>) -> f64 {
    let (Some(b), Some(mo)) = (&m.barde, modele) else {
        return 0.0;
    };
    modeles_barde()
        .iter()
        .find(|x| x.nom_en == b.armure)
        .and_then(|x| x.poids_kg(mo.grande))
        .unwrap_or(0.0)
}

fn charge_monture(
    etat: &Etat,
    m: &campagne::biens::Monture,
    modele: Option<&Monture>,
) -> Option<ChargeBien> {
    let racine = etat.inventaire_de_la_monture(m.id)?;
    let poids_kg = kg(etat.inventaire.poids_total(racine).0) + poids_barde_kg(m, modele);
    let Some(mo) = modele else {
        return Some(ChargeBien {
            poids_kg,
            limites_kg: Vec::new(),
            categorie: "limite inconnue".into(),
            depassement: false,
        });
    };
    let (l, i, h) = seuils_charge_monture(mo);
    let c = categorie_charge_monture(mo, poids_kg);
    Some(ChargeBien {
        poids_kg,
        limites_kg: vec![l, i, h],
        categorie: match c {
            CategorieCharge::Legere => "légère",
            CategorieCharge::Intermediaire => "intermédiaire",
            CategorieCharge::Lourde => "lourde",
            CategorieCharge::Depassement => "dépassement",
        }
        .into(),
        depassement: c == CategorieCharge::Depassement,
    })
}

/// La charge d'un véhicule ou d'une monture pour la page Inventaire (limites en livres, comme
/// celles des personnages : 1 kg = 2 livres).
pub(crate) fn charge_vue_inventaire(
    etat: &Etat,
    type_id: &str,
    ref_id: &str,
) -> Option<crate::inventaire::ChargeVue> {
    let lb = |kg: f64| kg * 2.0;
    if type_id == PROPRIETAIRE_MONTURE {
        let m = etat
            .montures
            .get(&campagne::biens::monture_de_ref(ref_id)?)?;
        let modele = MONTURES.iter().find(|x| x.nom == m.modele);
        let c = charge_monture(etat, m, modele)?;
        let [l, i, h] = <[f64; 3]>::try_from(c.limites_kg.clone()).ok()?;
        return Some(crate::inventaire::ChargeVue {
            nom: if c.depassement {
                "surcharge".into()
            } else {
                c.categorie
            },
            poids_lb: lb(c.poids_kg),
            limites_lb: [lb(l), lb(i), lb(h)],
        });
    }
    if type_id == PROPRIETAIRE_VEHICULE {
        let v = etat
            .vehicules
            .get(&campagne::biens::vehicule_de_ref(ref_id)?)?;
        let c = charge_vehicule(etat, v)?;
        let cap = *c.limites_kg.first()?;
        return Some(crate::inventaire::ChargeVue {
            nom: if c.depassement {
                "surcharge".into()
            } else {
                "légère".into()
            },
            poids_lb: lb(c.poids_kg),
            limites_lb: [lb(cap); 3],
        });
    }
    None
}

fn barde_vue(m: &campagne::biens::Monture, modele: Option<&Monture>) -> Option<BardeVue> {
    let b = m.barde.as_ref()?;
    let mo = modele?;
    let armure = modeles_barde().iter().find(|x| x.nom_en == b.armure)?;
    let vitesse = armure
        .genre
        .ralentissante()
        .and_then(|bd| vitesse_avec_barde(mo.vitesse_base_m, Some(bd)))
        .map(|v| {
            format!(
                "{} m par round au lieu de {} m",
                nombre(v),
                mo.vitesse_base_m
            )
        });
    Some(BardeVue {
        nom: armure.nom.clone(),
        genre: nom_genre(armure.genre).into(),
        bonus_ca: armure.bonus_ca,
        dex_max: armure.dex_max,
        malus_armure: armure.malus_armure,
        prix: b.prix.into(),
        poids_kg: armure.poids_kg(mo.grande),
        vitesse,
        prix_revente: Montant::pc(b.prix.pc / 2).into(),
    })
}

/// Modèle d'inventaire d'une barde rangée : « barde:Chainmail:G » (nom anglais de l'armure, taille).
fn id_modele_barde(nom_en: &str, grande: bool) -> String {
    format!("barde:{nom_en}:{}", if grande { "G" } else { "M" })
}

fn lire_modele_barde(id: &str) -> Option<(String, bool)> {
    let (nom, taille) = id.strip_prefix("barde:")?.rsplit_once(':')?;
    Some((nom.to_string(), taille == "G"))
}

/// Les bardes de la bonne taille rangées dans l'inventaire de la monture.
fn bardes_rangees(
    etat: &Etat,
    m: &campagne::biens::Monture,
    modele: Option<&Monture>,
) -> Vec<BardeRangeeVue> {
    let (Some(racine), Some(mo)) = (etat.inventaire_de_la_monture(m.id), modele) else {
        return Vec::new();
    };
    let registre = &etat.inventaire;
    registre
        .exemplaires()
        .filter(|e| registre.racine_de(e.id) == Ok(racine))
        .filter_map(|e| {
            let id = e.modele.as_ref()?;
            let (_, grande) = lire_modele_barde(id)?;
            (grande == mo.grande).then(|| BardeRangeeVue {
                objet: e.id,
                libelle: registre
                    .modele(id)
                    .map_or_else(|| id.clone(), |x| x.nom.clone()),
            })
        })
        .collect()
}

fn bardes_possibles(modele: Option<&Monture>) -> Vec<BardePossibleVue> {
    let grande = modele.is_some_and(|m| m.grande);
    modeles_barde()
        .iter()
        .map(|b: &ModeleBarde| BardePossibleVue {
            nom_en: b.nom_en.clone(),
            libelle: format!(
                "{} ({}){}{}",
                b.nom,
                nom_genre(b.genre),
                b.prix(grande).map_or(String::new(), |p| format!(" — {p}")),
                b.poids_kg(grande)
                    .map_or(String::new(), |p| format!(" · {} kg", nombre(p)))
            ),
            prix: b.prix(grande).map(Into::into),
        })
        .collect()
}

fn notes_monture(m: &campagne::biens::Monture) -> Vec<String> {
    let mut n = vec![
        "Mettre ou enlever une barde prend cinq fois plus de temps (Manuel p. 162).".to_string(),
        "Les chevaux sont de taille G, le poney et le chien de selle de taille M : le prix de la barde est quatre fois (G) ou deux fois (M) celui de l'armure, son poids le double (G) ou le même (M).".to_string(),
    ];
    if m.barde.is_some() {
        n.push("Un animal en armure ne peut transporter que son cavalier et ses fontes (Manuel p. 162).".into());
    }
    n
}

fn ameliorations_du_vehicule(
    v: &campagne::biens::Vehicule,
    m: Option<&ModeleVehicule>,
) -> (Vec<AmelioreeVue>, Vec<AmeliorationPossibleVue>) {
    let catalogue = ameliorations_navires();
    let installees = v
        .ameliorations
        .iter()
        .enumerate()
        .map(|(index, i)| {
            let a = catalogue.iter().find(|a| a.id == i.id);
            AmelioreeVue {
                index,
                nom: a.map_or(i.id.clone(), |a| a.nom.clone()),
                quantite: i.quantite,
                prix: i.prix.into(),
                effets: a.map(|a| texte_effets(&a.effets)).unwrap_or_default(),
            }
        })
        .collect();
    let Some(m) = m.filter(|m| !m.terrestre) else {
        return (installees, Vec::new());
    };
    let base = m.prix.unwrap_or(v.prix);
    let possibles = catalogue
        .iter()
        .filter(|a| a.cumulable || !v.ameliorations.iter().any(|i| i.id == a.id))
        .map(|a| AmeliorationPossibleVue {
            id: a.id.clone(),
            nom: a.nom.clone(),
            prerequis: a.prerequis.clone(),
            cout: texte_cout(a),
            prix: cout_amelioration(a, m, base, 1).map(Into::into),
            cumulable: a.cumulable,
            a_la_construction: a.a_la_construction,
            unite: match &a.cout {
                CoutAmelioration::ParUnite {
                    unite,
                    compteur: None,
                    ..
                } => Some(unite.clone()),
                _ => None,
            },
            effets: texte_effets(&a.effets),
            texte: a.texte.clone(),
        })
        .collect();
    (installees, possibles)
}

pub fn page_biens(c: &mut campagne::Campagne, r: &Referentiel) -> PageBiens {
    let etat = c.etat(r).clone();
    let proprietaires: Vec<ChoixCible> = etat
        .localites
        .values()
        .map(|l| ProprietaireBien::Localite(l.id))
        .chain(
            etat.organisations
                .values()
                .map(|o| ProprietaireBien::Organisation(o.id)),
        )
        .chain(
            etat.personnages
                .values()
                .map(|p| ProprietaireBien::Personnage(p.id)),
        )
        .map(|p| choix_proprietaire(&etat, p))
        .collect();
    let toutes_equipes: Vec<IdEquipe> = etat.equipes.keys().copied().collect();
    PageBiens {
        modeles_vehicules: modeles_vehicules()
            .iter()
            .map(|m| ModeleVue {
                id: m.id.clone(),
                nom: m.nom.clone(),
                resume: details_modele(m).join(" · "),
                prix: m.prix.map(Into::into),
            })
            .collect(),
        modeles_montures: MONTURES
            .iter()
            .map(|m| ModeleVue {
                id: m.nom.to_string(),
                nom: m.nom.to_string(),
                resume: format!(
                    "{} km/h ({} km par jour)",
                    m.vitesse_km_h, m.vitesse_km_jour
                ),
                prix: prix_monture(m.nom).map(Into::into),
            })
            .collect(),
        proprietaires,
        detenteurs: etat
            .personnages
            .values()
            .map(|p| choix_personnage(&etat, p.id))
            .chain(toutes_equipes.iter().map(|e| choix_equipe(&etat, r, *e)))
            // un partisan qui a une équipe est déjà dans la liste des équipes
            .chain(etat.prestige_don.iter().flat_map(|(maitre, d)| {
                (0..d.partisans.len())
                    .filter(|i| d.partisans[*i].equipe.is_none())
                    .map(|i| choix_partisan(&etat, *maitre, i))
            }))
            .collect(),
        payeurs: etat
            .localites
            .values()
            .map(|l| ChoixCible {
                cible: Cible {
                    genre: GenreCible::Localite,
                    id: l.id.0,
                },
                libelle: format!("Trésorerie de {} ({})", l.nom, l.tresorerie),
            })
            .chain(etat.personnages.values().map(|p| ChoixCible {
                cible: Cible {
                    genre: GenreCible::Personnage,
                    id: p.id.0,
                },
                libelle: format!("Compte de {} ({})", p.nom, p.compte),
            }))
            .collect(),
        vehicules: etat
            .vehicules
            .values()
            .map(|v| {
                let m = modeles_vehicules().iter().find(|m| m.id == v.modele);
                FicheVehicule {
                    id: v.id.0,
                    nom: v.nom.clone(),
                    modele: m.map_or(v.modele.clone(), |m| m.nom.clone()),
                    proprietaire: choix_proprietaire(&etat, v.proprietaire),
                    prix: v.prix.into(),
                    details: m.map(details_modele).unwrap_or_default(),
                    jauge: jauge(&etat, r, &v.equipes, m),
                    equipes: v
                        .equipes
                        .iter()
                        .map(|e| choix_equipe(&etat, r, *e))
                        .collect(),
                    equipes_possibles: toutes_equipes
                        .iter()
                        .filter(|e| !v.equipes.contains(e))
                        .map(|e| choix_equipe(&etat, r, *e))
                        .collect(),
                    gestionnaires: v
                        .gestionnaires
                        .iter()
                        .filter_map(|g| etat.gestionnaires.get(g))
                        .map(|g| Choix {
                            id: g.id.0,
                            libelle: g.nom.clone(),
                        })
                        .collect(),
                    gestionnaires_possibles: etat
                        .gestionnaires
                        .values()
                        .filter(|g| !v.gestionnaires.contains(&g.id))
                        .map(|g| Choix {
                            id: g.id.0,
                            libelle: g.nom.clone(),
                        })
                        .collect(),
                    inventaire: etat.inventaire_du_vehicule(v.id),
                    charge: charge_vehicule(&etat, v),
                    passagers: v.passagers,
                    passagers_max: capacites_vehicule(v).passagers_max,
                    ameliorations: ameliorations_du_vehicule(v, m).0,
                    ameliorations_possibles: ameliorations_du_vehicule(v, m).1,
                    vitesse_ajustee: {
                        let cap = capacites_vehicule(v);
                        match (m.and_then(|m| m.vitesse_km_h), cap.vitesse_pct) {
                            (Some(h), p) if p != 0 => Some(format!(
                                "{} km/h ({:+} %)",
                                nombre(h * f64::from(100 + p) / 100.0),
                                p
                            )),
                            _ => None,
                        }
                    },
                    bonus_navigation: capacites_vehicule(v).bonus_navigation,
                }
            })
            .collect(),
        montures: etat
            .montures
            .values()
            .map(|m| {
                let modele = MONTURES.iter().find(|x| x.nom == m.modele);
                FicheMonture {
                    id: m.id.0,
                    nom: m.nom.clone(),
                    modele: m.modele.clone(),
                    proprietaire: choix_proprietaire(&etat, m.proprietaire),
                    detenteur: m.detenteur.map(|d| match d {
                        DetenteurMonture::Personnage(p) => choix_personnage(&etat, p),
                        DetenteurMonture::Equipe(e) => choix_equipe(&etat, r, e),
                        DetenteurMonture::Partisan { maitre, index } => {
                            choix_partisan(&etat, maitre, index)
                        }
                    }),
                    prix: m.prix.into(),
                    details: modele
                        .map(|x| {
                            vec![
                                format!(
                                    "Vitesse : {} km/h ({} km par jour)",
                                    x.vitesse_km_h, x.vitesse_km_jour
                                ),
                                format!(
                                    "Chargée de {} à {} kg : {} km/h ({} km par jour)",
                                    x.plage_reduite_kg.0,
                                    x.plage_reduite_kg.1,
                                    x.vitesse_reduite_km_h,
                                    x.vitesse_reduite_km_jour
                                ),
                            ]
                        })
                        .unwrap_or_default(),
                    inventaire: etat.inventaire_de_la_monture(m.id),
                    charge: charge_monture(&etat, m, modele),
                    barde: barde_vue(m, modele),
                    bardes_possibles: bardes_possibles(modele),
                    bardes_rangees: bardes_rangees(&etat, m, modele),
                    notes: notes_monture(m),
                }
            })
            .collect(),
    }
}

// ------------------------------------------------------------------ actions

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "camelCase"))]
pub enum ActionBien {
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AcheterVehicule {
        modele: String,
        nom: String,
        proprietaire: Cible,
        payeur: Cible,
        prix: Option<String>,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    ModifierVehicule {
        id: u32,
        nom: String,
        proprietaire: Cible,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AffecterEquipe { id: u32, equipe: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    RetirerEquipe { id: u32, equipe: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AffecterGestionnaire { id: u32, gestionnaire: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    RetirerGestionnaire { id: u32, gestionnaire: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    RetirerVehicule { id: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AcheterMonture {
        modele: String,
        nom: String,
        proprietaire: Cible,
        detenteur: Option<Cible>,
        payeur: Cible,
        prix: Option<String>,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    ModifierMonture {
        id: u32,
        nom: String,
        proprietaire: Cible,
        detenteur: Option<Cible>,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    RetirerMonture { id: u32 },
    /// Crée l'inventaire d'un bien acheté avant que les inventaires soient liés.
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    CreerInventaire { genre: String, id: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    InstallerAmelioration {
        id: u32,
        amelioration: String,
        quantite: Option<u32>,
        payeur: Cible,
        prix: Option<String>,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    RetirerAmelioration { id: u32, index: usize },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    Passagers { id: u32, nombre: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    EquiperBarde {
        id: u32,
        armure: String,
        payeur: Cible,
        prix: Option<String>,
    },
    /// Range la barde dans l'inventaire de la monture.
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    RetirerBarde { id: u32 },
    /// Vend la barde portée ; `prix` vide : la moitié du prix payé.
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    VendreBarde {
        id: u32,
        beneficiaire: Cible,
        prix: Option<String>,
    },
    /// Rééquipe une barde rangée dans l'inventaire, sans la repayer.
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    EquiperBardeRangee { id: u32, objet: u64 },
}

/// L'achat d'un bien, avec la création de son inventaire dans le même geste (annulable d'un coup).
fn avec_inventaire(achat: Evenement, type_id: &str, ref_id: String, libelle: &str) -> Evenement {
    Evenement::Groupe {
        libelle: libelle.to_string(),
        evenements: vec![
            achat,
            Evenement::Inventaire(Commande::CreerInventaire {
                proprietaire: Proprietaire {
                    type_id: type_id.to_string(),
                    ref_id,
                },
                conteneur: Conteneur {
                    capacite: None,
                    types_autorises: None,
                    contenu_compte: true,
                },
            }),
        ],
    }
}

/// Prix saisi, ou à défaut celui du catalogue ; sans l'un ni l'autre, l'utilisateur doit le saisir.
fn prix_choisi(saisi: Option<String>, catalogue: Option<Montant>) -> Result<Montant, String> {
    match saisi
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        Some(t) => Montant::depuis_texte(&t)
            .filter(|m| !m.est_negatif())
            .ok_or_else(|| "Prix illisible : écrivez par exemple 50 ou 2 po 5 pa".to_string()),
        None => catalogue
            .ok_or_else(|| "Ce modèle n'a pas de prix au catalogue : saisissez-le".to_string()),
    }
}

pub fn action(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    a: ActionBien,
) -> Result<(Evenement, String), String> {
    use ActionBien as A;
    let etat = c.etat(r).clone();
    let vehicule = |id: u32| {
        etat.vehicules
            .get(&IdVehicule(id))
            .cloned()
            .ok_or_else(|| "Véhicule introuvable".to_string())
    };
    let modifie = |v: &campagne::biens::Vehicule| Evenement::VehiculeModifie {
        id: v.id,
        nom: v.nom.clone(),
        proprietaire: v.proprietaire,
        equipes: v.equipes.clone(),
        gestionnaires: v.gestionnaires.clone(),
    };
    Ok(match a {
        A::AcheterVehicule {
            modele,
            nom,
            proprietaire: p,
            payeur: pay,
            prix,
        } => {
            let m = modeles_vehicules()
                .iter()
                .find(|m| m.id == modele)
                .ok_or("Modèle de véhicule introuvable")?;
            let nom = if nom.trim().is_empty() {
                m.nom.clone()
            } else {
                nom.trim().to_string()
            };
            let id = c.nouvel_id_vehicule();
            let achat = Evenement::VehiculeAchete {
                id,
                modele,
                nom: nom.clone(),
                proprietaire: proprietaire(&p)?,
                payeur: payeur(&pay)?,
                prix: prix_choisi(prix, m.prix)?,
            };
            (
                avec_inventaire(
                    achat,
                    PROPRIETAIRE_VEHICULE,
                    campagne::biens::ref_vehicule(id),
                    &format!("{nom} acheté"),
                ),
                format!("{nom} acheté"),
            )
        }
        A::ModifierVehicule {
            id,
            nom,
            proprietaire: p,
        } => {
            let mut v = vehicule(id)?;
            if nom.trim().is_empty() {
                return Err("Donnez un nom au véhicule".into());
            }
            v.nom = nom.trim().to_string();
            v.proprietaire = proprietaire(&p)?;
            (modifie(&v), format!("{} modifié", v.nom))
        }
        A::AffecterEquipe { id, equipe } => {
            let mut v = vehicule(id)?;
            v.equipes.push(IdEquipe(equipe));
            (modifie(&v), format!("Équipe affectée à {}", v.nom))
        }
        A::RetirerEquipe { id, equipe } => {
            let mut v = vehicule(id)?;
            v.equipes.retain(|e| e.0 != equipe);
            (modifie(&v), format!("Équipe retirée de {}", v.nom))
        }
        A::AffecterGestionnaire { id, gestionnaire } => {
            let mut v = vehicule(id)?;
            v.gestionnaires.push(IdGestionnaire(gestionnaire));
            (modifie(&v), format!("Gestionnaire affecté à {}", v.nom))
        }
        A::RetirerGestionnaire { id, gestionnaire } => {
            let mut v = vehicule(id)?;
            v.gestionnaires.retain(|g| g.0 != gestionnaire);
            (modifie(&v), format!("Gestionnaire retiré de {}", v.nom))
        }
        A::RetirerVehicule { id } => {
            let v = vehicule(id)?;
            (
                Evenement::VehiculeRetire { id: v.id },
                format!("{} retiré", v.nom),
            )
        }
        A::AcheterMonture {
            modele,
            nom,
            proprietaire: p,
            detenteur: d,
            payeur: pay,
            prix,
        } => {
            let m = MONTURES
                .iter()
                .find(|m| m.nom == modele)
                .ok_or("Modèle de monture introuvable")?;
            let nom = if nom.trim().is_empty() {
                m.nom.to_string()
            } else {
                nom.trim().to_string()
            };
            let id = c.nouvel_id_monture();
            let achat = Evenement::MontureAchetee {
                id,
                modele,
                nom: nom.clone(),
                proprietaire: Some(proprietaire(&p)?),
                detenteur: d.as_ref().map(detenteur).transpose()?,
                payeur: payeur(&pay)?,
                prix: prix_choisi(prix, prix_monture(m.nom))?,
            };
            (
                avec_inventaire(
                    achat,
                    PROPRIETAIRE_MONTURE,
                    campagne::biens::ref_monture(id),
                    &format!("{nom} acheté(e)"),
                ),
                format!("{nom} acheté(e)"),
            )
        }
        A::ModifierMonture {
            id,
            nom,
            proprietaire: p,
            detenteur: d,
        } => {
            let m = etat
                .montures
                .get(&IdMonture(id))
                .ok_or("Monture introuvable")?;
            if nom.trim().is_empty() {
                return Err("Donnez un nom à la monture".into());
            }
            (
                Evenement::MontureModifiee {
                    id: m.id,
                    nom: nom.trim().to_string(),
                    proprietaire: Some(proprietaire(&p)?),
                    detenteur: d.as_ref().map(detenteur).transpose()?,
                },
                format!("{} modifié(e)", nom.trim()),
            )
        }
        A::RetirerMonture { id } => {
            let m = etat
                .montures
                .get(&IdMonture(id))
                .ok_or("Monture introuvable")?;
            (
                Evenement::MontureRetiree { id: m.id },
                format!("{} retiré(e)", m.nom),
            )
        }
        A::CreerInventaire { genre, id } => {
            let (type_id, ref_id) = match genre.as_str() {
                "vehicule" => {
                    vehicule(id)?;
                    (
                        PROPRIETAIRE_VEHICULE,
                        campagne::biens::ref_vehicule(IdVehicule(id)),
                    )
                }
                "monture" => {
                    etat.montures
                        .get(&IdMonture(id))
                        .ok_or("Monture introuvable")?;
                    (
                        PROPRIETAIRE_MONTURE,
                        campagne::biens::ref_monture(IdMonture(id)),
                    )
                }
                _ => return Err("Genre de bien inconnu".into()),
            };
            (
                Evenement::Inventaire(Commande::CreerInventaire {
                    proprietaire: Proprietaire {
                        type_id: type_id.to_string(),
                        ref_id,
                    },
                    conteneur: Conteneur {
                        capacite: None,
                        types_autorises: None,
                        contenu_compte: true,
                    },
                }),
                "Inventaire créé".to_string(),
            )
        }
        A::InstallerAmelioration {
            id,
            amelioration,
            quantite,
            payeur: pay,
            prix,
        } => {
            let v = vehicule(id)?;
            let m = modeles_vehicules()
                .iter()
                .find(|m| m.id == v.modele)
                .ok_or("Modèle de véhicule introuvable")?;
            if m.terrestre {
                return Err("Les améliorations ne concernent que les bateaux".into());
            }
            let a = ameliorations_navires()
                .iter()
                .find(|a| a.id == amelioration)
                .ok_or("Amélioration introuvable")?;
            if !a.cumulable && v.ameliorations.iter().any(|i| i.id == a.id) {
                return Err(format!("« {} » est déjà installée", a.nom));
            }
            let quantite = quantite.unwrap_or(1).max(1);
            let base = m.prix.unwrap_or(v.prix);
            (
                Evenement::VehiculeAmelioree {
                    id: v.id,
                    amelioration,
                    quantite,
                    payeur: payeur(&pay)?,
                    prix: prix_choisi(prix, cout_amelioration(a, m, base, quantite))?,
                },
                format!("{} installée sur {}", a.nom, v.nom),
            )
        }
        A::RetirerAmelioration { id, index } => {
            let v = vehicule(id)?;
            (
                Evenement::VehiculeAmeliorationRetiree { id: v.id, index },
                format!("Amélioration retirée de {}", v.nom),
            )
        }
        A::Passagers { id, nombre } => {
            let v = vehicule(id)?;
            (
                Evenement::VehiculePassagers { id: v.id, nombre },
                format!("{nombre} passager(s) à bord de {}", v.nom),
            )
        }
        A::EquiperBarde {
            id,
            armure,
            payeur: pay,
            prix,
        } => {
            let m = etat
                .montures
                .get(&IdMonture(id))
                .ok_or("Monture introuvable")?;
            let modele = MONTURES
                .iter()
                .find(|x| x.nom == m.modele)
                .ok_or("Modèle de monture introuvable")?;
            let b = modeles_barde()
                .iter()
                .find(|b| b.nom_en == armure)
                .ok_or("Armure introuvable pour la barde")?;
            (
                Evenement::MontureBardee {
                    id: m.id,
                    armure,
                    payeur: payeur(&pay)?,
                    prix: prix_choisi(prix, b.prix(modele.grande))?,
                    gratuite: false,
                },
                format!("Barde équipée sur {}", m.nom),
            )
        }
        A::RetirerBarde { id } => {
            let m = etat
                .montures
                .get(&IdMonture(id))
                .ok_or("Monture introuvable")?;
            let b = m
                .barde
                .as_ref()
                .ok_or("Cette monture ne porte pas de barde")?;
            let racine = etat
                .inventaire_de_la_monture(m.id)
                .ok_or("Créez d'abord l'inventaire de la monture : la barde y sera rangée")?;
            let modele = MONTURES
                .iter()
                .find(|x| x.nom == m.modele)
                .ok_or("Modèle de monture introuvable")?;
            let armure = modeles_barde()
                .iter()
                .find(|x| x.nom_en == b.armure)
                .ok_or("Armure introuvable pour la barde")?;
            let modele_id = id_modele_barde(&b.armure, modele.grande);
            (
                Evenement::Groupe {
                    libelle: format!("Barde rangée dans l'inventaire de {}", m.nom),
                    evenements: vec![
                        Evenement::MontureDebardee { id: m.id },
                        Evenement::Inventaire(Commande::AjouterModele(inventaire::Modele {
                            id: modele_id.clone(),
                            nom: format!(
                                "Barde : {} (taille {})",
                                armure.nom,
                                if modele.grande { "G" } else { "M" }
                            ),
                            type_objet: "barde".to_string(),
                            valeur: armure.prix(modele.grande),
                            poids: armure
                                .poids_kg(modele.grande)
                                .map(|kg| inventaire::Poids((kg * 1000.0).round() as i64)),
                            source: "Manuel des joueurs (VF), p. 162".to_string(),
                            proprietes: std::collections::BTreeMap::new(),
                        })),
                        Evenement::Inventaire(Commande::Ajouter {
                            modele: modele_id,
                            quantite: 1,
                            dans: racine,
                        }),
                    ],
                },
                format!("Barde rangée dans l'inventaire de {}", m.nom),
            )
        }
        A::VendreBarde {
            id,
            beneficiaire,
            prix,
        } => {
            let m = etat
                .montures
                .get(&IdMonture(id))
                .ok_or("Monture introuvable")?;
            let b = m
                .barde
                .as_ref()
                .ok_or("Cette monture ne porte pas de barde")?;
            (
                Evenement::MontureBardeVendue {
                    id: m.id,
                    beneficiaire: payeur(&beneficiaire)?,
                    prix: prix_choisi(prix, Some(Montant::pc(b.prix.pc / 2)))?,
                },
                format!("Barde de {} vendue", m.nom),
            )
        }
        A::EquiperBardeRangee { id, objet } => {
            let m = etat
                .montures
                .get(&IdMonture(id))
                .ok_or("Monture introuvable")?;
            if m.barde.is_some() {
                return Err("Rangez ou vendez d'abord la barde actuelle".into());
            }
            let racine = etat
                .inventaire_de_la_monture(m.id)
                .ok_or("Cette monture n'a pas d'inventaire")?;
            let modele = MONTURES
                .iter()
                .find(|x| x.nom == m.modele)
                .ok_or("Modèle de monture introuvable")?;
            let registre = &etat.inventaire;
            if registre.racine_de(objet) != Ok(racine) {
                return Err("Cette barde n'est pas dans l'inventaire de la monture".into());
            }
            let modele_id = registre
                .exemplaire(objet)
                .ok()
                .and_then(|e| e.modele.clone())
                .ok_or("Objet introuvable")?;
            let (armure, grande) =
                lire_modele_barde(&modele_id).ok_or("Cet objet n'est pas une barde")?;
            if grande != modele.grande {
                return Err("Cette barde n'est pas de la bonne taille pour cette monture".into());
            }
            // son prix (pour une revente) : celui de l'armure pour cette taille
            let valeur = modeles_barde()
                .iter()
                .find(|x| x.nom_en == armure)
                .and_then(|x| x.prix(grande))
                .unwrap_or(Montant::ZERO);
            (
                Evenement::Groupe {
                    libelle: format!("Barde rééquipée sur {}", m.nom),
                    evenements: vec![
                        Evenement::MontureBardee {
                            id: m.id,
                            armure,
                            // rien à payer : le prix est nul, le payeur n'est pas débité
                            payeur: Payeur::Localite(IdLocalite(0)),
                            prix: valeur,
                            gratuite: true,
                        },
                        Evenement::Inventaire(Commande::Retirer { id: objet }),
                    ],
                },
                format!("Barde rééquipée sur {}", m.nom),
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn monde() -> (campagne::Campagne, Referentiel, IdLocalite, IdPersonnage) {
        let r = regles::referentiel_officiel();
        let mut c = campagne::Campagne::nouvelle("Test");
        let l = c.nouvel_id_localite();
        c.executer(
            Evenement::LocaliteCreee {
                id: l,
                nom: "Port".into(),
                genre: "ville".into(),
                population: None,
                categorie: CategorieLocalite::Officielle {
                    id: "petite-bourgade".into(),
                },
                notes: String::new(),
            },
            &r,
        )
        .unwrap();
        let pj = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id: pj,
                nom: "Héros".into(),
                compte_initial: Montant::po(20_000),
                prestige: None,
            },
            &r,
        )
        .unwrap();
        (c, r, l, pj)
    }

    #[test]
    fn achat_d_un_drakkar_au_prix_du_catalogue_avec_sa_jauge_d_equipage() {
        let (mut c, r, l, pj) = monde();
        let (ev, _) = action(
            &mut c,
            &r,
            ActionBien::AcheterVehicule {
                modele: "drakkar".into(),
                nom: String::new(),
                proprietaire: Cible {
                    genre: GenreCible::Localite,
                    id: l.0,
                },
                payeur: Cible {
                    genre: GenreCible::Personnage,
                    id: pj.0,
                },
                prix: None,
            },
        )
        .unwrap();
        c.executer(ev, &r).unwrap();
        let page = page_biens(&mut c, &r);
        let v = &page.vehicules[0];
        assert_eq!(v.nom, "Drakkar");
        assert_eq!(v.prix.pc, Montant::po(10_000).pc);
        let j = v.jauge.as_ref().unwrap();
        assert_eq!((j.recrutes, j.requis_min, j.complete), (0, 50, false));
        assert_eq!(page.payeurs.len(), 2);
    }

    #[test]
    fn monture_sans_prix_au_catalogue_demande_le_prix() {
        let (mut c, r, _, pj) = monde();
        let a = ActionBien::AcheterMonture {
            modele: "Chien de selle".into(),
            nom: "Rex".into(),
            proprietaire: Cible {
                genre: GenreCible::Personnage,
                id: pj.0,
            },
            detenteur: Some(Cible {
                genre: GenreCible::Personnage,
                id: pj.0,
            }),
            payeur: Cible {
                genre: GenreCible::Personnage,
                id: pj.0,
            },
            prix: None,
        };
        assert!(action(&mut c, &r, a).is_err());
    }
}

#[cfg(test)]
mod tests_patrimoine {
    use super::tests::monde;
    use super::*;
    use crate::statistiques::page_statistiques;

    #[test]
    fn vehicules_et_montures_comptent_dans_le_patrimoine_au_prix_d_achat() {
        let (mut c, r, l, pj) = monde();
        let cible_pj = Cible {
            genre: GenreCible::Personnage,
            id: pj.0,
        };
        for a in [
            ActionBien::AcheterVehicule {
                modele: "barque".into(),
                nom: String::new(),
                proprietaire: Cible {
                    genre: GenreCible::Localite,
                    id: l.0,
                },
                payeur: cible_pj.clone(),
                prix: Some("50".into()),
            },
            ActionBien::AcheterMonture {
                modele: "Poney".into(),
                nom: String::new(),
                // propriétaire : la localité ; confié à un PJ, sans changer de propriétaire
                proprietaire: Cible {
                    genre: GenreCible::Localite,
                    id: l.0,
                },
                detenteur: Some(cible_pj.clone()),
                payeur: cible_pj.clone(),
                prix: None,
            },
        ] {
            let (ev, _) = action(&mut c, &r, a).unwrap();
            c.executer(ev, &r).unwrap();
        }
        let etat = c.etat(&r).clone();
        let stats = page_statistiques(&etat, &r);
        let part = |cle: &str| {
            stats
                .repartition
                .iter()
                .find(|p| p.cle == cle)
                .unwrap()
                .valeur
                .pc
        };
        assert_eq!(part("vehicules"), Montant::po(50).pc);
        assert_eq!(part("montures"), Montant::po(30).pc);
        // la barque et le poney appartiennent à la localité : le poney, confié au PJ,
        // reste dans le patrimoine de la ville
        let ville = stats.localites.iter().find(|x| x.id == l.0).unwrap();
        assert!(ville
            .composantes
            .iter()
            .any(|p| p.cle == "vehicules" && p.valeur.pc == Montant::po(50).pc));
        assert!(ville
            .composantes
            .iter()
            .any(|p| p.cle == "montures" && p.valeur.pc == Montant::po(30).pc));
        let perso = stats.personnages.iter().find(|x| x.id == pj.0).unwrap();
        assert!(perso
            .composantes
            .iter()
            .all(|p| p.cle != "montures" || p.valeur.pc == 0));
    }

    #[cfg(test)]
    mod tests_complets {
        use super::tests::monde;
        use super::*;

        fn cible_pj(pj: IdPersonnage) -> Cible {
            Cible {
                genre: GenreCible::Personnage,
                id: pj.0,
            }
        }

        fn jouer(c: &mut campagne::Campagne, r: &Referentiel, a: ActionBien) -> Result<(), String> {
            let (ev, _) = action(c, r, a)?;
            c.executer(ev, r).map(|_| ()).map_err(|e| e.to_string())
        }

        #[test]
        fn un_achat_cree_l_inventaire_et_la_charge_du_bateau() {
            let (mut c, r, l, pj) = monde();
            jouer(
                &mut c,
                &r,
                ActionBien::AcheterVehicule {
                    modele: "drakkar".into(),
                    nom: String::new(),
                    proprietaire: Cible {
                        genre: GenreCible::Localite,
                        id: l.0,
                    },
                    payeur: cible_pj(pj),
                    prix: None,
                },
            )
            .unwrap();
            let page = page_biens(&mut c, &r);
            let v = &page.vehicules[0];
            assert!(v.inventaire.is_some());
            let charge = v.charge.as_ref().unwrap();
            assert_eq!(charge.limites_kg, vec![50_000.0]);
            assert_eq!(charge.categorie, "dans la limite");
            assert!(!charge.depassement);
            assert_eq!(v.passagers_max, Some(120));
            // l'inventaire apparaît dans la page Inventaire, avec le nom du bateau
            let inv = crate::inventaire::page_inventaire(c.etat(&r));
            assert!(inv
                .inventaires
                .iter()
                .any(|i| i.proprietaire == "Cargaison de Drakkar"));
            // annuler l'achat enlève le bateau et son inventaire d'un seul geste
            c.annuler().unwrap();
            assert!(c.etat(&r).vehicules.is_empty());
        }

        #[test]
        fn les_ameliorations_ne_concernent_que_les_bateaux_et_pas_en_double() {
            let (mut c, r, l, pj) = monde();
            for modele in ["chariot", "barque"] {
                jouer(
                    &mut c,
                    &r,
                    ActionBien::AcheterVehicule {
                        modele: modele.into(),
                        nom: String::new(),
                        proprietaire: Cible {
                            genre: GenreCible::Localite,
                            id: l.0,
                        },
                        payeur: cible_pj(pj),
                        prix: Some("1".into()),
                    },
                )
                .unwrap();
            }
            let page = page_biens(&mut c, &r);
            let chariot = page
                .vehicules
                .iter()
                .find(|v| v.modele == "Chariot")
                .unwrap();
            let barque = page
                .vehicules
                .iter()
                .find(|v| v.modele == "Barque")
                .unwrap();
            assert!(chariot.ameliorations_possibles.is_empty());
            assert!(!barque.ameliorations_possibles.is_empty());
            let essai = |id: u32, amelioration: &str| ActionBien::InstallerAmelioration {
                id,
                amelioration: amelioration.into(),
                quantite: None,
                payeur: cible_pj(pj),
                prix: None,
            };
            assert!(jouer(&mut c, &r, essai(chariot.id, "grand_safran")).is_err());
            jouer(&mut c, &r, essai(barque.id, "grand_safran")).unwrap();
            // une amélioration non cumulable ne s'installe qu'une fois
            assert!(jouer(&mut c, &r, essai(barque.id, "grand_safran")).is_err());
            let page = page_biens(&mut c, &r);
            let barque = page
                .vehicules
                .iter()
                .find(|v| v.modele == "Barque")
                .unwrap();
            assert_eq!(barque.bonus_navigation, 1);
            assert_eq!(barque.ameliorations.len(), 1);
            // la figure de proue demande un prix
            assert!(jouer(&mut c, &r, essai(barque.id, "figure_de_proue")).is_err());
        }

        #[test]
        fn la_monture_a_une_charge_une_barde_et_des_sacoches() {
            let (mut c, r, _, pj) = monde();
            jouer(
                &mut c,
                &r,
                ActionBien::AcheterMonture {
                    modele: "Cheval léger".into(),
                    nom: "Éclair".into(),
                    proprietaire: cible_pj(pj),
                    detenteur: None,
                    payeur: cible_pj(pj),
                    prix: None,
                },
            )
            .unwrap();
            let page = page_biens(&mut c, &r);
            let m = &page.montures[0];
            assert!(m.inventaire.is_some());
            let charge = m.charge.as_ref().unwrap();
            assert_eq!(charge.limites_kg, vec![87.5, 175.0, 262.5]);
            assert_eq!(charge.categorie, "légère");
            // une barde intermédiaire (cotte de mailles, 150 po ×4 pour un cheval, 20 kg ×2)
            jouer(
                &mut c,
                &r,
                ActionBien::EquiperBarde {
                    id: m.id,
                    armure: "Chainmail".into(),
                    payeur: cible_pj(pj),
                    prix: None,
                },
            )
            .unwrap();
            let page = page_biens(&mut c, &r);
            let m = &page.montures[0];
            let b = m.barde.as_ref().unwrap();
            assert_eq!(b.genre, "intermédiaire");
            assert_eq!(b.prix.pc, Montant::po(600).pc);
            assert_eq!(b.poids_kg, Some(40.0));
            assert_eq!(b.vitesse.as_deref(), Some("12 m par round au lieu de 18 m"));
            // le poids de la barde compte dans la charge
            assert_eq!(m.charge.as_ref().unwrap().poids_kg, 40.0);
            assert!(m.notes.iter().any(|n| n.contains("cavalier et ses fontes")));
            // retirer la barde
            jouer(&mut c, &r, ActionBien::RetirerBarde { id: m.id }).unwrap();
            assert!(page_biens(&mut c, &r).montures[0].barde.is_none());
        }

        #[test]
        fn suivi_des_passagers() {
            let (mut c, r, l, pj) = monde();
            jouer(
                &mut c,
                &r,
                ActionBien::AcheterVehicule {
                    modele: "carrosse".into(),
                    nom: String::new(),
                    proprietaire: Cible {
                        genre: GenreCible::Localite,
                        id: l.0,
                    },
                    payeur: cible_pj(pj),
                    prix: Some("1".into()),
                },
            )
            .unwrap();
            let id = page_biens(&mut c, &r).vehicules[0].id;
            jouer(&mut c, &r, ActionBien::Passagers { id, nombre: 2 }).unwrap();
            let v = &page_biens(&mut c, &r).vehicules[0];
            assert_eq!((v.passagers, v.passagers_max), (2, Some(4)));
        }
    }
}

#[cfg(test)]
mod tests_barde {
    use super::tests::monde;
    use super::*;

    fn cible_pj(pj: IdPersonnage) -> Cible {
        Cible {
            genre: GenreCible::Personnage,
            id: pj.0,
        }
    }

    fn jouer(c: &mut campagne::Campagne, r: &Referentiel, a: ActionBien) -> Result<(), String> {
        let (ev, _) = action(c, r, a)?;
        c.executer(ev, r).map(|_| ()).map_err(|e| e.to_string())
    }

    #[test]
    fn la_barde_retiree_est_rangee_puis_rééquipée_ou_vendue() {
        let (mut c, r, _, pj) = monde();
        jouer(
            &mut c,
            &r,
            ActionBien::AcheterMonture {
                modele: "Cheval léger".into(),
                nom: "Éclair".into(),
                proprietaire: cible_pj(pj),
                detenteur: None,
                payeur: cible_pj(pj),
                prix: None,
            },
        )
        .unwrap();
        let id = page_biens(&mut c, &r).montures[0].id;
        let compte = |c: &mut campagne::Campagne| c.etat(&r).personnages[&pj].compte;
        jouer(
            &mut c,
            &r,
            ActionBien::EquiperBarde {
                id,
                armure: "Chainmail".into(),
                payeur: cible_pj(pj),
                prix: None,
            },
        )
        .unwrap();
        let apres_achat = compte(&mut c);
        // retirer = ranger dans l'inventaire (40 kg), rien n'est perdu
        jouer(&mut c, &r, ActionBien::RetirerBarde { id }).unwrap();
        let page = page_biens(&mut c, &r);
        let m = &page.montures[0];
        assert!(m.barde.is_none());
        assert_eq!(m.bardes_rangees.len(), 1);
        assert_eq!(m.charge.as_ref().unwrap().poids_kg, 40.0);
        // rééquiper sans payer
        let objet = m.bardes_rangees[0].objet;
        jouer(&mut c, &r, ActionBien::EquiperBardeRangee { id, objet }).unwrap();
        assert_eq!(compte(&mut c), apres_achat);
        let page = page_biens(&mut c, &r);
        assert!(page.montures[0].barde.is_some());
        assert!(page.montures[0].bardes_rangees.is_empty());
        // vendre : la moitié du prix payé (600 po) va au bénéficiaire
        jouer(
            &mut c,
            &r,
            ActionBien::VendreBarde {
                id,
                beneficiaire: cible_pj(pj),
                prix: None,
            },
        )
        .unwrap();
        assert_eq!(compte(&mut c), apres_achat + Montant::po(300));
        assert!(page_biens(&mut c, &r).montures[0].barde.is_none());
        // plus rien à vendre ni à ranger
        assert!(jouer(&mut c, &r, ActionBien::RetirerBarde { id }).is_err());
    }

    #[test]
    fn ranger_la_barde_demande_un_inventaire() {
        let (mut c, r, _, pj) = monde();
        // monture sans inventaire (ancien achat) : événements joués à la main
        let id = c.nouvel_id_monture();
        c.executer(
            Evenement::MontureAchetee {
                id,
                modele: "Poney".into(),
                nom: "Pip".into(),
                proprietaire: Some(ProprietaireBien::Personnage(pj)),
                detenteur: None,
                payeur: Payeur::Personnage(pj),
                prix: Montant::po(30),
            },
            &r,
        )
        .unwrap();
        c.executer(
            Evenement::MontureBardee {
                id,
                armure: "Padded".into(),
                payeur: Payeur::Personnage(pj),
                prix: Montant::po(10),
                gratuite: false,
            },
            &r,
        )
        .unwrap();
        let err = action(&mut c, &r, ActionBien::RetirerBarde { id: id.0 }).unwrap_err();
        assert!(err.contains("Créez d'abord l'inventaire"), "{err}");
    }
}
