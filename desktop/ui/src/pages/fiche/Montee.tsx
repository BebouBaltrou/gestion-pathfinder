import { useEffect, useState } from "react";
import type { Outils } from "../../App";
import { api } from "../../api";
import {
  ApercuMontee,
  CARACS,
  EssaiMontee,
  FicheVue,
  LignePersonnage,
  Montee as DonneesMontee,
  NOMS_CARAC,
} from "../../fiche-types";
import { signe, useCatalogue } from "./commun";
import { SelectionDons, SelectionRangs, SelectionSorts } from "./selections";

/** Assistant de montée de niveau : tout est contrôlé par le cœur avant de valider. */
export function Montee({
  outils,
  pj,
  fiche,
  onFerme,
}: {
  outils: Outils;
  pj: LignePersonnage;
  fiche: FicheVue;
  onFerme: () => void;
}) {
  const cat = useCatalogue();
  const [classe, setClasse] = useState("");
  const [apercu, setApercu] = useState<ApercuMontee | null>(null);
  const [de, setDe] = useState("");
  const [rangs, setRangs] = useState<Record<string, number>>({});
  const [predilection, setPredilection] = useState<"PointDeVie" | "Rang" | null>(null);
  const [dons, setDons] = useState<string[]>([]);
  const [augmentation, setAugmentation] = useState<(typeof CARACS)[number] | "">("");
  const [sorts, setSorts] = useState<string[]>([]);
  const [essai, setEssai] = useState<EssaiMontee | null>(null);

  useEffect(() => {
    if (cat && !classe) setClasse(cat.classes[0]?.id ?? "");
  }, [cat, classe]);

  useEffect(() => {
    if (classe) api.apercuMontee(pj.id, classe).then(setApercu);
  }, [classe, pj.id]);

  const montee = (): DonneesMontee | null => {
    if (!apercu) return null;
    const dv = apercu.deViePropose ?? Number(de);
    return {
      classe,
      de_vie: Number.isFinite(dv) && dv > 0 ? dv : 0,
      rangs,
      predilection: apercu.predilectionPossible ? predilection : null,
      dons: dons.filter(Boolean),
      augmentation: apercu.augmentation && augmentation ? augmentation : null,
      sorts_appris: sorts,
      echange: null,
    };
  };

  useEffect(() => {
    const m = montee();
    if (!m) return;
    const t = window.setTimeout(() => api.essaiMontee(pj.id, m).then(setEssai), 200);
    return () => window.clearTimeout(t);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [apercu, de, rangs, predilection, dons, augmentation, sorts]);

  if (!cat || !apercu) return null;
  const sortsAttendus = apercu.sortsConnusGagnes;
  const rangsDisponibles = apercu.rangsAPlacer + (predilection === "Rang" ? 1 : 0);
  const dejaPlaces: Record<string, number> = {};
  fiche.competences.forEach((c) => (dejaPlaces[c.cle] = c.rangs));
  const valider = async () => {
    const m = montee();
    if (!m) return;
    const r = await outils.agir(api.evenementFiche(pj.id, { MonteeDeNiveau: m }));
    if (r?.ok) onFerme();
  };
  const prete = essai !== null && essai.erreurs.length === 0;

  return (
    <div className="fond-modal" role="dialog" aria-modal="true" aria-label="Montée de niveau">
      <div className="modal carte">
        <div className="section-tete">
          <h2>
            Niveau {apercu.niveauDePersonnage} — {cat.classes.find((c) => c.id === classe)?.nom} {apercu.niveauDeClasse}
          </h2>
          <span className="espace" />
          <button className="bouton" onClick={onFerme}>Fermer</button>
        </div>
        {!apercu.niveauAtteint && (
          <p className="alerte-texte">
            Ce niveau n'est pas encore atteint (niveau permis : {apercu.niveauAtteignable}).
          </p>
        )}
        {cat.classes.length > 1 && (
          <select className="champ choix" aria-label="Classe" value={classe} onChange={(e) => setClasse(e.target.value)}>
            {cat.classes.map((c) => <option key={c.id} value={c.id}>{c.nom}</option>)}
          </select>
        )}

        <div className="grille-fiche">
          {[
            ["Bonus de base à l'attaque", apercu.bba],
            ["Vigueur", apercu.vigueur],
            ["Réflexes", apercu.reflexes],
            ["Volonté", apercu.volonte],
          ].map(([nom, v]) => (
            <div className="carte chiffre-fiche" key={String(nom)}>
              <span className="etiquette">{String(nom)}</span>
              <span className="valeur">
                {signe((v as [number, number])[0])} → {signe((v as [number, number])[1])}
              </span>
            </div>
          ))}
        </div>
        {apercu.capacites.length > 0 && (
          <div>
            <h3>Nouvelles capacités</h3>
            <ul>{apercu.capacites.map((c, i) => <li key={i}>{c}</li>)}</ul>
          </div>
        )}

        <h3>Points de vie</h3>
        {apercu.deViePropose !== null ? (
          <p>Dé de vie d{apercu.deVie} : <strong>{apercu.deViePropose}</strong> (méthode de la campagne, voir Paramètres)</p>
        ) : (
          <div className="ligne">
            <label htmlFor="de-vie">Résultat du d{apercu.deVie}</label>
            <input id="de-vie" className="champ court" inputMode="numeric" value={de} onChange={(e) => setDe(e.target.value)} />
          </div>
        )}

        {apercu.predilectionPossible && (
          <>
            <h3>Classe de prédilection</h3>
            <div className="groupe-boutons">
              <button className={"bouton" + (predilection === "PointDeVie" ? " principal" : "")} onClick={() => setPredilection("PointDeVie")}>+1 point de vie</button>
              <button className={"bouton" + (predilection === "Rang" ? " principal" : "")} onClick={() => setPredilection("Rang")}>+1 rang de compétence</button>
            </div>
          </>
        )}

        {apercu.augmentation && (
          <>
            <h3>Augmentation de caractéristique (+1)</h3>
            <div className="groupe-boutons">
              {CARACS.map((c) => (
                <button key={c} className={"bouton" + (augmentation === c ? " principal" : "")} onClick={() => setAugmentation(c)}>
                  {NOMS_CARAC[c]}
                </button>
              ))}
            </div>
          </>
        )}

        {apercu.donsAChoisir > 0 && (
          <>
            <h3>Don{apercu.donsAChoisir > 1 ? "s" : ""} à choisir</h3>
            <SelectionDons cat={cat} nombre={apercu.donsAChoisir} valeur={dons} onChange={setDons} />
          </>
        )}

        {sortsAttendus.length > 0 && (
          <>
            <h3>Nouveaux sorts connus</h3>
            <SelectionSorts
              sorts={cat.sortsEnsorceleur}
              attendus={sortsAttendus}
              valeur={sorts}
              onChange={setSorts}
              exclus={fiche.incantateurs.flatMap((i) => i.sorts.map((s) => s.nomEn))}
            />
          </>
        )}

        <h3>Compétences</h3>
        <SelectionRangs
          cat={cat}
          apercu={essai?.apres ?? fiche}
          rangs={rangs}
          onChange={setRangs}
          disponibles={rangsDisponibles}
          maximum={apercu.niveauDePersonnage}
          dejaPlaces={dejaPlaces}
        />

        {essai && essai.erreurs.length > 0 && (
          <section className="alerte grave" role="alert">
            <span className="point" />
            <div className="texte">
              <strong>À corriger avant de valider :</strong>
              <ul>{essai.erreurs.map((m, i) => <li key={i}>{m}</li>)}</ul>
            </div>
          </section>
        )}
        {essai?.apres && essai.apres.problemes.length > 0 && (
          <p className="doux">Après ce niveau, la fiche aura encore : {essai.apres.problemes.join(" ; ")}</p>
        )}
        {essai?.apres && (
          <p className="doux">
            Après ce niveau : {essai.apres.pv.total} PV · CA {essai.apres.ca.total} · Vigueur {signe(essai.apres.vigueur.total)} · Réflexes{" "}
            {signe(essai.apres.reflexes.total)} · Volonté {signe(essai.apres.volonte.total)}
          </p>
        )}
        <div className="ligne">
          <button className="bouton principal" disabled={!prete} onClick={valider}>Valider ce niveau</button>
          <button className="bouton" onClick={onFerme}>Annuler</button>
        </div>
      </div>
    </div>
  );
}
