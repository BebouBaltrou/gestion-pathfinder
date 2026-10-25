import { useEffect, useState } from "react";
import type { Outils } from "../../App";
import { api } from "../../api";
import type { FicheVue } from "../../fiche-types";
import type { CommandeInventaire, EmplacementVue, InventaireVue, ObjetVue } from "../../inventaire-types";
import { ListeVirtuelle } from "./ListeVirtuelle";

/** Place de chaque emplacement sur le schéma (colonne, ligne) : présentation seulement. */
const PLACES: Record<string, [number, number]> = {
  visage: [2, 1], tete: [3, 1], cou: [4, 1],
  epaules: [2, 2], torse: [4, 2],
  mains: [1, 3], corps: [3, 3], poignets: [5, 3],
  main_directrice: [1, 4], anneau: [2, 4], taille: [3, 4], main_non_directrice: [5, 4],
  pieds: [3, 5],
};

/** Silhouette stylisée, sans illustration travaillée. */
function Silhouette() {
  return (
    <svg className="silhouette" viewBox="0 0 120 300" aria-hidden="true" focusable="false">
      <circle cx="60" cy="34" r="22" />
      <rect x="44" y="58" width="32" height="14" rx="4" />
      <rect x="28" y="72" width="64" height="100" rx="16" />
      <rect x="10" y="76" width="16" height="96" rx="8" />
      <rect x="94" y="76" width="16" height="96" rx="8" />
      <rect x="34" y="172" width="22" height="110" rx="9" />
      <rect x="64" y="172" width="22" height="110" rx="9" />
    </svg>
  );
}

