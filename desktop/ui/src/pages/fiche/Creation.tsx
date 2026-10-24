import { useEffect, useState } from "react";
import type { Outils } from "../../App";
import { api } from "../../api";
import {
  CARACS,
  FicheDonnees,
  FicheVue,
  LignePersonnage,
  NOMS_CARAC,
  RegleInfo,
} from "../../fiche-types";
import { SaisieNombre } from "../../composants/jeu";
import { Choix, Pas, signe, useCatalogue } from "./commun";
import { SelectionDons, SelectionRangs, SelectionSorts } from "./selections";

const ETAPES = ["Identité", "Caractéristiques", "Compétences", "Dons", "Sorts", "Récapitulatif"];

/** Assistant de création : un niveau 1 contrôlé pas à pas par le cœur de règles. */
export function Creation({ outils, pj, onFini }: { outils: Outils; pj: LignePersonnage; onFini: (niveauVise: number) => void }) {
  const cat = useCatalogue();
  const [etape, setEtape] = useState(0);
  const [regle, setRegle] = useState<RegleInfo | null>(null);
  const [apercu, setApercu] = useState<FicheVue | null>(null);
  const [erreur, setErreur] = useState<string | null>(null);
  const [fiche, setFiche] = useState<FicheDonnees | null>(null);
  const [niveauVise, setNiveauVise] = useState<number | null>(1);

  useEffect(() => {
    api.reglesCampagne().then((l) => setRegle(l.find((r) => r.id === "pj.generation") ?? null));
  }, []);

  useEffect(() => {
    if (cat && !fiche) {
      const classe = cat.classes[0];
      setFiche({
        nom: pj.nom,
        race: "",
        bonus_racial_libre: null,
        scores_base: { For: 10, Dex: 10, Con: 10, Int: 10, Sag: 10, Cha: 10 },
        dommages: {},
        drain: {},
        augmentations: [],
        niveaux: [{ classe: classe.id, de_vie_obtenu: classe.deVie, rangs: {}, predilection: "PointDeVie", dons: [] }],
        classes_de_predilection: [classe.id],
        lignage: null,
        connaissance_de_lignage: null,
        effets: [],
        sorts_connus: [],
        equipement: { armure: null, bouclier: null, armes: [], objets: [], charge_declaree: "Legere" },
        etats: [],
        temporaires: [],
      });
    }
  }, [cat, fiche, pj.nom]);

  useEffect(() => {
    if (!fiche || !fiche.race) {
      setApercu(null);
      return;
    }
    const t = window.setTimeout(() => {
      api.apercuFiche(fiche).then(
        (v) => {
          setApercu(v);
          setErreur(null);
        },
        (e) => {
          setApercu(null);
          setErreur(String(e));
        },
      );
    }, 200);
    return () => window.clearTimeout(t);
  }, [fiche]);

  if (!cat || !fiche) return null;
  const classe = cat.classes.find((c) => c.id === fiche.niveaux[0].classe)!;
  const race = cat.races.find((r) => r.id === fiche.race);
  const lignage = classe.lignages.find((l) => l.id === fiche.lignage);
  const maj = (p: Partial<FicheDonnees>) => setFiche({ ...fiche, ...p });
  const majNiveau = (p: Partial<FicheDonnees["niveaux"][number]>) =>
    maj({ niveaux: [{ ...fiche.niveaux[0], ...p }] });

  const methode = regle?.valeur ?? "achat_15";
  const points = methode.startsWith("achat_") ? Number(methode.slice(6)) : null;
  const cout = (s: number) => cat.coutAchat.find(([sc]) => sc === s)?.[1] ?? 0;
  const depense = CARACS.reduce((t, c) => t + cout(fiche.scores_base[c]), 0);
  const modInt = apercu?.caracteristiques.find((c) => c.id === "int")?.modificateur ?? 0;
  const rangsDisponibles =
    Math.max(1, classe.rangsParNiveau + modInt) + (race?.rangSupplementaire ?? 0) + (fiche.niveaux[0].predilection === "Rang" ? 1 : 0);
  const donsAttendus = 1 + (race?.bonusDon ?? 0);
  const sortsAttendus: [number, number][] = (classe.sortsConnus[0] ?? [])
    .map((n, i) => [i, n ?? 0] as [number, number])
    .filter(([, n]) => n > 0);
  const choixLibreManquant = race?.choixLibre != null && !fiche.bonus_racial_libre;

  const creer = async () => {
    const r = await outils.agir(api.creerFiche(pj.id, fiche));
    if (!r?.ok) return;
    const niveau = niveauVise ?? 1;
    if (niveau > 1) {
      // les XP du niveau choisi sont accordés ; les niveaux se montent ensuite un à un
      const g = await outils.agir(api.accorderNiveauDeDepart(pj.id, niveau));
      if (!g?.ok) return;
    }
    onFini(niveau);
  };

  const peutAvancer = [
    !!race && !!fiche.lignage && !choixLibreManquant && (!lignage?.choixConnaissance || !!fiche.connaissance_de_lignage),
    points === null || (depense === points && CARACS.every((c) => fiche.scores_base[c] >= 7 && fiche.scores_base[c] <= 18)),
    true,
    true,
    true,
    true,
  ][etape];

  return (
    <section className="carte creation">
      <div className="section-tete">
        <h2>Fiche de {pj.nom}</h2>
        <span className="espace" />
        <button className="bouton" onClick={() => onFini(1)}>Annuler</button>
      </div>
      <ol className="etapes">
        {ETAPES.map((n, i) => (
          <li key={n} className={i === etape ? "actif" : i < etape ? "fait" : ""}>
            <button onClick={() => setEtape(i)} disabled={i > etape && !peutAvancer}>{n}</button>
          </li>
        ))}
      </ol>

      {etape === 0 && (
        <div className="section">
          <div>
            <label className="etiquette-champ" htmlFor="c-nom">Nom du personnage</label>
            <input id="c-nom" className="champ" value={fiche.nom} onChange={(e) => maj({ nom: e.target.value })} />
          </div>
          <div>
            <label className="etiquette-champ">Race</label>
            <Choix
              etiquette="Race"
              vide="Choisir une race…"
              valeur={fiche.race}
              options={cat.races.map((r) => ({ v: r.id, l: r.nom }))}
              onChange={(v) => maj({ race: v, bonus_racial_libre: null })}
            />
          </div>
          {race && (
            <div className="carte">
              <p className="doux">
                Taille {race.taille} · vitesse {race.vitesse} pi · ajustements :{" "}
                {race.ajustements.map(([c, v]) => `${v > 0 ? "+" : "−"}${Math.abs(v)} ${c.toUpperCase()}`).join(", ") || "aucun fixe"}
                {race.choixLibre != null ? ` · +${race.choixLibre} à la caractéristique de votre choix` : ""}
              </p>
              {race.choixLibre != null && (
                <div className="groupe-boutons">
                  {CARACS.map((c) => (
                    <button key={c} className={"bouton" + (fiche.bonus_racial_libre === c ? " principal" : "")} onClick={() => maj({ bonus_racial_libre: c })}>
                      {NOMS_CARAC[c]}
                    </button>
                  ))}
                </div>
              )}
              <details>
                <summary>Traits de la race</summary>
                {race.traits.map((t) => <p key={t.nom}><strong>{t.nom}.</strong> {t.texte}</p>)}
              </details>
            </div>
          )}
          <div>
            <label className="etiquette-champ">Lignage ({classe.nom})</label>
            <Choix
              etiquette="Lignage"
              vide="Choisir un lignage…"
              valeur={fiche.lignage ?? ""}
              options={classe.lignages.map((l) => ({ v: l.id, l: l.nom }))}
              onChange={(v) => maj({ lignage: v || null, connaissance_de_lignage: null })}
            />
          </div>
          {lignage && (
            <div className="carte">
              <p className="doux">
                Compétence de classe : {lignage.competenceDeClasse} · Arcane : {lignage.arcane}
              </p>
              {lignage.choixConnaissance && (
                <Choix
                  etiquette="Sous-type de Connaissances du lignage"
                  vide="Choisir la Connaissance…"
                  valeur={fiche.connaissance_de_lignage ?? ""}
                  options={cat.competences
                    .filter((c) => c.id.startsWith("knowledge:"))
                    .map((c) => ({ v: c.id.split(":")[1], l: c.nom }))}
                  onChange={(v) => maj({ connaissance_de_lignage: v || null })}
                />
              )}
            </div>
          )}
        </div>
      )}

      {etape === 1 && (
        <div className="section">
          <p className="doux">
            Méthode de la campagne : {regle?.choix.find(([v]) => v === methode)?.[1] ?? methode} (modifiable dans Paramètres).
            {points === null && " Saisissez ici les scores obtenus avec vos dés."}
          </p>
          {points !== null && (
            <p>
              Points dépensés : <strong>{depense}</strong> / {points}
              {depense > points && <span className="alerte-texte"> (trop)</span>}
            </p>
          )}
          <table className="tableau">
            <thead><tr><th>Caractéristique</th><th>Score de base</th><th>Coût</th><th>Score final</th></tr></thead>
            <tbody>
              {CARACS.map((c) => {
                const fin = apercu?.caracteristiques.find((x) => x.id === c.toLowerCase());
                return (
                  <tr key={c}>
                    <td>{NOMS_CARAC[c]}</td>
                    <td>
                      {points !== null ? (
                        <Pas
                          etiquette={NOMS_CARAC[c]}
                          valeur={fiche.scores_base[c]}
                          min={7}
                          max={18}
                          onChange={(n) => maj({ scores_base: { ...fiche.scores_base, [c]: n } })}
                        />
                      ) : (
                        <input
                          className="champ court"
                          inputMode="numeric"
                          aria-label={NOMS_CARAC[c]}
                          value={fiche.scores_base[c]}
                          onChange={(e) => maj({ scores_base: { ...fiche.scores_base, [c]: Number(e.target.value) || 0 } })}
                        />
                      )}
                    </td>
                    <td className="num">{points !== null ? cout(fiche.scores_base[c]) : ""}</td>
                    <td className="num">{fin ? `${fin.score.total} (${signe(fin.modificateur)})` : "—"}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {etape === 2 && (
        <div className="section">
          <h3>Classe de prédilection</h3>
          <div className="groupe-boutons">
            <button className={"bouton" + (fiche.niveaux[0].predilection === "PointDeVie" ? " principal" : "")} onClick={() => majNiveau({ predilection: "PointDeVie" })}>+1 point de vie</button>
            <button className={"bouton" + (fiche.niveaux[0].predilection === "Rang" ? " principal" : "")} onClick={() => majNiveau({ predilection: "Rang" })}>+1 rang de compétence</button>
          </div>
          <h3>Compétences</h3>
          <p className="doux">Au niveau 1, au plus 1 rang par compétence. Rangs : {classe.rangsParNiveau} + modificateur d'Intelligence (au moins 1) + bonus de race.</p>
          <SelectionRangs
            cat={cat}
            apercu={apercu}
            rangs={fiche.niveaux[0].rangs}
            onChange={(rangs) => majNiveau({ rangs })}
            disponibles={rangsDisponibles}
            maximum={1}
          />
        </div>
      )}

      {etape === 3 && (
        <div className="section">
          <p className="doux">
            {classe.donBonusNiveau1 ? `Don gratuit de la classe : ${classe.donBonusNiveau1}. ` : ""}
            Dons à choisir : {donsAttendus}.
          </p>
          <SelectionDons
            cat={cat}
            nombre={donsAttendus}
            valeur={fiche.niveaux[0].dons}
            onChange={(dons) => majNiveau({ dons })}
            imposes={race?.donImpose ? [race.donImpose] : []}
          />
          {apercu && apercu.problemes.filter((p) => p.includes("don")).map((p, i) => <p key={i} className="alerte-texte">{p}</p>)}
        </div>
      )}

      {etape === 4 && (
        <div className="section">
          <p className="doux">
            Sorts connus au niveau 1 : {sortsAttendus.map(([n, k]) => `${k} de niveau ${n}`).join(", ")}. Les sorts de lignage s'ajoutent
            automatiquement aux niveaux prévus.
          </p>
          <SelectionSorts
            sorts={cat.sortsEnsorceleur}
            attendus={sortsAttendus}
            valeur={fiche.sorts_connus}
            onChange={(sorts_connus) => maj({ sorts_connus })}
          />
        </div>
      )}

      {etape === 5 && (
        <div className="section">
          {erreur && <p className="alerte-texte">{erreur}</p>}
          {apercu && (
            <>
              <div className="grille-fiche">
                <div className="carte chiffre-fiche"><span className="etiquette">Points de vie</span><span className="valeur">{apercu.pv.total}</span></div>
                <div className="carte chiffre-fiche"><span className="etiquette">Classe d'armure</span><span className="valeur">{apercu.ca.total}</span></div>
                <div className="carte chiffre-fiche"><span className="etiquette">Initiative</span><span className="valeur">{signe(apercu.initiative.total)}</span></div>
                <div className="carte chiffre-fiche"><span className="etiquette">Vigueur / Réflexes / Volonté</span><span className="valeur">{signe(apercu.vigueur.total)} / {signe(apercu.reflexes.total)} / {signe(apercu.volonte.total)}</span></div>
              </div>
              {apercu.problemes.length > 0 ? (
                <section className="alerte grave" role="alert">
                  <span className="point" />
                  <div className="texte">
                    <strong>À corriger avant de créer la fiche :</strong>
                    <ul>{apercu.problemes.map((m, i) => <li key={i}>{m}</li>)}</ul>
                  </div>
                </section>
              ) : (
                <p>La fiche est valide.</p>
              )}
              <SaisieNombre etiquette="Niveau du personnage à la création" min={1} max={20} valeur={niveauVise} onChange={setNiveauVise} />
              {pj.acolyteDe && (
                <p className="doux">
                  {pj.nom} est l'acolyte de {pj.acolyteDe} : son niveau est limité par le tableau du don Prestige et par le niveau de son maître (au
                  moins deux niveaux de moins).
                </p>
              )}
              {(niveauVise ?? 1) > 1 && (
                <p className="doux">
                  La fiche est créée au niveau 1, puis les XP du niveau {niveauVise} (selon la vitesse de progression de la campagne, ou le
                  jalon) lui sont accordés et l'assistant vous fait monter les niveaux un par un, avec tous les choix (dé de vie, rangs, dons,
                  sorts).
                </p>
              )}
              <button className="bouton principal" disabled={apercu.problemes.length > 0 || niveauVise === null} onClick={creer}>Créer la fiche</button>
            </>
          )}
        </div>
      )}

      <div className="ligne">
        <button className="bouton" disabled={etape === 0} onClick={() => setEtape(etape - 1)}>Précédent</button>
        {etape < ETAPES.length - 1 && (
          <button className="bouton principal" disabled={!peutAvancer} onClick={() => setEtape(etape + 1)}>Suivant</button>
        )}
      </div>
    </section>
  );
}
