import { useEffect, useState } from "react";
import type { Outils } from "../../App";
import { api } from "../../api";
import type { FicheVue } from "../../fiche-types";
import type { CommandeInventaire, EmplacementVue, InventaireVue, ObjetVue } from "../../inventaire-types";
import "../../equipement.css";
import { ListeVirtuelle } from "./ListeVirtuelle";

/** Disposition autour du mannequin (présentation seulement) : colonne de gauche, colonne de droite, rangée du bas. */
const GAUCHE = ["tete", "visage", "cou", "epaules", "torse"];
const DROITE = ["corps", "poignets", "mains", "taille", "pieds"];
const BAS = ["main_directrice", "anneau", "main_non_directrice"];
const PLACES = [...GAUCHE, ...DROITE, ...BAS];

/** Pictogrammes au trait (24 × 24), un par emplacement. */
const GLYPHES: Record<string, string> = {
  tete: "M4 15a8 8 0 0 1 16 0v2H4zM9 17v3M15 17v3",
  visage: "M3 10h18v4a3 3 0 0 1-3 3h-2l-2-2h-4l-2 2H6a3 3 0 0 1-3-3z",
  cou: "M6 4c0 6 3 9 6 9s6-3 6-9M12 13v3m0 0l-2 2 2 3 2-3z",
  epaules: "M7 4h10l3 16H4zM12 4v16",
  torse: "M8 4L3 8l2 4 3-1v9h8v-9l3 1 2-4-5-4a4 4 0 0 1-8 0z",
  corps: "M5 5l7-2 7 2v8c0 4-3 6-7 8-4-2-7-4-7-8z",
  mains: "M8 21v-6l-3-4 2-1 2 2V5h2v6V4h2v7V5h2v7V8h2v8c0 3-2 5-5 5z",
  poignets: "M6 8h12v8H6zM6 12h12",
  main_directrice: "M14 4l6-1-1 6-9 9-3-3zM7 16l-3 3M5 14l5 5",
  main_non_directrice: "M12 3l8 3v6c0 5-4 8-8 9-4-1-8-4-8-9V6z",
  anneau: "M12 9a6 6 0 1 0 0 12 6 6 0 0 0 0-12zM10 9L8 4h8l-2 5",
  taille: "M3 9h18v6H3zM10 9v6h4V9",
  pieds: "M8 3h6v9l5 3v5H5v-4l3-2z",
};
const GLYPHE_DEFAUT = "M12 3l9 9-9 9-9-9z";

function Pictogramme({ id }: { id: string }) {
  return (
    <svg viewBox="0 0 24 24" className="eq-pict" aria-hidden="true" focusable="false">
      <path d={GLYPHES[id] ?? GLYPHE_DEFAUT} />
    </svg>
  );
}

/** Mannequin stylisé : un corps d'aventurier, sans illustration travaillée. */
function Mannequin() {
  return (
    <svg className="eq-mannequin" viewBox="0 0 200 440" aria-hidden="true" focusable="false">
      <circle cx="100" cy="48" r="30" />
      <rect x="88" y="76" width="24" height="20" rx="7" />
      <path d="M54 104 Q100 90 146 104 L140 202 Q100 214 60 202 Z" />
      <rect x="22" y="104" width="26" height="176" rx="13" transform="rotate(5 35 190)" />
      <rect x="152" y="104" width="26" height="176" rx="13" transform="rotate(-5 165 190)" />
      <path d="M62 198 Q100 212 138 198 L142 238 Q100 252 58 238 Z" />
      <rect x="60" y="232" width="36" height="170" rx="17" />
      <rect x="104" y="232" width="36" height="170" rx="17" />
      <rect x="54" y="398" width="46" height="30" rx="13" />
      <rect x="100" y="398" width="46" height="30" rx="13" />
    </svg>
  );
}

