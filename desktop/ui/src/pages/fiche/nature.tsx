import { useState } from "react";
import {
  CARACS,
  NOMS_CARAC,
  type CatalogueCreation,
  type ClasseVue,
  type EvenementFiche,
  type FicheDonnees,
  type FicheVue,
} from "../../fiche-types";
import { Choix, signe } from "./commun";

// ------------------------------------------------------------------ création : pacte avec la nature

export function ChoixPacte({
  cat,
  fiche,
  maj,
  apercu,
  classe,
}: {
  cat: CatalogueCreation;
  fiche: FicheDonnees;
  maj: (p: Partial<FicheDonnees>) => void;
  apercu: FicheVue | null;
  classe: ClasseVue;
}) {
  const domaines = cat.domaines.filter((d) => classe.pacteDomaines.includes(d.id));
  const animal = cat.animaux.find((a) => a.id === fiche.compagnon?.animal);
  const problemes = (apercu?.problemes ?? []).filter((p) => /pacte|domaine|compagnon/i.test(p));
  return (
    <div className="section">
      <p className="doux">
        Pacte avec la nature (niveau 1, définitif) : le druide devient si intime avec la nature qu'il acquiert un domaine de prêtre (Air, Climat,
        Eau, Faune, Feu, Flore ou Terre), ou bien il se lie à un compagnon animal.
      </p>
      <div className="groupe-boutons">
        <button
          className={"bouton" + (fiche.pacte === "domaine" ? " principal" : "")}
          onClick={() => maj({ pacte: "domaine", compagnon: null })}
        >
          Un domaine
        </button>
        <button
          className={"bouton" + (fiche.pacte === "compagnon" ? " principal" : "")}
          onClick={() => maj({ pacte: "compagnon", domaines: [] })}
        >
          Un compagnon animal
        </button>
      </div>
      {fiche.pacte === "domaine" && (
        <div className="liste-domaines">
          <p className="doux">
            Les pouvoirs et les sorts de domaine sont ceux d'un prêtre dont le niveau serait égal au niveau de druide ; le druide gagne aussi un
            emplacement de sort de domaine par niveau de sort.
          </p>
          {domaines.map((d) => (
            <div className="carte" key={d.id}>
              <label className="ligne">
                <input type="radio" name="pacte-domaine" checked={fiche.domaines[0] === d.id} onChange={() => maj({ domaines: [d.id] })} />
                <strong>{d.nom}</strong>
              </label>
              <details>
                <summary>Pouvoirs et sorts</summary>
                <p className="doux">{d.description}</p>
                {d.pouvoirs.map((p) => (
                  <p key={p.nom}>
                    <strong>{p.nom} ({p.genre}, niveau {p.niveau}).</strong> {p.texte}
                  </p>
                ))}
                <p className="doux">Sorts de domaine : {d.sorts.map((s) => `${s.niveau}e ${s.nom}`).join(", ")}</p>
              </details>
            </div>
          ))}
        </div>
      )}
      {fiche.pacte === "compagnon" && (
        <div className="section">
          <div>
            <label className="etiquette-champ">Animal</label>
            <Choix
              etiquette="Animal compagnon"
              vide="Choisir un animal…"
              valeur={fiche.compagnon?.animal ?? ""}
              options={cat.animaux.map((a) => ({ v: a.id, l: a.nom }))}
              onChange={(v) =>
                maj({
                  compagnon: v
                    ? { animal: v, nom: fiche.compagnon?.nom ?? "", ignore_developpement: false, augmentations: [] }
                    : null,
                })
              }
            />
          </div>
          {animal && fiche.compagnon && (
            <>
              <div>
                <label className="etiquette-champ" htmlFor="c-compagnon">Nom du compagnon</label>
                <input
                  id="c-compagnon"
                  className="champ"
                  value={fiche.compagnon.nom}
                  onChange={(e) => maj({ compagnon: { ...fiche.compagnon!, nom: e.target.value } })}
                />
              </div>
              <div className="carte">
                <p>
                  Taille {animal.taille} · vitesse {animal.vitesse} · armure naturelle +{animal.armureNaturelle}
                </p>
                <p className="doux">Attaques : {animal.attaques}</p>
                <p className="doux">Caractéristiques de départ : {animal.caracteristiques}</p>
                {animal.particularites && <p className="doux">{animal.particularites}</p>}
                <p className="doux">
                  Développement au niveau {animal.niveauDeveloppement} : {animal.developpement}.
                </p>
                <label className="ligne">
                  <input
                    type="checkbox"
                    checked={fiche.compagnon.ignore_developpement}
                    onChange={(e) => maj({ compagnon: { ...fiche.compagnon!, ignore_developpement: e.target.checked } })}
                  />
                  Ignorer ce développement : +2 Dextérité et +2 Constitution à la place
                </label>
              </div>
            </>
          )}
        </div>
      )}
      {problemes.map((p, i) => (
        <p key={i} className="alerte-texte">{p}</p>
      ))}
    </div>
  );
}

