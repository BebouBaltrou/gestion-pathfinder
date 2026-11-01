import { useEffect, useState } from "react";
import type { Outils } from "../App";
import {
  Acquisition,
  ActionBatiment,
  api,
  ApercuCreation,
  Capitaux,
  Choix,
  Cible,
  EtatVente,
  FicheBatiment,
  FicheEtage,
  FicheSalle,
  formaterPo,
  PageBatiments,
  RessourceId,
  SaisieBatiment,
} from "../api";
import { icones } from "../icones";
import { texteCapitaux } from "../texte";
import { ChampMontant } from "./Localites";
import { BlocCapitaux } from "./Tableau";

export const COULEUR_RESSOURCE: Record<RessourceId, string> = {
  po: "var(--po)",
  marchandises: "var(--marchandises)",
  influence: "var(--influence)",
  travail: "var(--travail)",
  magie: "var(--magie)",
};
export const NOM_RESSOURCE: Record<RessourceId, string> = {
  po: "po",
  marchandises: "Marchandises",
  influence: "Influence",
  travail: "Travail",
  magie: "Magie",
};

const SAISIE_VIDE: SaisieBatiment = {
  localite: null,
  modele: null,
  nom: "",
  classement: "",
  acquisition: "plan",
  facteur: 1,
  seulementManquants: false,
  prix: "",
  payeur: null,
};

const ACQUISITIONS: [Acquisition, string][] = [
  ["plan", "Seulement le plan (salles à construire, rien n'est lancé)"],
  ["possede", "Déjà possédé (gratuit, déjà construit)"],
  ["construire", "Construire avec les capitaux de la localité"],
  ["acheterCapitaux", "Acheter les capitaux en po, puis construire"],
  ["acheterBatiment", "Acheter le bâtiment terminé (prix en po, sans durée)"],
];


/** Accélération ×1 à ×4 (GC p. 95, arrondi au jour supérieur, PC-10). */
export function ChoixFacteur(props: { id: string; valeur: number; changer: (f: number) => void; capital?: string }) {
  const cap = props.capital ?? "Travail";
  return (
    <select
      id={props.id}
      className="champ moyen"
      value={props.valeur}
      title="Guide de campagne p. 95. Arrondi au jour supérieur (PC-10)."
      onChange={(e) => props.changer(Number(e.target.value))}
    >
      <option value={1}>aucune</option>
      {[2, 3, 4].map((k) => (
        <option key={k} value={k}>×{k} {cap}, durée ÷{k}</option>
      ))}
    </select>
  );
}

export function ChoixPayeur(props: { id: string; valeur: number | null; changer: (p: number | null) => void; tresorerie: string; payeurs: Choix[] }) {
  return (
    <select
      id={props.id}
      className="champ"
      style={{ maxWidth: 320 }}
      value={props.valeur ?? ""}
      onChange={(e) => props.changer(e.target.value === "" ? null : Number(e.target.value))}
    >
      <option value="">{props.tresorerie}</option>
      {props.payeurs.map((p) => (
        <option key={p.id} value={p.id}>{p.libelle}</option>
      ))}
    </select>
  );
}

