import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, PagePersonnages } from "../api";
import type { LignePersonnage } from "../fiche-types";
import { ChampMontant } from "./Localites";

/**
 * Suivi d'un personnage dans l'intermède : où il se trouve (usure des capitaux et des
 * entreprises, GC p. 82), gains et dépenses hors de l'application, nom.
 */
export function SuiviPersonnage({ outils, pj }: { outils: Outils; pj: LignePersonnage }) {
  const [p, setP] = useState<PagePersonnages | null>(null);
  const [delta, setDelta] = useState("");
  const [motif, setMotif] = useState("");
  const [nom, setNom] = useState(pj.nom);

  useEffect(() => {
    api.pagePersonnages().then(setP);
  }, [outils.version]);
  useEffect(() => setNom(pj.nom), [pj.id, pj.nom]);

  const fiche = p?.personnages.find((x) => x.id === pj.id);
  if (!p || !fiche) return null;

  return (
    <>
      <section className="carte section">
        <h2>Où se trouve {pj.nom}</h2>
        <div className="ligne" style={{ gap: 12 }}>
          <select
            className="champ moyen"
            aria-label="Se trouve à"
            value={fiche.localite ?? ""}
            onChange={(e) => outils.agir(api.deplacerPersonnage(pj.id, e.target.value === "" ? null : Number(e.target.value)))}
          >
            <option value="">hors localité</option>
            {p.localites.map((l) => (
              <option key={l.id} value={l.id}>{l.libelle}</option>
            ))}
          </select>
          <span className="doux">La présence des PJ compte pour l'usure des capitaux et des entreprises (Guide de campagne p. 82).</span>
        </div>
      </section>

      <section className="carte section">
        <h2>Gain ou dépense</h2>
        <form
          className="ligne operation"
          onSubmit={async (e) => {
            e.preventDefault();
            const r = await outils.agir(api.ajusterCompte(pj.id, delta, motif));
            if (r?.ok) {
              setDelta("");
              setMotif("");
            }
          }}
        >
          <div>
            <label className="etiquette-champ" htmlFor="pj-delta">Montant (négatif pour une dépense)</label>
            <ChampMontant id="pj-delta" valeur={delta} changer={setDelta} signe />
          </div>
          <div style={{ flex: 1 }}>
            <label className="etiquette-champ" htmlFor="pj-motif">Motif</label>
            <input id="pj-motif" className="champ large" placeholder="butin, achat…" value={motif} onChange={(e) => setMotif(e.target.value)} />
          </div>
          <button type="submit" className="bouton principal" disabled={!delta.trim() || !motif.trim()}>Appliquer</button>
        </form>
      </section>

      <details className="carte repli">
        <summary>Renommer le personnage</summary>
        <form
          className="ligne operation"
          onSubmit={(e) => {
            e.preventDefault();
            // le Prestige calculé à partir de la fiche n'est pas enregistré : on ne le touche pas
            outils.agir(api.modifierPersonnage(pj.id, nom, pj.prestigeCalcule || pj.prestige === null ? "" : String(pj.prestige)));
          }}
        >
          <input className="champ moyen" aria-label="Nom du personnage" value={nom} onChange={(e) => setNom(e.target.value)} />
          <button type="submit" className="bouton" disabled={!nom.trim() || nom.trim() === pj.nom}>Enregistrer</button>
        </form>
      </details>

      <details className="carte repli">
        <summary>Supprimer le personnage</summary>
        <div className="ligne operation">
          <button
            className="bouton danger"
            onClick={() => {
              if (window.confirm(`Supprimer ${pj.nom} ?\n\nSon compte, sa fiche et ses liens (acolyte, partisans) disparaissent de la campagne.\n\nVous pourrez annuler juste après avec le bouton « Annuler ».`)) {
                outils.agir(api.supprimerPersonnage(pj.id));
              }
            }}
          >
            Supprimer {pj.nom}…
          </button>
        </div>
      </details>
    </>
  );
}
