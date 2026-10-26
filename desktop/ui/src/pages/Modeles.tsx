import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, PageModeles } from "../api";

/** Tous les modèles personnels (bâtiments et organisations), toutes campagnes confondues. */
export function Modeles({ outils }: { outils: Outils }) {
  const [page, setPage] = useState<PageModeles | null>(null);

  useEffect(() => {
    api.pageModeles().then(setPage);
  }, [outils.version]);

  const supprimer = async (id: string, nom: string, genre: "batiment" | "organisation") => {
    if (!window.confirm(`Supprimer le modèle « ${nom} » ? Les bâtiments déjà construits ne sont pas touchés.`)) return;
    const r = await outils.agir(genre === "batiment" ? api.supprimerModeleBatiment(id) : api.supprimerModeleOrganisation(id));
    if (r?.ok) setPage(null);
  };

  const groupes: [string, "batiment" | "organisation", PageModeles["batiments"]][] = page
    ? [
        ["Bâtiments", "batiment", page.batiments],
        ["Organisations", "organisation", page.organisations],
      ]
    : [];

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Modèles personnels</h1>
        <p>
          Modèles enregistrés depuis un bâtiment ou une organisation. Ils servent à construire à l'identique, dans
          toutes les campagnes.
        </p>
      </div>
      {groupes.map(([titre, genre, liste]) => (
        <section className="section" key={titre}>
          <h2>{titre}</h2>
          {liste.length === 0 && <p className="doux">Aucun modèle personnel.</p>}
          <div className="liste-regles">
            {liste.map((m) => (
              <div className="carte regle" key={m.id}>
                <div className="regle-texte">
                  <div className="regle-nom">
                    <strong>{m.nom}</strong>
                    <span className="puce or">personnel</span>
                  </div>
                  <p className="doux">{m.detail}</p>
                </div>
                <button type="button" className="bouton discret" onClick={() => supprimer(m.id, m.nom, genre)}>
                  Supprimer
                </button>
              </div>
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}
