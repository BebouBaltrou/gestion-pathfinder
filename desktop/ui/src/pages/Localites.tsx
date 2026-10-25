import { useEffect, useState } from "react";
import type { Outils } from "../App";
import {
  api,
  Capitaux,
  ChoixCategorie,
  FicheLocalite,
  formaterPo,
  PageLocalites,
  RessourceTransfert,
  SaisieLocalite,
  Somme,
} from "../api";
import { icones } from "../icones";
import { BlocCapitaux } from "./Tableau";

const SAISIE_VIDE: SaisieLocalite = {
  nom: "",
  genre: "",
  population: "",
  officielle: true,
  categorieOfficielle: "",
  categoriePersoNom: "",
  categoriePersoLimite: "",
  notes: "",
};

const CAPITAUX_VIDES: Capitaux = { marchandises: 0, influence: 0, travail: 0, magie: 0 };

const TYPES_CAPITAUX: [keyof Capitaux, string, string][] = [
  ["marchandises", "Marchandises", "var(--marchandises)"],
  ["influence", "Influence", "var(--influence)"],
  ["travail", "Travail", "var(--travail)"],
  ["magie", "Magie", "var(--magie)"],
];

export function Localites({ outils }: { outils: Outils }) {
  const [p, setP] = useState<PageLocalites | null>(null);
  const [selection, setSelection] = useState<number | null>(null);
  const [creation, setCreation] = useState(false);
  const [modeleChoisi, setModeleChoisi] = useState("");

  useEffect(() => {
    api.pageLocalites().then((x) => {
      setP(x);
      if (x && x.localites.length === 0) setCreation(true);
    });
  }, [outils.version]);

  if (!p) return null;
  const modele = p.modeles.find((m) => m.id === modeleChoisi);
  const choisie = p.localites.find((l) => l.id === selection) ?? (creation ? undefined : p.localites[0]);

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Localités</h1>
        <p>Villes, villages et lieux-dits où vos personnages possèdent des biens</p>
      </div>

      <div className="maitre-detail">
        <aside className="liste-maitre" aria-label="Liste des localités">
          <button
            className={"bouton nouvelle" + (creation ? " actif" : "")}
            onClick={() => {
              setCreation(true);
              setSelection(null);
            }}
          >
            {icones.plus({})} Nouvelle localité
          </button>
          {p.localites.map((l) => (
            <button
              key={l.id}
              className={"element-liste" + (!creation && choisie?.id === l.id ? " actif" : "")}
              aria-current={!creation && choisie?.id === l.id ? "true" : undefined}
              onClick={() => {
                setSelection(l.id);
                setCreation(false);
              }}
            >
              <span className="nom">{l.nom}</span>
              <span className={"montant" + (l.tresorerie.pa < 0 ? " negatif" : "")}>{formaterPo(l.tresorerie.pa)}</span>
              <span className="doux">{l.genre || l.categorie.split(" · ")[0]}</span>
            </button>
          ))}
        </aside>

        <div className="detail">
          {creation || !choisie ? (
            <section className="carte section">
              <h2>Nouvelle localité</h2>
              <div className="formulaire">
                <label htmlFor="loc-modele">Localité prête à l'emploi</label>
                <select
                  id="loc-modele"
                  className="champ"
                  value={modeleChoisi}
                  onChange={(e) => setModeleChoisi(e.target.value)}
                >
                  <option value="">Aucune (saisie libre)</option>
                  {p.modeles.map((m) => (
                    <option key={m.id} value={m.id}>
                      {m.libelle}
                      {m.officiel ? "" : " (à confirmer)"}
                    </option>
                  ))}
                </select>
                {modele && (
                  <p className="doux">
                    Source : {modele.source}.{" "}
                    {modele.remarque ? <strong>À confirmer : {modele.remarque}</strong> : "Valeurs du wiki."}
                  </p>
                )}
              </div>
              <Formulaire
                key={"creation-" + modeleChoisi}
                initiale={modele?.saisie ?? SAISIE_VIDE}
                categories={p.categories}
                bouton="Créer la localité"
                principal
                valider={async (s) => {
                  const avant = new Set(p.localites.map((l) => l.id));
                  const r = await outils.agir(api.creerLocalite(s));
                  if (r?.ok) {
                    const apres = await api.pageLocalites();
                    const nouvelle = apres?.localites.find((l) => !avant.has(l.id));
                    setCreation(false);
                    setModeleChoisi("");
                    setSelection(nouvelle?.id ?? null);
                  }
                  return !!r?.ok;
                }}
              />
            </section>
          ) : (
            <Detail key={choisie.id} l={choisie} p={p} outils={outils} />
          )}
        </div>
      </div>
    </div>
  );
}

