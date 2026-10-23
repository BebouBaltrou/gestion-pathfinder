export function EnAttente({ nom }: { nom: string }) {
  return (
    <div className="page">
      <div className="titre-page">
        <h1>{nom}</h1>
      </div>
      <div className="carte vide">
        <p>
          Cette page arrive bientôt dans la nouvelle interface. En attendant, l'ancienne application reste disponible et
          travaille sur les mêmes campagnes : ce que vous y faites apparaît ici.
        </p>
      </div>
    </div>
  );
}