/** Schéma de cases autour d'un corps stylisé, résumé des bonus actifs et jeux d'équipement. */
export function Equipement(props: {
  outils: Outils;
  inventaire: InventaireVue;
  emplacements: EmplacementVue[];
  glisse: ObjetVue | null;
  onGlisser: (o: ObjetVue | null) => void;
  cmd: (c: CommandeInventaire, message: string) => void;
}) {
  const { outils, inventaire, emplacements, glisse, onGlisser, cmd } = props;
  const [fiche, setFiche] = useState<FicheVue | null>(null);
  const [recherche, setRecherche] = useState("");
  const [jeu, setJeu] = useState("");
  const [nouveauJeu, setNouveauJeu] = useState("");
  const pj = /^PJ-(\d+)$/.exec(inventaire.refProprietaire)?.[1];

  useEffect(() => {
    if (!pj) return setFiche(null);
    api.fichePersonnage(Number(pj)).then(setFiche, () => setFiche(null));
  }, [pj, outils.version]);

  useEffect(() => setJeu(inventaire.jeux[0] ?? ""), [inventaire.racine, inventaire.jeux.length]);

  const equipes = inventaire.objets.filter((o) => o.equipe.length > 0);
  const occupants = (slot: string) => equipes.filter((o) => o.equipe.includes(slot));
  const connus = emplacements.filter((e) => PLACES[e.id]);
  const autres = emplacements.filter((e) => !PLACES[e.id]);

  const deposer = (e: React.DragEvent, slot: EmplacementVue) => {
    e.preventDefault();
    const id = Number(e.dataTransfer.getData("text/plain"));
    onGlisser(null);
    const objet = inventaire.objets.find((o) => o.id === id);
    if (objet) cmd({ Equiper: { id, emplacement: slot.id } }, `${objet.nom} équipé`);
  };

  const etat = (slot: EmplacementVue) => {
    if (!glisse) return null;
    if (!glisse.emplacementsPossibles.includes(slot.id)) return { classe: "incompatible", texte: "✕ Incompatible" };
    if (slot.capacite > 0 && occupants(slot.id).length >= slot.capacite) return { classe: "occupe", texte: "● Occupé" };
    return { classe: "compatible", texte: "✓ Compatible" };
  };

  const caseSlot = (slot: EmplacementVue) => {
    const ici = occupants(slot.id);
    const indicateur = etat(slot);
    return (
      <div
        key={slot.id}
        className={"case-emplacement " + (indicateur?.classe ?? "")}
        style={PLACES[slot.id] ? { gridColumn: PLACES[slot.id][0], gridRow: PLACES[slot.id][1] } : undefined}
        onDragOver={(e) => indicateur?.classe === "compatible" && e.preventDefault()}
        onDrop={(e) => deposer(e, slot)}
        aria-label={`${slot.zone} : ${slot.nom}`}
      >
        <div className="zone">{slot.zone}</div>
        <div className="nom-slot">{slot.nom}{slot.capacite > 1 && ` (${ici.length}/${slot.capacite})`}</div>
        {ici.map((o) => (
          <div key={o.id} className="occupant">
            <span>{o.nom}{o.colle && " ⚠"}</span>
            <button className="bouton discret" disabled={o.colle} onClick={() => cmd({ Deequiper: { id: o.id } }, `${o.nom} déséquipé`)} aria-label={`Déséquiper ${o.nom}`}>
              Retirer
            </button>
          </div>
        ))}
        {ici.length === 0 && <div className="vide-slot">vide</div>}
        {indicateur && <div className={"indicateur " + indicateur.classe}>{indicateur.texte}</div>}
      </div>
    );
  };

  const eq = fiche?.equipement;
  const charge = inventaire.charge;
  const [leg, inter, lourde] = charge?.limitesLb ?? [0, 0, 0];
  const pourcent = charge && lourde > 0 ? Math.min(100, Math.round((charge.poidsLb / lourde) * 100)) : 0;

  const mots = recherche.toLowerCase().split(/\s+/).filter(Boolean);
  const candidats = inventaire.objets.filter(
    (o) => o.equipe.length === 0 && o.emplacementsPossibles.length > 0 && mots.every((m) => o.nom.toLowerCase().includes(m)),
  );
  const nomSlot = (id: string) => emplacements.find((e) => e.id === id)?.nom ?? id;

  return (
    <div className="equipement">
      <div className="schema-et-reserve">
        <div>
          <div className="schema-corps">
            <Silhouette />
            {connus.map(caseSlot)}
          </div>
          {autres.length > 0 && <div className="autres-emplacements">{autres.map(caseSlot)}</div>}
        </div>

        <section className="reserve-equipable" aria-label="Objets à équiper">
          <h3>Dans l'inventaire</h3>
          <p className="doux">Glissez un objet sur une case du schéma, ou utilisez les boutons.</p>
          <input className="champ large" type="search" placeholder="Rechercher…" aria-label="Rechercher un objet à équiper" value={recherche} onChange={(e) => setRecherche(e.target.value)} />
          <ListeVirtuelle
            elements={candidats}
            hauteurLigne={64}
            cle={(o) => o.id}
            vide="Aucun objet à équiper."
            rendu={(o) => (
              <div
                className="objet a-equiper"
                draggable
                onDragStart={(e) => {
                  e.dataTransfer.setData("text/plain", String(o.id));
                  e.dataTransfer.effectAllowed = "move";
                  onGlisser(o);
                }}
                onDragEnd={() => onGlisser(null)}
              >
                <span className="nom-objet">{o.nom}</span>
                <span className="boutons-slots">
                  {o.emplacementsPossibles.map((s) => (
                    <button key={s} className="bouton discret" onClick={() => cmd({ Equiper: { id: o.id, emplacement: s } }, `${o.nom} équipé`)} aria-label={`Équiper ${o.nom} : ${nomSlot(s)}`}>
                      {nomSlot(s)}
                    </button>
                  ))}
                </span>
              </div>
            )}
          />
        </section>
      </div>

      <div className="resume-equipement">
        <section className="carte encadre">
          <h3>Charge</h3>
          <p>
            {inventaire.poids.texte} portés
            {charge ? ` — charge ${charge.nom}` : " — pas de fiche : charge non évaluée"}
          </p>
          {charge && (
            <>
              <div className="barre-charge" role="img" aria-label={`Charge ${charge.nom} : ${charge.poidsLb.toFixed(1)} lb sur ${lourde} lb`}>
                <div className={"remplissage " + charge.nom} style={{ width: `${pourcent}%` }} />
                <span className="repere" style={{ left: `${(leg / lourde) * 100}%` }} title={`Charge légère jusqu'à ${leg} lb`} />
                <span className="repere" style={{ left: `${(inter / lourde) * 100}%` }} title={`Charge intermédiaire jusqu'à ${inter} lb`} />
              </div>
              <p className="doux">
                {charge.poidsLb.toLocaleString("fr-FR", { maximumFractionDigits: 1 })} lb — limites {leg} / {inter} / {lourde} lb (légère / intermédiaire / lourde)
              </p>
            </>
          )}
        </section>

        <section className="carte encadre">
          <h3>Bonus actifs</h3>
          {!fiche && <p className="doux">Les bonus viennent de la fiche du personnage.</p>}
          {fiche && (
            <ul className="liste-bonus">
              <li>Classe d'armure : <strong>{fiche.ca.total}</strong> <span className="doux">{fiche.ca.explication}</span></li>
              {eq && eq.malusDArmure !== 0 && <li>Malus d'armure : {eq.malusDArmure} <span className="doux">({eq.sourceMalus})</span></li>}
              {eq?.dexMax && <li>Bonus de Dextérité maximal : {eq.dexMax}</li>}
              {eq && eq.echecSortsPct > 0 && <li>Échec des sorts profanes : {eq.echecSortsPct} %</li>}
              {eq?.objetsMagiques.map((m) => (
                <li key={m.nom}>{m.nom} : <span className="doux">{m.effets.join(", ")}</span></li>
              ))}
              {eq?.nonMaitrises.map((n) => (
                <li key={n} className="alerte-texte">⚠ Non maîtrisé : {n}</li>
              ))}
            </ul>
          )}
        </section>

        <section className="carte encadre">
          <h3>Jeux d'équipement</h3>
          <p className="doux">Enregistrez ce que le personnage porte (mode furtif, mode voyage…) et basculez en un clic.</p>
          <div className="ligne-action">
            <select className="champ" aria-label="Jeu enregistré" value={jeu} onChange={(e) => setJeu(e.target.value)} disabled={inventaire.jeux.length === 0}>
              {inventaire.jeux.length === 0 && <option value="">Aucun jeu enregistré</option>}
              {inventaire.jeux.map((j) => (
                <option key={j} value={j}>{j}</option>
              ))}
            </select>
            <button className="bouton principal" disabled={!jeu} onClick={() => cmd({ AppliquerJeu: { racine: inventaire.racine, nom: jeu } }, `Jeu « ${jeu} » appliqué`)}>
              Appliquer
            </button>
          </div>
          <div className="ligne-action">
            <input className="champ moyen" placeholder="Nom du nouveau jeu" aria-label="Nom du nouveau jeu" value={nouveauJeu} onChange={(e) => setNouveauJeu(e.target.value)} />
            <button
              className="bouton"
              disabled={!nouveauJeu.trim()}
              onClick={() => {
                cmd({ EnregistrerJeu: { racine: inventaire.racine, nom: nouveauJeu.trim() } }, `Jeu « ${nouveauJeu.trim()} » enregistré`);
                setNouveauJeu("");
              }}
            >
              Enregistrer l'équipement actuel
            </button>
          </div>
        </section>
      </div>
    </div>
  );
}