// ------------------------------------------------------------------ fiche

function Detail({ l, p, outils }: { l: FicheLocalite; p: PageLocalites; outils: Outils }) {
  return (
    <>
      <section className="carte fiche-localite">
        <div className="tete">
          <h2 className="nom">{l.nom}</h2>
          {l.genre && <span className="doux">{l.genre}</span>}
          {l.population !== null && <span className="doux">· {l.population.toLocaleString("fr-FR")} hab.</span>}
        </div>
        <div className="ligne" style={{ gap: 10 }}>
          <span className="doux">{l.categorie}</span>
          {l.personnalisee && <span className="badge perso">Personnalisé</span>}
        </div>
        <div className="ligne">
          <div className="tresor">
            <span className="etiquette">Trésorerie</span>
            <span className={"montant" + (l.tresorerie.pa < 0 ? " negatif" : "")}>{formaterPo(l.tresorerie.pa)}</span>
          </div>
          <BlocCapitaux c={l.capitaux} />
        </div>
        {l.transferts.length > 0 && (
          <div className="puces">
            {l.transferts.map((t) => (
              <span className="puce" key={t}>En route : {t}</span>
            ))}
          </div>
        )}
        {l.notes && <p className="doux notes">{l.notes}</p>}
      </section>

      <Operations l={l} p={p} outils={outils} />

      <details className="carte repli">
        <summary>Modifier la localité</summary>
        <Formulaire
          initiale={l.saisie}
          categories={p.categories}
          bouton="Enregistrer les modifications"
          valider={async (s) => !!(await outils.agir(api.modifierLocalite(l.id, s)))?.ok}
        />
      </details>

      <details className="carte repli">
        <summary>Supprimer la localité</summary>
        <div className="ligne operation">
          <button
            className="bouton danger"
            onClick={async () => {
              if (window.confirm(await api.avertissementSuppressionLocalite(l.id))) outils.agir(api.supprimerLocalite(l.id));
            }}
          >
            Supprimer {l.nom}…
          </button>
        </div>
      </details>
    </>
  );
}

// ------------------------------------------------------------------ formulaire

