import { useState } from "react";
import type { Outils } from "../App";
import { api } from "../api";
import { icones } from "../icones";

export function Campagnes({ outils }: { outils: Outils }) {
  const [nom, setNom] = useState("");
  const { campagnes, dossier } = outils.etat;

  const creer = async () => {
    const r = await outils.agir(api.creerCampagne(nom));
    if (r?.ok) {
      setNom("");
      outils.aller("tableau");
    }
  };

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Campagnes</h1>
        <p>Chaque campagne est enregistrée automatiquement après chaque action.</p>
      </div>

      <section className="carte section">
        <h2>Nouvelle campagne</h2>
        <form
          className="ligne"
          style={{ gap: 12 }}
          onSubmit={(e) => {
            e.preventDefault();
            creer();
          }}
        >
          <div>
            <label className="etiquette-champ" htmlFor="nom-campagne">Nom</label>
            <input
              id="nom-campagne"
              className="champ"
              value={nom}
              onChange={(e) => setNom(e.target.value)}
              placeholder="ex. L'Éveil des Seigneurs des Runes"
            />
          </div>
          <button className="bouton principal" type="submit" disabled={!nom.trim()} style={{ alignSelf: "flex-end", height: 44 }}>
            {icones.plus({})} Créer
          </button>
        </form>
      </section>

      <section className="section">
        <h2>Campagnes enregistrées</h2>
        {campagnes.length === 0 && <p className="doux">Aucune campagne pour l'instant.</p>}
        <div className="liste-campagnes">
          {campagnes.map((c) => (
            <div className="ligne-campagne" key={c.chemin}>
              {icones.campagnes({ taille: 20 })}
              <span className="nom">{c.nom}</span>
              {c.ouverte ? (
                <span className="puce or">ouverte</span>
              ) : (
                <button
                  className="bouton"
                  onClick={async () => {
                    const r = await outils.agir(api.ouvrirCampagne(c.chemin));
                    if (r?.ok) outils.aller("tableau");
                  }}
                >
                  Ouvrir
                </button>
              )}
            </div>
          ))}
        </div>
        <p className="doux">Dossier : {dossier}</p>
      </section>
    </div>
  );
}
