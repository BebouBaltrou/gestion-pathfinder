// Dialogues communs à l'aventure et au combat : santé, effets, repos, heure, objets.
// Chaque dialogue construit un événement d'aventure et le confie à `envoyer` (en aventure : l'événement
// du journal ; en combat : action du combat).

import { useEffect, useState } from "react";
import { api } from "../../api";
import type { Duree, Effet, EvenementAventure, PjAventure, Protection } from "../../aventure-types";
import { ChoixPj, Dialogue, SaisieNombre } from "../../composants/jeu";
import {
  CIBLES_D_EFFET,
  Consommable,
  ev,
  instantDe,
  TYPES_DE_BONUS,
  TYPES_D_ENERGIE,
} from "../../evenements-jeu";

export type Envoyer = (e: EvenementAventure) => Promise<unknown>;
type Fermer = () => void;

const pl = (n: number | null) => n ?? 0;

// ------------------------------------------------------------------ santé

type ModeSante = "degats" | "soin" | "temp" | "continu";

export function DialogueSante({
  pj,
  mode: modeInitial,
  envoyer,
  onFermer,
}: {
  pj: PjAventure;
  mode: ModeSante;
  envoyer: Envoyer;
  onFermer: Fermer;
}) {
  const [mode, setMode] = useState<ModeSante>(modeInitial);
  const [nombre, setNombre] = useState<number | null>(null);
  const [letaux, setLetaux] = useState(true);
  const [type, setType] = useState("");
  const [contourne, setContourne] = useState(false);
  const titre = { degats: "Prendre des dégâts", soin: "Se soigner", temp: "PV temporaires", continu: "Dégâts continus" }[mode];

  const valider = async () => {
    if (mode === "continu") {
      await envoyer(ev.degatsContinus(pj.id, !pj.degatsContinus));
    } else if (nombre !== null) {
      if (mode === "degats") {
        await envoyer(type ? ev.degatsTypes(pj.id, nombre, type, contourne, letaux) : ev.degats(pj.id, nombre, letaux));
      } else if (mode === "soin") await envoyer(ev.soin(pj.id, nombre));
      else await envoyer(ev.pvTemp(pj.id, nombre));
    } else return;
    onFermer();
  };

  return (
    <Dialogue
      titre={`${titre} — ${pj.nom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button className="bouton principal" disabled={mode !== "continu" && !nombre && nombre !== 0} onClick={valider}>
            Valider
          </button>
        </>
      }
    >
      <div className="rangee-boutons" role="tablist">
        {(
          [
            ["degats", "Dégâts"],
            ["soin", "Soin"],
            ["temp", "PV temporaires"],
            ["continu", "Dégâts continus"],
          ] as [ModeSante, string][]
        ).map(([m, n]) => (
          <button key={m} role="tab" aria-selected={mode === m} className={"bouton mini" + (mode === m ? " principal" : "")} onClick={() => setMode(m)}>
            {n}
          </button>
        ))}
      </div>
      {mode === "continu" ? (
        <p>
          {pj.degatsContinus
            ? "Les dégâts continus sont en cours. Un soin d'au moins 1 PV les arrête ; vous pouvez aussi les arrêter ici."
            : "Marquer que le personnage subit des dégâts continus (saignement…) : un mourant ne peut alors pas se stabiliser avant d'avoir été soigné."}
        </p>
      ) : (
        <>
          <SaisieNombre
            etiquette={mode === "degats" ? "Dégâts subis" : mode === "soin" ? "Points de vie rendus" : "PV temporaires reçus"}
            rappel={
              mode === "degats"
                ? "Résultat de vos dés ; les PV temporaires absorbent d'abord."
                : mode === "soin"
                  ? "Un soin retire aussi autant de dégâts non létaux ; jamais au-delà des dégâts subis."
                  : "Les PV temporaires de plusieurs sources se cumulent."
            }
            valeur={nombre}
            onChange={setNombre}
            autoFocus
          />
          {mode === "degats" && (
            <div className="champs">
              <label className="champ-g">
                <span>Type de dégâts (immunités, résistances et RD s'appliquent)</span>
                <select className="champ-jeu" value={type} onChange={(e) => setType(e.target.value)}>
                  <option value="">Sans type (rien à retirer)</option>
                  {TYPES_D_ENERGIE.map((t) => (
                    <option key={t} value={t}>{t}</option>
                  ))}
                  <option value="tranchant">tranchant</option>
                  <option value="contondant">contondant</option>
                  <option value="perforant">perforant</option>
                  <option value="autre">autre dégât physique</option>
                </select>
              </label>
              <label className="interrupteur">
                <input type="checkbox" checked={letaux} onChange={(e) => setLetaux(e.target.checked)} />
                Dégâts létaux (sinon non létaux)
              </label>
              {type !== "" && !TYPES_D_ENERGIE.includes(type) && (
                <label className="interrupteur">
                  <input type="checkbox" checked={contourne} onChange={(e) => setContourne(e.target.checked)} />
                  L'attaque contourne la réduction des dégâts
                </label>
              )}
            </div>
          )}
        </>
      )}
    </Dialogue>
  );
}

// ------------------------------------------------------------------ effets

type LigneBonus = { cible: string; type: string; valeur: number | null };
type LigneProtection = { genre: "reduction" | "resistance" | "immunite"; type: string; valeur: number | null };
const UNITES: [string, string][] = [
  ["rounds", "rounds"],
  ["minutes", "minutes"],
  ["heures", "heures"],
  ["jours", "jours"],
];

/** Duree saisie : un nombre et une unité, ou une durée sans chiffre. */
export function dureeDe(genre: string, n: number | null): Duree | null {
  switch (genre) {
    case "rounds":
    case "minutes":
    case "heures":
    case "jours":
      return n && n > 0 ? ({ [genre]: n } as Duree) : null;
    case "permanente":
    case "concentration":
    case "jusqua_decharge":
      return genre;
    case "debut":
      return { jusqua_prochain_tour: "debut" };
    case "fin":
      return { jusqua_prochain_tour: "fin" };
    default:
      return null;
  }
}

export function ChoixDuree({
  genre,
  nombre,
  onGenre,
  onNombre,
  auto,
}: {
  genre: string;
  nombre: number | null;
  onGenre: (g: string) => void;
  onNombre: (n: number | null) => void;
  /** Propose « celle du livre » (valeur `auto`) : le moteur lit la durée du sort. */
  auto?: string;
}) {
  const chiffree = UNITES.some(([u]) => u === genre);
  return (
    <div className="champs">
      <label className="champ-g">
        <span>Durée</span>
        <select className="champ-jeu" value={genre} onChange={(e) => onGenre(e.target.value)}>
          {auto && <option value="auto">{auto}</option>}
          {UNITES.map(([u, n]) => (
            <option key={u} value={u}>{n}</option>
          ))}
          <option value="permanente">permanente (jusqu'à ce qu'on la retire)</option>
          <option value="concentration">concentration</option>
          <option value="jusqua_decharge">jusqu'à décharge</option>
          <option value="debut">jusqu'au début de son prochain tour</option>
          <option value="fin">jusqu'à la fin de son prochain tour</option>
        </select>
      </label>
      {chiffree && <SaisieNombre etiquette="Combien" min={1} max={100000} valeur={nombre} onChange={onNombre} />}
    </div>
  );
}

export function DialogueEffet({
  pjs,
  pjId,
  envoyer,
  onFermer,
}: {
  pjs: PjAventure[];
  pjId: number;
  envoyer: Envoyer;
  onFermer: Fermer;
}) {
  const [cible, setCible] = useState(pjId);
  const [nom, setNom] = useState("");
  const [source, setSource] = useState("");
  const [genre, setGenre] = useState("minutes");
  const [n, setN] = useState<number | null>(10);
  const [bonus, setBonus] = useState<LigneBonus[]>([]);
  const [prot, setProt] = useState<LigneProtection[]>([]);
  const [etats, setEtats] = useState<string[]>([]);
  const [soinParRound, setSoinParRound] = useState<number | null>(null);
  const [catalogue, setCatalogue] = useState<[string, string][]>([]);
  useEffect(() => {
    api.catalogueEtats().then(setCatalogue);
  }, []);

  const duree = dureeDe(genre, n);
  const valide = nom.trim() !== "" && duree !== null && bonus.every((b) => b.valeur !== null) && prot.every((p) => (p.genre === "immunite" ? p.type !== "" : p.valeur !== null && (p.genre === "reduction" || p.type !== "")));

  const valider = async () => {
    if (!duree) return;
    const effets: Effet[] = bonus.map((b) => ({ cible: b.cible, type_bonus: b.type, valeur: pl(b.valeur), source: nom.trim() }));
    const protections: Protection[] = prot.map((p) =>
      p.genre === "reduction"
        ? { Reduction: { valeur: pl(p.valeur) } }
        : p.genre === "resistance"
          ? { Resistance: { type_degats: p.type, valeur: pl(p.valeur) } }
          : { Immunite: { type_degats: p.type } },
    );
    await envoyer(
      ev.effet({
        pj: cible,
        nom: nom.trim(),
        source: source.trim() || "saisi à la main",
        duree,
        effets,
        etats,
        soinParRound: pl(soinParRound),
        protections,
      }),
    );
    onFermer();
  };

  return (
    <Dialogue
      large
      titre="Appliquer un effet"
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button className="bouton principal" disabled={!valide} onClick={valider}>Appliquer</button>
        </>
      }
    >
      {pjs.length > 1 && <ChoixPj liste={pjs} valeur={cible} onChange={setCible} />}
      <div className="champs">
        <label className="champ-g">
          <span>Nom de l'effet</span>
          <input className="champ-jeu" value={nom} onChange={(e) => setNom(e.target.value)} placeholder="Bénédiction, poison, potion…" autoFocus />
        </label>
        <label className="champ-g">
          <span>Source (sort, objet, créature…)</span>
          <input className="champ-jeu" value={source} onChange={(e) => setSource(e.target.value)} placeholder="sort lancé par Kyra" />
        </label>
      </div>
      <ChoixDuree genre={genre} nombre={n} onGenre={setGenre} onNombre={setN} />
      <div className="bloc">
        <h3>Bonus et malus (les bonus de même type ne se cumulent pas, sauf esquive, circonstance et sans type)</h3>
        {bonus.map((b, i) => (
          <div className="champs" key={i}>
            <label className="champ-g">
              <span>Ce qui change</span>
              <select className="champ-jeu" value={b.cible} onChange={(e) => setBonus(bonus.map((x, j) => (j === i ? { ...x, cible: e.target.value } : x)))}>
                {CIBLES_D_EFFET.map(([c, t]) => (
                  <option key={c} value={c}>{t}</option>
                ))}
              </select>
            </label>
            <label className="champ-g">
              <span>Type de bonus</span>
              <select className="champ-jeu" value={b.type} onChange={(e) => setBonus(bonus.map((x, j) => (j === i ? { ...x, type: e.target.value } : x)))}>
                {TYPES_DE_BONUS.map(([c, t]) => (
                  <option key={c} value={c}>{t}</option>
                ))}
              </select>
            </label>
            <SaisieNombre etiquette="Valeur (− pour un malus)" min={-99} max={99} valeur={b.valeur} onChange={(v) => setBonus(bonus.map((x, j) => (j === i ? { ...x, valeur: v } : x)))} />
            <button className="bouton discret danger" onClick={() => setBonus(bonus.filter((_, j) => j !== i))}>Retirer</button>
          </div>
        ))}
        <div className="rangee-boutons">
          <button className="bouton mini" onClick={() => setBonus([...bonus, { cible: "ca", type: "sans_type", valeur: 1 }])}>+ Un bonus ou un malus</button>
        </div>
      </div>
      <div className="bloc">
        <h3>Protections</h3>
        {prot.map((p, i) => (
          <div className="champs" key={i}>
            <label className="champ-g">
              <span>Genre</span>
              <select className="champ-jeu" value={p.genre} onChange={(e) => setProt(prot.map((x, j) => (j === i ? { ...x, genre: e.target.value as LigneProtection["genre"] } : x)))}>
                <option value="reduction">réduction des dégâts (RD)</option>
                <option value="resistance">résistance à une énergie</option>
                <option value="immunite">immunité à un type de dégâts</option>
              </select>
            </label>
            {p.genre !== "reduction" && (
              <label className="champ-g">
                <span>Type de dégâts</span>
                <select className="champ-jeu" value={p.type} onChange={(e) => setProt(prot.map((x, j) => (j === i ? { ...x, type: e.target.value } : x)))}>
                  <option value="">—</option>
                  {TYPES_D_ENERGIE.map((t) => (
                    <option key={t} value={t}>{t}</option>
                  ))}
                  {p.genre === "immunite" && <option value="poison">poison</option>}
                </select>
              </label>
            )}
            {p.genre !== "immunite" && (
              <SaisieNombre etiquette="Valeur" min={1} max={99} valeur={p.valeur} onChange={(v) => setProt(prot.map((x, j) => (j === i ? { ...x, valeur: v } : x)))} />
            )}
            <button className="bouton discret danger" onClick={() => setProt(prot.filter((_, j) => j !== i))}>Retirer</button>
          </div>
        ))}
        <div className="rangee-boutons">
          <button className="bouton mini" onClick={() => setProt([...prot, { genre: "resistance", type: "", valeur: 10 }])}>+ Une protection</button>
        </div>
      </div>
      <div className="champs">
        <label className="champ-g">
          <span>États donnés tant que l'effet dure</span>
          <select
            multiple
            size={4}
            className="champ-jeu"
            value={etats}
            onChange={(e) => setEtats(Array.from(e.target.selectedOptions).map((o) => o.value))}
          >
            {catalogue.map(([id, n]) => (
              <option key={id} value={id}>{n}</option>
            ))}
          </select>
        </label>
        <SaisieNombre etiquette="PV rendus au début de chaque tour (guérison accélérée)" min={0} max={99} valeur={soinParRound} onChange={setSoinParRound} />
      </div>
    </Dialogue>
  );
}

// ------------------------------------------------------------------ repos

export function DialogueRepos({
  pjs,
  envoyer,
  onFermer,
}: {
  pjs: PjAventure[];
  envoyer: Envoyer;
  onFermer: Fermer;
}) {
  const [choisis, setChoisis] = useState<number[]>(pjs.map((p) => p.id));
  const [heures, setHeures] = useState<number | null>(8);
  const [alitement, setAlitement] = useState(false);
  const [interrompu, setInterrompu] = useState(false);
  return (
    <Dialogue
      titre="Repos"
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button
            className="bouton principal"
            disabled={!heures || choisis.length === 0 || (alitement && heures < 24)}
            onClick={async () => {
              await envoyer(ev.repos(choisis, pl(heures), alitement, interrompu));
              onFermer();
            }}
          >
            Se reposer
          </button>
        </>
      }
    >
      <div className="bloc">
        <h3>Qui se repose</h3>
        <div className="rangee-boutons">
          {pjs.map((p) => (
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
      <SaisieNombre
        etiquette="Durée du repos (heures)"
        rappel="8 heures sans interruption : PV rendus (1 par niveau), caractéristiques (1 point), emplacements de sorts et usages quotidiens."
        min={1}
        max={240}
        valeur={heures}
        onChange={setHeures}
      />
      <label className="interrupteur">
        <input type="checkbox" checked={alitement} onChange={(e) => setAlitement(e.target.checked)} />
        Alitement complet pendant 24 h (le double de PV et de points de caractéristique)
      </label>
      <label className="interrupteur">
        <input type="checkbox" checked={interrompu} onChange={(e) => setInterrompu(e.target.checked)} />
        Le repos a été interrompu (le temps passe, mais rien n'est récupéré)
      </label>
    </Dialogue>
  );
}

// ------------------------------------------------------------------ heure

export function DialogueHeure({
  jour,
  heure,
  minute,
  envoyer,
  onFermer,
}: {
  jour: number;
  heure: number;
  minute: number;
  envoyer: Envoyer;
  onFermer: Fermer;
}) {
  const [j, setJ] = useState<number | null>(jour);
  const [h, setH] = useState<number | null>(heure);
  const [m, setM] = useState<number | null>(minute);
  const [recul, setRecul] = useState<Awaited<ReturnType<typeof api.aventureApercuRecul>>>(null);
  const [reactiver, setReactiver] = useState<number[]>([]);
  const [confirme, setConfirme] = useState(false);
  const valide = j !== null && h !== null && m !== null;
  const instant = valide ? instantDe(j!, h!, m!) : null;
  const actuel = instantDe(jour, heure, minute);
  const enArriere = instant !== null && instant < actuel;

  useEffect(() => {
    if (instant !== null && enArriere) api.aventureApercuRecul(instant).then(setRecul);
    else setRecul(null);
    setReactiver([]);
    setConfirme(false);
  }, [instant, enArriere]);

  const bloque = enArriere && (!recul || recul.avantMinuit || (recul.crees.length > 0 && !confirme));
  return (
    <Dialogue
      titre="Régler l'heure"
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button
            className="bouton principal"
            disabled={!valide || instant === actuel || bloque}
            onClick={async () => {
              await envoyer(ev.regleHeure(instant!, reactiver, confirme));
              onFermer();
            }}
          >
            {enArriere ? "Reculer l'horloge" : "Avancer l'horloge"}
          </button>
        </>
      }
    >
      <div className="champs">
        <SaisieNombre etiquette="Jour" min={1} max={99999} valeur={j} onChange={setJ} />
        <SaisieNombre etiquette="Heure" min={0} max={23} valeur={h} onChange={setH} />
        <SaisieNombre etiquette="Minute" min={0} max={59} valeur={m} onChange={setM} />
      </div>
      {!enArriere && instant !== null && instant > actuel && <p className="doux">Les effets arrivés à échéance dans l'intervalle prendront fin.</p>}
      {enArriere && recul?.avantMinuit && (
        <div className="avertissement">
          Reculer avant minuit n'est pas possible ici : utilisez « Annuler », qui défait aussi le jour d'intermède.
        </div>
      )}
      {enArriere && recul && !recul.avantMinuit && (
        <div className="bloc">
          <h3>Un recul ne ressuscite pas un effet expiré sans votre accord</h3>
          {recul.reactivables.length === 0 && <p className="doux vide">Aucun effet expiré dans cet intervalle.</p>}
          {recul.reactivables.map(([id, nomEffet, qui]) => (
            <label key={id} className="interrupteur">
              <input type="checkbox" checked={reactiver.includes(id)} onChange={(e) => setReactiver(e.target.checked ? [...reactiver, id] : reactiver.filter((x) => x !== id))} />
              Réactiver « {nomEffet} » ({qui})
            </label>
          ))}
          {recul.crees.length > 0 && (
            <div className="avertissement">
              Ces effets ont été créés après l'heure visée et seront retirés :{" "}
              {recul.crees.map(([, n, q]) => `« ${n} » (${q})`).join(", ")}.
              <label className="interrupteur">
                <input type="checkbox" checked={confirme} onChange={(e) => setConfirme(e.target.checked)} />
                Je confirme le retrait
              </label>
            </div>
          )}
        </div>
      )}
    </Dialogue>
  );
}

// ------------------------------------------------------------------ objets

export function DialogueObjet({
  pjs,
  pjId,
  envoyer,
  onFermer,
}: {
  pjs: PjAventure[];
  pjId: number;
  envoyer: Envoyer;
  onFermer: Fermer;
}) {
  const [cible, setCible] = useState(pjId);
  const [genre, setGenre] = useState<Consommable["genre"]>("potion");
  const [nom, setNom] = useState("");
  const [sort, setSort] = useState("");
  const [niveauSort, setNiveauSort] = useState<number | null>(1);
  const [cl, setCl] = useState<number | null>(1);
  const [arcanique, setArcanique] = useState(true);
  const [quantite, setQuantite] = useState<number | null>(1);
  const [charges, setCharges] = useState<number | null>(50);
  const [parJour, setParJour] = useState<number | null>(null);
  const avecSort = ["potion", "parchemin", "baguette", "baton"].includes(genre);
  const aCharges = ["baguette", "baton", "charges"].includes(genre);
  const valide =
    nom.trim() !== "" &&
    (!avecSort || (sort.trim() !== "" && niveauSort !== null && cl !== null)) &&
    (quantite ?? 0) > 0 &&
    (!aCharges || (charges ?? 0) > 0 || parJour !== null);

  return (
    <Dialogue
      large
      titre="Ajouter un objet"
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button
            className="bouton principal"
            disabled={!valide}
            onClick={async () => {
              const objet: Consommable = {
                nom: nom.trim(),
                genre,
                sort: avecSort ? sort.trim() : null,
                niveau_du_sort: avecSort ? pl(niveauSort) : 0,
                niveau_de_lanceur: avecSort ? pl(cl) : 0,
                arcanique,
                quantite: genre === "baguette" || genre === "baton" ? 1 : pl(quantite),
                charges: aCharges && charges ? { restantes: charges, max: charges } : null,
                par_jour: parJour ? { restantes: parJour, max: parJour } : null,
                notes: "",
              };
              await envoyer(ev.objetAjoute(cible, objet));
              onFermer();
            }}
          >
            Ajouter
          </button>
        </>
      }
    >
      {pjs.length > 1 && <ChoixPj liste={pjs} valeur={cible} onChange={setCible} />}
      <div className="champs">
        <label className="champ-g">
          <span>Genre</span>
          <select className="champ-jeu" value={genre} onChange={(e) => setGenre(e.target.value as Consommable["genre"])}>
            <option value="potion">Potion</option>
            <option value="parchemin">Parchemin</option>
            <option value="baguette">Baguette</option>
            <option value="baton">Bâton</option>
            <option value="charges">Objet à charges ou à usages quotidiens</option>
            <option value="munition">Munitions</option>
            <option value="autre">Autre objet</option>
          </select>
        </label>
        <label className="champ-g">
          <span>Nom</span>
          <input className="champ-jeu" value={nom} onChange={(e) => setNom(e.target.value)} placeholder="Potion de soins légers" autoFocus />
        </label>
      </div>
      {avecSort && (
        <div className="champs">
          <label className="champ-g">
            <span>Sort contenu (nom anglais du catalogue : Cure Light Wounds…)</span>
            <input className="champ-jeu" value={sort} onChange={(e) => setSort(e.target.value)} />
          </label>
          <SaisieNombre etiquette="Niveau du sort" min={0} max={9} valeur={niveauSort} onChange={setNiveauSort} rappel={genre === "potion" ? "3 au plus (potion)" : genre === "baguette" ? "4 au plus (baguette)" : undefined} />
          <SaisieNombre etiquette="Niveau de lanceur de l'objet" min={1} max={20} valeur={cl} onChange={setCl} rappel={genre === "baton" ? "8 au moins (bâton)" : undefined} />
          {genre === "parchemin" && (
            <label className="interrupteur">
              <input type="checkbox" checked={arcanique} onChange={(e) => setArcanique(e.target.checked)} />
              Parchemin profane (sinon divin)
            </label>
          )}
        </div>
      )}
      <div className="champs">
        {genre !== "baguette" && genre !== "baton" && <SaisieNombre etiquette="Quantité" min={1} max={999} valeur={quantite} onChange={setQuantite} />}
        {aCharges && (
          <SaisieNombre
            etiquette="Charges"
            min={0}
            max={genre === "baton" ? 10 : 50}
            valeur={charges}
            onChange={setCharges}
            rappel={genre === "baton" ? "10 au plus" : genre === "baguette" ? "50 au plus" : undefined}
          />
        )}
        {genre === "charges" && <SaisieNombre etiquette="Usages par jour (rendus au repos)" min={0} max={99} valeur={parJour} onChange={setParJour} />}
      </div>
    </Dialogue>
  );
}
