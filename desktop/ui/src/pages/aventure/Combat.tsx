// Écran de combat : initiative, tours, actions, sorts, objets, santé, fin de combat.
// Le combat est une copie de travail : rien n'entre dans la campagne avant la validation finale.
// Toutes les règles sont dans le cœur ; cet écran envoie des actions et affiche ce qu'il renvoie.

import { AnimatePresence, motion } from "motion/react";
import { useEffect, useMemo, useState } from "react";
import type { Outils } from "../../App";
import { api, Retour } from "../../api";
import type {
  ActionCombat,
  ActionDef,
  ApercuFin,
  ApercuSort,
  CombatVue,
  Duree,
  EvenementAventure,
  PjCombat,
  SortConnuVue,
  TypeAction,
  Validation,
} from "../../aventure-types";
import { Alertes, Dialogue, Gemmes, SaisieDe, SaisieNombre } from "../../composants/jeu";
import { act, ev } from "../../evenements-jeu";
import { Demandes } from "../Aventure";
import { CartePj } from "./CartePj";
import { ChoixDuree, DialogueEffet, DialogueObjet, DialogueSante, dureeDe, Envoyer } from "./dialogues";

type Ouvert =
  | { g: "sante"; pj: number; mode: "degats" | "soin" | "temp" | "continu" }
  | { g: "effet"; pj: number }
  | { g: "objetAjout"; pj: number }
  | { g: "agir"; pj: number }
  | { g: "sort"; pj: number }
  | { g: "objet"; pj: number }
  | { g: "initiative"; pj: number }
  | { g: "concentration"; pj: number }
  | { g: "preparer"; pj: number }
  | { g: "fin" }
  | { g: "maintenir"; id: number; nom: string; pj: string }
  | null;

const NOM_TYPE: Record<TypeAction, string> = {
  simple: "Action simple",
  mouvement: "Action de mouvement",
  complexe: "Action complexe",
  rapide: "Action rapide",
  immediate: "Action immédiate",
  libre: "Action libre",
  aucune: "Sans action",
};

export function Combat({ outils, vue }: { outils: Outils; vue: CombatVue }) {
  const [ouvert, setOuvert] = useState<Ouvert>(null);

  const agir = (a: ActionCombat): Promise<Retour | null> => outils.agir(api.combatAction(a));
  const envoyer: Envoyer = (e: EvenementAventure) => agir(act.aventure(e));
  const pjs = vue.personnages;
  const actif = pjs.find((p) => p.id === vue.actif) ?? null;
  const nom = (id: number) => pjs.find((p) => p.id === id)?.nom ?? `PJ ${id}`;
  const enCours = vue.phase === "enCours";

  // ordre d'affichage : le personnage qui joue en premier
  const ordre = useMemo(() => [...pjs].sort((a, b) => Number(b.id === vue.actif) - Number(a.id === vue.actif)), [pjs, vue.actif]);

  return (
    <div className="page aventure combat">
      <div className="combat-entete">
        <div className="titre-page" style={{ marginRight: "auto" }}>
          <h1>Combat</h1>
          <p>
            {enCours
              ? "Copie de travail : la campagne ne change qu'à la validation de la fin du combat."
              : "Saisissez le résultat de votre d20 d'initiative pour chaque personnage."}
          </p>
        </div>
        {enCours && (
          <div className="combat-round" aria-live="polite">
            <strong>{vue.surprise ? "Surprise" : `Round ${vue.round}`}</strong>
            <span>{vue.horloge.texte} · {vue.roundsJoues} round(s) achevé(s)</span>
          </div>
        )}
        <div className="groupe-boutons">
          <button className="bouton" disabled={!vue.peutAnnuler} onClick={() => outils.agir(api.combatAnnuler())} title="Annuler la dernière action du combat">
            ↶ Annuler
          </button>
          <button className="bouton" disabled={!vue.peutRetablir} onClick={() => outils.agir(api.combatRetablir())} title="Rétablir">
            ↷ Rétablir
          </button>
        </div>
        {enCours && (
          <button className="bouton principal" onClick={() => agir(act.tourSuivant())}>
            Tour suivant ▶
          </button>
        )}
        <button className="bouton" onClick={() => setOuvert({ g: "fin" })}>
          Fin du combat…
        </button>
      </div>

      {vue.phase === "initiative" && <PhaseInitiative vue={vue} agir={agir} />}

      {enCours && (
        <>
          <BarreInitiative vue={vue} onChoisir={(id) => setOuvert({ g: "initiative", pj: id })} />
          {vue.alertes.length > 0 && (
            <Alertes
              alertes={vue.alertes}
              onConfirmer={(a) => envoyer(ev.alerteConfirmee(a.id))}
              onMaintenir={(a) => setOuvert({ g: "maintenir", id: a.id, nom: a.nom, pj: a.personnage })}
            />
          )}
          {vue.demandes.length > 0 && <Demandes demandes={vue.demandes} envoyer={envoyer} />}
          {actif && (
            <PanneauActif
              pj={actif}
              onAgir={() => setOuvert({ g: "agir", pj: actif.id })}
              onSort={() => setOuvert({ g: "sort", pj: actif.id })}
              onObjet={() => setOuvert({ g: "objet", pj: actif.id })}
              onPreparer={() => setOuvert({ g: "preparer", pj: actif.id })}
              onConcentration={() => setOuvert({ g: "concentration", pj: actif.id })}
              onRetarder={() => setOuvert({ g: "initiative", pj: actif.id })}
            />
          )}
        </>
      )}

      <div className="grille-pj">
        <AnimatePresence initial={false}>
          {ordre.map((pj) => (
            <motion.div layout key={pj.id} transition={{ type: "spring", stiffness: 380, damping: 36 }} style={{ minWidth: 0 }}>
              <CartePj
                pj={pj}
                actif={enCours && pj.id === vue.actif}
                grandeJauge={pj.id === vue.actif}
                onSante={(mode) => setOuvert({ g: "sante", pj: pj.id, mode })}
                onEffet={() => setOuvert({ g: "effet", pj: pj.id })}
                onObjet={() => setOuvert({ g: "objetAjout", pj: pj.id })}
                onRetirerEffet={(e) => envoyer(ev.effetRetire(e.id))}
                onUtiliserRessource={(r) => envoyer(ev.ressourceUtilisee(pj.id, r.cle, 1))}
                pied={enCours && <EconomieAffichee pj={pj} />}
              />
            </motion.div>
          ))}
        </AnimatePresence>
      </div>

      <section className="carte">
        <div className="section-tete">
          <h2>Journal du combat</h2>
          <span className="aide">Chaque action est enregistrée avec les dés saisis : le combat est rejouable.</span>
        </div>
        <div className="journal-jeu" style={{ marginTop: 12 }}>
          {vue.journal.map(([r, t], i) => (
            <div className="ligne-j" key={i}>
              <span className="quand">round {r}</span>
              <span>{t}</span>
            </div>
          ))}
        </div>
      </section>

      <AnimatePresence>
        {ouvert?.g === "sante" && (
          <DialogueSante pj={pjs.find((p) => p.id === ouvert.pj)!} mode={ouvert.mode} envoyer={envoyer} onFermer={() => setOuvert(null)} />
        )}
        {ouvert?.g === "effet" && <DialogueEffet pjs={pjs} pjId={ouvert.pj} envoyer={envoyer} onFermer={() => setOuvert(null)} />}
        {ouvert?.g === "objetAjout" && <DialogueObjet pjs={pjs} pjId={ouvert.pj} envoyer={envoyer} onFermer={() => setOuvert(null)} />}
        {ouvert?.g === "agir" && <DialogueAgir pj={pjs.find((p) => p.id === ouvert.pj)!} agir={agir} onFermer={() => setOuvert(null)} />}
        {ouvert?.g === "sort" && <DialogueSort pj={pjs.find((p) => p.id === ouvert.pj)!} pjs={pjs} agir={agir} onFermer={() => setOuvert(null)} />}
        {ouvert?.g === "objet" && <DialogueUtiliserObjet pj={pjs.find((p) => p.id === ouvert.pj)!} pjs={pjs} agir={agir} onFermer={() => setOuvert(null)} />}
        {ouvert?.g === "initiative" && <DialogueInitiative vue={vue} pj={ouvert.pj} nom={nom(ouvert.pj)} agir={agir} onFermer={() => setOuvert(null)} />}
        {ouvert?.g === "concentration" && <DialogueConcentration pj={pjs.find((p) => p.id === ouvert.pj)!} agir={agir} onFermer={() => setOuvert(null)} />}
        {ouvert?.g === "preparer" && <DialoguePreparer pj={pjs.find((p) => p.id === ouvert.pj)!} agir={agir} onFermer={() => setOuvert(null)} />}
        {ouvert?.g === "fin" && <DialogueFin vue={vue} outils={outils} onFermer={() => setOuvert(null)} />}
        {ouvert?.g === "maintenir" && <DialogueMaintenirCombat id={ouvert.id} nom={ouvert.nom} pj={ouvert.pj} envoyer={envoyer} onFermer={() => setOuvert(null)} />}
      </AnimatePresence>
    </div>
  );
}

