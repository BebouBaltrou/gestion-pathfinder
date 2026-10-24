import { Fragment, useEffect, useState } from "react";
import type { Outils } from "../../App";
import { api, formaterPo } from "../../api";
import type { Charge, EffetDonnees, EvenementFiche, FicheVue, LignePersonnage } from "../../fiche-types";
import { Chiffre, Choix, Detail, signe, useCatalogue } from "./commun";
import { Montee } from "./Montee";
import { PrestigeDon } from "./PrestigeDon";

/** `niveauVise` : niveau choisi à la création ; l'assistant de montée s'ouvre tant qu'il n'est pas atteint. */
type Props = { outils: Outils; pj: LignePersonnage; fiche: FicheVue; niveauVise?: number };
type Onglet = "combat" | "competences" | "dons" | "sorts" | "equipement" | "etats" | "histoire" | "prestige";

const ONGLETS: [Onglet, string][] = [
  ["combat", "Combat"],
  ["competences", "Compétences"],
  ["dons", "Dons et capacités"],
  ["sorts", "Sorts"],
  ["equipement", "Équipement"],
  ["etats", "États et effets"],
  ["histoire", "XP et historique"],
];

export function FicheComplete(p: Props) {
  const [onglet, setOnglet] = useState<Onglet>("combat");
  const [monter, setMonter] = useState(false);
  const [avecPrestige, setAvecPrestige] = useState(false);
  const f = p.fiche;
  const id = f.identite;
  useEffect(() => {
    api.prestigeDon(p.pj.id).then((v) => setAvecPrestige(!!v?.actif));
  }, [p.pj.id, p.outils.version]);
  useEffect(() => {
    if ((p.niveauVise ?? 1) > id.niveau && id.niveauxEnAttente > 0) setMonter(true);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id.niveau, p.niveauVise]);
  const agir = (e: EvenementFiche) => p.outils.agir(api.evenementFiche(p.pj.id, e));
  const progression =
    id.xpSuivant !== null ? Math.min(1, id.xp / id.xpSuivant) : id.niveau >= 20 ? 1 : 0;

  return (
    <div className="fiche">
      <section className="carte entete-fiche">
        <div>
          <h2 className="nom-fiche">{id.nom}</h2>
          <p className="doux">
            {id.race} · {id.classes}
            {id.lignage ? ` · lignage ${id.lignage.toLowerCase()}` : ""} · niveau {id.niveau}
          </p>
          <p className="doux">
            Compte : {formaterPo(id.compte.pa)}
            {id.prestige !== null && (
              <>
                {" "}· Prestige {id.prestige}
                {id.prestigeCalcule ? " (niveau + modificateur de Charisme)" : ""}
              </>
            )}
          </p>
        </div>
        <div className="xp">
          <div className="doux">
            {id.xp.toLocaleString("fr-FR")} XP
            {id.xpSuivant !== null ? ` / ${id.xpSuivant.toLocaleString("fr-FR")} pour le niveau ${id.niveau + 1}` : ""}
          </div>
          <div className="barre"><div style={{ width: `${progression * 100}%` }} /></div>
          {id.niveauxEnAttente > 0 && (
            <button className="bouton principal" onClick={() => setMonter(true)}>
              Monter de niveau ({id.niveauxEnAttente} en attente)
            </button>
          )}
        </div>
      </section>

      {f.problemes.length > 0 && (
        <section className="alerte grave" role="alert">
          <span className="point" />
          <div className="texte">
            <strong>La fiche n'est pas valide :</strong>
            <ul>{f.problemes.map((m, i) => <li key={i}>{m}</li>)}</ul>
          </div>
        </section>
      )}

      {monter && <Montee outils={p.outils} pj={p.pj} fiche={f} onFerme={() => setMonter(false)} />}

      <div className="onglets" role="tablist">
        {[...ONGLETS, ...(avecPrestige ? ([["prestige", "Prestige"]] as [Onglet, string][]) : [])].map(([k, nom]) => (
          <button key={k} role="tab" aria-selected={onglet === k} className={"onglet" + (onglet === k ? " actif" : "")} onClick={() => setOnglet(k)}>
            {nom}
          </button>
        ))}
      </div>

      {onglet === "combat" && <Combat f={f} />}
      {onglet === "competences" && <Competences f={f} />}
      {onglet === "dons" && <Dons f={f} />}
      {onglet === "sorts" && <Sorts f={f} agir={agir} />}
      {onglet === "equipement" && <Equipement f={f} agir={agir} />}
      {onglet === "etats" && <Etats f={f} agir={agir} />}
      {onglet === "histoire" && <Histoire f={f} agir={agir} />}
      {onglet === "prestige" && <PrestigeDon outils={p.outils} pj={p.pj} />}
    </div>
  );
}

