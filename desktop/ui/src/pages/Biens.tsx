import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, ActionBien, CibleBien, ChoixCible, formaterPo, JaugeEquipage, ModeleBien, PageBiens, Retour } from "../api";
import { ChampMontant } from "./Localites";

const cle = (c: CibleBien) => `${c.genre}:${c.id}`;
const trouver = (liste: ChoixCible[], k: string) => liste.find((x) => cle(x.cible) === k)?.cible;

/** Formulaire d'achat commun aux véhicules et aux montures. */
function FormulaireAchat(props: {
  id: string;
  modeles: ModeleBien[];
  libelleCible: string;
  cibles: ChoixCible[];
  payeurs: ChoixCible[];
  acheter: (modele: string, nom: string, cible: CibleBien, payeur: CibleBien, prix: string | null) => Promise<Retour | null | undefined>;
}) {
  const [modele, setModele] = useState("");
  const [nom, setNom] = useState("");
  const [cibleCle, setCibleCle] = useState("");
  const [payeurCle, setPayeurCle] = useState("");
  const [prix, setPrix] = useState("");
  const m = props.modeles.find((x) => x.id === (modele || props.modeles[0]?.id));
  const cible = trouver(props.cibles, cibleCle) ?? props.cibles[0]?.cible;
  const payeur = trouver(props.payeurs, payeurCle) ?? props.payeurs[0]?.cible;
  const pret = !!m && !!cible && !!payeur && (m.prix !== null || prix.trim() !== "");
  return (
    <form
      className="formulaire"
      onSubmit={async (e) => {
        e.preventDefault();
        if (!m || !cible || !payeur) return;
        const r = await props.acheter(m.id, nom, cible, payeur, prix.trim() || null);
        if (r?.ok) {
          setNom("");
          setPrix("");
        }
      }}
    >
      <label htmlFor={`${props.id}-modele`}>Modèle</label>
      <select id={`${props.id}-modele`} className="champ" value={m?.id ?? ""} onChange={(e) => setModele(e.target.value)}>
        {props.modeles.map((x) => (
          <option key={x.id} value={x.id}>{x.nom}{x.prix ? ` — ${formaterPo(x.prix.pc)}` : " — prix à saisir"}</option>
        ))}
      </select>
      {m && m.resume && (
        <>
          <span className="etiquette-grille">Manuel</span>
          <span className="doux">{m.resume}</span>
        </>
      )}
      <label htmlFor={`${props.id}-nom`}>Nom</label>
      <input id={`${props.id}-nom`} className="champ" placeholder={m?.nom ?? ""} value={nom} onChange={(e) => setNom(e.target.value)} />
      <label htmlFor={`${props.id}-cible`}>{props.libelleCible}</label>
      <select id={`${props.id}-cible`} className="champ" value={cible ? cle(cible) : ""} onChange={(e) => setCibleCle(e.target.value)}>
        {props.cibles.map((x) => (
          <option key={cle(x.cible)} value={cle(x.cible)}>{x.libelle}</option>
        ))}
      </select>
      <label htmlFor={`${props.id}-payeur`}>Payé par</label>
      <select id={`${props.id}-payeur`} className="champ" value={payeur ? cle(payeur) : ""} onChange={(e) => setPayeurCle(e.target.value)}>
        {props.payeurs.map((x) => (
          <option key={cle(x.cible)} value={cle(x.cible)}>{x.libelle}</option>
        ))}
      </select>
      <label htmlFor={`${props.id}-prix`}>Prix</label>
      <ChampMontant id={`${props.id}-prix`} valeur={prix} changer={setPrix} />
      <span className="doux" style={{ gridColumn: "2" }}>
        {m?.prix ? `Laissez vide pour le prix du catalogue (${formaterPo(m.prix.pc)}).` : "Ce modèle n'a pas de prix au catalogue : saisissez-le."}
      </span>
      <span />
      <div>
        <button type="submit" className="bouton principal" disabled={!pret}>Acheter</button>
      </div>
    </form>
  );
}

