// Le don Prestige : score, modificateurs, acolyte (un vrai personnage avec sa fiche) et partisans
// (simples effectifs par niveau). Tout est calculé et contrôlé par le cœur.

import { useEffect, useState } from "react";
import type { Outils } from "../../App";
import { api } from "../../api";
import { SaisieNombre } from "../../composants/jeu";
import type { LignePersonnage, PrestigeDonVue } from "../../fiche-types";

export function PrestigeDon({ outils, pj }: { outils: Outils; pj: LignePersonnage }) {
  const [v, setV] = useState<PrestigeDonVue | null>(null);
  const [nomAcolyte, setNomAcolyte] = useState("");
  const [nomPartisan, setNomPartisan] = useState("");
  const [niveauPartisan, setNiveauPartisan] = useState<number | null>(1);
  useEffect(() => {
    api.prestigeDon(pj.id).then(setV);
  }, [pj.id, outils.version]);
  if (!v) return null;

  if (!v.actif) {
    return (
      <div className="section">
        <p className="doux">
          Ce personnage n'a pas le don Prestige (niveau {v.niveauMinimal} minimum). Une fois le don choisi à la montée de niveau, cet onglet
          permet de recruter un acolyte et des partisans.
        </p>
      </div>
    );
  }

  const modifier = (id: string, compte: number) => {
    const actuels: Record<string, number> = {};
    v.modificateurs.forEach((m) => {
      if (m.compte > 0) actuels[m.id] = m.compte;
    });
    if (compte > 0) actuels[id] = compte;
    else delete actuels[id];
    return outils.agir(api.prestigeEvenement({ PrestigeDonRegle: { pj: pj.id, modificateurs: actuels } }));
  };

  const groupe = (titre: string, portee: string) => (
    <div className="bloc">
      <h3>{titre}</h3>
      {v.modificateurs
        .filter((m) => m.portee === portee)
        .map((m) => (
          <label className="interrupteur" key={m.id}>
            {m.cumulable ? (
              <input
                type="number"
                min={0}
                max={20}
                value={m.compte}
                style={{ width: 56 }}
                aria-label={m.libelle}
                onChange={(e) => modifier(m.id, Math.max(0, Number(e.target.value) || 0))}
              />
            ) : (
              <input type="checkbox" checked={m.compte > 0} onChange={(e) => modifier(m.id, e.target.checked ? 1 : 0)} />
            )}
            <span>
              {m.libelle}{" "}
              <strong>
                ({m.valeur > 0 ? "+" : ""}
                {m.valeur})
              </strong>
            </span>
          </label>
        ))}
    </div>
  );

  const recruterAcolyte = async () => {
    const nom = nomAcolyte.trim();
    if (!nom) return;
    const r = await outils.agir(api.recruterAcolyte(pj.id, nom));
    if (r?.ok) setNomAcolyte("");
  };

  return (
    <div className="section">
      <div className="grille-fiche">
        <div className="carte chiffre-fiche">
          <span className="etiquette">Score de Prestige (acolyte)</span>
          <span className="valeur">{v.scoreAcolyte ?? "?"}</span>
        </div>
        <div className="carte chiffre-fiche">
          <span className="etiquette">Score de Prestige (partisans)</span>
          <span className="valeur">{v.scorePartisans ?? "?"}</span>
        </div>
        <div className="carte chiffre-fiche">
          <span className="etiquette">Niveau maximal de l'acolyte</span>
          <span className="valeur">{v.niveauMaxAcolyte}</span>
          <span className="doux">tableau : {v.niveauTableau}, et au moins 2 niveaux sous {pj.nom}</span>
        </div>
      </div>
      <p className="doux">
        Score de base : niveau du personnage + modificateur de Charisme, puis les modificateurs ci-dessous. Source : don Prestige (Leadership,
        Core).
      </p>

      <div className="carte section">
        <h2>Modificateurs</h2>
        {groupe("Réputation (acolyte et partisans)", "tous")}
        {groupe("Seulement pour l'acolyte", "acolyte")}
        {groupe("Seulement pour les partisans", "partisans")}
      </div>

      <div className="carte section">
        <h2>Acolyte</h2>
        {v.acolyte ? (
          <>
            <p>
              <strong>{v.acolyte.nom}</strong>
              {v.acolyte.aUneFiche
                ? ` · niveau ${v.acolyte.niveau} · ${v.acolyte.xp} XP`
                : " · pas encore de fiche (ouvrez-le dans Personnages pour la créer)"}
            </p>
            <p className="doux">
              Il reçoit automatiquement une part de l'XP de {pj.nom} : XP gagnés × niveau de l'acolyte ÷ niveau de {pj.nom}, sans jamais atteindre le
              niveau « {pj.nom} moins un ».
            </p>
            <div className="rangee-boutons">
              <button
                className="bouton danger"
                onClick={() => {
                  if (v.acolyte && window.confirm(`${v.acolyte.nom} cesse d'être votre acolyte ? Son personnage et sa fiche sont conservés.`))
                    outils.agir(api.prestigeEvenement({ AcolyteDelie: { acolyte: v.acolyte.id } }));
                }}
              >
                Délier l'acolyte
              </button>
            </div>
          </>
        ) : v.niveauMaxAcolyte === 0 ? (
          <p className="doux">Le score actuel ne permet pas d'avoir un acolyte.</p>
        ) : (
          <>
            <p className="doux">Pas d'acolyte pour l'instant. Niveau maximal : {v.niveauMaxAcolyte}.</p>
            {v.candidats.length > 0 && (
              <div className="rangee-boutons">
                {v.candidats.map(([id, nom]) => (
                  <button key={id} className="bouton" onClick={() => outils.agir(api.prestigeEvenement({ AcolyteLie: { acolyte: id, maitre: pj.id } }))}>
                    Prendre {nom} comme acolyte
                  </button>
                ))}
              </div>
            )}
            <div className="champs">
              <label className="champ-g">
                <span>Nom du nouvel acolyte</span>
                <input className="champ-jeu" value={nomAcolyte} onChange={(e) => setNomAcolyte(e.target.value)} />
              </label>
              <button className="bouton principal" disabled={!nomAcolyte.trim()} onClick={recruterAcolyte}>
                Recruter un nouvel acolyte
              </button>
            </div>
          </>
        )}
      </div>

      <div className="carte section">
        <h2>Partisans</h2>
        <p className="doux">Ils n'ont ni fiche ni XP ; le tableau est relu à chaque montée de niveau de {pj.nom}.</p>
        <table className="tableau-simple">
          <thead>
            <tr>
              <th>Niveau</th>
              <th>Permis</th>
              <th>Recrutés</th>
            </tr>
          </thead>
          <tbody>
            {v.places.map((p) => (
              <tr key={p.niveau}>
                <td>{p.niveau}</td>
                <td>{p.permis}</td>
                <td>{p.pris}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {v.partisans.map((p) => (
          <div className="ligne-objet" key={p.index}>
            <span className="nom">
              {p.nom || "(sans nom)"} <span className="doux">niveau {p.niveau}</span>
            </span>
            <button className="bouton mini danger" onClick={() => outils.agir(api.prestigeEvenement({ PartisanRetire: { maitre: pj.id, index: p.index } }))}>
              Retirer
            </button>
          </div>
        ))}
        <div className="champs">
          <SaisieNombre etiquette="Niveau du partisan (1 à 6)" min={1} max={6} valeur={niveauPartisan} onChange={setNiveauPartisan} />
          <label className="champ-g">
            <span>Nom (facultatif)</span>
            <input className="champ-jeu" value={nomPartisan} onChange={(e) => setNomPartisan(e.target.value)} />
          </label>
          <button
            className="bouton principal"
            disabled={niveauPartisan === null}
            onClick={async () => {
              const r = await outils.agir(
                api.prestigeEvenement({ PartisanAjoute: { maitre: pj.id, niveau: niveauPartisan ?? 1, nom: nomPartisan.trim() } }),
              );
              if (r?.ok) setNomPartisan("");
            }}
          >
            Ajouter un partisan
          </button>
        </div>
      </div>
    </div>
  );
}
