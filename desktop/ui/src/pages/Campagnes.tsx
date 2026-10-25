import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, PageParametres } from "../api";
import { icones } from "../icones";

export function Campagnes({ outils }: { outils: Outils }) {
  const [nom, setNom] = useState("");
  const [cal, setCal] = useState<PageParametres["calendrier"] | null>(null);
  const { campagnes, dossier } = outils.etat;

  useEffect(() => {
    api.calendrierParDefaut().then(setCal);
  }, []);

  const creer = async () => {
    const saisie = cal && { jourSemaine: cal.jourSemaine, jour: cal.jour, mois: cal.mois, annee: cal.annee, bissextile: cal.bissextile };
    const r = await outils.agir(api.creerCampagne(nom, saisie ?? undefined));
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
          style={{ gap: 12, flexWrap: "wrap" }}
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
          {cal && (
            <div>
              <span className="etiquette-champ">Date de départ (jour 1)</span>
              <div className="ligne" style={{ gap: 8 }}>
                <select className="champ petit" aria-label="Jour de la semaine du jour 1" value={cal.jourSemaine} onChange={(e) => setCal({ ...cal, jourSemaine: Number(e.target.value) })}>
                  {cal.joursSemaine.map((j, i) => (
                    <option key={j} value={i}>{j}</option>
                  ))}
                </select>
                <input
                  className="champ mini"
                  type="number"
                  min={1}
                  max={31}
                  aria-label="Jour du mois"
                  value={cal.jour}
                  onChange={(e) => setCal({ ...cal, jour: Math.max(1, Math.min(31, Number(e.target.value) || 1)) })}
                />
                <select className="champ petit" aria-label="Mois" value={cal.mois} onChange={(e) => setCal({ ...cal, mois: Number(e.target.value) })}>
                  {cal.nomsMois.map((m, i) => (
                    <option key={m} value={i + 1}>{m}</option>
                  ))}
                </select>
                <input
                  className="champ mini"
                  type="number"
                  min={1}
                  max={9999}
                  aria-label="Année"
                  value={cal.annee}
                  onChange={(e) => setCal({ ...cal, annee: Math.max(1, Math.min(9999, Number(e.target.value) || 1)) })}
                />
                <span>AR</span>
              </div>
            </div>
          )}
          <button className="bouton principal" type="submit" disabled={!nom.trim()} style={{ alignSelf: "flex-end", height: 44 }}>
            {icones.plus({})} Créer
          </button>
        </form>
      </section>

      <section className="carte section">
        <h2>Exporter et importer</h2>
        <div className="ligne" style={{ gap: 10 }}>
          <button className="bouton" disabled={!outils.etat.entete} title="Copie complète (fichier .json) à ranger ou à transmettre" onClick={() => outils.agir(api.exporterCampagne())}>
            Exporter la campagne…
          </button>
          <button
            className="bouton"
            disabled={!outils.etat.entete}
            title="Page lisible : personnages, localités, bâtiments, organisations, gestionnaires, historique. Ouvrez-la dans le navigateur puis « Imprimer → PDF »."
            onClick={() => outils.agir(api.exporterRecapitulatif())}
          >
            Récapitulatif (HTML / PDF)…
          </button>
          <button className="bouton" disabled={!outils.etat.entete} title="Fichier CSV lisible par Excel ou LibreOffice" onClick={() => outils.agir(api.exporterHistorique())}>
            Historique (tableur)…
          </button>
          <button
            className="bouton"
            title="Ajoute une campagne exportée (.json) à vos campagnes et l'ouvre"
            onClick={async () => {
              const r = await outils.agir(api.importerCampagne());
              if (r?.ok && outils.etat.entete) outils.aller("tableau");
            }}
          >
            Importer une campagne…
          </button>
        </div>
        {outils.etat.simulation && <p className="doux">En simulation, les exports portent sur la campagne simulée.</p>}
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
        <div className="ligne" style={{ gap: 10 }}>
          <span className="doux">Dossier : {dossier}</span>
          <button className="bouton discret" onClick={() => outils.agir(api.ouvrirDossier())}>Ouvrir le dossier</button>
        </div>
      </section>
    </div>
  );
}