function Formulaire(props: {
  initiale: SaisieLocalite;
  categories: ChoixCategorie[];
  bouton: string;
  principal?: boolean;
  valider: (s: SaisieLocalite) => Promise<boolean>;
}) {
  const [s, setS] = useState<SaisieLocalite>(props.initiale);
  const [suggestion, setSuggestion] = useState<ChoixCategorie | null>(null);
  const maj = (champs: Partial<SaisieLocalite>) => setS((x) => ({ ...x, ...champs }));

  useEffect(() => {
    let actuel = true;
    api.categorieSuggeree(s.population).then((c) => actuel && setSuggestion(c));
    return () => {
      actuel = false;
    };
  }, [s.population]);

  const categorie = props.categories.find((c) => c.id === s.categorieOfficielle);
  const complet =
    s.nom.trim() !== "" && (s.officielle ? !!categorie : /^\d+$/.test(s.categoriePersoLimite.trim()));

  return (
    <form
      className="formulaire"
      onSubmit={async (e) => {
        e.preventDefault();
        if (await props.valider(s)) {
          if (props.principal) setS(SAISIE_VIDE);
        }
      }}
    >
      <label htmlFor="loc-nom">Nom</label>
      <input id="loc-nom" className="champ" value={s.nom} onChange={(e) => maj({ nom: e.target.value })} />

      <label htmlFor="loc-genre">Genre</label>
      <div className="ligne" style={{ gap: 8 }}>
        <input
          id="loc-genre"
          className="champ moyen"
          value={s.genre}
          placeholder="ville, village, lieu-dit…"
          onChange={(e) => maj({ genre: e.target.value })}
        />
        {["ville", "village", "lieu-dit"].map((g) => (
          <button type="button" className="bouton discret" key={g} onClick={() => maj({ genre: g })}>
            {g}
          </button>
        ))}
      </div>

      <label htmlFor="loc-pop">Population</label>
      <div className="ligne" style={{ gap: 10 }}>
        <input
          id="loc-pop"
          className="champ petit"
          inputMode="numeric"
          value={s.population}
          placeholder="facultatif"
          onChange={(e) => maj({ population: e.target.value })}
        />
        {suggestion && (
          <span className="doux">
            Catégorie officielle : {suggestion.nom}
            {s.officielle && s.categorieOfficielle !== suggestion.id && (
              <button type="button" className="bouton discret" onClick={() => maj({ categorieOfficielle: suggestion.id })}>
                Appliquer
              </button>
            )}
          </span>
        )}
      </div>

      <span className="etiquette-grille">Catégorie</span>
      <div className="pile">
        <div className="segments" role="radiogroup" aria-label="Type de catégorie">
          <button type="button" role="radio" aria-checked={s.officielle} className={s.officielle ? "actif" : ""} onClick={() => maj({ officielle: true })}>
            Officielle
          </button>
          <button type="button" role="radio" aria-checked={!s.officielle} className={!s.officielle ? "actif" : ""} onClick={() => maj({ officielle: false })}>
            Personnalisée
          </button>
        </div>
        {s.officielle ? (
          <>
            <select
              className="champ"
              aria-label="Catégorie officielle"
              value={s.categorieOfficielle}
              onChange={(e) => maj({ categorieOfficielle: e.target.value })}
            >
              <option value="">Choisir…</option>
              {props.categories.map((c) => (
                <option key={c.id} value={c.id}>{c.libelle}</option>
              ))}
            </select>
            {categorie ? (
              <span className="doux">
                Population : {categorie.sourcePopulation}. Limite de dépense : {categorie.sourceLimite}.
                {categorie.pointAConfirmer && <span className="badge confirmer">À confirmer : {categorie.pointAConfirmer}</span>}
              </span>
            ) : (
              <span className="doux">La catégorie fixe la limite de dépense par jour.</span>
            )}
          </>
        ) : (
          <>
            <div className="ligne" style={{ gap: 10 }}>
              <input
                className="champ moyen"
                aria-label="Nom de la catégorie"
                placeholder="nom (ex. Avant-poste)"
                value={s.categoriePersoNom}
                onChange={(e) => maj({ categoriePersoNom: e.target.value })}
              />
              <input
                className="champ petit"
                aria-label="Limite de dépense par jour"
                inputMode="numeric"
                placeholder="limite/jour"
                value={s.categoriePersoLimite}
                onChange={(e) => maj({ categoriePersoLimite: e.target.value })}
              />
              <span className="doux">unités de capital par jour</span>
            </div>
            <span><span className="badge perso">Personnalisé</span></span>
          </>
        )}
      </div>

      <label htmlFor="loc-notes">Notes</label>
      <textarea id="loc-notes" className="champ zone" rows={2} value={s.notes} onChange={(e) => maj({ notes: e.target.value })} />

      <span />
      <div>
        <button type="submit" className={"bouton" + (props.principal ? " principal" : "")} disabled={!complet}>
          {props.bouton}
        </button>
      </div>
    </form>
  );
}

// ------------------------------------------------------------------ opérations

type Action = "injection" | "retrait" | "transfert" | "ajustement";

const ACTIONS: [Action, string][] = [
  ["injection", "Injection depuis un PJ"],
  ["retrait", "Retrait vers un PJ"],
  ["transfert", "Transfert"],
  ["ajustement", "Ajustement"],
];

