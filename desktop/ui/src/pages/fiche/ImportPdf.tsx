import { useState } from "react";
import type { Outils } from "../../App";
import { api } from "../../api";
import type { ApercuImport, GraviteImport, LigneImport } from "../../fiche-types";

const TITRES: Record<GraviteImport, { titre: string; aide: string }> = {
  ecart: {
    titre: "Écarts",
    aide: "Les règles donnent une autre valeur que la feuille.",
  },
  a_valider: {
    titre: "À valider",
    aide: "Les règles ne permettent pas de trancher (objet, don ou texte libre non appliqué).",
  },
  accord: { titre: "D'accord", aide: "La feuille et les règles donnent la même valeur." },
};

/** Import d'une fiche PDF : rapport de cohérence, puis « corriger » ou « garder » pour chaque écart. */
export function ImportPdf({ outils, onFini }: { outils: Outils; onFini: () => void }) {
  const [apercu, setApercu] = useState<ApercuImport | null>(null);
  const [erreur, setErreur] = useState<string | null>(null);
  const [gardes, setGardes] = useState<Set<string>>(new Set());
  const [po, setPo] = useState("0");
  const [ouvrirAccords, setOuvrirAccords] = useState(false);

  const choisir = async () => {
    setErreur(null);
    try {
      const a = await api.choisirFichePdf();
      if (a) {
        setApercu(a);
        setGardes(new Set());
      }
    } catch (e) {
      setErreur(String(e));
    }
  };

  const basculer = (cle: string, garder: boolean) =>
    setGardes((g) => {
      const n = new Set(g);
      if (garder) n.add(cle);
      else n.delete(cle);
      return n;
    });

  const importer = async () => {
    if (!apercu) return;
    const r = await outils.agir(api.importerFichePdf(apercu.chemin, [...gardes], Math.max(0, Number(po) || 0)));
    if (r?.ok) {
      setApercu(null);
      onFini();
    }
  };

  if (!apercu) {
    return (
      <section className="carte section">
        <h2>Importer une fiche PDF</h2>
        <p className="doux">
          Le programme lit la fiche, la compare aux règles du Manuel des joueurs, puis te montre chaque écart :
          tu choisis de suivre les règles ou de garder la valeur de la feuille (elle sera alors marquée « personnalisé »).
          Les sorts de la feuille ne sont pas importés.
        </p>
        {erreur && <p className="alerte">{erreur}</p>}
        <button className="bouton principal" onClick={choisir}>Choisir un PDF…</button>
      </section>
    );
  }

  const par = (g: GraviteImport) => apercu.lignes.filter((l) => l.gravite === g);

  const Ligne = ({ l }: { l: LigneImport }) => {
    const garde = gardes.has(l.cle);
    return (
      <div className="element-liste">
        <span className="nom">
          {l.libelle}
          <small className="doux"> {l.rubrique}</small>
        </span>
        <span>
          feuille <b>{l.feuille}</b> · règles <b>{l.regles}</b>
        </span>
        <span className="doux">{l.explication}</span>
        {l.gravite !== "accord" && (
          <span className="choix-import">
            <label>
              <input type="radio" name={l.cle} checked={!garde} onChange={() => basculer(l.cle, false)} /> Corriger
              {" "}(valeur des règles)
            </label>
            {l.peut_garder ? (
              <label>
                <input type="radio" name={l.cle} checked={garde} onChange={() => basculer(l.cle, true)} /> Garder
                {" "}la valeur de la feuille ({l.delta !== null && l.delta > 0 ? "+" : ""}{l.delta} personnalisé)
              </label>
            ) : (
              <small className="doux">Garder n'est pas possible pour cette ligne : la fiche suit les règles.</small>
            )}
          </span>
        )}
      </div>
    );
  };

  return (
    <section className="carte section">
      <h2>Importer « {apercu.nom} »</h2>
      <p className="doux">
        Niveau {apercu.niveau} · {apercu.pv} PV (la feuille ne donne que le total : les dés de vie sont répartis pour le
        retrouver) · dons reconnus : {apercu.dons.length}
      </p>
      {(["ecart", "a_valider"] as GraviteImport[]).map(
        (g) =>
          par(g).length > 0 && (
            <div key={g}>
              <h3>
                {TITRES[g].titre} ({par(g).length})
              </h3>
              <p className="doux">{TITRES[g].aide}</p>
              {par(g).map((l) => (
                <Ligne key={l.cle} l={l} />
              ))}
            </div>
          ),
      )}
      <h3>
        {TITRES.accord.titre} ({par("accord").length}){" "}
        <button className="bouton" onClick={() => setOuvrirAccords(!ouvrirAccords)}>
          {ouvrirAccords ? "Masquer" : "Afficher"}
        </button>
      </h3>
      {ouvrirAccords && par("accord").map((l) => <Ligne key={l.cle} l={l} />)}
      {apercu.non_verifie.length > 0 && (
        <>
          <h3>Non vérifié</h3>
          <ul>
            {apercu.non_verifie.map((n) => (
              <li key={n} className="doux">{n}</li>
            ))}
          </ul>
        </>
      )}
      <div className="formulaire">
        <div>
          <label className="etiquette-champ" htmlFor="po-import">Or du personnage (po)</label>
          <input id="po-import" className="champ large" inputMode="numeric" value={po} onChange={(e) => setPo(e.target.value)} />
        </div>
        <div className="ligne">
          <button className="bouton principal" onClick={importer}>
            Importer le personnage ({gardes.size} valeur{gardes.size > 1 ? "s" : ""} gardée{gardes.size > 1 ? "s" : ""})
          </button>
          <button className="bouton" onClick={() => setApercu(null)}>Choisir un autre PDF</button>
        </div>
      </div>
    </section>
  );
}