// ------------------------------------------------------------------ combat

function Combat({ f }: { f: FicheVue }) {
  return (
    <div className="section">
      <div className="grille-fiche">
        {f.caracteristiques.map((c) => (
          <Chiffre key={c.id} nom={c.nom} v={c.score} sous={`modificateur ${signe(c.modificateur)}`} />
        ))}
      </div>
      <div className="grille-fiche">
        <Chiffre nom="Points de vie" v={f.pv} />
        <Chiffre nom="Classe d'armure" v={f.ca} />
        <Chiffre nom="CA de contact" v={f.caContact} />
        <Chiffre nom="CA pris au dépourvu" v={f.caDepourvu} />
        <Chiffre nom="Initiative" v={f.initiative} avecSigne />
        <Chiffre nom="Vigueur" v={f.vigueur} avecSigne />
        <Chiffre nom="Réflexes" v={f.reflexes} avecSigne />
        <Chiffre nom="Volonté" v={f.volonte} avecSigne />
        <Chiffre nom="Bonus de base à l'attaque" v={f.bba} avecSigne />
        <Chiffre nom="BMO" v={f.bmo} avecSigne />
        <Chiffre nom="DMD" v={f.dmd} />
        <div className="carte chiffre-fiche">
          <div className="chiffre-bouton">
            <span className="etiquette">Vitesse</span>
            <span className="valeur">{f.vitesse} pi</span>
          </div>
          <span className="doux">{(f.vitesse * 0.3).toLocaleString("fr-FR", { maximumFractionDigits: 1 })} m</span>
        </div>
      </div>
      {f.attaquesIteratives.length > 1 && (
        <p className="doux">Attaques itératives : {f.attaquesIteratives.map(signe).join(" / ")}</p>
      )}
      <h2>Armes</h2>
      {f.equipement.armes.length === 0 && <p className="doux">Aucune arme portée (onglet Équipement).</p>}
      <div className="liste-regles">
        {f.equipement.armes.map((a) => (
          <div className="carte regle" key={a.nomEn}>
            <div className="regle-texte">
              <strong>{a.nom}</strong>
              <p className="doux">
                {a.critique ? `Critique ${a.critique}` : ""}
                {a.portee && a.portee !== "—" ? ` · portée ${a.portee}` : ""}
                {a.typeDegats ? ` · ${a.typeDegats}` : ""}
                {!a.maitrise ? " · non maîtrisée (−4)" : ""}
              </p>
            </div>
            <div className="attaque-arme">
              <span title={a.attaque.explication}>Attaque {signe(a.attaque.total)}</span>
              <span>Dégâts {a.degats}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

// ------------------------------------------------------------------ compétences

function Competences({ f }: { f: FicheVue }) {
  const [tout, setTout] = useState(false);
  const [ouverte, setOuverte] = useState<string | null>(null);
  const liste = f.competences.filter((c) => tout || c.rangs > 0 || c.deClasse);
  const bilan = f.rangsParNiveau;
  return (
    <div className="section">
      <label className="case">
        <input type="checkbox" checked={tout} onChange={(e) => setTout(e.target.checked)} /> Montrer toutes les compétences
      </label>
      <table className="tableau">
        <thead>
          <tr><th>Compétence</th><th>Total</th><th>Rangs</th><th /></tr>
        </thead>
        <tbody>
          {liste.map((c) => (
            <Fragment key={c.cle}>
              <tr onClick={() => setOuverte(ouverte === c.cle ? null : c.cle)} className="cliquable">
                <td>
                  {c.nom}
                  {c.formationRequise && c.rangs === 0 && <small className="doux"> (formation requise)</small>}
                </td>
                <td className="num">{signe(c.valeur.total)}</td>
                <td className="num">{c.rangs}</td>
                <td>{c.deClasse && <span className="puce or">de classe</span>}</td>
              </tr>
              {ouverte === c.cle && (
                <tr>
                  <td colSpan={4}><Detail v={c.valeur} /></td>
                </tr>
              )}
            </Fragment>
          ))}
        </tbody>
      </table>
      <p className="doux">
        Rangs placés par niveau :{" "}
        {bilan.map(([n, places, dispo]) => `niveau ${n} : ${places}/${dispo}`).join(" · ")}
      </p>
    </div>
  );
}

// ------------------------------------------------------------------ dons

function Dons({ f }: { f: FicheVue }) {
  return (
    <div className="section">
      <h2>Dons</h2>
      <div className="liste-regles">
        {f.dons.map((d, i) => (
          <div className="carte" key={d.nomEn + i}>
            <strong>{d.nom}</strong>
            {d.inutilisable && <p className="alerte-texte">Inutilisable : prérequis manquant ({d.inutilisable})</p>}
            {d.resume && <p className="doux">{d.resume}</p>}
            {d.avantage && (
              <details>
                <summary>Avantage</summary>
                <p>{d.avantage}</p>
              </details>
            )}
          </div>
        ))}
      </div>
      <h2>Traits de la race</h2>
      <div className="liste-regles">
        {f.traits.map((t) => (
          <details className="carte" key={t.nom}>
            <summary>{t.nom}</summary>
            <p>{t.texte}</p>
          </details>
        ))}
      </div>
      <h2>Capacités de classe</h2>
      <div className="liste-regles">
        {f.capacites.map((t, i) => (
          <div className="carte" key={i}>
            <small className="doux">{t.nom}</small>
            <p>{t.texte}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

// ------------------------------------------------------------------ sorts

function Sorts({ f, agir }: { f: FicheVue; agir: (e: EvenementFiche) => void }) {
  if (f.incantateurs.length === 0) return <p className="doux">Ce personnage ne lance pas de sorts.</p>;
  return (
    <div className="section">
      {f.equipement.echecSortsPct > 0 && (
        <p className="alerte-texte">
          Armure et bouclier : {f.equipement.echecSortsPct} % de chances d'échec des sorts profanes à composante gestuelle.
        </p>
      )}
      {f.incantateurs.map((inc) => (
        <div className="section" key={inc.classe}>
          <div className="section-tete">
            <h2>{inc.classe}</h2>
            <span className="aide">niveau de lanceur {inc.niveauDeLanceur}</span>
            <span className="espace" />
            <button className="bouton" onClick={() => agir("Repos")}>Repos : rendre les sorts</button>
          </div>
          <div className="grille-fiche">
            {inc.emplacements.map((e) => (
              <div className="carte chiffre-fiche" key={e.niveauDeSort}>
                <div className="chiffre-bouton">
                  <span className="etiquette">{e.niveauDeSort === 0 ? "Tours de magie" : `Niveau ${e.niveauDeSort}`}</span>
                  <span className="valeur">
                    {e.niveauDeSort === 0 ? "∞" : `${e.restants ?? e.parJour}/${e.parJour}`}
                  </span>
                </div>
                <span className="doux">
                  {e.connus} connu(s) · DD {e.dd}
                  {e.bonus > 0 ? ` · +${e.bonus} bonus` : ""}
                </span>
              </div>
            ))}
          </div>
          <table className="tableau">
            <thead><tr><th>Sort</th><th>Niv.</th><th>École</th><th /></tr></thead>
            <tbody>
              {inc.sorts.map((s) => (
                <tr key={s.nomEn}>
                  <td>{s.nom}{s.deLignage && <small className="doux"> (lignage)</small>}</td>
                  <td className="num">{s.niveau}</td>
                  <td className="doux">{s.ecole}</td>
                  <td>
                    {s.niveau > 0 && (
                      <button
                        className="bouton discret"
                        disabled={(inc.emplacements.find((e) => e.niveauDeSort === s.niveau)?.restants ?? 0) === 0}
                        onClick={() => agir({ SortLance: { classe: "ensorceleur", niveau_de_sort: s.niveau, sort: s.nomEn } })}
                      >
                        Lancer
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ))}
    </div>
  );
}

// ------------------------------------------------------------------ équipement

const CHARGES: [Charge, string][] = [
  ["Legere", "Légère"],
  ["Intermediaire", "Intermédiaire"],
  ["Lourde", "Lourde"],
  ["Surcharge", "Surcharge"],
];

function Equipement({ f, agir }: { f: FicheVue; agir: (e: EvenementFiche) => void }) {
  const cat = useCatalogue();
  const [objet, setObjet] = useState("");
  const [quantite, setQuantite] = useState("1");
  const [arme, setArme] = useState("");
  const eq = f.equipement;
  if (!cat) return null;
  const protections = (genre: (g: string) => boolean) =>
    cat.armures.filter((a) => genre(a.genre)).map((a) => ({ v: a.nomEn, l: a.nom }));
  return (
    <div className="section">
      <div className="grille-fiche">
        <div className="carte chiffre-fiche">
          <div className="chiffre-bouton">
            <span className="etiquette">Poids porté</span>
            <span className="valeur">{eq.poidsTotal.toLocaleString("fr-FR", { maximumFractionDigits: 1 })} lb</span>
          </div>
          <span className="doux">
            {eq.limites ? `limites ${eq.limites[0]} / ${eq.limites[1]} / ${eq.limites[2]} lb` : ""}
          </span>
        </div>
        <div className="carte chiffre-fiche">
          <div className="chiffre-bouton">
            <span className="etiquette">Malus d'armure</span>
            <span className="valeur">{eq.malusDArmure}</span>
          </div>
          <span className="doux">{eq.sourceMalus || "aucun"}</span>
        </div>
        <div className="carte chiffre-fiche">
          <div className="chiffre-bouton">
            <span className="etiquette">Échec des sorts</span>
            <span className="valeur">{eq.echecSortsPct} %</span>
          </div>
          <span className="doux">Bonus de DEX max : {eq.dexMax ?? "aucun plafond"}</span>
        </div>
      </div>
      {eq.nonMaitrises.length > 0 && (
        <p className="alerte-texte">Porté sans maîtrise : {eq.nonMaitrises.join(", ")} (son malus s'applique aussi aux attaques).</p>
      )}
      {eq.poidsInconnu.length > 0 && (
        <p className="doux">Poids inconnu dans le Core (compté pour 0) : {eq.poidsInconnu.join(", ")}.</p>
      )}

      <h2>Charge portée</h2>
      <p className="doux">Indiquée par le joueur : le poids ne la déduit pas automatiquement pour l'instant.</p>
      <div className="groupe-boutons">
        {CHARGES.map(([v, l]) => (
          <button
            key={v}
            className={"bouton" + (eq.charge.toLowerCase() === l.toLowerCase() ? " principal" : "")}
            onClick={() => agir({ ChargeDeclaree: v })}
          >
            {l}
          </button>
        ))}
      </div>

      <h2>Armure et bouclier</h2>
      <div className="ligne">
        <Choix
          etiquette="Armure"
          vide="Aucune armure"
          valeur={eq.armure?.nomEn ?? ""}
          options={protections((g) => g.startsWith("armure"))}
          onChange={(v) => agir({ Equiper: { emplacement: "armure", objet: v || null } })}
        />
        <Choix
          etiquette="Bouclier"
          vide="Aucun bouclier"
          valeur={eq.bouclier?.nomEn ?? ""}
          options={protections((g) => g === "bouclier")}
          onChange={(v) => agir({ Equiper: { emplacement: "bouclier", objet: v || null } })}
        />
      </div>

      <h2>Armes</h2>
      <div className="puces">
        {eq.armes.map((a) => (
          <span className="puce" key={a.nomEn}>
            {a.nom}{" "}
            <button className="croix" aria-label={`Ranger ${a.nom}`} onClick={() => agir({ ArmeRetiree: a.nomEn })}>×</button>
          </span>
        ))}
      </div>
      <div className="ligne">
        <Choix
          etiquette="Arme à prendre"
          vide="Prendre une arme…"
          valeur={arme}
          options={cat.armes.filter((a) => a.genre === "arme").map((a) => ({ v: a.nomEn, l: a.nom }))}
          onChange={setArme}
        />
        <button className="bouton" disabled={!arme} onClick={() => { agir({ ArmeAjoutee: arme }); setArme(""); }}>Ajouter</button>
      </div>

      <h2>Matériel et munitions</h2>
      <table className="tableau">
        <tbody>
          {eq.objets.map((o) => (
            <tr key={o.nomEn}>
              <td>{o.nom}</td>
              <td className="num">× {o.quantite}</td>
              <td>
                <button className="bouton discret" onClick={() => agir({ ObjetRetire: { nom_en: o.nomEn, quantite: 1 } })}>−1</button>
                <button className="bouton discret" onClick={() => agir({ ObjetAjoute: { nom_en: o.nomEn, quantite: 1 } })}>+1</button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <div className="ligne">
        <Choix
          etiquette="Objet à ajouter"
          vide="Ajouter un objet…"
          valeur={objet}
          options={[
            ...cat.materiel.map((a) => ({ v: a.nomEn, l: a.nom })),
            ...cat.armes.filter((a) => a.genre !== "arme").map((a) => ({ v: a.nomEn, l: a.nom })),
          ]}
          onChange={setObjet}
        />
        <input className="champ court" inputMode="numeric" aria-label="Quantité" value={quantite} onChange={(e) => setQuantite(e.target.value)} />
        <button
          className="bouton"
          disabled={!objet || !(Number(quantite) > 0)}
          onClick={() => { agir({ ObjetAjoute: { nom_en: objet, quantite: Number(quantite) } }); setObjet(""); setQuantite("1"); }}
        >
          Ajouter
        </button>
      </div>
    </div>
  );
}

// ------------------------------------------------------------------ états et effets temporaires

const CIBLES: [string, string][] = [
  ["carac.for", "Force"], ["carac.dex", "Dextérité"], ["carac.con", "Constitution"],
  ["carac.int", "Intelligence"], ["carac.sag", "Sagesse"], ["carac.cha", "Charisme"],
  ["ca", "Classe d'armure"], ["attaque", "Jets d'attaque"], ["degats", "Dégâts"], ["init", "Initiative"],
  ["jds.vig", "Vigueur"], ["jds.ref", "Réflexes"], ["jds.vol", "Volonté"], ["pv", "Points de vie"], ["comp.*", "Toutes les compétences"],
];
const TYPES: [string, string][] = [
  ["alteration", "altération"], ["armure", "armure"], ["bouclier", "bouclier"], ["parade", "parade"], ["esquive", "esquive"],
  ["moral", "moral"], ["chance", "chance"], ["circonstance", "circonstance"], ["racial", "racial"], ["objet", "d'objet"],
  ["taille", "de taille"], ["naturel", "naturel"], ["perspicacite", "perspicacité"], ["competence", "de compétence"],
  ["sacre", "sacré"], ["profane", "profane"], ["sans_type", "sans type"],
];

function Etats({ f, agir }: { f: FicheVue; agir: (e: EvenementFiche) => void }) {
  const cat = useCatalogue();
  const [etat, setEtat] = useState("");
  const [nom, setNom] = useState("");
  const [cible, setCible] = useState("carac.for");
  const [type, setType] = useState("alteration");
  const [valeur, setValeur] = useState("2");
  const [rounds, setRounds] = useState("");
  const [passe, setPasse] = useState("1");
  if (!cat) return null;
  const effet = (): EffetDonnees => ({ cible, type_bonus: type, valeur: Number(valeur) || 0, source: nom.trim() });
  return (
    <div className="section">
      <h2>États</h2>
      <div className="liste-regles">
        {f.etats.length === 0 && <p className="doux">Aucun état.</p>}
        {f.etats.map((e) => (
          <div className="carte regle" key={e.id}>
            <div className="regle-texte">
              <strong>{e.nom}</strong>
              {e.note && <p className="alerte-texte">{e.note}</p>}
              <details>
                <summary>Texte du livre (traduction non officielle)</summary>
                <p>{e.texte}</p>
              </details>
            </div>
            <button className="bouton" onClick={() => agir({ EtatRetire: e.id })}>Retirer</button>
          </div>
        ))}
      </div>
      <div className="ligne">
        <Choix
          etiquette="État à ajouter"
          vide="Ajouter un état…"
          valeur={etat}
          options={cat.etats.map((e) => ({ v: e.id, l: e.nom }))}
          onChange={setEtat}
        />
        <button className="bouton" disabled={!etat} onClick={() => { agir({ EtatAjoute: etat }); setEtat(""); }}>Ajouter</button>
      </div>

      <h2>Effets temporaires</h2>
      <p className="doux">Sorts sur soi, potions, bénédictions : l'effet est chiffré selon les types de bonus du Core (un même type ne se cumule pas).</p>
      <div className="liste-regles">
        {f.temporaires.length === 0 && <p className="doux">Aucun effet temporaire.</p>}
        {f.temporaires.map((t) => (
          <div className="carte regle" key={t.nom}>
            <div className="regle-texte">
              <strong>{t.nom}</strong>
              <p className="doux">
                {t.effets.join(", ")} ·{" "}
                {t.roundsRestants === null ? "jusqu'à ce qu'on le retire" : `${t.roundsRestants} round(s) restant(s)`}
              </p>
            </div>
            <button className="bouton" onClick={() => agir({ EffetTemporaireRetire: t.nom })}>Retirer</button>
          </div>
        ))}
      </div>
      <form
        className="carte formulaire-effet"
        onSubmit={(e) => {
          e.preventDefault();
          agir({
            EffetTemporaireAjoute: {
              nom: nom.trim(),
              effets: [effet()],
              rounds_restants: rounds.trim() ? Math.max(1, Number(rounds) || 1) : null,
            },
          });
          setNom("");
        }}
      >
        <div>
          <label className="etiquette-champ" htmlFor="eff-nom">Nom</label>
          <input id="eff-nom" className="champ" value={nom} onChange={(e) => setNom(e.target.value)} placeholder="ex. Force de taureau" />
        </div>
        <div>
          <label className="etiquette-champ" htmlFor="eff-cible">Ce qui change</label>
          <select id="eff-cible" className="champ choix" value={cible} onChange={(e) => setCible(e.target.value)}>
            {CIBLES.map(([v, l]) => <option key={v} value={v}>{l}</option>)}
          </select>
        </div>
        <div>
          <label className="etiquette-champ" htmlFor="eff-type">Type de bonus</label>
          <select id="eff-type" className="champ choix" value={type} onChange={(e) => setType(e.target.value)}>
            {TYPES.map(([v, l]) => <option key={v} value={v}>{l}</option>)}
          </select>
        </div>
        <div>
          <label className="etiquette-champ" htmlFor="eff-val">Valeur</label>
          <input id="eff-val" className="champ court" inputMode="numeric" value={valeur} onChange={(e) => setValeur(e.target.value)} />
        </div>
        <div>
          <label className="etiquette-champ" htmlFor="eff-dur">Durée (rounds)</label>
          <input id="eff-dur" className="champ court" inputMode="numeric" value={rounds} onChange={(e) => setRounds(e.target.value)} placeholder="sans limite" />
        </div>
        <button className="bouton principal" type="submit" disabled={!nom.trim()}>Ajouter</button>
      </form>
      <div className="ligne">
        <span>Faire passer</span>
        <input className="champ court" inputMode="numeric" aria-label="Rounds" value={passe} onChange={(e) => setPasse(e.target.value)} />
        <button className="bouton" disabled={!(Number(passe) > 0)} onClick={() => agir({ TempsEcoule: { rounds: Number(passe) } })}>round(s)</button>
        <span className="doux">Les jours de la campagne font aussi passer le temps (1 jour = 14 400 rounds).</span>
      </div>
      {f.notesEtats.length > 0 && <p className="doux">Non calculé automatiquement : voir les notes des états.</p>}
    </div>
  );
}

// ------------------------------------------------------------------ XP et historique

function Histoire({ f, agir }: { f: FicheVue; agir: (e: EvenementFiche) => void }) {
  const [xp, setXp] = useState("");
  const [motif, setMotif] = useState("");
  const [jalon, setJalon] = useState(String(f.identite.niveau + 1));
  return (
    <div className="section">
      <h2>Gagner de l'expérience</h2>
      <form
        className="ligne"
        onSubmit={(e) => {
          e.preventDefault();
          agir({ XpGagnes: { nombre: Number(xp), motif: motif.trim() } });
          setXp("");
          setMotif("");
        }}
      >
        <input className="champ court" inputMode="numeric" aria-label="Points d'expérience" placeholder="XP" value={xp} onChange={(e) => setXp(e.target.value)} />
        <input className="champ" aria-label="Motif" placeholder="Motif (facultatif)" value={motif} onChange={(e) => setMotif(e.target.value)} />
        <button className="bouton principal" type="submit" disabled={!(Number(xp) > 0)}>Ajouter</button>
      </form>
      <h2>Jalon accordé par le MJ</h2>
      <p className="doux">Utile quand la vitesse de progression de la campagne est « Jalons » (page Paramètres).</p>
      <div className="ligne">
        <input className="champ court" inputMode="numeric" aria-label="Niveau accordé" value={jalon} onChange={(e) => setJalon(e.target.value)} />
        <button className="bouton" disabled={!(Number(jalon) >= 1 && Number(jalon) <= 20)} onClick={() => agir({ JalonAccorde: { niveau: Number(jalon) } })}>
          Accorder ce niveau
        </button>
      </div>
      <h2>Historique de la fiche</h2>
      <ol className="historique">
        {[...f.historique].reverse().map((h, i) => <li key={i}>{h}</li>)}
      </ol>
    </div>
  );
}
