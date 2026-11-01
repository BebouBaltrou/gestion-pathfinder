// Construit l'interface dans dist/ : un seul script, une feuille de style, la police Inter.
// MOCK=1 : données d'exemple sans Tauri (aperçu dans un navigateur, captures d'écran).
import { build } from "esbuild";
import { cpSync, mkdirSync, rmSync, existsSync } from "node:fs";

const mock = process.env.MOCK === "1";
const dist = process.env.DIST || "dist";
rmSync(dist, { recursive: true, force: true });
mkdirSync(dist, { recursive: true });
cpSync("ui/index.html", `${dist}/index.html`);

// la police n'est disponible qu'après « npm install » ; sans elle, police du système
const police = existsSync("node_modules/@fontsource-variable/inter");

await build({
  entryPoints: ["ui/src/main.tsx"],
  bundle: true,
  minify: !mock,
  sourcemap: mock,
  format: "esm",
  target: "es2022",
  jsx: "automatic",
  outfile: `${dist}/app.js`,
  define: {
    "process.env.NODE_ENV": mock ? '"development"' : '"production"',
    __MOCK__: mock ? "true" : "false",
    __MAJ_TOKEN__: JSON.stringify(process.env.VITE_MAJ_TOKEN || ""),
  },
  loader: { ".woff2": "file", ".woff": "file" },
  assetNames: "polices/[name]-[hash]",
  nodePaths: process.env.NODE_PATH ? process.env.NODE_PATH.split(":") : [],
  logLevel: "info",
});

if (police) {
  await build({
    entryPoints: ["ui/polices.css"],
    bundle: true,
    minify: true,
    outfile: `${dist}/polices.css`,
    loader: { ".woff2": "file", ".woff": "file" },
    assetNames: "polices/[name]-[hash]",
    logLevel: "info",
  });
}
