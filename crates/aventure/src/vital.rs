//! État vital d'un personnage : dégâts subis, dégâts non létaux, PV temporaires.
//!
//! On garde les **dégâts subis** (pas les PV restants) : si le maximum change (Constitution,
//! niveau), le personnage garde les mêmes blessures.
//! Règles : Core Rulebook p. 189-191 (lectures à valider, voir `docs/POINTS_A_CONFIRMER_AVENTURE.md`).

#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vital {
    /// Dégâts létaux subis (peut dépasser le maximum : PV négatifs).
    pub degats: i32,
    /// Dégâts non létaux subis (total, dont les parts bloquées ci-dessous).
    pub non_letaux: i32,
    pub temporaires: i32,
    /// Part des dégâts non létaux due à la soif : non récupérable avant d'avoir bu (p. 444).
    pub bloques_soif: i32,
    /// Part due à la faim : non récupérable avant d'avoir mangé (p. 444).
    pub bloques_faim: i32,
    /// Rounds écoulés depuis le dernier point de guérison des dégâts non létaux.
    pub reste_non_letal: i64,
    /// Personnage mourant stabilisé : il ne perd plus de PV (p. 190).
    #[cfg_attr(feature = "serde", serde(default))]
    pub stable: bool,
}

/// État de santé d'un personnage (pour l'affichage ; les états de la fiche sont dans `etats_de_sante`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Sante {
    Conscient,
    /// Dégâts non létaux égaux aux PV : chancelant.
    Chancelant,
    /// Exactement 0 PV : hors de combat.
    HorsDeCombat,
    Inconscient,
    Mourant,
    Stable,
    Mort,
}

impl Sante {
    pub fn nom(self) -> &'static str {
        match self {
            Sante::Conscient => "conscient",
            Sante::Chancelant => "chancelant",
            Sante::HorsDeCombat => "hors de combat",
            Sante::Inconscient => "inconscient",
            Sante::Mourant => "mourant",
            Sante::Stable => "stable",
            Sante::Mort => "mort",
        }
    }
}

/// Ce qu'un soin a réellement fait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoinApplique {
    pub pv_rendus: i32,
    pub non_letaux_retires: i32,
}

impl Vital {
    pub fn pv_actuels(&self, pv_max: i32) -> i32 {
        pv_max - self.degats
    }

    /// Dégâts non létaux qu'on peut guérir (hors parts bloquées par la faim ou la soif).
    pub fn non_letaux_guerissables(&self) -> i32 {
        (self.non_letaux - self.bloques_soif - self.bloques_faim).max(0)
    }

    /// Les PV temporaires absorbent les dégâts avant les PV (p. 191), létaux ou non.
    pub fn prendre_degats(&mut self, nombre: i32, letaux: bool) {
        let nombre = nombre.max(0);
        let absorbe = self.temporaires.min(nombre);
        self.temporaires -= absorbe;
        let reste = nombre - absorbe;
        if letaux {
            self.degats += reste;
            if reste > 0 {
                // lecture à valider : de nouveaux dégâts létaux rompent la stabilité
                self.stable = false;
            }
        } else {
            self.non_letaux += reste;
        }
    }

    /// Soin : rend des PV, jamais plus que les dégâts subis (p. 191), et retire autant de
    /// dégâts non létaux (hors parts bloquées).
    pub fn soigner(&mut self, nombre: i32) -> SoinApplique {
        let nombre = nombre.max(0);
        let pv_rendus = nombre.min(self.degats.max(0));
        self.degats -= pv_rendus;
        let non_letaux_retires = nombre.min(self.non_letaux_guerissables());
        self.non_letaux -= non_letaux_retires;
        SoinApplique {
            pv_rendus,
            non_letaux_retires,
        }
    }

    /// Guérison naturelle d'une nuit : PV (pas de non létal), jamais plus que les dégâts.
    pub fn guerir_naturellement(&mut self, pv: i32) -> i32 {
        let rendus = pv.max(0).min(self.degats.max(0));
        self.degats -= rendus;
        rendus
    }

    /// PV temporaires : les lots se cumulent (décision d'Antoine, 24/10/2026).
    pub fn donner_temporaires(&mut self, nombre: i32) {
        self.temporaires = self.temporaires.saturating_add(nombre.max(0));
    }