// ------------------------------------------------------------------ fiche : forme animale

export function PanneauFormes({ f, agir }: { f: FicheVue; agir: (e: EvenementFiche) => void }) {
  const formes = f.formes;
  const p = f.pouvoirs.find((x) => x.cle === "forme_animale");
  if (!formes) return null;
  const categories = ["animal", "élémentaire", "plante"];
  const vide = p?.restants !== null && p?.restants !== undefined && p.restants <= 0;
  return (
    <div className="section">
      <section className="carte">
        <h3>Forme animale</h3>
        {!p ? (
          <p className="doux">La forme animale s'acquiert au niveau 4 de druide.</p>
        ) : (
          <p>
            {p.max === null ? "À volonté" : `${p.restants} utilisation(s) restante(s) sur ${p.max} aujourd'hui`} · chaque transformation dure {formes.dureeHeures}{" "}
            heure(s) (1 par niveau de druide) et consomme une utilisation.
          </p>
        )}
        {formes.enCours ? (
          <div className="ligne">
            <strong>{formes.enCours}</strong>
            {formes.roundsRestants !== null && <span className="doux"> — {Math.round(formes.roundsRestants / 10)} min environ restantes</span>}
            <span className="espace" />
            <button className="bouton" onClick={() => agir("FormeQuittee")}>Reprendre sa forme normale</button>
          </div>
        ) : (
          <p className="doux">Sous sa forme normale.</p>
        )}
      </section>
      {categories.map((cat) => {
        const liste = formes.formes.filter((x) => x.categorie === cat);
        if (liste.length === 0) return null;
        return (
          <section className="carte" key={cat}>
            <h3>{cat === "animal" ? "Animaux" : cat === "plante" ? "Plantes" : "Élémentaires"}</h3>
            <table className="tableau">
              <thead>
                <tr><th>Forme</th><th>Bonus</th><th>Niveau</th><th /></tr>
              </thead>
              <tbody>
                {liste.map((x) => (
                  <tr key={x.id} className={x.disponible ? "" : "ignore"}>
                    <td>{x.nom}</td>
                    <td className="doux">{x.effets.join(", ")}</td>
                    <td className="num">{x.niveau}</td>
                    <td>
                      <button
                        className="bouton discret"
                        disabled={!x.disponible || vide || !p}
                        title={x.disponible ? x.texte : `Niveau ${x.niveau} de druide requis`}
                        onClick={() => agir({ FormeAdoptee: { forme: x.id } })}
                      >
                        Se transformer
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </section>
        );
      })}
      <p className="doux">
        Les vitesses, attaques naturelles et aptitudes de la forme suivent le sort correspondant (forme bestiale, corps élémentaire ou forme
        végétale) : elles ne sont pas calculées ici.
      </p>
    </div>
  );
}

// ------------------------------------------------------------------ fiche : compagnon animal

export function PanneauCompagnon({ f, agir }: { f: FicheVue; agir: (e: EvenementFiche) => void }) {
  const c = f.compagnon;
  const [augm, setAugm] = useState<string[] | null>(null);
  if (!c) return <p className="doux">Pas de compagnon animal.</p>;
  const actuelles = augm ?? c.augmentations;
  const lignes = Array.from({ length: c.augmentationsPermises }, (_, i) => actuelles[i] ?? "");
  const enregistrer = (changes: { ignore?: boolean; augmentations?: string[]; nom?: string }) => {
    const aug = (changes.augmentations ?? c.augmentations).filter(Boolean);
    const traduire = (id: string) => CARACS.find((k) => k.toLowerCase() === id) ?? "Sag";
    agir({
      CompagnonModifie: {
        animal: c.animalId,
        nom: changes.nom ?? c.nom,
        ignore_developpement: changes.ignore ?? c.ignoreDeveloppement,
        augmentations: aug.map(traduire),
      },
    });
  };
  return (
    <div className="section">
      <section className="carte">
        <h3>
          {c.nom} <small className="doux">({c.animal}, taille {c.taille})</small>
        </h3>
        <p className="doux">
          {c.dv} DV · {c.pv} PV (moyenne) · vitesse {c.vitesse} · BBA {signe(c.bba)}
        </p>
        <div className="grille-fiche">
          <div className="carte chiffre-fiche"><span className="etiquette">CA</span><span className="valeur">{c.ca}</span><span className="modificateur">contact {c.caContact} · dépourvu {c.caDepourvu}</span></div>
          <div className="carte chiffre-fiche"><span className="etiquette">Vigueur / Réflexes / Volonté</span><span className="valeur">{signe(c.vigueur)} / {signe(c.reflexes)} / {signe(c.volonte)}</span></div>
          <div className="carte chiffre-fiche"><span className="etiquette">Attaque</span><span className="valeur">{signe(c.bonusAttaque)}</span><span className="modificateur">{c.attaques}</span></div>
        </div>
        <table className="tableau">
          <thead><tr><th>Caractéristique</th><th>Score</th><th>Mod.</th></tr></thead>
          <tbody>
            {c.caracteristiques.map((k) => (
              <tr key={k.id}><td>{k.nom}</td><td className="num">{k.score.total}</td><td className="num">{signe(k.modificateur)}</td></tr>
            ))}
          </tbody>
        </table>
        <p className="doux">
          {c.competences} rangs de compétence · {c.dons} don(s) · {c.tours} tour(s) supplémentaire(s)
          {c.particularites ? ` · ${c.particularites}` : ""}
        </p>
        {c.developpementApplique && <p className="doux">Développement appliqué.</p>}
      </section>
      <section className="carte">
        <h3>Choix du compagnon</h3>
        <label className="ligne">
          <input type="checkbox" checked={c.ignoreDeveloppement} onChange={(e) => enregistrer({ ignore: e.target.checked })} />
          Ignorer le développement : +2 Dextérité et +2 Constitution à la place
        </label>
        {c.augmentationsPermises === 0 ? (
          <p className="doux">Le compagnon gagne une augmentation de caractéristique aux niveaux 4, 9, 14 et 20 de druide.</p>
        ) : (
          <>
            <p className="doux">Augmentations de caractéristique gagnées : {c.augmentationsPermises} (+1 chacune).</p>
            <div className="groupe-boutons">
              {lignes.map((v, i) => (
                <Choix
                  key={i}
                  etiquette={`Augmentation ${i + 1}`}
                  vide="Choisir…"
                  valeur={v}
                  options={CARACS.map((k) => ({ v: k.toLowerCase(), l: NOMS_CARAC[k] }))}
                  onChange={(x) => {
                    const n = [...lignes];
                    n[i] = x;
                    setAugm(n);
                    enregistrer({ augmentations: n });
                  }}
                />
              ))}
            </div>
          </>
        )}
      </section>
      {c.capacites.map((t, i) => (
        <details className="carte" key={i}>
          <summary>{t.nom}</summary>
          <p>{t.texte}</p>
        </details>
      ))}
    </div>
  );
}
