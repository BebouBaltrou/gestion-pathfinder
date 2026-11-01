import { useCallback, useEffect, useRef, useState } from "react";
import { api, EtatApp, Retour } from "./api";
import { chercherMiseAJour, miseAJourActive, Nouvelle } from "./composants/MiseAJour";
import { icones } from "./icones";
import { Campagnes } from "./pages/Campagnes";
import { EnAttente } from "./pages/EnAttente";
import { Localites } from "./pages/Localites";
import { Batiments } from "./pages/Batiments";
import { Gestionnaires } from "./pages/Gestionnaires";
import { Historique } from "./pages/Historique";
import { Notes } from "./pages/Notes";
import { PanneauNotes } from "./composants/PanneauNotes";
import { Inventaire } from "./pages/Inventaire";
import { Parametres } from "./pages/Parametres";
import { Modeles } from "./pages/Modeles";
import { Points } from "./pages/Points";
import { Referentiel } from "./pages/Referentiel";
import { Organisations } from "./pages/Organisations";
import { Personnages } from "./pages/Personnages";
import { Recettes } from "./pages/Recettes";
import { Aventure } from "./pages/Aventure";
import { Tableau } from "./pages/Tableau";

export type Page =
  | "tableau" | "aventure" | "recettes" | "historique" | "notes"
  | "localites" | "batiments" | "organisations" | "gestionnaires" | "personnages" | "inventaire"
  | "referentiel" | "points" | "parametres" | "campagnes" | "modeles";

const NAVIGATION: { titre: string; pages: [Page, string][] }[] = [
  { titre: "Table de jeu", pages: [["aventure", "Aventure et combat"]] },
  { titre: "Campagne", pages: [["tableau", "Tableau de bord"], ["recettes", "Recettes"], ["notes", "Notes"], ["historique", "Historique"]] },
  {
    titre: "Domaines",
    pages: [["localites", "Localités"], ["batiments", "Bâtiments"], ["organisations", "Organisations"], ["gestionnaires", "Gestionnaires"], ["personnages", "Personnages"], ["inventaire", "Inventaire"], ["modeles", "Modèles personnels"]],
  },
  { titre: "Référence", pages: [["referentiel", "Référentiel"], ["points", "Points à confirmer"], ["parametres", "Paramètres"], ["campagnes", "Campagnes"]] },
];

export const NOMS_PAGES: Record<Page, string> = Object.fromEntries(
  NAVIGATION.flatMap((g) => g.pages),
) as Record<Page, string>;

type Notification = { id: number; genre: "ok" | "erreur" | "alerte"; texte: string };

/** Contexte passé aux pages : agir, puis tout se rafraîchit. */
export type Outils = {
  agir: (p: Promise<Retour>) => Promise<Retour | null>;
  aller: (p: Page) => void;
  version: number;
  etat: EtatApp;
};