function Jauge({ j }: { j: JaugeEquipage }) {
  const cible = Math.max(j.requisMax, j.requisMin, 1);
  const pctRecrute = Math.min(100, (j.recrutes / cible) * 100);
  const pctCours = Math.min(100 - pctRecrute, (j.enCours / cible) * 100);
  const plage = j.requisMin === j.requisMax ? `${j.requisMin}` : `${j.requisMin} à ${j.requisMax}`;
  return (
    <div className="pile" style={{ gap: 4 }}>
      <span>
        Équipage : <strong>{j.recrutes}</strong> / {plage}
        {j.enCours > 0 && <span className="doux"> (+ {j.enCours} en cours de recrutement)</span>}
        {j.complete && <span className="badge" style={{ marginLeft: 8, border: "1px solid var(--bord-fort)" }}>équipage complet</span>}
      </span>
      <div className="barre" role="progressbar" aria-valuemin={0} aria-valuemax={j.requisMin} aria-valuenow={j.recrutes} aria-label="Équipage recruté">
        <div style={{ width: `${pctRecrute}%`, display: "inline-block" }} />
        <div style={{ width: `${pctCours}%`, display: "inline-block", opacity: 0.4 }} />
      </div>
    </div>
  );
}

export function Biens({ outils }: { outils: Outils }) {
  const [p, setP] = useState<PageBiens | null>(null);
  const [equipeChoisie, setEquipeChoisie] = useState<Record<number, string>>({});
  const [gestionnaireChoisi, setGestionnaireChoisi] = useState<Record<number, string>>({});

  useEffect(() => {
    api.pageBiens().then(setP);
  }, [outils.version]);

  if (!p) return null;
  const agir = (a: ActionBien) => outils.agir(api.actionBien(a));

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Biens</h1>
        <p>Véhicules et montures de la campagne : propriétaire, équipes, gestionnaires, équipage. Caractéristiques du Manuel des joueurs ; les attributions sont des règles personnalisées.</p>
      </div>

      <section className="carte section">
        <h2>Acheter un véhicule</h2>
        <FormulaireAchat
          id="veh"
          modeles={p.modelesVehicules}
          libelleCible="Propriétaire"
          cibles={p.proprietaires}
          payeurs={p.payeurs}
          acheter={(modele, nom, cible, payeur, prix) => agir({ type: "acheterVehicule", modele, nom, proprietaire: cible, payeur, prix })}
        />
      </section>

      <section className="section">
        <h2>Véhicules</h2>
        {p.vehicules.length === 0 && <p className="doux">Aucun véhicule pour l'instant.</p>}
        {p.vehicules.map((v) => (
          <div className="carte pile" key={v.id}>
            <div className="ligne" style={{ gap: 12 }}>
              <strong style={{ fontSize: 17, color: "var(--texte-fort)" }}>{v.nom}</strong>
              <span className="doux">{v.modele}</span>
              <strong style={{ color: "var(--po)" }} title="Compté dans le patrimoine du propriétaire">{formaterPo(v.prix.pc)}</strong>
              <button className="bouton discret" onClick={() => agir({ type: "retirerVehicule", id: v.id })}>Retirer</button>
            </div>
            <div className="ligne" style={{ gap: 10 }}>
              <label className="doux" htmlFor={`veh-prop-${v.id}`}>Propriétaire</label>
              <select
                id={`veh-prop-${v.id}`}
                className="champ"
                style={{ maxWidth: 320 }}
                value={cle(v.proprietaire.cible)}
                onChange={(e) => {
                  const c = trouver(p.proprietaires, e.target.value);
                  if (c) agir({ type: "modifierVehicule", id: v.id, nom: v.nom, proprietaire: c });
                }}
              >
                {p.proprietaires.map((x) => (
                  <option key={cle(x.cible)} value={cle(x.cible)}>{x.libelle}</option>
                ))}
              </select>
            </div>
            {v.details.length > 0 && (
              <ul className="doux" style={{ margin: 0, paddingLeft: 18 }}>
                {v.details.map((d) => <li key={d}>{d}</li>)}
              </ul>
            )}
            {v.jauge && <Jauge j={v.jauge} />}
            <div className="ligne" style={{ gap: 10 }}>
              <span className="doux">Équipes :</span>
              {v.equipes.length === 0 && <span className="doux">aucune</span>}
              {v.equipes.map((e) => (
                <span className="puce or" key={cle(e.cible)}>
                  {e.libelle}{" "}
                  <button className="bouton discret" style={{ height: 22, padding: "0 4px" }} aria-label={`Retirer ${e.libelle}`} onClick={() => agir({ type: "retirerEquipe", id: v.id, equipe: e.cible.id })}>✕</button>
                </span>
              ))}
            </div>
            {v.equipesPossibles.length > 0 && (
              <div className="ligne" style={{ gap: 8 }}>
                <select className="champ" style={{ maxWidth: 320 }} aria-label="Affecter une équipe" value={equipeChoisie[v.id] ?? ""} onChange={(e) => setEquipeChoisie({ ...equipeChoisie, [v.id]: e.target.value })}>
                  <option value="">Affecter une équipe…</option>
                  {v.equipesPossibles.map((e) => (
                    <option key={cle(e.cible)} value={e.cible.id}>{e.libelle}</option>
                  ))}
                </select>
                <button
                  className="bouton"
                  disabled={!equipeChoisie[v.id]}
                  onClick={async () => {
                    const r = await agir({ type: "affecterEquipe", id: v.id, equipe: Number(equipeChoisie[v.id]) });
                    if (r?.ok) setEquipeChoisie({ ...equipeChoisie, [v.id]: "" });
                  }}
                >
                  Affecter
                </button>
              </div>
            )}
            <div className="ligne" style={{ gap: 10 }}>
              <span className="doux">Gestionnaires :</span>
              {v.gestionnaires.length === 0 && <span className="doux">aucun</span>}
              {v.gestionnaires.map((g) => (
                <span className="puce or" key={g.id}>
                  {g.libelle}{" "}
                  <button className="bouton discret" style={{ height: 22, padding: "0 4px" }} aria-label={`Retirer ${g.libelle}`} onClick={() => agir({ type: "retirerGestionnaire", id: v.id, gestionnaire: g.id })}>✕</button>
                </span>
              ))}
            </div>
            {v.gestionnairesPossibles.length > 0 && (
              <div className="ligne" style={{ gap: 8 }}>
                <select className="champ" style={{ maxWidth: 320 }} aria-label="Affecter un gestionnaire" value={gestionnaireChoisi[v.id] ?? ""} onChange={(e) => setGestionnaireChoisi({ ...gestionnaireChoisi, [v.id]: e.target.value })}>
                  <option value="">Affecter un gestionnaire…</option>
                  {v.gestionnairesPossibles.map((g) => (
                    <option key={g.id} value={g.id}>{g.libelle}</option>
                  ))}
                </select>
                <button
                  className="bouton"
                  disabled={!gestionnaireChoisi[v.id]}
                  onClick={async () => {
                    const r = await agir({ type: "affecterGestionnaire", id: v.id, gestionnaire: Number(gestionnaireChoisi[v.id]) });
                    if (r?.ok) setGestionnaireChoisi({ ...gestionnaireChoisi, [v.id]: "" });
                  }}
                >
                  Affecter
                </button>
              </div>
            )}
          </div>
        ))}
      </section>

      <section className="carte section">
        <h2>Acheter une monture</h2>
        <FormulaireAchat
          id="mon"
          modeles={p.modelesMontures}
          libelleCible="Confiée à"
          cibles={p.detenteurs}
          payeurs={p.payeurs}
          acheter={(modele, nom, cible, payeur, prix) => agir({ type: "acheterMonture", modele, nom, detenteur: cible, payeur, prix })}
        />
      </section>

      <section className="section">
        <h2>Montures</h2>
        {p.montures.length === 0 && <p className="doux">Aucune monture pour l'instant.</p>}
        {p.montures.map((m) => (
          <div className="carte pile" key={m.id}>
            <div className="ligne" style={{ gap: 12 }}>
              <strong style={{ fontSize: 17, color: "var(--texte-fort)" }}>{m.nom}</strong>
              <span className="doux">{m.modele}</span>
              <strong style={{ color: "var(--po)" }} title="Compté dans le patrimoine du détenteur">{formaterPo(m.prix.pc)}</strong>
              <button className="bouton discret" onClick={() => agir({ type: "retirerMonture", id: m.id })}>Retirer</button>
            </div>
            <div className="ligne" style={{ gap: 10 }}>
              <label className="doux" htmlFor={`mon-det-${m.id}`}>Confiée à</label>
              <select
                id={`mon-det-${m.id}`}
                className="champ"
                style={{ maxWidth: 320 }}
                value={cle(m.detenteur.cible)}
                onChange={(e) => {
                  const c = trouver(p.detenteurs, e.target.value);
                  if (c) agir({ type: "modifierMonture", id: m.id, nom: m.nom, detenteur: c });
                }}
              >
                {p.detenteurs.map((x) => (
                  <option key={cle(x.cible)} value={cle(x.cible)}>{x.libelle}</option>
                ))}
              </select>
            </div>
            {m.details.length > 0 && (
              <ul className="doux" style={{ margin: 0, paddingLeft: 18 }}>
                {m.details.map((d) => <li key={d}>{d}</li>)}
              </ul>
            )}
          </div>
        ))}
      </section>
    </div>
  );
}