    /// Le temps passe : 1 point de dégâts non létaux guéri par heure et par niveau (p. 191).
    /// Renvoie le nombre de points guéris.
    pub fn laisser_passer(&mut self, rounds: i64, niveau: u32) -> i32 {
        self.reste_non_letal += rounds.max(0);
        let heures = self.reste_non_letal / temps::ROUNDS_PAR_HEURE;
        self.reste_non_letal %= temps::ROUNDS_PAR_HEURE;
        let a_guerir = (heures * niveau as i64).min(i32::MAX as i64) as i32;
        let gueris = a_guerir.min(self.non_letaux_guerissables());
        self.non_letaux -= gueris;
        if self.non_letaux_guerissables() == 0 {
            // rien à guérir : on ne garde pas d'heures « en réserve »
            self.reste_non_letal = 0;
        }
        gueris
    }

    /// Mange (`soif` = faux) ou boit (`soif` = vrai) : la part bloquée devient guérissable.
    pub fn debloquer(&mut self, soif: bool) {
        if soif {
            self.bloques_soif = 0;
        } else {
            self.bloques_faim = 0;
        }
    }

    /// État de santé (Core p. 189-191). `con_score` : score de Constitution (mort à −Con).
    pub fn sante(&self, pv_max: i32, con_score: i32) -> Sante {
        let pv = self.pv_actuels(pv_max);
        if pv <= -con_score {
            Sante::Mort
        } else if pv < 0 {
            if self.stable {
                Sante::Stable
            } else {
                Sante::Mourant
            }
        } else if self.non_letaux > pv {
            Sante::Inconscient
        } else if pv == 0 {
            Sante::HorsDeCombat
        } else if self.non_letaux == pv {
            Sante::Chancelant
        } else {
            Sante::Conscient
        }
    }

    /// États du Core (`data/core/etats.json`) qui découlent de la santé.
    pub fn etats_de_sante(&self, pv_max: i32, con_score: i32) -> Vec<&'static str> {
        let pv = self.pv_actuels(pv_max);
        let mut v = Vec::new();
        if pv <= -con_score {
            v.push("dead");
        } else if pv < 0 {
            v.push(if self.stable { "stable" } else { "dying" });
            v.push("unconscious");
        } else {
            if pv == 0 {
                v.push("disabled");
            }
            if pv == 0 || self.non_letaux == pv {
                v.push("staggered");
            }
            if self.non_letaux > pv {
                v.push("unconscious");
            }
        }
        v
    }

    pub fn a_des_degats_de_survie(&self) -> bool {
        self.bloques_soif > 0 || self.bloques_faim > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporaires_absorbent_d_abord() {
        let mut v = Vital::default();
        v.donner_temporaires(4);
        v.prendre_degats(6, true);
        assert_eq!((v.temporaires, v.degats), (0, 2));
        v.donner_temporaires(5);
        v.donner_temporaires(3); // les lots se cumulent
        assert_eq!(v.temporaires, 8);
        v.prendre_degats(3, false);
        assert_eq!((v.temporaires, v.non_letaux), (5, 0));
    }

    #[test]
    fn soin_limite_aux_degats_et_retire_le_non_letal() {
        let mut v = Vital {
            degats: 4,
            non_letaux: 6,
            ..Default::default()
        };
        let s = v.soigner(10);
        assert_eq!(
            s,
            SoinApplique {
                pv_rendus: 4,
                non_letaux_retires: 6
            }
        );
        assert_eq!((v.degats, v.non_letaux), (0, 0));
        assert_eq!(v.pv_actuels(20), 20);
    }

    #[test]
    fn pv_negatifs_et_maximum_qui_change() {
        let v = Vital {
            degats: 25,
            ..Default::default()
        };
        assert_eq!(v.pv_actuels(20), -5);
        assert_eq!(v.pv_actuels(30), 5);
    }

    #[test]
    fn non_letal_gueri_par_heure_et_par_niveau() {
        let mut v = Vital {
            non_letaux: 10,
            ..Default::default()
        };
        assert_eq!(v.laisser_passer(599, 3), 0);
        assert_eq!(v.laisser_passer(1, 3), 3); // 1 heure complète
        assert_eq!(v.non_letaux, 7);
        assert_eq!(v.laisser_passer(2 * 600, 3), 6);
        assert_eq!(v.laisser_passer(600, 3), 1); // il ne reste que 1 point
        assert_eq!(v.non_letaux, 0);
    }

    #[test]
    fn degats_de_survie_bloques_jusqu_a_manger_ou_boire() {
        let mut v = Vital {
            non_letaux: 5,
            bloques_soif: 3,
            ..Default::default()
        };
        assert_eq!(v.non_letaux_guerissables(), 2);
        assert_eq!(v.laisser_passer(10 * 600, 5), 2);
        assert_eq!(v.non_letaux, 3);
        assert_eq!(v.soigner(10).non_letaux_retires, 0);
        v.debloquer(true);
        assert_eq!(v.laisser_passer(600, 5), 3);
        assert!(!v.a_des_degats_de_survie());
    }
}