export function App() {
  const [etat, setEtat] = useState<EtatApp | null>(null);
  const [page, setPage] = useState<Page>("tableau");
  const [version, setVersion] = useState(0);
  const [panneauNotes, setPanneauNotes] = useState(false);
  const [notifs, setNotifs] = useState<Notification[]>([]);
  const prochain = useRef(1);

  const notifier = useCallback((genre: Notification["genre"], texte: string) => {
    const id = prochain.current++;
    setNotifs((n) => [...n.slice(-3), { id, genre, texte }]);
    window.setTimeout(() => setNotifs((n) => n.filter((x) => x.id !== id)), genre === "ok" ? 4500 : 9000);
  }, []);

  const rafraichir = useCallback(async () => {
    try {
      const e = await api.etatApp();
      setEtat(e);
      setVersion((v) => v + 1);
      if (!e.entete) setPage("campagnes");
    } catch (err) {
      notifier("erreur", String(err));
    }
  }, [notifier]);

  const agir = useCallback(
    async (p: Promise<Retour>) => {
      try {
        const r = await p;
        notifier(!r.ok ? "erreur" : r.alerte ? "alerte" : "ok", r.message);
        await rafraichir();
        return r;
      } catch (err) {
        notifier("erreur", String(err));
        return null;
      }
    },
    [notifier, rafraichir],
  );

  useEffect(() => {
    rafraichir();
  }, [rafraichir]);

  // Mise à jour : une seule vérification au lancement, si l'option est active
  const [maj, setMaj] = useState<Nouvelle | null>(null);
  useEffect(() => {
    if (miseAJourActive()) chercherMiseAJour().then(setMaj);
  }, []);

  // Ctrl+Z / Ctrl+Y (hors des champs de saisie)
  useEffect(() => {
    const clavier = (e: KeyboardEvent) => {
      if (e.altKey && e.key.toLowerCase() === "n") {
        e.preventDefault();
        setPanneauNotes((v) => !v);
        return;
      }
      const cible = e.target as HTMLElement | null;
      if (cible && ["INPUT", "TEXTAREA", "SELECT"].includes(cible.tagName)) return;
      if (!(e.ctrlKey || e.metaKey)) return;
      const k = e.key.toLowerCase();
      if (k === "z" && !e.shiftKey) {
        e.preventDefault();
        agir(api.annuler());
      } else if (k === "y" || (k === "z" && e.shiftKey)) {
        e.preventDefault();
        agir(api.retablir());
      }
    };
    window.addEventListener("keydown", clavier);
    return () => window.removeEventListener("keydown", clavier);
  }, [agir]);

  if (!etat) return <div className="coquille" />;

  const entete = etat.entete;
  const pastilles = new Map(entete?.pastilles ?? []);
  const outils: Outils = { agir, aller: setPage, version, etat };
  const pageActive: Page = entete ? page : "campagnes";

  return (
    <div className="coquille">
      <nav className="navigation" aria-label="Navigation principale">
        <div className="marque">
          <div className="marque-logo">
            <svg width="46" height="46" viewBox="0 0 48 48" aria-hidden="true">
              <path d="M24 2 L25.9 19.4 L33.2 14.8 L28.6 22.1 L46 24 L28.6 25.9 L33.2 33.2 L25.9 28.6 L24 46 L22.1 28.6 L14.8 33.2 L19.4 25.9 L2 24 L19.4 22.1 L14.8 14.8 L22.1 19.4 Z" fill="currentColor" />
              <circle cx="24" cy="24" r="3" fill="var(--panneau)" />
            </svg>
          </div>
          <div className="marque-texte">
            <div className="marque-nom">Gestion Pathfinder</div>
            <div className="marque-sous">Guide de campagne</div>
          </div>
        </div>
        {NAVIGATION.map((g) => (
          <div className="groupe-nav" key={g.titre}>
            <div className="groupe-nav-titre">{g.titre}</div>
            {g.pages.map(([id, nom]) => (
              <button
                key={id}
                className={"lien-nav" + (pageActive === id ? " actif" : "")}
                onClick={() => setPage(id)}
                disabled={!entete && id !== "campagnes"}
                aria-current={pageActive === id ? "page" : undefined}
                title={nom}
              >
                {icones[id]({ taille: 18 })}
                <span className="libelle">{nom}</span>
                {pastilles.get(id) && <span className="pastille">{pastilles.get(id)}</span>}
              </button>
            ))}
          </div>
        ))}
      </nav>

      <div className="zone-principale">
        <header className="barre-haut">
          <div className="campagne">
            <small>Campagne</small>
            <strong>{entete?.campagne ?? "Aucune campagne ouverte"}</strong>
          </div>
          {entete && (
            <>
              <div className="date" title={`Jour ${entete.jour} de la campagne`}>
                {icones.calendrier({ taille: 18 })}
                <strong>{entete.date}</strong>
                <span className="jour-campagne">jour {entete.jour}</span>
              </div>
              <button
                className={"bouton" + (panneauNotes ? " principal" : "")}
                aria-pressed={panneauNotes}
                title="Notes de la partie (Alt+N)"
                onClick={() => setPanneauNotes((v) => !v)}
              >
                {icones.notes({})} <span className="masquable">Notes</span>
              </button>
              <button
                className="bouton icone"
                aria-label="Annuler"
                title={entete.derniereAction ? `Annuler : ${entete.derniereAction} (Ctrl+Z)` : "Rien à annuler"}
                disabled={!entete.peutAnnuler || etat.combatEnCours}
                onClick={() => agir(api.annuler())}
              >
                {icones.annuler({})}
              </button>
              <button
                className="bouton icone"
                aria-label="Rétablir"
                title="Rétablir (Ctrl+Y)"
                disabled={!entete.peutRetablir || etat.combatEnCours}
                onClick={() => agir(api.retablir())}
              >
                {icones.retablir({})}
              </button>
              {!etat.simulation && (
                <button
                  className="bouton"
                  title="Essayer des actions sur une copie de la campagne, puis les abandonner ou les garder"
                  onClick={() => agir(api.simulation("demarrer"))}
                >
                  {icones.fiole({})} <span className="masquable">Simuler</span>
                </button>
              )}
              <div className="groupe-boutons">
                <button className="bouton" disabled={etat.combatEnCours} onClick={() => agir(api.avancer(7))}>+7 j</button>
                <button className="bouton" disabled={etat.combatEnCours} onClick={() => agir(api.avancer(30))}>+30 j</button>
              </div>
              <button className="bouton principal" disabled={etat.combatEnCours} onClick={() => agir(api.avancer(1))}>
                {icones.suivant({})} Jour suivant
              </button>
            </>
          )}
        </header>

        {etat.combatEnCours && (
          <div className="bandeau-simulation" role="status">
            <strong>COMBAT EN COURS</strong>
            <span>Le reste de la campagne est verrouillé jusqu'à la fin du combat (validation ou abandon).</span>
            <span className="espace" />
            <button className="bouton principal" onClick={() => setPage("aventure")}>Retourner au combat</button>
          </div>
        )}

        {etat.simulation && (
          <div className="bandeau-simulation" role="status">
            <strong>MODE SIMULATION</strong>
            <span>
              Rien n'est enregistré. {etat.simulation[0]} action(s) et {etat.simulation[1]} jour(s) d'avance sur la
              campagne réelle.
            </span>
            <span className="espace" />
            <button className="bouton" onClick={() => agir(api.simulation("abandonner"))}>Abandonner</button>
            <button className="bouton principal" onClick={() => agir(api.simulation("garder"))}>Garder la simulation</button>
          </div>
        )}

        {maj && (
          <div className="doux" role="status" style={{ padding: "6px 12px" }}>
            Version {maj.version} disponible.{" "}
            <a href={maj.page} target="_blank" rel="noreferrer">
              Voir la nouveauté
            </a>{" "}
            <button className="bouton discret" onClick={() => setMaj(null)}>
              Masquer
            </button>
          </div>
        )}
        <main className="contenu">
          {pageActive === "tableau" && <Tableau outils={outils} />}
          {pageActive === "aventure" && <Aventure outils={outils} />}
          {pageActive === "campagnes" && <Campagnes outils={outils} />}
          {pageActive === "localites" && <Localites outils={outils} />}
          {pageActive === "personnages" && <Personnages outils={outils} />}
          {pageActive === "inventaire" && <Inventaire outils={outils} />}
          {pageActive === "notes" && <Notes outils={outils} />}
          {pageActive === "batiments" && <Batiments outils={outils} />}
          {pageActive === "organisations" && <Organisations outils={outils} />}
          {pageActive === "gestionnaires" && <Gestionnaires outils={outils} />}
          {pageActive === "historique" && <Historique outils={outils} />}
          {pageActive === "points" && <Points outils={outils} />}
          {pageActive === "referentiel" && <Referentiel outils={outils} />}
          {pageActive === "parametres" && <Parametres outils={outils} />}
          {pageActive === "modeles" && <Modeles outils={outils} />}
          {pageActive === "recettes" && <Recettes outils={outils} />}
          {!["tableau", "aventure", "campagnes", "localites", "personnages", "inventaire", "notes", "batiments", "recettes", "organisations", "gestionnaires", "historique", "points", "parametres", "referentiel", "modeles"].includes(pageActive) && <EnAttente nom={NOMS_PAGES[pageActive]} />}
        </main>
      </div>

      {panneauNotes && entete && <PanneauNotes outils={outils} onFermer={() => setPanneauNotes(false)} />}

      <div className="notifications" aria-live="polite">
        {notifs.map((n) => (
          <div key={n.id} className={"notification " + n.genre}>
            <span className="point" />
            <span className="texte">{n.texte}</span>
            <button aria-label="Fermer" onClick={() => setNotifs((x) => x.filter((y) => y.id !== n.id))}>
              {icones.fermer({ taille: 16 })}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
