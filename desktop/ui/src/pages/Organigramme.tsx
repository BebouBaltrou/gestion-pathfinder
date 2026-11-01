import { useEffect, useState } from "react";
import { api, CartePersonnage, formaterPo, NoeudOrga, Organigramme as OrganigrammeDonnees } from "../api";
import { Portrait } from "../composants/Portrait";

const CLE = "gestion-pathfinder.personnage-joueur";

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
      <div className="noeud">
        <span className="doux">{LIBELLES[n.genre] ?? n.genre}</span>
        <strong>{n.nom}</strong>
      </div>
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

/** Même encart que sur le tableau de bord, pour le personnage choisi à la racine. */
function Encart({ p }: { p: CartePersonnage }) {
  return (
    <div className="carte carte-perso">
      <div className="perso-tete">
        <Portrait id={p.id} nom={p.nom} />
        <div className="perso-infos">
          <span className="perso-nom">{p.nom}</span>
          <span className="doux">{[p.race, p.classes].filter(Boolean).join(" – ") || "fiche non créée"}</span>
          {p.niveau !== null && <span className="doux">Niveau {p.niveau}</span>}
        </div>
      </div>
      <div className="puces">
        {p.meilleureCaracteristique && <span className="puce tag-carac">{p.meilleureCaracteristique}</span>}
        {p.meilleureCompetence && <span className="puce tag-comp">{p.meilleureCompetence}</span>}
      </div>
      <div className="perso-pied doux">
        {p.lieu ?? "en voyage / hors localité"} · {formaterPo(p.compte.pc)}
      </div>
    </div>
  );
}

function personnageEnregistre(): number | null {
  try {
    const v = window.localStorage.getItem(CLE);
    return v === null ? null : Number(v);
  } catch {
    return null;
  }
}

export function Organigramme() {
  const [donnees, setDonnees] = useState<OrganigrammeDonnees | null>(null);
  const [personnages, setPersonnages] = useState<CartePersonnage[]>([]);
  const [joueur, setJoueur] = useState<number | null>(personnageEnregistre);
  const [choix, setChoix] = useState(false);

  useEffect(() => {
    api.organigramme().then(setDonnees);
    api.tableauDeBord().then((t) => setPersonnages(t?.personnages ?? []));
  }, []);

  const choisir = (id: number) => {
    setJoueur(id);
    setChoix(false);
    try {
      window.localStorage.setItem(CLE, String(id));
    } catch {
      /* sans stockage, le choix ne dure que jusqu'à la fermeture */
    }
  };

  if (!donnees?.global) return null;
  const choisi = personnages.find((p) => p.id === joueur);

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Organigramme</h1>
        <p>Qui gère quoi, localité par localité. Votre personnage est au sommet.</p>
      </div>

      <div className="racine-orga">
        {choisi && !choix ? (
          <>
            <Encart p={choisi} />
            <button type="button" className="bouton discret" onClick={() => setChoix(true)}>
              Changer de personnage
            </button>
          </>
        ) : (
          <div className="carte racine-vide">
            <span className="doux">Racine</span>
            <strong>Sélectionnez votre personnage</strong>
            {personnages.length === 0 && <p className="doux">Aucun personnage pour l'instant.</p>}
          </div>
        )}
        {(!choisi || choix) && personnages.length > 0 && (
          <div className="choix-personnages">
            {personnages.map((p) => (
              <button key={p.id} type="button" className="choix-perso" onClick={() => choisir(p.id)}>
                <Encart p={p} />
              </button>
            ))}
          </div>
        )}
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
