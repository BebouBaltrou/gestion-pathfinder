// Page « Aventure » : horloge de campagne, état des personnages (santé, effets, ressources, objets),
// repos. Quand un combat est en cours, la page montre le combat (composant Combat).

import { useCallback, useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, Retour } from "../api";
import type { AlerteVue, AventureVue, CombatVue, DemandeVue, EffetVue, EvenementAventure, ObjetVue, RessourceVue } from "../aventure-types";
import { Alertes, Dialogue, SaisieDe } from "../composants/jeu";
import { ev, PasHoraire } from "../evenements-jeu";
import { CartePj } from "./aventure/CartePj";
import { Combat } from "./aventure/Combat";
import { DialogueEffet, DialogueHeure, DialogueObjet, DialogueRepos, DialogueSante, ChoixDuree } from "./aventure/dialogues";
import { SaisieNombre } from "../composants/jeu";

type Ouvert =
  | { genre: "sante"; pj: number; mode: "degats" | "soin" | "temp" | "continu" }
  | { genre: "effet"; pj: number }
  | { genre: "objet"; pj: number }
  | { genre: "repos" }
  | { genre: "heure" }
  | { genre: "combat" }
  | { genre: "maintenir"; alerte: AlerteVue }
  | null;

export function Aventure({ outils }: { outils: Outils }) {
  const [vue, setVue] = useState<AventureVue | null>(null);
  const [combat, setCombat] = useState<CombatVue | null>(null);
  const [ouvert, setOuvert] = useState<Ouvert>(null);

  const charger = useCallback(async () => {
    const [v, c] = await Promise.all([api.aventureVue(), api.combatVue()]);
    setVue(v);
    setCombat(c);
  }, []);
  useEffect(() => {
    charger();
  }, [charger, outils.version]);

  /** Envoie un événement d'aventure au journal de la campagne. */
  const envoyer = useCallback(async (e: EvenementAventure): Promise<Retour | null> => outils.agir(api.aventureEvenement(e)), [outils]);

  if (!vue) return <div className="page" />;
  if (combat) return <Combat outils={outils} vue={combat} />;
  if (!vue.actif) return <Demarrage outils={outils} />;

  const h = vue.horloge!;
  const pjs = vue.personnages;
  const pas = (p: PasHoraire, nom: string) => (
    <button key={p} className="bouton" onClick={() => envoyer(ev.pas(p))}>{nom}</button>
  );

  return (
    <div className="page aventure">
      <div className="entete-jeu">
        <div className="titre-page">
          <h1>Aventure</h1>
          <p>Le temps qui passe, la santé, les effets et les ressources de vos personnages.</p>
        </div>
        <button className="bouton principal" disabled={pjs.length === 0} onClick={() => setOuvert({ genre: "combat" })}>
          Commencer un combat
        </button>
        <button
          className="bouton danger"
          onClick={() => {
            if (window.confirm("Terminer l'aventure ? Les personnages et la campagne restent ; seule l'horloge d'aventure s'arrête (annulable).")) outils.agir(api.aventureTerminer());
          }}
        >
          Terminer l'aventure
        </button>
      </div>

      <div className="carte horloge-carte">
        <div className="horloge-grande">
          <span className="heure">{String(h.heure).padStart(2, "0")}:{String(h.minute).padStart(2, "0")}</span>
          <span className="jour">jour {h.jour} de la campagne — {outils.etat.entete?.date}</span>
        </div>
        <div className="horloge-pas" role="group" aria-label="Faire passer le temps">
          {pas("minutes5", "+5 min")}
          {pas("minutes10", "+10 min")}
          {pas("minutes30", "+30 min")}
          {pas("heure1", "+1 h")}
        </div>
        <span className="espace" />
        <button className="bouton" onClick={() => setOuvert({ genre: "heure" })}>Régler l'heure…</button>
        <button className="bouton" disabled={pjs.length === 0} onClick={() => setOuvert({ genre: "repos" })}>Repos…</button>
      </div>

      {vue.alertes.length > 0 && (
        <Alertes
          alertes={vue.alertes}
          onConfirmer={(a) => envoyer(ev.alerteConfirmee(a.id))}
          onMaintenir={(a) => setOuvert({ genre: "maintenir", alerte: a })}
        />
      )}

      {vue.demandes.length > 0 && <Demandes demandes={vue.demandes} envoyer={envoyer} />}

      {vue.disponibles.length > 0 && (
        <div className="carte">
          <div className="section-tete">
            <h2>Faire entrer un personnage dans l'aventure</h2>
            <span className="aide">Seuls les personnages qui ont une fiche peuvent jouer.</span>
          </div>
          <div className="rangee-boutons" style={{ marginTop: 12 }}>
            {vue.disponibles.map(([id, nom]) => (
              <button key={id} className="bouton" onClick={() => envoyer(ev.entree(id))}>+ {nom}</button>
            ))}
          </div>
        </div>
      )}

      {pjs.length === 0 && vue.disponibles.length === 0 && (
        <div className="carte">
          <p>Aucun personnage n'a de fiche. Créez-en une dans la page « Personnages » pour jouer.</p>
          <button className="bouton" onClick={() => outils.aller("personnages")}>Aller aux personnages</button>
        </div>
      )}

      <div className="grille-pj">
        {pjs.map((pj) => (
          <CartePj
            key={pj.id}
            pj={pj}
            onSante={(mode) => setOuvert({ genre: "sante", pj: pj.id, mode })}
            onEffet={() => setOuvert({ genre: "effet", pj: pj.id })}
            onObjet={() => setOuvert({ genre: "objet", pj: pj.id })}
            onRetirerEffet={(e: EffetVue) => envoyer(ev.effetRetire(e.id))}
            onUtiliserObjet={(o: ObjetVue) => envoyer(ev.objetUtilise(pj.id, o.id, 1))}
            onUtiliserRessource={(r: RessourceVue) => envoyer(ev.ressourceUtilisee(pj.id, r.cle, 1))}
            onRendreRessource={(r: RessourceVue) => envoyer(ev.ressourceRendue(pj.id, r.cle, 1))}
            onActualiserRessources={() => outils.agir(api.aventureActualiserRessources(pj.id))}
          />
        ))}
      </div>

      <section className="carte">
        <div className="section-tete">
          <h2>Journal d'aventure</h2>
          <span className="aide">Distinct du journal d'intermède et de celui des combats.</span>
        </div>
        <div className="journal-jeu" style={{ marginTop: 12 }}>
          {vue.journal.length === 0 && <span className="doux">Rien pour l'instant.</span>}
          {vue.journal.map((l, i) => (
            <div className="ligne-j" key={i}>
              <span className="quand">{l.quand}</span>
              <span>{l.texte}</span>
            </div>
          ))}
        </div>
      </section>

      {ouvert?.genre === "sante" && (
        <DialogueSante pj={pjs.find((p) => p.id === ouvert.pj)!} mode={ouvert.mode} envoyer={envoyer} onFermer={() => setOuvert(null)} />
      )}
      {ouvert?.genre === "effet" && <DialogueEffet pjs={pjs} pjId={ouvert.pj} envoyer={envoyer} onFermer={() => setOuvert(null)} />}
      {ouvert?.genre === "objet" && <DialogueObjet pjs={pjs} pjId={ouvert.pj} envoyer={envoyer} onFermer={() => setOuvert(null)} />}
      {ouvert?.genre === "repos" && <DialogueRepos pjs={pjs} envoyer={envoyer} onFermer={() => setOuvert(null)} />}
      {ouvert?.genre === "heure" && <DialogueHeure jour={h.jour} heure={h.heure} minute={h.minute} envoyer={envoyer} onFermer={() => setOuvert(null)} />}
      {ouvert?.genre === "maintenir" && <DialogueMaintenir alerte={ouvert.alerte} envoyer={envoyer} onFermer={() => setOuvert(null)} />}
      {ouvert?.genre === "combat" && <DialogueDebutCombat vue={vue} onFermer={() => setOuvert(null)} outils={outils} />}
    </div>
  );
}

// ------------------------------------------------------------------ démarrage

function Demarrage({ outils }: { outils: Outils }) {
  const [heure, setHeure] = useState<number | null>(8);
  const [minute, setMinute] = useState<number | null>(0);
  const [massifs, setMassifs] = useState(false);
  return (
    <div className="page aventure">
      <div className="titre-page">
        <h1>Aventure</h1>
        <p>
          Suivez vos personnages pendant la partie : le temps qui passe heure par heure, la santé, les effets des sorts et les
          combats, round par round. Les dés restent sur la table : l'application vous demande les résultats.
        </p>
      </div>
      <div className="carte" style={{ maxWidth: 640 }}>
        <div className="bloc">
          <h2>Démarrer l'aventure</h2>
          <p className="doux">
            L'horloge part du jour courant de la campagne ({outils.etat.entete?.date}). Quand elle passe minuit, un jour d'intermède passe
            aussi (chantiers, salaires, recettes).
          </p>
          <div className="champs">
            <SaisieNombre etiquette="Heure de départ" min={0} max={23} valeur={heure} onChange={setHeure} />
            <SaisieNombre etiquette="Minute" min={0} max={59} valeur={minute} onChange={setMinute} />
          </div>
          <label className="interrupteur">
            <input type="checkbox" checked={massifs} onChange={(e) => setMassifs(e.target.checked)} />
            Règle optionnelle des dégâts massifs (jet de Vigueur DD 15 après une attaque d'au moins 50 PV et la moitié des PV)
          </label>
          <div className="rangee-boutons">
            <button
              className="bouton principal"
              disabled={heure === null || minute === null}
              onClick={() => outils.agir(api.aventureDemarrer(heure ?? 0, minute ?? 0, false, massifs))}
            >
              Démarrer l'aventure
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

// ------------------------------------------------------------------ demandes de dés

export function Demandes({
  demandes,
  envoyer,
}: {
  demandes: DemandeVue[];
  envoyer: (e: EvenementAventure) => Promise<unknown>;
}) {
  const [valeurs, setValeurs] = useState<Record<string, number | null>>({});
  return (
    <div className="demandes">
      {demandes.map((d) => {
        const cle = `${d.pj}-${d.genre}`;
        return (
          <div className="demande" key={cle}>
            <strong>{d.nom} : {d.texte}</strong>
            <SaisieDe
              etiquette="Résultat du d20"
              rappel={d.dd ? `DD ${d.dd}` : undefined}
              min={1}
              max={20}
              valeur={valeurs[cle] ?? null}
              onChange={(v) => setValeurs({ ...valeurs, [cle]: v })}
            />
            <div className="rangee-boutons">
              <button
                className="bouton principal"
                disabled={valeurs[cle] == null}
                onClick={() => {
                  const v = valeurs[cle]!;
                  if (d.genre === "stabilisation") envoyer(ev.stabilisation(d.pj, v));
                  else if (d.genre === "massifs") envoyer(ev.degatsMassifs(d.pj, v));
                }}
              >
                Valider le jet
              </button>
            </div>
          </div>
        );
      })}
    </div>
  );
}

// ------------------------------------------------------------------ dialogues de la page

function DialogueMaintenir({ alerte, envoyer, onFermer }: { alerte: AlerteVue; envoyer: (e: EvenementAventure) => Promise<unknown>; onFermer: () => void }) {
  const [genre, setGenre] = useState("minutes");
  const [n, setN] = useState<number | null>(10);
  const duree = ["rounds", "minutes", "heures", "jours"].includes(genre) ? (n ? { [genre]: n } : null) : genre === "permanente" || genre === "concentration" || genre === "jusqua_decharge" ? genre : null;
  return (
    <Dialogue
      titre={`Maintenir « ${alerte.nom} »`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button
            className="bouton principal"
            disabled={!duree}
            onClick={async () => {
              await envoyer(ev.effetRetabli(alerte.id, duree as never));
              onFermer();
            }}
          >
            Remettre l'effet
          </button>
        </>
      }
    >
      <p className="doux">L'effet revient pour la durée saisie, comptée à partir de maintenant ({alerte.personnage}).</p>
      <ChoixDuree genre={genre} nombre={n} onGenre={setGenre} onNombre={setN} />
    </Dialogue>
  );
}

function DialogueDebutCombat({ vue, onFermer, outils }: { vue: AventureVue; onFermer: () => void; outils: Outils }) {
  const [choisis, setChoisis] = useState<number[]>(vue.personnages.map((p) => p.id));
  return (
    <Dialogue
      titre="Commencer un combat"
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button
            className="bouton principal"
            disabled={choisis.length === 0}
            onClick={async () => {
              await outils.agir(api.combatDemarrer(choisis));
              onFermer();
            }}
          >
            Commencer
          </button>
        </>
      }
    >
      <p className="doux">
        Le combat travaille sur une copie de vos personnages : rien n'est changé dans la campagne tant que vous ne validez pas la
        fin du combat. L'horloge avance alors round par round (6 secondes).
      </p>
      <div className="bloc">
        <h3>Personnages engagés</h3>
        <div className="rangee-boutons">
          {vue.personnages.map((p) => (
            <label key={p.id} className="interrupteur">
              <input
                type="checkbox"
                checked={choisis.includes(p.id)}
                onChange={(e) => setChoisis(e.target.checked ? [...choisis, p.id] : choisis.filter((x) => x !== p.id))}
              />
              {p.nom}
            </label>
          ))}
        </div>
      </div>
    </Dialogue>
  );
}