/** Champ de montant en po, avec sa lecture par le cœur (« = 12 po 5 pa »). */
export function ChampMontant(props: { id: string; valeur: string; changer: (v: string) => void; signe?: boolean }) {
  const [lu, setLu] = useState<Somme | null>(null);
  useEffect(() => {
    let actuel = true;
    if (props.valeur.trim() === "") setLu(null);
    else api.lireMontant(props.valeur).then((m) => actuel && setLu(m));
    return () => {
      actuel = false;
    };
  }, [props.valeur]);
  const illisible = props.valeur.trim() !== "" && lu === null;
  return (
    <span className="champ-montant">
      <input
        id={props.id}
        className={"champ petit" + (illisible ? " invalide" : "")}
        value={props.valeur}
        placeholder={props.signe ? "ex. -12 ou 4 po" : "ex. 12 ou 4 po 7 pa"}
        title="Saisie en po : « 12 », « 12,5 » (= 12 po 5 pa), « 4 po 7 pa » ou « 7 pa »"
        onChange={(e) => props.changer(e.target.value)}
      />
      <span className={"lecture" + (illisible ? " invalide" : "")}>
        {illisible ? "montant illisible" : lu ? `= ${formaterPo(lu.pa)}` : ""}
      </span>
    </span>
  );
}

