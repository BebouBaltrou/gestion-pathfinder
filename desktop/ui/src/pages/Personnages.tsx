import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, formaterPo } from "../api";
import type { FicheVue, LignePersonnage } from "../fiche-types";
import { Creation } from "./fiche/Creation";
import { FicheComplete } from "./fiche/FicheComplete";
import { ImportPdf } from "./fiche/ImportPdf";
import { SuiviPersonnage } from "./SuiviPersonnage";

/** Liste des personnages joueurs et, pour celui choisi, sa fiche. */
export function Personnages({ outils }: { outils: Outils }) {
  const [liste, setListe] = useState<LignePersonnage[]>([]);
  const [choisi, setChoisi] = useState<number | null>(null);
  const [fiche, setFiche] = useState<FicheVue | null>(null);
  const [creation, setCreation] = useState(false);
  const [importPdf, setImportPdf] = useState(false);
  const [niveauVise, setNiveauVise] = useState(1);
  const [nom, setNom] = useState("");
  const [po, setPo] = useState("0");

  useEffect(() => {
    api.listePersonnages().then((l) => {
      setListe(l);
      setChoisi((c) => (c !== null && l.some((p) => p.id === c) ? c : (l[0]?.id ?? null)));
    });
  }, [outils.version]);

  useEffect(() => setNiveauVise(1), [choisi]);

  useEffect(() => {
    if (choisi === null) {
      setFiche(null);
      return;
    }
    api.fichePersonnage(choisi).then(setFiche, () => setFiche(null));
  }, [choisi, outils.version]);

  const pj = liste.find((p) => p.id === choisi) ?? null;

  const ajouter = async () => {
    const r = await outils.agir(api.creerPersonnage(nom, Math.max(0, Number(po) || 0)));
    if (r?.ok) {
      setNom("");
      setPo("0");
    }
  };

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Personnages</h1>
        <p>Les PJ de la campagne, leur compte et leur fiche. Tout se recalcule à partir des règles du Core.</p>
      </div>

      <div className="deux-colonnes">
        <div className="colonne-etroite">
          <section className="section">
            <h2>Joueurs</h2>
            <div className="liste-campagnes">
              {liste.length === 0 && <p className="doux">Aucun personnage pour l'instant.</p>}
              {liste.map((p) => (
                <button
                  key={p.id}
                  className={"ligne-campagne carte-pj" + (p.id === choisi ? " choisi" : "")}
                  onClick={() => {
                    setChoisi(p.id);
                    setCreation(false);
                  }}
                >
                  <span className="nom">
                    {p.nom}
                    <small className="doux">
                      {p.fiche ? `${p.fiche.race} · ${p.fiche.classes}` : "pas encore de fiche"}
                    </small>
                  </span>
                  {p.acolyteDe && <span className="puce">acolyte de {p.acolyteDe}</span>}
                  {p.fiche && p.fiche.niveauxEnAttente > 0 && <span className="puce or">niveau à monter</span>}
                </button>
              ))}
            </div>
          </section>
          <section className="carte section">
            <h2>Nouveau personnage</h2>
            <form
              className="formulaire"
              onSubmit={(e) => {
                e.preventDefault();
                ajouter();
              }}
            >
              <div>
                <label className="etiquette-champ" htmlFor="nom-pj">Nom</label>
                <input id="nom-pj" className="champ large" value={nom} onChange={(e) => setNom(e.target.value)} />
              </div>
              <div>
                <label className="etiquette-champ" htmlFor="po-pj">Or de départ (po)</label>
                <input
                  id="po-pj"
                  className="champ large"
                  inputMode="numeric"
                  value={po}
                  onChange={(e) => setPo(e.target.value)}
                />
              </div>
              <button className="bouton principal" type="submit" disabled={!nom.trim()}>Ajouter</button>
            </form>
            <p className="doux">ou</p>
            <button className="bouton" onClick={() => setImportPdf(true)}>Importer une fiche PDF…</button>
          </section>
        </div>

        <div className="colonne-large">
          {importPdf && <ImportPdf outils={outils} onFini={() => setImportPdf(false)} />}
          {!pj && <div className="carte vide"><p>Ajoutez un personnage pour commencer.</p></div>}
          {pj && !fiche && !creation && (
            <div className="carte vide">
              <h2>{pj.nom}</h2>
              <p>
                Compte : {formaterPo(pj.compte.pa)}
                {pj.lieu ? ` · à ${pj.lieu}` : ""}. Ce personnage n'a pas encore de fiche.
              </p>
              <button className="bouton principal" onClick={() => setCreation(true)}>Créer la fiche</button>
            </div>
          )}
          {pj && !fiche && creation && (
            <Creation
              outils={outils}
              pj={pj}
              onFini={(n) => {
                setNiveauVise(n);
                setCreation(false);
              }}
            />
          )}
          {pj && fiche && <FicheComplete outils={outils} pj={pj} fiche={fiche} niveauVise={niveauVise} />}
          {pj && <SuiviPersonnage outils={outils} pj={pj} />}
        </div>
      </div>
    </div>
  );
}