export function Batiments({ outils }: { outils: Outils }) {
  const [p, setP] = useState<PageBatiments | null>(null);
  const [selection, setSelection] = useState<number | null>(null);
  const [creation, setCreation] = useState(false);

  useEffect(() => {
    api.pageBatiments().then(setP);
  }, [outils.version]);

  if (!p) return null;
  const tous = p.groupes.flatMap((g) => g.batiments);
  const choisi = tous.find((b) => b.id === selection) ?? (creation ? undefined : tous[0]);
  const peutCreer = p.localites.length > 0;

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Bâtiments</h1>
        <p>Étages et salles, construction (Guide de campagne p. 85 et 92-95), recettes de chaque salle</p>
      </div>

      <div className="maitre-detail">
        <aside className="liste-maitre" aria-label="Liste des bâtiments">
          <button
            className={"bouton nouvelle" + (creation ? " actif" : "")}
            disabled={!peutCreer}
            onClick={() => {
              setCreation(true);
              setSelection(null);
            }}
          >
            {icones.plus({})} Nouveau bâtiment
          </button>
          {!peutCreer && <p className="doux">Créez d'abord une localité.</p>}
          {p.groupes.map((g) => (
            <div className="groupe-liste" key={g.localite}>
              <span className="titre-groupe">{g.localite}</span>
              {g.batiments.map((b) => {
                const actif = !creation && choisi?.id === b.id;
                return (
                  <button
                    key={b.id}
                    className={"element-liste" + (actif ? " actif" : "")}
                    aria-current={actif ? "true" : undefined}
                    onClick={() => {
                      setSelection(b.id);
                      setCreation(false);
                    }}
                  >
                    <span className="nom">{b.nom}</span>
                    {b.enTravaux > 0 && <span className="puce or">{b.enTravaux} en travaux</span>}
                  </button>
                );
              })}
            </div>
          ))}
        </aside>

        <div className="detail">
          {creation ? (
            <Creation
              p={p}
              outils={outils}
              apres={async (avant) => {
                const apres = await api.pageBatiments();
                setCreation(false);
                setSelection(apres?.groupes.flatMap((g) => g.batiments).find((b) => !avant.has(b.id))?.id ?? null);
              }}
              existants={new Set(tous.map((b) => b.id))}
            />
          ) : choisi ? (
            <Detail key={choisi.id} id={choisi.id} p={p} outils={outils} />
          ) : (
            <div className="carte vide">
              <p>Aucun bâtiment pour l'instant. Créez-en un pour y construire des salles.</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// ------------------------------------------------------------------ création

function Creation(props: { p: PageBatiments; outils: Outils; existants: Set<number>; apres: (avant: Set<number>) => void }) {
  const { p } = props;
  const [s, setS] = useState<SaisieBatiment>({ ...SAISIE_VIDE, localite: p.localites[0]?.id ?? null });
  const [ap, setAp] = useState<ApercuCreation | null>(null);
  const maj = (c: Partial<SaisieBatiment>) => setS((x) => ({ ...x, ...c }));

  useEffect(() => {
    let actuel = true;
    if (!s.modele) setAp(null);
    else api.apercuCreationBatiment(s).then((x) => actuel && setAp(x));
    return () => {
      actuel = false;
    };
  }, [s.localite, s.modele, s.acquisition, s.facteur, s.seulementManquants]);

  const modele = p.modeles.find((m) => m.id === s.modele);
  const avecFacteur = s.acquisition === "construire" || s.acquisition === "acheterCapitaux";
  const tresorerie = ap?.tresorerie ?? "Trésorerie de la localité";

  return (
    <section className="carte section">
      <h2>Nouveau bâtiment</h2>
      <form
        className="formulaire"
        onSubmit={async (e) => {
          e.preventDefault();
          const r = await props.outils.agir(api.creerBatiment(s));
          if (r?.ok) props.apres(props.existants);
        }}
      >
        <label htmlFor="bat-loc">Localité</label>
        <select id="bat-loc" className="champ moyen" value={s.localite ?? ""} onChange={(e) => maj({ localite: Number(e.target.value), payeur: null })}>
          {p.localites.map((l) => (
            <option key={l.id} value={l.id}>{l.libelle}</option>
          ))}
        </select>

        <label htmlFor="bat-modele">Modèle</label>
        <div className="pile">
          <select id="bat-modele" className="champ" value={s.modele ?? ""} onChange={(e) => maj({ modele: e.target.value || null })}>
            <option value="">aucun (bâtiment vide)</option>
            {p.modeles.map((m) => (
              <option key={m.id} value={m.id}>{m.libelle}</option>
            ))}
          </select>
          {modele?.personnel && (
            <span className="ligne" style={{ gap: 10 }}>
              <span className="badge perso" style={{ marginLeft: 0 }}>Personnalisé</span>
              <span className="doux">Modèle personnel : étages et salles comme enregistrés.</span>
              <button
                type="button"
                className="bouton discret"
                onClick={async () => {
                  const r = await props.outils.agir(api.supprimerModeleBatiment(modele.id));
                  if (r?.ok) maj({ modele: null });
                }}
              >
                Supprimer ce modèle
              </button>
            </span>
          )}
        </div>

        <label htmlFor="bat-nom">Nom</label>
        <input id="bat-nom" className="champ" placeholder="ex. Le Dragon vert" value={s.nom} onChange={(e) => maj({ nom: e.target.value })} />
        <label htmlFor="bat-classement">Classement</label>
        <input id="bat-classement" className="champ" placeholder="auberge, temple, entrepôt…" value={s.classement} onChange={(e) => maj({ classement: e.target.value })} />

        {ap && (
          <>
            <span className="etiquette-grille">Salles</span>
            <span className="doux">{ap.composition} <em>({ap.source})</em></span>
            <span className="etiquette-grille">Acquisition</span>
            <div className="pile" role="radiogroup" aria-label="Acquisition">
              {ACQUISITIONS.map(([a, nom]) => (
                <label key={a} className="choix-radio">
                  <input type="radio" name="acq" checked={s.acquisition === a} onChange={() => maj({ acquisition: a })} /> {nom}
                </label>
              ))}
              {s.acquisition === "plan" && (
                <span className="doux">Les salles sont posées « à construire » au rez-de-chaussée ; lancez-les ensuite salle par salle.</span>
              )}
              {s.acquisition === "possede" && <span className="doux">Toutes les salles sont créées construites, sans rien payer.</span>}
              {avecFacteur && (
                <>
                  <span className="ligne" style={{ gap: 10 }}>
                    <span className="doux">Accélération</span>
                    <ChoixFacteur id="bat-facteur" valeur={s.facteur} changer={(f) => maj({ facteur: f })} />
                  </span>
                  <span>Capitaux nécessaires : <strong>{texteCapitaux(ap.besoin)}</strong></span>
                  <span className="doux">
                    Un PJ doit être présent dans la localité. Chaque jour, les chantiers puisent dans les réserves, dans la limite de dépense.
                  </span>
                </>
              )}
              {s.acquisition === "acheterCapitaux" && (
                <>
                  <label className="choix-radio">
                    <input type="checkbox" checked={s.seulementManquants} onChange={(e) => maj({ seulementManquants: e.target.checked })} /> N'acheter que ce qui manque dans les réserves
                  </label>
                  <span>
                    À acheter : <strong>{texteCapitaux(ap.aAcheter)}</strong> = <strong>{ap.prixCapitauxPo} po</strong>
                  </span>
                  <span className="doux">Valeurs d'achat : Table 2-1, Guide de campagne p. 78.</span>
                </>
              )}
              {s.acquisition === "acheterBatiment" && (
                <span className="ligne" style={{ gap: 10 }}>
                  <span className="doux">Prix payé</span>
                  <ChampMontant id="bat-prix" valeur={s.prix} changer={(v) => maj({ prix: v })} />
                  <span className="doux">prix du livre : {ap.prixLivrePo} po (vide = ce prix)</span>
                </span>
              )}
              {(s.acquisition === "acheterCapitaux" || s.acquisition === "acheterBatiment") && (
                <span className="ligne" style={{ gap: 10 }}>
                  <span className="doux">Payé par</span>
                  <ChoixPayeur id="bat-payeur" valeur={s.payeur} changer={(x) => maj({ payeur: x })} tresorerie={tresorerie} payeurs={p.payeurs} />
                </span>
              )}
            </div>
          </>
        )}

        <span />
        <div>
          <button type="submit" className="bouton principal" disabled={!s.nom.trim() || s.localite === null}>
            Créer le bâtiment
          </button>
        </div>
      </form>
    </section>
  );
}

// ------------------------------------------------------------------ vente et abandon

/** Boutons « Vendre… » et « Abandonner… » d'un bien, avec leurs formulaires (GC p. 84). */
export function BlocVente({ v, outils }: { v: EtatVente; outils: Outils }) {
  const [mode, setMode] = useState<null | "vente" | "abandon">(null);
  const [sansAcheteur, setSansAcheteur] = useState(false);
  const [po, setPo] = useState(v.produit);
  const [jours, setJours] = useState("");
  const [influence, setInfluence] = useState(0);
  const agir = async (p: Promise<import("../api").Retour>) => {
    const r = await outils.agir(p);
    if (r?.ok) setMode(null);
  };
  const act = (a: ActionBatiment) => agir(api.actionBatiment(a));

  if (v.enCours) {
    return (
      <div className="ligne" style={{ gap: 10 }}>
        <span className="badge perso" style={{ marginLeft: 0 }}>en vente : {v.enCours}</span>
        <button className="bouton discret" onClick={() => act({ type: "annulerVente", cible: v.cible })}>Annuler la vente</button>
      </div>
    );
  }
  return (
    <div className="pile">
      <div className="ligne" style={{ gap: 8 }}>
        <button
          className="bouton discret"
          title="Moitié du prix d'achat, acheteur trouvé en 3d6 jours dans 75 % des cas (GC p. 84)"
          onClick={() => {
            setPo(v.produit);
            setSansAcheteur(false);
            setMode(mode === "vente" ? null : "vente");
          }}
        >
          Vendre…
        </button>
        <button className="bouton discret" title="Sans compensation, immédiat et définitif (GC p. 84)" onClick={() => setMode(mode === "abandon" ? null : "abandon")}>
          Abandonner…
        </button>
      </div>

      {mode === "abandon" && (
        <div className="carte encadre">
          <strong style={{ color: "var(--orange)" }}>Abandonner {v.libelle} ?</strong>
          <p className="doux">Sans aucune compensation ; le bien disparaît (annulable avec Ctrl+Z).</p>
          <div className="ligne" style={{ gap: 8 }}>
            <button className="bouton danger" onClick={() => act({ type: "abandonner", cible: v.cible, sansAcheteur: false })}>Abandonner</button>
            <button className="bouton" onClick={() => setMode(null)}>Annuler</button>
          </div>
        </div>
      )}

      {mode === "vente" && (
        <div className="carte encadre">
          <strong style={{ color: "var(--or)" }}>Vendre {v.libelle}</strong>
          <p className="doux">{v.valeur}</p>
          <div className="segments" role="radiogroup" aria-label="Acheteur">
            <button role="radio" aria-checked={!sansAcheteur} className={!sansAcheteur ? "actif" : ""} onClick={() => setSansAcheteur(false)}>Acheteur trouvé (75 % des cas)</button>
            <button role="radio" aria-checked={sansAcheteur} className={sansAcheteur ? "actif" : ""} onClick={() => setSansAcheteur(true)}>Aucun acheteur (25 %)</button>
          </div>
          {sansAcheteur ? (
            <>
              <p className="doux">Personne ne reprend le bien : vous le gardez, ou vous l'abandonnez sans compensation.</p>
              <div className="ligne" style={{ gap: 8 }}>
                <button className="bouton" onClick={() => act({ type: "garderSansAcheteur", cible: v.cible })}>Garder le bien</button>
                <button className="bouton danger" onClick={() => act({ type: "abandonner", cible: v.cible, sansAcheteur: true })}>Abandonner le bien</button>
                <button className="bouton" onClick={() => setMode(null)}>Annuler</button>
              </div>
            </>
          ) : (
            <form
              className="pile"
              onSubmit={(e) => {
                e.preventDefault();
                agir(api.miseEnVente(v.cible, jours, influence, po));
              }}
            >
              <div className="ligne operation">
                <div>
                  <label className="etiquette-champ" htmlFor="vente-po">Produit</label>
                  <ChampMontant id="vente-po" valeur={po} changer={setPo} />
                </div>
                <div>
                  <label className="etiquette-champ" htmlFor="vente-jours">Jours pour trouver l'acheteur</label>
                  <input id="vente-jours" className="champ mini" inputMode="numeric" placeholder="3d6" value={jours} onChange={(e) => setJours(e.target.value)} />
                </div>
                <div>
                  <label className="etiquette-champ" htmlFor="vente-inf">Influence dépensée</label>
                  <input
                    id="vente-inf"
                    className="champ mini"
                    type="number"
                    min={0}
                    max={v.influenceDispo}
                    value={influence}
                    onChange={(e) => setInfluence(Math.max(0, Math.min(v.influenceDispo, Number(e.target.value) || 0)))}
                  />
                </div>
              </div>
              <p className="doux">
                Lancez 3d6, puis retirez 1d6 par point d'Influence dépensé (minimum 0). Le bien reste à vous jusqu'à la vente.
              </p>
              <div className="ligne" style={{ gap: 8 }}>
                <button type="submit" className="bouton principal" disabled={!jours.trim() || !po.trim()}>Mettre en vente</button>
                <button type="button" className="bouton" onClick={() => setMode(null)}>Annuler</button>
              </div>
            </form>
          )}
        </div>
      )}
    </div>
  );
}

// ------------------------------------------------------------------ fiche

function Detail({ id, p, outils }: { id: number; p: PageBatiments; outils: Outils }) {
  const [b, setB] = useState<FicheBatiment | null>(null);
  const [facteur, setFacteur] = useState(1);
  const [payeur, setPayeur] = useState<number | null>(null);
  const [nomEtage, setNomEtage] = useState("");

  useEffect(() => {
    api.ficheBatiment(id).then(setB);
  }, [id, outils.version]);

  if (!b) return null;
  const agir = async (a: ActionBatiment) => outils.agir(api.actionBatiment(a));
  const bonus = b.bonus;

  return (
    <>
      <section className="carte fiche-localite">
        <div className="tete">
          <h2 className="nom">{b.nom}</h2>
          {b.classement && <span className="doux">{b.classement}</span>}
          {b.modele && <span className="badge perso">modèle : {b.modele}</span>}
        </div>
        <span className="doux">{b.lieu}</span>
        <div className="ligne">
          <div>
            <span className="etiquette-champ">Reste à dépenser</span>
            <BlocCapitaux c={b.reste} />
          </div>
          <div>
            <span className="etiquette-champ">Durée restante estimée</span>
            <strong>{b.dureeRestante} jours</strong>
          </div>
        </div>
        <div className="ligne" style={{ gap: 10 }}>
          <span className="doux">Recettes des salles (bonus maximum par type) :</span>
          {bonus.length === 0 && <span className="doux">aucune recette active</span>}
          {bonus.map((x) => (
            <span key={x.ressource} className="puce" style={{ color: COULEUR_RESSOURCE[x.ressource] }}>
              {NOM_RESSOURCE[x.ressource]} +{x.valeur}
            </span>
          ))}
        </div>
        <span className="doux">{b.equipes}</span>
        <span className="doux">{b.gestionnaires}</span>
        <button className="bouton discret" onClick={() => agir({ type: "favori", batiment: b.id, actif: !b.favori })}>
          {b.favori ? "★ Retirer des favoris" : "☆ Mettre en favori"}
        </button>
        <label className="choix-radio" title="Option personnalisée : sinon, les étages avancent en même temps">
          <input type="checkbox" checked={b.etagesSuccessifs} onChange={(e) => agir({ type: "etagesSuccessifs", batiment: b.id, actif: e.target.checked })} />
          Étages construits l'un après l'autre <span className="badge perso">Personnalisé</span>
        </label>
        <BlocVente v={b.vente} outils={outils} />
        <div>
          <button
            className="bouton discret"
            title="Étages et salles de ce bâtiment, réutilisables dans toutes vos campagnes (un modèle du même nom est remplacé)"
            onClick={() => outils.agir(api.enregistrerModeleBatiment(b.id))}
          >
            ★ Enregistrer comme modèle personnel
          </button>
        </div>
        {b.notes && <p className="doux notes">{b.notes}</p>}
      </section>

      <section className="carte ligne" style={{ gap: 14 }}>
        <label className="doux" htmlFor="det-facteur">Accélération des prochains chantiers</label>
        <ChoixFacteur id="det-facteur" valeur={facteur} changer={setFacteur} />
        <label className="doux" htmlFor="det-payeur">Achats payés par</label>
        <ChoixPayeur id="det-payeur" valeur={payeur} changer={setPayeur} tresorerie={b.tresorerie} payeurs={b.payeurs} />
      </section>

      {b.etages.map((e) => (
        <Etage key={e.id} e={e} b={b} p={p} facteur={facteur} payeur={payeur} outils={outils} />
      ))}

      <form
        className="ligne operation"
        onSubmit={async (ev) => {
          ev.preventDefault();
          const r = await agir({ type: "ajouterEtage", batiment: b.id, nom: nomEtage });
          if (r?.ok) setNomEtage("");
        }}
      >
        <input className="champ moyen" aria-label="Nom du nouvel étage" placeholder="nom du nouvel étage" value={nomEtage} onChange={(e) => setNomEtage(e.target.value)} />
        <button type="submit" className="bouton" disabled={!nomEtage.trim()}>{icones.plus({})} Ajouter un étage</button>
      </form>
    </>
  );
}

function Etage(props: { e: FicheEtage; b: FicheBatiment; p: PageBatiments; facteur: number; payeur: number | null; outils: Outils }) {
  const { e, b, p } = props;
  const [ajout, setAjout] = useState(false);
  const [recherche, setRecherche] = useState("");
  const [salle, setSalle] = useState("");
  const [deja, setDeja] = useState(false);
  const agir = (a: ActionBatiment) => props.outils.agir(api.actionBatiment(a));
  const rech = recherche.trim().toLowerCase();
  const choix = p.salles.filter((s) => !rech || s.nom.toLowerCase().includes(rech));

  return (
    <section className="carte etage">
      <div className="ligne" style={{ gap: 12 }}>
        <h3 style={{ color: "var(--or)" }}>{e.nom}</h3>
        <select
          className="champ moyen"
          aria-label={`Mode de construction de ${e.nom}`}
          value={e.sequentiel ? "seq" : "par"}
          onChange={(ev) => agir({ type: "modeEtage", batiment: b.id, etage: e.id, sequentiel: ev.target.value === "seq" })}
        >
          <option value="par">salles en parallèle</option>
          <option value="seq">salles l'une après l'autre</option>
        </select>
        <span className="badge perso" style={{ marginLeft: 0 }}>Personnalisé</span>
      </div>
      <div className="ligne" style={{ gap: 12 }}>
        <span className="doux">Durée restante : {e.dureeRestante} j</span>
        <span className="doux">· {texteCapitaux(e.reste)}</span>
      </div>

      {e.salles.map((s) => (
        <Salle key={s.id} s={s} p={p} facteur={props.facteur} payeur={props.payeur} outils={props.outils} />
      ))}

      {ajout ? (
        <div className="ligne operation">
          <input className="champ moyen" aria-label="Chercher une salle" placeholder="chercher une salle" value={recherche} onChange={(ev) => setRecherche(ev.target.value)} />
          <select className="champ" aria-label="Salle à ajouter" style={{ maxWidth: 460 }} value={salle} onChange={(ev) => setSalle(ev.target.value)}>
            <option value="">Choisir…</option>
            {choix.map((x) => (
              <option key={x.id} value={x.id}>{x.libelle}</option>
            ))}
          </select>
          <label className="choix-radio"><input type="checkbox" checked={deja} onChange={(ev) => setDeja(ev.target.checked)} /> déjà construite</label>
          <button
            className="bouton principal"
            disabled={!salle}
            onClick={() => agir({ type: "ajouterSalle", batiment: b.id, etage: e.id, reference: salle, deja })}
          >
            Ajouter la salle
          </button>
          <button className="bouton" onClick={() => setAjout(false)}>Fermer</button>
        </div>
      ) : (
        <div>
          <button className="bouton" onClick={() => { setAjout(true); setSalle(""); }}>{icones.plus({})} Ajouter une salle</button>
        </div>
      )}
    </section>
  );
}

// ------------------------------------------------------------------ salle

function Salle(props: { s: FicheSalle; p: PageBatiments; facteur: number; payeur: number | null; outils: Outils }) {
  const { s } = props;
  const [amelioration, setAmelioration] = useState("");
  const agir = (a: ActionBatiment) => props.outils.agir(api.actionBatiment(a));
  const ameliorationChoisie = s.ameliorations.find((a) => a.id === amelioration);
  const regler = (c: Partial<{ ressource: RessourceId | null; revenusActifs: boolean; brisee: boolean }>) =>
    agir({
      type: "reglerSalle",
      salle: s.id,
      ressource: s.ressource,
      revenusActifs: s.revenusActifs,
      brisee: s.brisee,
      ...c,
    });
  const couleur = s.statut === "construite" ? "var(--vert)" : s.statut === "en_travaux" ? "var(--orange)" : "var(--texte-doux)";

  return (
    <div className="salle">
      <div className="ligne" style={{ gap: 10 }}>
        <strong style={{ color: "var(--texte-fort)" }}>{s.nom}</strong>
        {s.typeSalle && <span className="doux">{s.typeSalle}</span>}
        <span className="badge" style={{ color: couleur, border: `1px solid ${couleur}`, marginLeft: 0 }}>{s.statutTexte}</span>
        {s.brisee && <span className="badge" style={{ color: "var(--rouge)", border: "1px solid var(--rouge)" }}>brisée</span>}
        <span className="doux">{s.source}</span>
        {s.pointAConfirmer && <span className="badge confirmer">À confirmer : {s.pointAConfirmer}</span>}
      </div>
      <span className="doux">{s.infos}</span>

      {s.admises.length > 0 && (
        <div className="ligne" style={{ gap: 12 }}>
          <span className="doux">Génère :</span>
          {s.admises.map((x) => (
            <label key={x} className="choix-radio" style={{ color: COULEUR_RESSOURCE[x] }}>
              {/* un seul type de revenu à la fois ; recliquer le type choisi le décoche */}
              <input type="radio" name={`res-${s.id}`} checked={s.ressource === x} onClick={() => regler({ ressource: s.ressource === x ? null : x })} onChange={() => undefined} />
              {NOM_RESSOURCE[x]}
            </label>
          ))}
          <label className="choix-radio"><input type="checkbox" checked={s.revenusActifs} onChange={(e) => regler({ revenusActifs: e.target.checked })} /> revenus actifs</label>
          <label className="choix-radio" title="Moitié des recettes (GC p. 92)"><input type="checkbox" checked={s.brisee} onChange={(e) => regler({ brisee: e.target.checked })} /> brisée</label>
        </div>
      )}

      {s.statut === "a_construire" && (
        <div className="ligne" style={{ gap: 8 }}>
          <button className="bouton" disabled={!s.constructible} onClick={() => agir({ type: "lancerConstruction", salle: s.id, facteur: props.facteur })}>
            Lancer la construction{props.facteur > 1 ? ` (×${props.facteur})` : ""}
          </button>
          {s.prixAchatPo !== null && (
            <button className="bouton" title="Payé par le choix « Achats payés par » ci-dessus" onClick={() => agir({ type: "acheterSalle", salle: s.id, payeur: props.payeur })}>
              Acheter terminée ({s.prixAchatPo} po)
            </button>
          )}
          <button className="bouton" title="Marque la salle construite sans rien payer" onClick={() => agir({ type: "sallePossedee", salle: s.id })}>Déjà possédée</button>
          <button className="bouton discret" onClick={() => agir({ type: "retirerSalle", salle: s.id })}>Retirer</button>
        </div>
      )}

      {s.chantier && (
        <div className="pile">
          <div className="ligne" style={{ gap: 16 }}>
            <span className="doux">capitaux {s.chantier.capitauxFaits}/{s.chantier.capitauxTotal}</span>
            <div className="barre" style={{ width: 160 }}><div style={{ width: `${Math.min(100, (s.chantier.capitauxFaits / Math.max(1, s.chantier.capitauxTotal)) * 100)}%` }} /></div>
            <span className="doux">jours {s.chantier.jours}/{s.chantier.duree}</span>
            <div className="barre" style={{ width: 160 }}><div style={{ width: `${s.chantier.duree === 0 ? 100 : Math.min(100, (s.chantier.jours / s.chantier.duree) * 100)}%` }} /></div>
            <span className="doux">{s.chantier.estimation}</span>
          </div>
          {s.chantier.manque && <span className="doux">{s.chantier.manque}</span>}
        </div>
      )}

      {s.statut === "construite" && (
        <div className="ligne" style={{ gap: 8 }}>
          {s.ameliorations.length > 0 && (
            <>
              <select className="champ moyen" aria-label="Améliorer en…" value={amelioration} onChange={(e) => setAmelioration(e.target.value)}>
                <option value="">Améliorer en…</option>
                {s.ameliorations.map((a) => (
                  <option key={a.id} value={a.id}>{a.nom}</option>
                ))}
              </select>
              {ameliorationChoisie?.detail && <span className="doux">{ameliorationChoisie.detail}</span>}
              {ameliorationChoisie && (
                <button className="bouton" onClick={() => agir({ type: "ameliorer", salle: s.id, vers: ameliorationChoisie.id, facteur: props.facteur })}>
                  Lancer l'amélioration
                </button>
              )}
            </>
          )}
          <button className="bouton discret" onClick={() => agir({ type: "retirerSalle", salle: s.id })}>Retirer</button>
        </div>
      )}

      {s.vente && <BlocVente v={s.vente} outils={props.outils} />}

      <Extensions s={s} facteur={props.facteur} payeur={props.payeur} outils={props.outils} />

      {s.reparation && (
        <div className="ligne" style={{ gap: 8 }}>
          <button className="bouton" title="Moitié du prix de construction, payée par le choix « Achats payés par »" onClick={() => agir({ type: "reparer", salle: s.id, enCapitaux: false, payeur: props.payeur })}>
            Réparer ({s.reparation[0]})
          </button>
          {s.reparation[1] && (
            <button className="bouton" title="Moitié du coût (arrondi inférieur), pris dans les réserves de la localité" onClick={() => agir({ type: "reparer", salle: s.id, enCapitaux: true, payeur: null })}>
              Réparer en capitaux ({s.reparation[1]})
            </button>
          )}
        </div>
      )}
    </div>
  );
}

// ------------------------------------------------------------------ extensions (GC p. 108)

function Extensions(props: { s: FicheSalle; facteur: number; payeur: number | null; outils: Outils }) {
  const { s } = props;
  const [choix, setChoix] = useState<string | null>(null);
  const [deja, setDeja] = useState(false);
  const [cout, setCout] = useState<Capitaux>({ marchandises: 0, influence: 0, travail: 0, magie: 0 });
  const [prix, setPrix] = useState("");
  const [duree, setDuree] = useState("");
  const agir = (a: ActionBatiment) => props.outils.agir(api.actionBatiment(a));
  const possible = s.extensionsPossibles.find((x) => x.id === choix);

  return (
    <>
      {s.extensions.map((x) => (
        <div className="ligne" style={{ gap: 10 }} key={x.reference}>
          <span className="badge perso" style={{ marginLeft: 0 }}>{x.nom}</span>
          <span className="badge" style={{ border: "1px solid var(--bord-fort)", marginLeft: 0 }}>{x.statutTexte}</span>
          {x.description && <span className="doux">{x.description}</span>}
          <span className="doux">{x.detail}</span>
          {x.manque && <span className="doux">{x.manque}</span>}
          {x.aConstruire && (
            <>
              <button
                className="bouton"
                disabled={!x.lancable}
                title={x.lancable ? undefined : "La salle doit d'abord être construite ou lancée"}
                onClick={() => agir({ type: "lancerExtension", salle: s.id, reference: x.reference, facteur: props.facteur })}
              >
                Lancer{props.facteur > 1 ? ` (×${props.facteur})` : ""}
              </button>
              <button className="bouton" title="Payé par le choix « Achats payés par » ci-dessus" onClick={() => agir({ type: "acheterExtension", salle: s.id, reference: x.reference, payeur: props.payeur })}>
                Acheter faite ({x.prixPo} po)
              </button>
              <button className="bouton discret" onClick={() => agir({ type: "extensionPossedee", salle: s.id, reference: x.reference })}>Déjà possédée</button>
              <button className="bouton discret" onClick={() => agir({ type: "retirerExtension", salle: s.id, reference: x.reference })}>Retirer</button>
            </>
          )}
        </div>
      ))}

      {s.extensionsPossibles.length > 0 && (
        <div className="ligne" style={{ gap: 8 }}>
          <span className="doux">Extensions :</span>
          {s.extensionsPossibles.map((x) => (
            <button
              key={x.id}
              className="bouton discret"
              title={x.aide}
              onClick={() => {
                setChoix(x.id);
                setDeja(false);
                setPrix("");
                setDuree("");
                setCout({ marchandises: 0, influence: 0, travail: 0, magie: 0 });
              }}
            >
              + {x.nom}
            </button>
          ))}
        </div>
      )}

      {possible && (
        <form
          className="carte encadre pile"
          onSubmit={async (e) => {
            e.preventDefault();
            const r = await agir({
              type: "ajouterExtension",
              salle: s.id,
              reference: possible.id,
              deja,
              cout: possible.saisieRequise ? cout : null,
              prix,
              duree,
            });
            if (r?.ok) setChoix(null);
          }}
        >
          <strong style={{ color: "var(--or)" }}>{possible.nom} sur {s.nom}</strong>
          <span className="doux">{possible.detail}</span>
          {possible.saisieRequise && (
            <div className="ligne operation">
              {(["marchandises", "influence", "travail", "magie"] as const).map((k) => (
                <div key={k}>
                  <label className="etiquette-champ" htmlFor={`ext-${k}`}>{NOM_RESSOURCE[k]}</label>
                  <input id={`ext-${k}`} className="champ mini" inputMode="numeric" value={cout[k] === 0 ? "" : cout[k]} placeholder="0" onChange={(e) => setCout({ ...cout, [k]: Math.max(0, Math.trunc(Number(e.target.value.replace(/\D/g, "")) || 0)) })} />
                </div>
              ))}
              <div>
                <label className="etiquette-champ" htmlFor="ext-prix">Prix (po)</label>
                <input id="ext-prix" className="champ mini" inputMode="numeric" value={prix} onChange={(e) => setPrix(e.target.value)} />
              </div>
              <div>
                <label className="etiquette-champ" htmlFor="ext-duree">Durée (jours)</label>
                <input id="ext-duree" className="champ mini" inputMode="numeric" value={duree} onChange={(e) => setDuree(e.target.value)} />
              </div>
              <span className="badge perso" style={{ marginLeft: 0 }}>Personnalisé</span>
            </div>
          )}
          <label className="choix-radio"><input type="checkbox" checked={deja} onChange={(e) => setDeja(e.target.checked)} /> déjà faite (gratuit)</label>
          <div className="ligne" style={{ gap: 8 }}>
            <button type="submit" className="bouton principal" disabled={possible.saisieRequise && (!prix.trim() || !duree.trim())}>Ajouter l'extension</button>
            <button type="button" className="bouton" onClick={() => setChoix(null)}>Annuler</button>
          </div>
        </form>
      )}
    </>
  );
}

export { formaterPo };
export type { Cible };
