// Carte d'un personnage dans l'aventure ou le combat : santé, états, effets, emplacements de sorts,
// ressources quotidiennes et objets. Les boutons d'action sont fournis par la page.

import { ReactNode } from "react";
import type { EffetVue, ObjetVue, PjAventure, RessourceVue } from "../../aventure-types";
import { EtatsPuces, Gemmes, JaugePv, ListeEffets, PastilleSante } from "../../composants/jeu";

export function CartePj({
  pj,
  actif = false,
  grandeJauge = false,
  onRetirerEffet,
  onUtiliserObjet,
  onUtiliserRessource,
  onRendreRessource,
  onSante,
  onEffet,
  onObjet,
  onActualiserRessources,
  entete,
  pied,
}: {
  pj: PjAventure;
  actif?: boolean;
  grandeJauge?: boolean;
  onRetirerEffet?: (e: EffetVue) => void;
  onUtiliserObjet?: (o: ObjetVue) => void;
  onUtiliserRessource?: (r: RessourceVue) => void;
  onRendreRessource?: (r: RessourceVue) => void;
  onSante?: (mode: "degats" | "soin" | "temp" | "continu") => void;
  onEffet?: () => void;
  onObjet?: () => void;
  onActualiserRessources?: () => void;
  entete?: ReactNode;
  pied?: ReactNode;
}) {
  const hors = pj.santeCode === "mort" || pj.santeCode === "mourant" || pj.santeCode === "inconscient";
  return (
    <article className={"carte carte-pj" + (actif ? " actif" : "") + (hors ? " hors-jeu" : "")} aria-label={`Fiche de jeu de ${pj.nom}`}>
      <div className="tete">
        <h2>{pj.nom}</h2>
        <span className="niveau">niv. {pj.niveau}</span>
        <PastilleSante code={pj.santeCode} />
        {entete}
      </div>
      <JaugePv pj={pj} grande={grandeJauge} />
      {onSante && (
        <div className="rangee-boutons">
          <button className="bouton mini" onClick={() => onSante("degats")}>Prendre des dégâts</button>
          <button className="bouton mini" onClick={() => onSante("soin")}>Se soigner</button>
          <button className="bouton mini" onClick={() => onSante("temp")}>PV temporaires</button>
          <button className={"bouton mini" + (pj.degatsContinus ? " danger" : "")} onClick={() => onSante("continu")}>
            {pj.degatsContinus ? "Dégâts continus : en cours" : "Dégâts continus"}
          </button>
        </div>
      )}
      <EtatsPuces etats={pj.etats} />
      <div className="bloc">
        <h3>Effets en cours</h3>
        <ListeEffets effets={pj.effets} onRetirer={onRetirerEffet} />
        {onEffet && (
          <div className="rangee-boutons">
            <button className="bouton mini" onClick={onEffet}>+ Appliquer un effet</button>
          </div>
        )}
      </div>
      {pj.caster && pj.emplacements.length > 0 && (
        <div className="bloc">
          <h3>Emplacements de sorts</h3>
          {pj.emplacements.map((e) => (
            <Gemmes key={e.niveau} niveau={e.niveau} restants={e.restants} total={e.total} dd={e.dd} />
          ))}
        </div>
      )}
      {(pj.ressources.length > 0 || onActualiserRessources) && (
        <div className="bloc">
          <h3>Pouvoirs à usages quotidiens</h3>
          <div className="liste-compacte">
            {pj.ressources.map((r) => (
              <div className="ligne-objet" key={r.cle}>
                <span className="nom" title={r.source}>{r.nom}</span>
                <span className="reste-compte">
                  {r.restantes}/{r.max} {r.unite}
                </span>
                {onUtiliserRessource && (
                  <button className="bouton mini" disabled={r.restantes === 0} onClick={() => onUtiliserRessource(r)}>
                    Utiliser
                  </button>
                )}
                {onRendreRessource && r.restantes < r.max && (
                  <button className="bouton mini discret" onClick={() => onRendreRessource(r)}>Rendre 1</button>
                )}
              </div>
            ))}
            {pj.ressources.length === 0 && <span className="doux">Aucune ressource suivie.</span>}
          </div>
          {onActualiserRessources && (
            <div className="rangee-boutons">
              <button className="bouton mini" onClick={onActualiserRessources} title="Calcule les usages d'après le lignage et le Charisme de la fiche">
                Calculer d'après la fiche
              </button>
            </div>
          )}
        </div>
      )}
      <div className="bloc">
        <h3>Objets</h3>
        <div className="liste-compacte">
          {pj.inventaire.length === 0 && <span className="doux">Aucun consommable.</span>}
          {pj.inventaire.map((o) => (
            <div className="ligne-objet" key={o.id}>
              <span className="nom">
                {o.nom}
                <span className="doux"> · {o.genre}{o.sort ? ` (${o.sort})` : ""}</span>
              </span>
              <span className="reste-compte">
                {o.charges ? `${o.charges[0]}/${o.charges[1]} charges` : `× ${o.quantite}`}
                {o.parJour ? ` · ${o.parJour[0]}/${o.parJour[1]} aujourd'hui` : ""}
              </span>
              {onUtiliserObjet && (
                <button className="bouton mini" onClick={() => onUtiliserObjet(o)}>
                  {o.genreCode === "munition" ? "Tirer 1" : "Utiliser"}
                </button>
              )}
            </div>
          ))}
        </div>
        {onObjet && (
          <div className="rangee-boutons">
            <button className="bouton mini" onClick={onObjet}>+ Ajouter un objet</button>
          </div>
        )}
      </div>
      {pied}
    </article>
  );
}
