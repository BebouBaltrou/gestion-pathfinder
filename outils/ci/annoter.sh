#!/usr/bin/env bash
# Lance une commande ; en cas d'échec, publie ses erreurs comme annotation GitHub
# (lisible via l'API checks, sans télécharger les journaux).
set -o pipefail
"$@" 2>&1 | tee /tmp/sortie.txt
code=${PIPESTATUS[0]}
if [ "$code" -ne 0 ]; then
  extrait=$(grep -E -A14 '^(error|warning)(\[|:)|panicked|FAILED|^failures:' /tmp/sortie.txt | head -n 160)
  [ -z "$extrait" ] && extrait=$(tail -n 80 /tmp/sortie.txt)
  extrait=${extrait//'%'/'%25'}
  extrait=${extrait//$'\r'/}
  extrait=${extrait//$'\n'/'%0A'}
  echo "::error title=$1 $2 a échoué::$extrait"
fi
exit "$code"
