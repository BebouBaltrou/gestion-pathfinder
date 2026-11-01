import { useEffect, useState } from "react";
import { api, NoeudOrga, Organigramme as OrganigrammeDonnees } from "../api";

const LIBELLES: Record<string, string> = {
  joueur: "Joueur",
  localite: "Localité",
  batiment: "Bâtiment",
  organisation: "Organisation",
  gestionnaire: "Gestionnaire",
};

/** Un élément de l'arbre, avec ce qui lui est rattaché en dessous. */
function Ligne({ n }: { n: NoeudOrga }) {
  return (
    <li className={"orga-" + n.genre}>
      <span className="doux">{LIBELLES[n.genre] ?? n.genre}</span>
      <strong>{n.nom}</strong>
      {n.enfants.length > 0 && (
        <ul>
          {n.enfants.map((e) => (
            <Ligne key={e.genre + e.id} n={e} />
          ))}
        </ul>
      )}
    </li>
  );
}

export function Organigramme() {
  const [donnees, setDonnees] = useState<OrganigrammeDonnees | null>(null);

  useEffect(() => {
    api.organigramme().then(setDonnees);
  }, []);

  if (!donnees?.global) return null;

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Organigramme</h1>
        <p>Qui gère quoi, localité par localité. Le personnage du joueur est au sommet.</p>
      </div>

      <div className="carte racine-orga">
        <span className="doux">Racine</span>
        <strong>{donnees.global.nom}</strong>
        <span className="doux">Compte de campagne</span>
      </div>

      <div className="grille-orga">
        {donnees.localites.map((l) => (
          <article className="carte colonne-orga" key={l.id}>
            <h2>{l.nom}</h2>
            {l.enfants.length === 0 ? (
              <p className="doux">Aucun bâtiment, organisation ni gestionnaire.</p>
            ) : (
              <ul className="arbre">
                {l.enfants.map((e) => (
                  <Ligne key={e.genre + e.id} n={e} />
                ))}
              </ul>
            )}
          </article>
        ))}
      </div>
    </div>
  );
}
