import { useEffect, useState } from "react";
import { api } from "../api";
import type { EntreeFabrication, SortieFabrication } from "../fabrication-types";

const ISSUE: Record<NonNullable<SortieFabrication["issue"]>, string> = {
  reussi: "Réussi : l'objet fonctionne.",
  echec: "Échec : temps et composantes perdus.",
  maudit: "Échec de 5 ou plus : l'objet est maudit (table 15-27 non modélisée).",
};

const entierVide = (v: string) => (v.trim() === "" ? 0 : Math.max(0, Math.floor(Number(v))));

/** Calcul d'une fabrication d'objet magique (Manuel des joueurs, ch. 15 ; Guide de campagne p. 85). */
export function Fabrication() {
  const [prix, setPrix] = useState("1000");
  const [lanceur, setLanceur] = useState("5");
  const [conditions, setConditions] = useState("0");
  const [potion, setPotion] = useState(false);
  const [acceleree, setAcceleree] = useState(false);
  const [po, setPo] = useState("0");
  const [marchandises, setMarchandises] = useState("0");
  const [magie, setMagie] = useState("0");
  const [jours, setJours] = useState("0");
  const [jet, setJet] = useState("");
  const [sortie, setSortie] = useState<SortieFabrication | null>(null);
  const [erreur, setErreur] = useState<string | null>(null);

  useEffect(() => {
    const entree: EntreeFabrication = {
      prix_po: entierVide(prix),
      niveau_lanceur: entierVide(lanceur),
      conditions_manquantes: entierVide(conditions),
      potion_ou_parchemin: potion,
      acceleree,
      po_payes: entierVide(po),
      marchandises: entierVide(marchandises),
      magie: entierVide(magie),
      jours_faits: entierVide(jours),
      jet: jet.trim() === "" ? null : Number(jet),
    };
    api
      .fabricationCalculer(entree)
      .then((s) => {
        setSortie(s);
        setErreur(null);
      })
      .catch((e) => setErreur(String(e)));
  }, [prix, lanceur, conditions, potion, acceleree, po, marchandises, magie, jours, jet]);

  return (
    <section className="page fabrication">
      <h1>Fabrication d'objet magique</h1>
      <p className="aide">
        Calcul seul : rien n'est enregistré dans la campagne. Manuel des joueurs, ch. 15 (p. 554-555) ; Guide de campagne p. 85.
      </p>
      <div className="grille-formulaire">
        <label>Prix de base (po)<input value={prix} onChange={(e) => setPrix(e.target.value)} inputMode="numeric" /></label>
        <label>Niveau de lanceur de sorts<input value={lanceur} onChange={(e) => setLanceur(e.target.value)} inputMode="numeric" /></label>
        <label>Conditions de création non remplies<input value={conditions} onChange={(e) => setConditions(e.target.value)} inputMode="numeric" /></label>
        <label className="case"><input type="checkbox" checked={potion} onChange={(e) => setPotion(e.target.checked)} /> Potion ou parchemin</label>
        <label className="case"><input type="checkbox" checked={acceleree} onChange={(e) => setAcceleree(e.target.checked)} /> Accélérée (DD +5, temps divisé par deux)</label>
      </div>
      <h2>Paiement des fournitures</h2>
      <div className="grille-formulaire">
        <label>Po<input value={po} onChange={(e) => setPo(e.target.value)} inputMode="numeric" /></label>
        <label>Marchandises (20 po l'unité)<input value={marchandises} onChange={(e) => setMarchandises(e.target.value)} inputMode="numeric" /></label>
        <label>Magie (100 po le point)<input value={magie} onChange={(e) => setMagie(e.target.value)} inputMode="numeric" /></label>
      </div>
      <h2>Avancement</h2>
      <div className="grille-formulaire">
        <label>Jours d'intermède déjà passés<input value={jours} onChange={(e) => setJours(e.target.value)} inputMode="numeric" /></label>
        <label>Résultat du test final (d20 + bonus)<input value={jet} onChange={(e) => setJet(e.target.value)} inputMode="numeric" placeholder="à saisir à la fin" /></label>
      </div>
      {erreur && <p className="alerte">{erreur}</p>}
      {sortie && (
        <div className="bilan">
          <p><strong>DD du test final :</strong> {sortie.dd}</p>
          <p><strong>Temps :</strong> {sortie.heures_requises} h, soit {sortie.jours_requis} jour(s) d'intermède. Reste {sortie.jours_restants} jour(s).</p>
          <p><strong>Fournitures :</strong> {(sortie.fournitures_pa / 10).toLocaleString("fr-FR")} po, payées au début.</p>
          <p className={sortie.suffisant ? "" : "alerte"}>
            <strong>Paiement :</strong> {(sortie.paiement_pa / 10).toLocaleString("fr-FR")} po {sortie.suffisant ? "(suffisant)" : "(insuffisant)"}
          </p>
          {sortie.terminee ? <p><strong>Fabrication terminée.</strong></p> : <p>Fabrication en cours.</p>}
          {sortie.issue && <p><strong>{ISSUE[sortie.issue]}</strong></p>}
        </div>
      )}
    </section>
  );
}