/** Mannequin entouré de cases par emplacement, réserve d'objets à équiper, charge, bonus et jeux d'équipement. */
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
  const par = (id: string) => emplacements.find((e) => e.id === id);
  const colonne = (ids: string[]) => ids.map(par).filter((e): e is EmplacementVue => !!e);
  const autres = emplacements.filter((e) => !PLACES.includes(e.id));
  const garnis = emplacements.filter((e) => occupants(e.id).length > 0).length;

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

  const caseSlot = (slot: EmplacementVue, cote: "gauche" | "droite" | "bas" | "libre") => {
    const ici = occupants(slot.id);
    const indicateur = etat(slot);
    return (
      <div
        key={slot.id}
        className={`eq-slot ${cote} ${indicateur?.classe ?? ""}${ici.length > 0 ? " garnie" : ""}`}
        onDragOver={(e) => indicateur?.classe === "compatible" && e.preventDefault()}
        onDrop={(e) => deposer(e, slot)}
        aria-label={`${slot.zone} : ${slot.nom}`}
      >
        <div className="eq-ico"><Pictogramme id={slot.id} /></div>
        <div className="eq-texte">
          <div className="eq-zone">{slot.zone}</div>
          <div className="eq-nom">{slot.nom}</div>
        </div>
        {slot.capacite > 1 && (
          <div className="eq-pions" aria-label={`${ici.length} sur ${slot.capacite}`}>
            {Array.from({ length: slot.capacite }, (_, i) => <i key={i} className={i < ici.length ? "plein" : ""} />)}
          </div>
        )}
        {ici.map((o) => (
          <div key={o.id} className="eq-objet">
            <span>{o.nom}{o.colle && " ⚠"}</span>
            <button className="eq-retirer" disabled={o.colle} onClick={() => cmd({ Deequiper: { id: o.id } }, `${o.nom} déséquipé`)} aria-label={`Déséquiper ${o.nom}`} title="Retirer">
              ✕
            </button>
          </div>
        ))}
        {ici.length === 0 && <div className="eq-vide">{glisse ? "Déposer ici" : "Libre"}</div>}
        {indicateur && <div className={"eq-etat " + indicateur.classe}>{indicateur.texte}</div>}
      </div>
    );
  };

  const eq = fiche?.equipement;
  const charge = inventaire.charge;
  const [leg, inter, lourde] = charge?.limitesLb ?? [0, 0, 0];
  const pourcent = charge && lourde > 0 ? Math.min(100, Math.round((charge.poidsLb / lourde) * 100)) : 0;
  const part = (a: number, b: number) => (lourde > 0 ? `${((b - a) / lourde) * 100}%` : "0%");

  const mots = recherche.toLowerCase().split(/\s+/).filter(Boolean);
  const candidats = inventaire.objets.filter(
    (o) => o.equipe.length === 0 && o.emplacementsPossibles.length > 0 && mots.every((m) => o.nom.toLowerCase().includes(m)),
  );
  const nomSlot = (id: string) => emplacements.find((e) => e.id === id)?.nom ?? id;

  return (
    <div className="eq">
      <div className="eq-haut">
        <section className="eq-scene" aria-label="Emplacements du corps">
          <div className="eq-bandeau">
            <span><b>{garnis}</b> / {emplacements.length} emplacements garnis</span>
            {glisse && <span className="eq-conseil">Déposez « {glisse.nom} » sur une case éclairée.</span>}
          </div>
          <div className="eq-plateau">
            <div className="eq-colonne gauche">{colonne(GAUCHE).map((s) => caseSlot(s, "gauche"))}</div>
            <div className="eq-corps">
              <Mannequin />
              {fiche && (
                <div className="eq-ca" title={fiche.ca.explication}>
                  <b>{fiche.ca.total}</b>
                  <small>Classe d'armure</small>
                </div>
              )}
            </div>
            <div className="eq-colonne droite">{colonne(DROITE).map((s) => caseSlot(s, "droite"))}</div>
            <div className="eq-bas">{colonne(BAS).map((s) => caseSlot(s, "bas"))}</div>
          </div>
          {autres.length > 0 && <div className="eq-autres">{autres.map((s) => caseSlot(s, "libre"))}</div>}
        </section>

        <section className="eq-reserve" aria-label="Objets à équiper">
          <h3>Dans l'inventaire</h3>
          <p className="doux">Glissez un objet sur une case du mannequin, ou touchez l'un de ses emplacements.</p>
          <input className="champ large" type="search" placeholder="Rechercher…" aria-label="Rechercher un objet à équiper" value={recherche} onChange={(e) => setRecherche(e.target.value)} />
          <div className="eq-compte">{candidats.length} objet(s) équipable(s)</div>
          <ListeVirtuelle
            elements={candidats}
            hauteurLigne={92}
            cle={(o) => o.id}
            vide="Aucun objet à équiper."
            rendu={(o) => (
              <div
                className="eq-carte"
                draggable
                onDragStart={(e) => {
                  e.dataTransfer.setData("text/plain", String(o.id));
                  e.dataTransfer.effectAllowed = "move";
                  onGlisser(o);
                }}
                onDragEnd={() => onGlisser(null)}
              >
                <span className="eq-carte-nom"><i className="eq-prise" aria-hidden="true">⋮⋮</i>{o.nom}</span>
                <span className="eq-chips">
                  {o.emplacementsPossibles.map((s) => (
                    <button key={s} className="eq-chip" onClick={() => cmd({ Equiper: { id: o.id, emplacement: s } }, `${o.nom} équipé`)} aria-label={`Équiper ${o.nom} : ${nomSlot(s)}`}>
                      <Pictogramme id={s} />
                      {nomSlot(s)}
                    </button>
                  ))}
                </span>
              </div>
            )}
          />
        </section>
      </div>

      <div className="eq-bas-panneaux">
        <section className="eq-panneau">
          <h3>Charge</h3>
          <p>
            {inventaire.poids.texte} portés
            {charge ? <> — charge <strong className={"eq-charge-nom " + charge.nom}>{charge.nom}</strong></> : " — pas de fiche : charge non évaluée"}
          </p>
          {charge && (
            <>
              <div className="eq-jauge" role="img" aria-label={`Charge ${charge.nom} : ${charge.poidsLb.toFixed(1)} lb sur ${lourde} lb`}>
                <span className="seg legere" style={{ width: part(0, leg) }} />
                <span className="seg intermediaire" style={{ width: part(leg, inter) }} />
                <span className="seg lourde" style={{ width: part(inter, lourde) }} />
                <span className="eq-marqueur" style={{ left: `${pourcent}%` }} />
              </div>
              <div className="eq-legende">
                <span className="legere">Légère ≤ {leg} lb</span>
                <span className="intermediaire">Intermédiaire ≤ {inter} lb</span>
                <span className="lourde">Lourde ≤ {lourde} lb</span>
              </div>
              <p className="doux">{charge.poidsLb.toLocaleString("fr-FR", { maximumFractionDigits: 1 })} lb portées sur {lourde} lb au maximum.</p>
            </>
          )}
        </section>

        <section className="eq-panneau">
          <h3>Bonus actifs</h3>
          {!fiche && <p className="doux">Les bonus viennent de la fiche du personnage.</p>}
          {fiche && (
            <ul className="eq-bonus">
              <li><span>Classe d'armure</span><strong>{fiche.ca.total}</strong><small className="doux">{fiche.ca.explication}</small></li>
              {eq && eq.malusDArmure !== 0 && <li><span>Malus d'armure</span><strong>{eq.malusDArmure}</strong><small className="doux">{eq.sourceMalus}</small></li>}
              {eq?.dexMax && <li><span>Bonus de Dextérité maximal</span><strong>{eq.dexMax}</strong></li>}
              {eq && eq.echecSortsPct > 0 && <li><span>Échec des sorts profanes</span><strong>{eq.echecSortsPct} %</strong></li>}
              {eq?.objetsMagiques.map((m) => (
                <li key={m.nom}><span>{m.nom}</span><small className="doux">{m.effets.join(", ")}</small></li>
              ))}
              {eq?.nonMaitrises.map((n) => (
                <li key={n} className="alerte-texte">⚠ Non maîtrisé : {n}</li>
              ))}
            </ul>
          )}
        </section>

        <section className="eq-panneau">
          <h3>Jeux d'équipement</h3>
          <p className="doux">Enregistrez ce que le personnage porte (mode furtif, mode voyage…) et basculez en un clic.</p>
          {inventaire.jeux.length > 0 && (
            <div className="eq-jeux" role="group" aria-label="Jeu enregistré">
              {inventaire.jeux.map((j) => (
                <button key={j} className={"eq-jeu" + (j === jeu ? " choisi" : "")} aria-pressed={j === jeu} onClick={() => setJeu(j)}>{j}</button>
              ))}
            </div>
          )}
          <div className="ligne-action">
            <button className="bouton principal" disabled={!jeu} onClick={() => cmd({ AppliquerJeu: { racine: inventaire.racine, nom: jeu } }, `Jeu « ${jeu} » appliqué`)}>
              {jeu ? `Appliquer « ${jeu} »` : "Aucun jeu enregistré"}
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