function Operations({ l, p, outils }: { l: FicheLocalite; p: PageLocalites; outils: Outils }) {
  const [action, setAction] = useState<Action>("injection");
  const [pj, setPj] = useState<number | "">(p.personnages[0]?.id ?? "");
  const [montant, setMontant] = useState("");
  const autres = p.localites.filter((x) => x.id !== l.id);
  const [vers, setVers] = useState<number | "">(autres[0]?.id ?? "");
  const [ressource, setRessource] = useState<RessourceTransfert>("po");
  const [quantite, setQuantite] = useState("");
  const [duree, setDuree] = useState("0");
  const [ajout, setAjout] = useState<Capitaux>(CAPITAUX_VIDES);
  const [retrait, setRetrait] = useState<Capitaux>(CAPITAUX_VIDES);
  const [motif, setMotif] = useState("");

  const vider = () => {
    setMontant("");
    setQuantite("");
    setMotif("");
    setAjout(CAPITAUX_VIDES);
    setRetrait(CAPITAUX_VIDES);
  };
  const lancer = async (p: Promise<import("../api").Retour>) => {
    const r = await outils.agir(p);
    if (r?.ok) vider();
  };

  const nombre = (v: string) => Math.max(0, Math.trunc(Number(v.replace(/\D/g, "")) || 0));
  const ajustementUtile =
    montant.trim() !== "" || Object.values(ajout).some((n) => n > 0) || Object.values(retrait).some((n) => n > 0);

  return (
    <section className="carte section">
      <h2>Opérations</h2>
      <div className="segments" role="tablist" aria-label="Opération">
        {ACTIONS.map(([a, nom]) => (
          <button key={a} role="tab" aria-selected={action === a} className={action === a ? "actif" : ""} onClick={() => setAction(a)}>
            {nom}
          </button>
        ))}
      </div>

      {(action === "injection" || action === "retrait") && (
        <form
          className="ligne operation"
          onSubmit={(e) => {
            e.preventDefault();
            if (pj !== "") lancer(api.mouvementPo(action === "injection", pj, l.id, montant));
          }}
        >
          {p.personnages.length === 0 ? (
            <p className="doux">Créez d'abord un personnage (page Personnages).</p>
          ) : (
            <>
              <div>
                <label className="etiquette-champ" htmlFor="op-pj">Personnage</label>
                <select id="op-pj" className="champ moyen" value={pj} onChange={(e) => setPj(Number(e.target.value))}>
                  {p.personnages.map((x) => (
                    <option key={x.id} value={x.id}>{x.libelle}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="etiquette-champ" htmlFor="op-montant">Montant</label>
                <ChampMontant id="op-montant" valeur={montant} changer={setMontant} />
              </div>
              <button type="submit" className="bouton principal" disabled={pj === "" || montant.trim() === ""}>
                {action === "injection" ? "Injecter" : "Retirer"}
              </button>
            </>
          )}
        </form>
      )}

      {action === "transfert" && (
        <form
          className="pile"
          onSubmit={(e) => {
            e.preventDefault();
            if (vers !== "") lancer(api.transfert(l.id, vers, ressource, quantite, duree));
          }}
        >
          {autres.length === 0 ? (
            <p className="doux">Il faut une autre localité pour envoyer un transfert.</p>
          ) : (
            <div className="ligne operation">
              <div>
                <label className="etiquette-champ" htmlFor="op-vers">Vers</label>
                <select id="op-vers" className="champ moyen" value={vers} onChange={(e) => setVers(Number(e.target.value))}>
                  {autres.map((x) => (
                    <option key={x.id} value={x.id}>{x.nom}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="etiquette-champ" htmlFor="op-quoi">Quoi</label>
                <select
                  id="op-quoi"
                  className="champ petit"
                  value={ressource}
                  onChange={(e) => {
                    setRessource(e.target.value as RessourceTransfert);
                    setQuantite("");
                  }}
                >
                  <option value="po">po</option>
                  <option value="marchandises">Marchandises</option>
                  <option value="travail">Travail</option>
                  <option value="magie">Magie</option>
                </select>
              </div>
              <div>
                <label className="etiquette-champ" htmlFor="op-quantite">{ressource === "po" ? "Montant" : "Quantité"}</label>
                {ressource === "po" ? (
                  <ChampMontant id="op-quantite" valeur={quantite} changer={setQuantite} />
                ) : (
                  <input id="op-quantite" className="champ petit" inputMode="numeric" placeholder="unités" value={quantite} onChange={(e) => setQuantite(e.target.value)} />
                )}
              </div>
              <div>
                <label className="etiquette-champ" htmlFor="op-duree">Durée (jours)</label>
                <input id="op-duree" className="champ mini" inputMode="numeric" value={duree} onChange={(e) => setDuree(e.target.value)} title="0 = arrivée immédiate" />
              </div>
              <button type="submit" className="bouton principal" disabled={vers === "" || quantite.trim() === "" || duree.trim() === ""}>
                Envoyer
              </button>
            </div>
          )}
          <p className="doux">L'Influence est attachée à sa communauté et ne se transfère pas (Guide de campagne p. 81).</p>
        </form>
      )}

      {action === "ajustement" && (
        <form
          className="pile"
          onSubmit={(e) => {
            e.preventDefault();
            lancer(api.ajustement(l.id, montant, ajout, retrait, motif));
          }}
        >
          <div>
            <label className="etiquette-champ" htmlFor="op-delta">po (+ ou −)</label>
            <ChampMontant id="op-delta" valeur={montant} changer={setMontant} signe />
          </div>
          <div className="grille-ajustement" role="group" aria-label="Capitaux à ajouter ou retirer">
            <span />
            {TYPES_CAPITAUX.map(([k, nom, couleur]) => (
              <span key={k} className="entete-capital" style={{ color: couleur }}>{nom}</span>
            ))}
            {(
              [
                ["Ajouter", ajout, setAjout],
                ["Retirer", retrait, setRetrait],
              ] as const
            ).map(([nom, valeurs, changer]) => (
              <Rangee key={nom} nom={nom} valeurs={valeurs} changer={(k, v) => changer({ ...valeurs, [k]: nombre(v) })} />
            ))}
          </div>
          <div className="ligne operation">
            <div style={{ flex: 1 }}>
              <label className="etiquette-champ" htmlFor="op-motif">Motif</label>
              <input id="op-motif" className="champ large" placeholder="récompense du MJ, correction…" value={motif} onChange={(e) => setMotif(e.target.value)} />
            </div>
            <button type="submit" className="bouton principal" disabled={!ajustementUtile || motif.trim() === ""}>
              Appliquer
            </button>
          </div>
        </form>
      )}
    </section>
  );
}

function Rangee(props: { nom: string; valeurs: Capitaux; changer: (k: keyof Capitaux, v: string) => void }) {
  return (
    <>
      <span className="doux">{props.nom}</span>
      {TYPES_CAPITAUX.map(([k, nom]) => (
        <input
          key={k}
          className="champ mini"
          inputMode="numeric"
          aria-label={`${props.nom} : ${nom}`}
          value={props.valeurs[k] === 0 ? "" : String(props.valeurs[k])}
          placeholder="0"
          onChange={(e) => props.changer(k, e.target.value)}
        />
      ))}
    </>
  );
}