// ------------------------------------------------------------------ initiative

function PhaseInitiative({ vue, agir }: { vue: CombatVue; agir: (a: ActionCombat) => Promise<Retour | null> }) {
  const [valeurs, setValeurs] = useState<Record<number, number | null>>({});
  const [depart, setDepart] = useState<Record<number, number | null>>({});
  const [surprise, setSurprise] = useState(false);
  const [agissants, setAgissants] = useState<number[]>([]);
  const pret = vue.sansInitiative.length === 0 && vue.egalites.length === 0;
  const tous = vue.initiatives.map((i) => ({ id: i.id, nom: i.nom }));
  return (
    <div className="carte" style={{ display: "flex", flexDirection: "column", gap: 18 }}>
      <h2>Initiative</h2>
      {vue.sansInitiative.map(([id, nom]) => (
        <div key={id} className="champs">
          <SaisieDe
            etiquette={`${nom} — résultat du d20`}
            rappel={`d20 + modificateur d'initiative de la fiche`}
            min={1}
            max={20}
            valeur={valeurs[id] ?? null}
            onChange={(v) => setValeurs({ ...valeurs, [id]: v })}
          />
          <button className="bouton principal" disabled={valeurs[id] == null} onClick={() => agir(act.initiative(id, valeurs[id]!))}>
            Valider
          </button>
        </div>
      ))}
      {vue.initiatives.length > 0 && (
        <div className="bloc">
          <h3>Déjà saisies</h3>
          <div className="liste-compacte">
            {vue.initiatives.map((i) => (
              <div className="ligne-objet" key={i.id}>
                <span className="nom">{i.nom}</span>
                <span className="reste-compte">
                  {i.d20} {i.modificateur >= 0 ? "+" : "−"} {Math.abs(i.modificateur)} = <strong>{i.total}</strong>
                  {i.departage !== null ? ` (départage ${i.departage})` : ""}
                </span>
                <button className="bouton mini discret" onClick={() => agir(act.initiative(i.id, 10))} hidden>
                  .
                </button>
              </div>
            ))}
          </div>
        </div>
      )}
      {vue.egalites.length > 0 && (
        <div className="avertissement">
          <strong>Égalité parfaite (même total, même modificateur) :</strong> chacun lance un d20 de départage.
          {vue.egalites.flat().map((nom) => {
            const i = vue.initiatives.find((x) => x.nom === nom);
            if (!i) return null;
            return (
              <div key={i.id} className="champs">
                <SaisieDe etiquette={`${nom} — d20 de départage`} min={1} max={20} valeur={depart[i.id] ?? null} onChange={(v) => setDepart({ ...depart, [i.id]: v })} />
                <button className="bouton" disabled={depart[i.id] == null} onClick={() => agir(act.departage(i.id, depart[i.id]!))}>
                  Valider
                </button>
              </div>
            );
          })}
        </div>
      )}
      {pret && (
        <div className="bloc">
          <label className="interrupteur">
            <input type="checkbox" checked={surprise} onChange={(e) => setSurprise(e.target.checked)} />
            Il y a un round de surprise (une seule action simple ou de mouvement, pour ceux qui ne sont pas surpris)
          </label>
          {surprise && (
            <div className="rangee-boutons">
              {tous.map((p) => (
                <label key={p.id} className="interrupteur">
                  <input type="checkbox" checked={agissants.includes(p.id)} onChange={(e) => setAgissants(e.target.checked ? [...agissants, p.id] : agissants.filter((x) => x !== p.id))} />
                  {p.nom} agit
                </label>
              ))}
            </div>
          )}
          <div className="rangee-boutons">
            <button className="bouton principal" disabled={surprise && agissants.length === 0} onClick={() => agir(act.demarrer(surprise ? agissants : null))}>
              Démarrer le combat
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

function BarreInitiative({ vue, onChoisir }: { vue: CombatVue; onChoisir: (id: number) => void }) {
  return (
    <div className="barre-initiative" role="list" aria-label="Ordre d'initiative">
      {vue.initiatives.map((i) => (
        <motion.button
          layout
          role="listitem"
          key={i.id}
          className={"jeton-init" + (i.actif ? " actif" : "") + (!i.joue ? " passe" : "")}
          onClick={() => onChoisir(i.id)}
          transition={{ type: "spring", stiffness: 420, damping: 34 }}
          title="Corriger l'initiative ou retarder"
        >
          <span className="total">{i.total}</span>
          <span className="nom">{i.nom}</span>
          <span className="calcul">
            {i.d20} {i.modificateur >= 0 ? "+" : "−"} {Math.abs(i.modificateur)}
          </span>
        </motion.button>
      ))}
    </div>
  );
}

function EconomieAffichee({ pj }: { pj: PjCombat }) {
  const e = pj.economie;
  const puce = (libre: boolean, nom: string) => (
    <span key={nom} className={"eco " + (libre ? "libre" : "depense")}>
      {nom}
    </span>
  );
  return (
    <div className="bloc">
      <h3>Actions du tour{e.limite ? " (une seule action simple ou de mouvement)" : ""}</h3>
      <div className="economie">
        {puce(e.simple, "Simple")}
        {puce(e.mouvement, "Mouvement")}
        {puce(e.complexe, "Complexe")}
        {puce(e.rapide, "Rapide")}
        {puce(e.pas, "Pas de 1,5 m")}
      </div>
      <span className="doux">
        {pj.degatsSubis > 0 ? `${pj.degatsSubis} dégâts subis` : "aucun dégât subi"}
        {pj.sortsLances > 0 ? ` · ${pj.sortsLances} sort(s) lancé(s)` : ""}
      </span>
    </div>
  );
}

function PanneauActif({
  pj,
  onAgir,
  onSort,
  onObjet,
  onPreparer,
  onConcentration,
  onRetarder,
}: {
  pj: PjCombat;
  onAgir: () => void;
  onSort: () => void;
  onObjet: () => void;
  onPreparer: () => void;
  onConcentration: () => void;
  onRetarder: () => void;
}) {
  const incapable = ["mourant", "stable", "mort", "inconscient"].includes(pj.santeCode);
  return (
    <div className="carte" style={{ display: "flex", flexWrap: "wrap", alignItems: "center", gap: 14 }}>
      <div style={{ marginRight: "auto" }}>
        <small className="doux">C'est le tour de</small>
        <h2 style={{ fontSize: 22 }}>{pj.nom}</h2>
      </div>
      {incapable ? (
        <span className="doux">Ce personnage ne peut pas agir ({pj.sante}).</span>
      ) : (
        <div className="rangee-boutons">
          <button className="bouton principal" onClick={onAgir}>Agir…</button>
          {pj.caster && <button className="bouton" onClick={onSort}>Lancer un sort…</button>}
          <button className="bouton" disabled={pj.inventaire.length === 0} onClick={onObjet}>Utiliser un objet…</button>
          <button className="bouton" onClick={onPreparer}>Préparer une action</button>
          <button className="bouton" onClick={onRetarder}>Retarder…</button>
          {pj.caster && <button className="bouton" onClick={onConcentration}>Test de concentration…</button>}
        </div>
      )}
    </div>
  );
}

// ------------------------------------------------------------------ agir

function DialogueAgir({ pj, agir, onFermer }: { pj: PjCombat; agir: (a: ActionCombat) => Promise<Retour | null>; onFermer: () => void }) {
  const [catalogue, setCatalogue] = useState<ActionDef[]>([]);
  const [recherche, setRecherche] = useState("");
  const [choix, setChoix] = useState<ActionDef | null>(null);
  const [typeVariable, setTypeVariable] = useState<TypeAction>("simple");
  const [deplacement, setDeplacement] = useState(false);
  const [libelle, setLibelle] = useState("");
  const [distance, setDistance] = useState(false);
  const [munition, setMunition] = useState<number | null>(null);
  const [tirs, setTirs] = useState<number | null>(1);
  useEffect(() => {
    api.catalogueActions().then(setCatalogue);
  }, []);
  const munitions = pj.inventaire.filter((o) => o.genreCode === "munition");
  const attaque = choix && ["attack", "full_attack", "charge"].includes(choix.id);
  const type: TypeAction | null = choix ? (choix.type === "variable" ? typeVariable : choix.type) : null;
  const groupes: TypeAction[] = ["simple", "mouvement", "complexe", "rapide", "immediate", "libre", "aucune"];
  const visibles = catalogue.filter((a) => `${a.nom} ${a.nom_en}`.toLowerCase().includes(recherche.toLowerCase()));

  const valider = async () => {
    if (!type) return;
    const nom = libelle.trim() || choix?.nom || "Action";
    const r = distance
      ? await agir(act.attaquer({ pj: pj.id, typeAction: type, libelle: nom, tirs: tirs ?? 1, munition, aDistanceAvecMunitions: true, deplacement }))
      : await agir(act.agir(pj.id, type, nom, deplacement));
    if (r?.ok) onFermer();
  };

  return (
    <Dialogue
      large
      titre={`Agir — ${pj.nom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Fermer</button>
          <button className="bouton principal" disabled={!choix || (distance && munition === null)} onClick={valider}>
            Faire cette action
          </button>
        </>
      }
    >
      <EconomieAffichee pj={pj} />
      <input className="champ-jeu" placeholder="Chercher une action (attaque, charge, parler…)" value={recherche} onChange={(e) => setRecherche(e.target.value)} autoFocus />
      <div className="bloc" style={{ maxHeight: 260, overflowY: "auto" }}>
        {groupes.map((g) => {
          const l = visibles.filter((a) => (a.type === "variable" ? false : a.type === g));
          if (l.length === 0) return null;
          return (
            <div key={g} className="bloc">
              <h3>{NOM_TYPE[g]}</h3>
              <div className="rangee-boutons">
                {l.map((a) => (
                  <button
                    key={a.id}
                    className={"bouton mini" + (choix?.id === a.id ? " principal" : "")}
                    onClick={() => {
                      setChoix(a);
                      setDeplacement(a.deplacement);
                      setDistance(false);
                      setLibelle("");
                    }}
                  >
                    {a.nom}
                  </button>
                ))}
              </div>
            </div>
          );
        })}
        {visibles.some((a) => a.type === "variable") && (
          <div className="bloc">
            <h3>Type d'action variable (précisez-le)</h3>
            <div className="rangee-boutons">
              {visibles
                .filter((a) => a.type === "variable")
                .map((a) => (
                  <button key={a.id} className={"bouton mini" + (choix?.id === a.id ? " principal" : "")} onClick={() => (setChoix(a), setDeplacement(false), setDistance(false))}>
                    {a.nom}
                  </button>
                ))}
            </div>
          </div>
        )}
      </div>
      {choix && (
        <div className="bloc">
          <h3>Cette action</h3>
          <div className="champs">
            {choix.type === "variable" && (
              <label className="champ-g">
                <span>Type d'action</span>
                <select className="champ-jeu" value={typeVariable} onChange={(e) => setTypeVariable(e.target.value as TypeAction)}>
                  {(["simple", "mouvement", "complexe", "rapide", "immediate", "libre", "aucune"] as TypeAction[]).map((t) => (
                    <option key={t} value={t}>{NOM_TYPE[t]}</option>
                  ))}
                </select>
              </label>
            )}
            <label className="champ-g">
              <span>Précision pour le journal (facultatif)</span>
              <input className="champ-jeu" value={libelle} onChange={(e) => setLibelle(e.target.value)} placeholder={choix.nom} />
            </label>
          </div>
          <label className="interrupteur">
            <input type="checkbox" checked={deplacement} onChange={(e) => setDeplacement(e.target.checked)} />
            Le personnage se déplace vraiment (incompatible avec un pas de 1,5 m)
          </label>
          {attaque && (
            <>
              <label className="interrupteur">
                <input type="checkbox" checked={distance} onChange={(e) => setDistance(e.target.checked)} />
                Attaque à distance avec munitions (décompte automatique ; refusée sans munition)
              </label>
              {distance && (
                <div className="champs">
                  <label className="champ-g">
                    <span>Munition</span>
                    <select className="champ-jeu" value={munition ?? ""} onChange={(e) => setMunition(e.target.value === "" ? null : Number(e.target.value))}>
                      <option value="">— choisir —</option>
                      {munitions.map((m) => (
                        <option key={m.id} value={m.id}>{m.nom} (× {m.quantite})</option>
                      ))}
                    </select>
                  </label>
                  <SaisieNombre etiquette="Tirs" min={1} max={20} valeur={tirs} onChange={setTirs} rappel="1 par attaque ; plusieurs à l'attaque à outrance" />
                </div>
              )}
            </>
          )}
          <span className="doux">{type ? NOM_TYPE[type] : ""}</span>
        </div>
      )}
    </Dialogue>
  );
}

// ------------------------------------------------------------------ sorts

function DialogueSort({
  pj,
  pjs,
  agir,
  onFermer,
}: {
  pj: PjCombat;
  pjs: PjCombat[];
  agir: (a: ActionCombat) => Promise<Retour | null>;
  onFermer: () => void;
}) {
  const [sort, setSort] = useState<SortConnuVue | null>(null);
  const [apercu, setApercu] = useState<ApercuSort | null>(null);
  const [erreur, setErreur] = useState<string | null>(null);
  const [defensif, setDefensif] = useState(false);
  const [dDefensif, setDDefensif] = useState<number | null>(null);
  const [d100, setD100] = useState<number | null>(null);
  const [cibles, setCibles] = useState<number[]>([pj.id]);
  const [genreDuree, setGenreDuree] = useState("auto");
  const [nDuree, setNDuree] = useState<number | null>(null);
  const [energie, setEnergie] = useState("feu");
  const [resultat, setResultat] = useState<number | null>(null);

  const restants = (niveau: number) => (niveau === 0 ? 99 : pj.emplacements.find((e) => e.niveau === niveau)?.restants ?? 0);
  const niveaux = [...new Set(pj.sortsConnus.map((s) => s.niveau))].sort((a, b) => a - b);

  const choisir = async (s: SortConnuVue) => {
    setSort(s);
    setApercu(null);
    setErreur(null);
    try {
      const a = await api.combatApercuSort(pj.id, s.nomEn);
      setApercu(a);
      setGenreDuree("auto");
      setNDuree(null);
    } catch (e) {
      setErreur(String(e));
    }
  };

  const duree: Duree | null = genreDuree === "auto" ? null : dureeDe(genreDuree, nDuree);
  const pret =
    apercu !== null &&
    (!defensif || dDefensif !== null) &&
    (apercu.echecArcaniquePct === 0 || d100 !== null) &&
    (genreDuree === "auto" || duree !== null);

  const lancer = async () => {
    if (!apercu || !sort) return;
    const r = await agir(
      act.lancer({
        pj: pj.id,
        sort: sort.nomEn,
        defensifD20: defensif ? dDefensif : null,
        echecArcaniqueD100: apercu.echecArcaniquePct > 0 ? d100 : null,
        cibles,
        duree,
        choixEnergie: apercu.choixEnergie ? energie : null,
        resultatDe: resultat,
      }),
    );
    if (r?.ok) onFermer();
  };

  return (
    <Dialogue
      large
      titre={`Lancer un sort — ${pj.nom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Fermer</button>
          <button className="bouton principal" disabled={!pret} onClick={lancer}>
            Lancer le sort
          </button>
        </>
      }
    >
      <div className="bloc">
        <h3>Emplacements restants</h3>
        {pj.emplacements.map((e) => (
          <Gemmes key={e.niveau} niveau={e.niveau} restants={e.restants} total={e.total} dd={e.dd} />
        ))}
      </div>
      {niveaux.map((n) => (
        <div className="bloc" key={n}>
          <h3>{n === 0 ? "Tours de magie (à volonté)" : `Sorts de niveau ${n}`}</h3>
          <div className="cartes-sorts">
            {pj.sortsConnus
              .filter((s) => s.niveau === n)
              .map((s) => (
                <button
                  key={s.nomEn}
                  className="carte-sort"
                  disabled={restants(n) === 0}
                  aria-pressed={sort?.nomEn === s.nomEn}
                  style={sort?.nomEn === s.nomEn ? { borderColor: "var(--magie)" } : undefined}
                  onClick={() => choisir(s)}
                  title={restants(n) === 0 ? "Plus d'emplacement de ce niveau" : undefined}
                >
                  <span className="niveau-sort">{n === 0 ? "Tour de magie" : `Niveau ${n}`}</span>
                  <span className="nom-sort">{s.nom}</span>
                </button>
              ))}
          </div>
        </div>
      ))}
      {erreur && <div className="avertissement">{erreur}</div>}
      {apercu && (
        <>
          <div className="details-sort">
            <strong>{apercu.nom}</strong>
            <div className="grille">
              <span>Emplacement : niveau {apercu.niveauEmplacement}{apercu.restants !== null ? ` (${apercu.restants} restant(s))` : " (à volonté)"}</span>
              <span>DD des jets de sauvegarde : {apercu.dd}</span>
              <span>Niveau de lanceur : {apercu.niveauDeLanceur}</span>
              <span>{apercu.typeAction}</span>
              <span>Composantes : {[apercu.verbal && "verbale", apercu.gestuel && "gestuelle"].filter(Boolean).join(" et ") || "aucune verbale ni gestuelle"}</span>
              {apercu.portee && <span>Portée : {apercu.portee}</span>}
              {apercu.jetDeSauvegarde && <span>Jet de sauvegarde : {apercu.jetDeSauvegarde}</span>}
              {apercu.dureeTexte && <span>Durée : {apercu.dureeTexte}</span>}
            </div>
            <div className="descr">{apercu.description}</div>
            {apercu.notes && <span className="doux">{apercu.notes}</span>}
            {!apercu.effetAutomatique && <span className="doux">Pas d'effet appliqué automatiquement : l'emplacement est décompté et le lancement noté ; appliquez l'effet à la main si besoin.</span>}
          </div>
          <div className="champs">
            <label className="interrupteur">
              <input type="checkbox" checked={defensif} onChange={(e) => setDefensif(e.target.checked)} />
              Lancement défensif (pour ne pas provoquer d'attaque d'opportunité)
            </label>
          </div>
          {defensif && (
            <SaisieDe
              etiquette="Test de concentration — d20"
              rappel={`d20 + ${apercu.bonusConcentration} contre DD ${apercu.ddLancementDefensif}`}
              min={1}
              max={20}
              valeur={dDefensif}
              onChange={setDDefensif}
            />
          )}
          {apercu.echecArcaniquePct > 0 && (
            <SaisieDe
              etiquette="Échec des sorts profanes — d100"
              rappel={`le sort échoue si le résultat est ${apercu.echecArcaniquePct} ou moins`}
              min={1}
              max={100}
              valeur={d100}
              onChange={setD100}
              pave={false}
            />
          )}
          {apercu.effetAutomatique && (
            <div className="bloc">
              <h3>Qui reçoit l'effet</h3>
              <div className="rangee-boutons">
                {pjs.map((p) => (
                  <label key={p.id} className="interrupteur">
                    <input type="checkbox" checked={cibles.includes(p.id)} onChange={(e) => setCibles(e.target.checked ? [...cibles, p.id] : cibles.filter((x) => x !== p.id))} />
                    {p.nom}
                  </label>
                ))}
              </div>
              <ChoixDuree
                genre={genreDuree}
                nombre={nDuree}
                onGenre={setGenreDuree}
                onNombre={setNDuree}
                auto={apercu.dureeSuggereeTexte ? `celle du livre : ${apercu.dureeSuggereeTexte}` : "à saisir (formule non simple)"}
              />
              {apercu.choixEnergie && (
                <label className="champ-g">
                  <span>Type d'énergie</span>
                  <select className="champ-jeu" value={energie} onChange={(e) => setEnergie(e.target.value)}>
                    {["acide", "froid", "électricité", "feu", "son"].map((t) => (
                      <option key={t} value={t}>{t}</option>
                    ))}
                  </select>
                </label>
              )}
            </div>
          )}
          <SaisieNombre etiquette="Résultat de vos dés (dégâts, soins… facultatif)" rappel="Noté dans le journal ; rien n'est appliqué à un ennemi (aucun ennemi n'est modélisé)." valeur={resultat} onChange={setResultat} />
        </>
      )}
    </Dialogue>
  );
}

// ------------------------------------------------------------------ objets

function DialogueUtiliserObjet({
  pj,
  pjs,
  agir,
  onFermer,
}: {
  pj: PjCombat;
  pjs: PjCombat[];
  agir: (a: ActionCombat) => Promise<Retour | null>;
  onFermer: () => void;
}) {
  const objets = pj.inventaire.filter((o) => o.genreCode !== "munition");
  const [id, setId] = useState<number | null>(objets[0]?.id ?? null);
  const objet = objets.find((o) => o.id === id) ?? null;
  const [cibles, setCibles] = useState<number[]>([pj.id]);
  const [genreDuree, setGenreDuree] = useState("auto");
  const [nDuree, setNDuree] = useState<number | null>(null);
  const [charges, setCharges] = useState<number | null>(1);
  const [dCl, setDCl] = useState<number | null>(null);
  const [dAccident, setDAccident] = useState<number | null>(null);
  const [d100, setD100] = useState<number | null>(null);
  const [type, setType] = useState<TypeAction>("simple");
  const [energie, setEnergie] = useState("feu");
  const [resultat, setResultat] = useState<number | null>(null);

  const parchemin = objet?.genreCode === "parchemin";
  const besoinCl = parchemin && objet !== null && objet.niveauDeLanceur > pj.niveauDeLanceur;
  const duree = genreDuree === "auto" ? null : dureeDe(genreDuree, nDuree);
  const valide = objet !== null && (genreDuree === "auto" || duree !== null) && (!besoinCl || dCl !== null);

  const utiliser = async () => {
    if (!objet) return;
    const r = await agir(
      act.utiliserObjet({
        pj: pj.id,
        objet: objet.id,
        cibles: objet.genreCode === "potion" ? [pj.id] : cibles,
        duree,
        choixEnergie: energie,
        charges: objet.genreCode === "baton" ? charges ?? 1 : 0,
        niveauDeLanceurD20: besoinCl ? dCl : null,
        accidentD20: dAccident,
        echecArcaniqueD100: d100,
        typeAction: objet.sort ? null : type,
        resultatDe: resultat,
      }),
    );
    if (r?.ok) onFermer();
  };

  return (
    <Dialogue
      large
      titre={`Utiliser un objet — ${pj.nom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Fermer</button>
          <button className="bouton principal" disabled={!valide} onClick={utiliser}>
            Utiliser
          </button>
        </>
      }
    >
      <div className="rangee-boutons">
        {objets.map((o) => (
          <button key={o.id} className={"bouton" + (o.id === id ? " principal" : "")} onClick={() => setId(o.id)}>
            {o.nom} {o.charges ? `(${o.charges[0]}/${o.charges[1]})` : `× ${o.quantite}`}
          </button>
        ))}
      </div>
      {objet && (
        <>
          <div className="details-sort">
            <strong>{objet.nom}</strong>
            <span>
              {objet.genre}
              {objet.sort ? ` — sort : ${objet.sort}` : ""} · niveau de lanceur {objet.niveauDeLanceur}
            </span>
            {objet.genreCode === "potion" && <span className="doux">Boire une potion : action simple ; le buveur est le lanceur et la cible (Core p. 477).</span>}
            {parchemin && <span className="doux">Il faut un parchemin du bon type (profane ou divin), un sort de votre liste et le score minimal (Core p. 490).</span>}
            {(objet.genreCode === "baguette" || objet.genreCode === "baton") && <span className="doux">Le sort doit être sur votre liste de classe ; action simple (Core p. 491, 496).</span>}
          </div>
          {objet.genreCode === "baton" && (
            <SaisieNombre etiquette="Charges dépensées" min={1} max={10} valeur={charges} onChange={setCharges} rappel="Un sort peut coûter plusieurs charges (voir l'objet)." />
          )}
          {!objet.sort && (
            <label className="champ-g">
              <span>Type d'action</span>
              <select className="champ-jeu" value={type} onChange={(e) => setType(e.target.value as TypeAction)}>
                {(["simple", "mouvement", "complexe", "rapide", "immediate", "libre"] as TypeAction[]).map((t) => (
                  <option key={t} value={t}>{NOM_TYPE[t]}</option>
                ))}
              </select>
            </label>
          )}
          {besoinCl && (
            <>
              <SaisieDe
                etiquette="Test de niveau de lanceur — d20"
                rappel={`d20 + votre niveau de lanceur (${pj.niveauDeLanceur}) contre DD ${objet.niveauDeLanceur + 1}`}
                min={1}
                max={20}
                valeur={dCl}
                onChange={setDCl}
              />
              <SaisieDe
                etiquette="Si le test est raté : test de Sagesse DD 5 — d20 (facultatif)"
                rappel="Un 1 naturel échoue toujours : l'accident consomme le parchemin"
                min={1}
                max={20}
                valeur={dAccident}
                onChange={setDAccident}
              />
            </>
          )}
          {parchemin && pj.echecArcaniquePct > 0 && (
            <SaisieDe
              etiquette="Échec des sorts profanes — d100"
              rappel={`échec à ${pj.echecArcaniquePct} ou moins si le sort a un geste`}
              min={1}
              max={100}
              valeur={d100}
              onChange={setD100}
              pave={false}
            />
          )}
          {objet.sort && objet.genreCode !== "potion" && (
            <div className="bloc">
              <h3>Qui reçoit l'effet</h3>
              <div className="rangee-boutons">
                {pjs.map((p) => (
                  <label key={p.id} className="interrupteur">
                    <input type="checkbox" checked={cibles.includes(p.id)} onChange={(e) => setCibles(e.target.checked ? [...cibles, p.id] : cibles.filter((x) => x !== p.id))} />
                    {p.nom}
                  </label>
                ))}
              </div>
            </div>
          )}
          {objet.sort && (
            <>
              <ChoixDuree genre={genreDuree} nombre={nDuree} onGenre={setGenreDuree} onNombre={setNDuree} auto="celle du livre (si la formule est simple)" />
              <label className="champ-g">
                <span>Type d'énergie (résistance à l'énergie seulement)</span>
                <select className="champ-jeu" value={energie} onChange={(e) => setEnergie(e.target.value)}>
                  {["acide", "froid", "électricité", "feu", "son"].map((t) => (
                    <option key={t} value={t}>{t}</option>
                  ))}
                </select>
              </label>
            </>
          )}
          <SaisieNombre etiquette="Résultat de vos dés (facultatif)" valeur={resultat} onChange={setResultat} />
        </>
      )}
    </Dialogue>
  );
}

// ------------------------------------------------------------------ petits dialogues

function DialogueInitiative({
  vue,
  pj,
  nom,
  agir,
  onFermer,
}: {
  vue: CombatVue;
  pj: number;
  nom: string;
  agir: (a: ActionCombat) => Promise<Retour | null>;
  onFermer: () => void;
}) {
  const i = vue.initiatives.find((x) => x.id === pj);
  const [total, setTotal] = useState<number | null>(i?.total ?? null);
  const actif = vue.actif === pj;
  return (
    <Dialogue
      titre={`Initiative — ${nom}`}
      onFermer={onFermer}
      actions={<button className="bouton" onClick={onFermer}>Fermer</button>}
    >
      <p className="doux">Initiative actuelle : <strong>{i?.total}</strong> ({i?.d20} {i && i.modificateur >= 0 ? "+" : "−"} {i ? Math.abs(i.modificateur) : 0}).</p>
      <SaisieNombre etiquette="Nouvelle valeur d'initiative" min={-20} max={60} valeur={total} onChange={setTotal} />
      <div className="rangee-boutons">
        <button
          className="bouton"
          disabled={total === null}
          onClick={async () => {
            const r = await agir(act.modifierInitiative(pj, total!));
            if (r?.ok) onFermer();
          }}
        >
          Corriger l'initiative
        </button>
        {actif && (
          <button
            className="bouton principal"
            disabled={total === null}
            onClick={async () => {
              const r = await agir(act.retarder(pj, total!));
              if (r?.ok) onFermer();
            }}
            title="Le personnage passe plus bas dans l'ordre (valeur plus basse que la sienne) ; c'est alors au suivant"
          >
            Retarder son action à cette valeur
          </button>
        )}
      </div>
    </Dialogue>
  );
}

function DialoguePreparer({ pj, agir, onFermer }: { pj: PjCombat; agir: (a: ActionCombat) => Promise<Retour | null>; onFermer: () => void }) {
  const [texte, setTexte] = useState("");
  return (
    <Dialogue
      titre={`Préparer une action — ${pj.nom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button
            className="bouton principal"
            disabled={texte.trim() === ""}
            onClick={async () => {
              const r = await agir(act.preparer(pj.id, texte.trim()));
              if (r?.ok) onFermer();
            }}
          >
            Préparer
          </button>
        </>
      }
    >
      <input className="champ-jeu" value={texte} onChange={(e) => setTexte(e.target.value)} placeholder="Tirer sur le premier ennemi qui franchit la porte" autoFocus />
      <p className="doux">L'action préparée est notée ; elle expire au prochain tour du personnage.</p>
    </Dialogue>
  );
}

const CAUSES: [string, string, (v: number | null) => Record<string, unknown> | string, string | null][] = [
  ["Blessure", "Dégâts subis", (v) => ({ Blessure: { degats: v ?? 0 } }), "dégâts reçus"],
  ["DegatsContinus", "Dégâts continus", (v) => ({ DegatsContinus: { derniers_degats: v ?? 0 } }), "derniers dégâts"],
  ["SortSurSoi", "Sort qui nous affecte", (v) => ({ SortSurSoi: { dd_du_sort: v ?? 0 } }), "DD du jet du sort"],
  ["Agrippe", "Agrippé ou immobilisé", (v) => ({ Agrippe: { bmc: v ?? 0 } }), "BMC de l'adversaire"],
  ["MouvementVigoureux", "Mouvement vigoureux", () => "MouvementVigoureux", null],
  ["MouvementViolent", "Mouvement violent", () => "MouvementViolent", null],
  ["MouvementTresViolent", "Mouvement extrêmement violent", () => "MouvementTresViolent", null],
  ["Entrave", "Entravé", () => "Entrave", null],
  ["Intemperies", "Mauvais temps (pluie, grésil)", () => "Intemperies", null],
  ["IntemperiesFortes", "Mauvais temps (grêle, débris)", () => "IntemperiesFortes", null],
  ["Libre", "Autre (DD donné par le livre)", (v) => ({ Libre: { dd: v ?? 0 } }), "DD"],
];

function DialogueConcentration({ pj, agir, onFermer }: { pj: PjCombat; agir: (a: ActionCombat) => Promise<Retour | null>; onFermer: () => void }) {
  const [cause, setCause] = useState(0);
  const [valeur, setValeur] = useState<number | null>(null);
  const [niveau, setNiveau] = useState<number | null>(1);
  const [d20, setD20] = useState<number | null>(null);
  const [effet, setEffet] = useState<number | "">("");
  const c = CAUSES[cause];
  const besoinValeur = c[3] !== null;
  return (
    <Dialogue
      titre={`Test de concentration — ${pj.nom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button
            className="bouton principal"
            disabled={d20 === null || niveau === null || (besoinValeur && valeur === null)}
            onClick={async () => {
              const r = await agir(act.concentration(pj.id, d20!, c[2](valeur), niveau!, effet === "" ? null : effet));
              if (r?.ok) onFermer();
            }}
          >
            Valider le test
          </button>
        </>
      }
    >
      <label className="champ-g">
        <span>Ce qui vous distrait</span>
        <select className="champ-jeu" value={cause} onChange={(e) => setCause(Number(e.target.value))}>
          {CAUSES.map((x, i) => (
            <option key={x[0]} value={i}>{x[1]}</option>
          ))}
        </select>
      </label>
      {besoinValeur && <SaisieNombre etiquette={c[3]!} min={0} max={500} valeur={valeur} onChange={setValeur} />}
      <SaisieNombre etiquette="Niveau du sort" min={0} max={9} valeur={niveau} onChange={setNiveau} />
      <label className="champ-g">
        <span>Sort à concentration maintenu (rompu si le test est raté)</span>
        <select className="champ-jeu" value={effet} onChange={(e) => setEffet(e.target.value === "" ? "" : Number(e.target.value))}>
          <option value="">Aucun (simple test)</option>
          {pj.effets.map((e) => (
            <option key={e.id} value={e.id}>{e.nom} — {e.duree}</option>
          ))}
        </select>
      </label>
      <SaisieDe etiquette="d20" rappel={`d20 + niveau de lanceur ${pj.niveauDeLanceur} + modificateur d'incantation (calculés par l'application)`} min={1} max={20} valeur={d20} onChange={setD20} />
    </Dialogue>
  );
}

function DialogueMaintenirCombat({
  id,
  nom,
  pj,
  envoyer,
  onFermer,
}: {
  id: number;
  nom: string;
  pj: string;
  envoyer: Envoyer;
  onFermer: () => void;
}) {
  const [genre, setGenre] = useState("minutes");
  const [n, setN] = useState<number | null>(10);
  const duree = dureeDe(genre, n);
  return (
    <Dialogue
      titre={`Maintenir « ${nom} »`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button
            className="bouton principal"
            disabled={!duree}
            onClick={async () => {
              await envoyer(ev.effetRetabli(id, duree!));
              onFermer();
            }}
          >
            Remettre l'effet
          </button>
        </>
      }
    >
      <p className="doux">L'effet revient pour la durée saisie, comptée à partir de maintenant ({pj}).</p>
      <ChoixDuree genre={genre} nombre={n} onGenre={setGenre} onNombre={setN} />
    </Dialogue>
  );
}

// ------------------------------------------------------------------ fin du combat

function DialogueFin({ vue, outils, onFermer }: { vue: CombatVue; outils: Outils; onFermer: () => void }) {
  const [apercu, setApercu] = useState<ApercuFin | null>(null);
  const [mode, setMode] = useState<"tout" | "partiel" | "abandon">("tout");
  const [choisis, setChoisis] = useState<number[]>(vue.personnages.map((p) => p.id));
  const [maintenus, setMaintenus] = useState<Record<number, { genre: string; n: number | null }>>({});
  useEffect(() => {
    api.combatApercuFin().then(setApercu).catch(() => setApercu(null));
  }, [vue.journal.length]);

  const contestations: [number, Duree][] = Object.entries(maintenus)
    .map(([id, m]) => [Number(id), dureeDe(m.genre, m.n)] as [number, Duree | null])
    .filter((x): x is [number, Duree] => x[1] !== null);
  const validation: Validation = mode === "tout" ? { genre: "tout" } : mode === "partiel" ? { genre: "partiel", pjs: choisis } : { genre: "abandon" };

  return (
    <Dialogue
      large
      titre="Fin du combat"
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Reprendre le combat</button>
          <button
            className={"bouton " + (mode === "abandon" ? "danger" : "principal")}
            disabled={mode === "partiel" && choisis.length === 0}
            onClick={async () => {
              if (mode === "abandon" && !window.confirm("Abandonner le combat ? Rien ne sera reporté dans la campagne et l'horloge reste au début du combat.")) return;
              const r = await outils.agir(api.combatTerminer(validation, mode === "abandon" ? [] : contestations));
              if (r?.ok) onFermer();
            }}
          >
            {mode === "abandon" ? "Abandonner le combat" : mode === "partiel" ? "Valider pour les personnages choisis" : "Valider le combat"}
          </button>
        </>
      }
    >
      {!apercu && <p className="doux">Calcul du récapitulatif…</p>}
      {apercu && (
        <>
          <div className="bloc">
            <h3>Récapitulatif — {apercu.roundsJoues} round(s) achevé(s)</h3>
            <table className="tableau-bilan">
              <thead>
                <tr><th>Personnage</th><th>PV avant → après</th><th>Dégâts subis</th><th>Soins reçus</th><th>Sorts lancés</th></tr>
              </thead>
              <tbody>
                {apercu.personnages.map((p) => (
                  <tr key={p.id}>
                    <td>{p.nom}</td>
                    <td>{p.pvAvant} → <strong>{p.pvApres}</strong></td>
                    <td>{p.degatsSubis}</td>
                    <td>{p.soinsRecus}</td>
                    <td>{p.sortsLances}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {apercu.dissipes.length > 0 && (
            <div className="avertissement" role="alert">
              <strong>Effets dissipés à la fin du combat</strong>
              <span className="doux">
                Simplification : les effets dont la durée est en rounds (ou « jusqu'à… ») s'arrêtent. Vous pouvez en maintenir en saisissant une durée.
              </span>
              {apercu.dissipes.map((d) => {
                const m = maintenus[d.id];
                return (
                  <div key={d.id} className="bloc">
                    <div>
                      <strong>{d.nom}</strong> ({d.personnage}) — {d.source} — durée {d.duree}
                      {d.perdu ? <>, temps perdu : <strong>{d.perdu}</strong></> : null}
                    </div>
                    <label className="interrupteur">
                      <input
                        type="checkbox"
                        checked={!!m}
                        onChange={(e) => {
                          const suite = { ...maintenus };
                          if (e.target.checked) suite[d.id] = { genre: "minutes", n: 10 };
                          else delete suite[d.id];
                          setMaintenus(suite);
                        }}
                      />
                      Maintenir cet effet
                    </label>
                    {m && (
                      <ChoixDuree genre={m.genre} nombre={m.n} onGenre={(g) => setMaintenus({ ...maintenus, [d.id]: { ...m, genre: g } })} onNombre={(n) => setMaintenus({ ...maintenus, [d.id]: { ...m, n } })} />
                    )}
                  </div>
                );
              })}
            </div>
          )}
          {apercu.arrondis.length > 0 && (
            <div className="bloc">
              <h3>Durées en minutes arrondies à la minute la plus proche (au moins 1)</h3>
              {apercu.arrondis.map((a, i) => (
                <div key={i}>
                  {a.nom} ({a.personnage}) : {a.avant} → <strong>{a.apres}</strong>
                </div>
              ))}
            </div>
          )}
          <div className="bloc">
            <h3>Effets qui continuent</h3>
            {apercu.persistants.length === 0 && <span className="doux">Aucun.</span>}
            {apercu.persistants.map(([n, p, fin], i) => (
              <div key={i}>
                {n} ({p}) — {fin ? `jusqu'au ${fin}` : "sans échéance"}
              </div>
            ))}
          </div>
        </>
      )}
      <div className="bloc">
        <h3>Que faire du combat ?</h3>
        <label className="interrupteur">
          <input type="radio" name="fin" checked={mode === "tout"} onChange={() => setMode("tout")} />
          Valider pour tous : santé, sorts dépensés, effets et objets sont reportés sur les fiches et l'aventure ; l'horloge avance de {apercu ? apercu.roundsJoues : "…"} round(s)
        </label>
        <label className="interrupteur">
          <input type="radio" name="fin" checked={mode === "partiel"} onChange={() => setMode("partiel")} />
          Valider seulement pour certains personnages (les autres reviennent à leur état d'avant le combat)
        </label>
        {mode === "partiel" && (
          <div className="rangee-boutons">
            {vue.personnages.map((p) => (
              <label key={p.id} className="interrupteur">
                <input type="checkbox" checked={choisis.includes(p.id)} onChange={(e) => setChoisis(e.target.checked ? [...choisis, p.id] : choisis.filter((x) => x !== p.id))} />
                {p.nom}
              </label>
            ))}
          </div>
        )}
        <label className="interrupteur">
          <input type="radio" name="fin" checked={mode === "abandon"} onChange={() => setMode("abandon")} />
          Abandonner le combat : rien n'est reporté, l'horloge reste au début du combat
        </label>
      </div>
    </Dialogue>
  );
}

