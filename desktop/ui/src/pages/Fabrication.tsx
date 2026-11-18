import { useEffect, useState } from "react";
import { api, type FicheLocalite } from "../api";
import type { EntreeFabrication, FabricationVue, SortieFabrication } from "../fabrication-types";

const ISSUE: Record<NonNullable<SortieFabrication["issue"]>, string> = {
  reussi: "Réussi : l'objet fonctionne.",
  echec: "Échec : temps et composantes perdus.",
  maudit: "Échec de 5 ou plus : l'objet est maudit (table 15-27 non modélisée).",
};

const entierVide = (v: string) => (v.trim() === "" ? 0 : Math.max(0, Math.floor(Number(v))));

/** Fabrication d'objet magique (Manuel des joueurs, ch. 15 ; Guide de campagne p. 85).
 *  Chaque « Enregistrer la journée » décompte un jour d'intermède et la Magie / Marchandises
 *  de la localité, et est inscrit au journal de la campagne. */
export function Fabrication() {
  const [localites, setLocalites] = useState<FicheLocalite[]>([]);
  const [localite, setLocalite] = useState("");
  const [cle, setCle] = useState("");
  const [nom, setNom] = useState("");
  const [prix, setPrix] = useState("1000");
  const [lanceur, setLanceur] = useState("5");
  const [conditions, setConditions] = useState("0");
  const [potion, setPotion] = useState(false);
  const [acceleree, setAcceleree] = useState(false);
  const [po, setPo] = useState("0");
  const [marchandises, setMarchandises] = useState("0");
  const [magie, setMagie] = useState("0");
  const [jet, setJet] = useState("");
  const [sortie, setSortie] = useState<SortieFabrication | null>(null);
  const [liste, setListe] = useState<FabricationVue[]>([]);
  const [message, setMessage] = useState<{ ok: boolean; texte: string } | null>(null);
  const [erreur, setErreur] = useState<string | null>(null);

  const chargerListe = () => api.fabricationsListe().then(setListe).catch(() => setListe([]));

  useEffect(() => {
    api.pageLocalites().then((p) => {
      if (!p) return;
      setLocalites(p.localites);
      setLocalite((l) => l || String(p.localites[0]?.id ?? ""));
    });
    chargerListe();
  }, []);

  // Aperçu du calcul, à chaque changement.
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
      jours_faits: 0,
      jet: jet.trim() === "" ? null : Number(jet),
    };
    api
      .fabricationCalculer(entree)
      .then((s) => {
        setSortie(s);
        setErreur(null);
      })
      .catch((e) => setErreur(String(e)));
  }, [prix, lanceur, conditions, potion, acceleree, po, marchandises, magie, jet]);

  const enregistrer = () => {
    if (!nom.trim() || !localite) {
      setMessage({ ok: false, texte: "Donne un nom à l'objet et une localité." });
      return;
    }
    const clef = cle || `fab-${Date.now()}`;
    api
      .fabricationAvancer({
        cle: clef,
        nom: nom.trim(),
        localite: Number(localite),
        prix_po: entierVide(prix),
        niveau_lanceur: entierVide(lanceur),
        conditions_manquantes: entierVide(conditions),
        potion_ou_parchemin: potion,
        acceleree,
        po_payes: entierVide(po),
        marchandises: entierVide(marchandises),
        magie: entierVide(magie),
        jet: jet.trim() === "" ? null : Number(jet),
      })
      .then((r) => {
        setMessage({ ok: r.ok, texte: r.message });
        if (r.ok) setCle(clef);
        chargerListe();
      })
      .catch((e) => setMessage({ ok: false, texte: String(e) }));
  };

  const reprendre = (f: FabricationVue) => {
    setCle(f.cle);
    setNom(f.nom);
    setMessage(null);
  };

  return (
    <section className="page fabrication">
      <h1>Fabrication d'objet magique</h1>
      <p className="aide">
        Manuel des joueurs, ch. 15 (p. 554-555) ; Guide de campagne p. 85. Les jours et les ressources sont inscrits au journal de la campagne.
      </p>

      <h2>Objet</h2>
      <div className="grille-formulaire">
        <label>Nom de l'objet<input value={nom} onChange={(e) => setNom(e.target.value)} /></label>
        <label>Localité
          <select value={localite} onChange={(e) => setLocalite(e.target.value)}>
            {localites.map((l) => (
              <option key={l.id} value={l.id}>{l.nom}</option>
            ))}
          </select>
        </label>
        <label>Prix de base (po)<input value={prix} onChange={(e) => setPrix(e.target.value)} inputMode="numeric" /></label>
        <label>Niveau de lanceur de sorts<input value={lanceur} onChange={(e) => setLanceur(e.target.value)} inputMode="numeric" /></label>
        <label>Conditions de création non remplies<input value={conditions} onChange={(e) => setConditions(e.target.value)} inputMode="numeric" /></label>
        <label className="case"><input type="checkbox" checked={potion} onChange={(e) => setPotion(e.target.checked)} /> Potion ou parchemin</label>
        <label className="case"><input type="checkbox" checked={acceleree} onChange={(e) => setAcceleree(e.target.checked)} /> Accélérée (DD +5, temps divisé par deux)</label>
      </div>

      <h2>Paiement (au premier jour, depuis la localité)</h2>
      <div className="grille-formulaire">
        <label>Po<input value={po} onChange={(e) => setPo(e.target.value)} inputMode="numeric" /></label>
        <label>Marchandises (20 po l'unité)<input value={marchandises} onChange={(e) => setMarchandises(e.target.value)} inputMode="numeric" /></label>
        <label>Magie (100 po le point)<input value={magie} onChange={(e) => setMagie(e.target.value)} inputMode="numeric" /></label>
      </div>

      <h2>Test final</h2>
      <div className="grille-formulaire">
        <label>Résultat du d20 + bonus (à saisir une fois le temps fait)<input value={jet} onChange={(e) => setJet(e.target.value)} inputMode="numeric" /></label>
      </div>

      {erreur && <p className="alerte">{erreur}</p>}
      {sortie && (
        <div className="bilan">
          <p><strong>DD du test final :</strong> {sortie.dd}</p>
          <p><strong>Temps :</strong> {sortie.heures_requises} h, soit {sortie.jours_requis} jour(s) d'intermède.</p>
          <p><strong>Fournitures :</strong> {(sortie.fournitures_pa / 10).toLocaleString("fr-FR")} po.</p>
          <p className={sortie.suffisant ? "" : "alerte"}>
            <strong>Paiement :</strong> {(sortie.paiement_pa / 10).toLocaleString("fr-FR")} po {sortie.suffisant ? "(suffisant)" : "(insuffisant)"}
          </p>
          {sortie.issue && <p><strong>{ISSUE[sortie.issue]}</strong></p>}
        </div>
      )}

      <div className="actions">
        <button className="bouton principal" onClick={enregistrer}>Enregistrer une journée de fabrication</button>
        {cle && <button className="bouton" onClick={() => { setCle(""); setNom(""); setMessage(null); }}>Nouvel objet</button>}
      </div>
      {message && <p className={message.ok ? "" : "alerte"}>{message.texte}</p>}

      <h2>Fabrications en cours</h2>
      {liste.length === 0 && <p className="aide">Aucune fabrication enregistrée.</p>}
      <ul>
        {liste.map((f) => (
          <li key={f.cle}>
            <button className="bouton" onClick={() => reprendre(f)}>{f.nom}</button>{" "}
            {f.heures_faites}/{f.heures_requises} h
            {f.terminee ? (f.issue ? ` : ${ISSUE[f.issue]}` : " : test final à saisir") : ""}
          </li>
        ))}
      </ul>
    </section>
  );
}
