// Fabrique toutes les images de l'appli à partir de la mascotte Bati dessinée ici en SVG
// (d'après site-codenco : src/app/api/bati/story/bati-svg.tsx et src/lib/bati/mascot-couleurs.ts, Bati bleu).
//
//   node images/fabriquer.mjs <dossier des polices TTF Be Vietnam Pro> [dossier des aperçus]
//
// sharp est pris dans ../site-codenco (rien à installer ici). Les polices : Be Vietnam Pro Bold et SemiBold en TTF
// (Google Fonts, licence OFL), le moteur de texte ne lit pas le WOFF2 du site.

import { createRequire } from "node:module";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const RACINE = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const sharp = createRequire(join(RACINE, "..", "site-codenco", "package.json"))("sharp");
const [POLICES, APERCUS] = process.argv.slice(2);
if (!POLICES) throw new Error("Donner le dossier des polices TTF Be Vietnam Pro");

// Couleurs du site : --color-bg, --color-blue, et le Bati bleu (COULEURS_BATI.bleu).
const NUIT = "#0a0d15";
const BLEU = "#4a8cff";
const BATI = { face: ["#4a5b97", "#2e3d78", "#1d2a5f"], rim: "#6aa8ff", glow: "#4a8cff", eyec: "#a8ceff" };

// ───────────────────────── La mascotte ─────────────────────────

/** Le corps carré (5 × 5 unités) de Bati bleu, visage « normal », un peu agrandi pour se lire en icône. */
function mascotte(id) {
  const visage = 1.2; // agrandissement des yeux et de la bouche autour du centre du visage
  const yOeil = 2.124; // 0,42 × 4,7 + 0,15 : le visage du site recentré dans un corps carré
  const eye = { w: 0.48, h: 0.52 };
  const oeil = (cx) => `<ellipse cx="${cx}" cy="${yOeil + eye.h / 2}" rx="${eye.w / 2}" ry="${eye.h / 2}"/>`;
  const g = 1.145; // (5 − (2 × 0,48 + 1,75)) / 2
  const yeux = oeil(g + eye.w / 2) + oeil(g + eye.w + 1.75 + eye.w / 2);
  // .bt-mouth « sourire » : 0,8 × 0,3, en haut à 0,56 × 4,7 (+ 0,15).
  const x0 = 2.1, y0 = 2.782, w = 0.8, h = 0.3;
  const bouche = `M${x0 + 0.0375 * w} ${y0 + 0.133 * h} C${x0 + 0.0875 * w} ${y0 + 1.1 * h} ${x0 + 0.9125 * w} ${y0 + 1.1 * h} ${x0 + 0.9625 * w} ${y0 + 0.133 * h}`;
  return {
    defs: `
    <linearGradient id="${id}-face" gradientUnits="userSpaceOnUse" x1="1.997" y1="-0.352" x2="3.003" y2="5.352">
      <stop offset="0" stop-color="${BATI.face[0]}"/><stop offset="0.45" stop-color="${BATI.face[1]}"/><stop offset="1" stop-color="${BATI.face[2]}"/>
    </linearGradient>
    <linearGradient id="${id}-voile" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#fff" stop-opacity="0.15"/><stop offset="0.12" stop-color="#fff" stop-opacity="0"/>
      <stop offset="0.7" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity="0.35"/>
    </linearGradient>
    <filter id="${id}-lueur" filterUnits="userSpaceOnUse" x="-1" y="-1" width="7" height="7"><feGaussianBlur stdDeviation="0.12"/></filter>
    <clipPath id="${id}-corps"><rect width="5" height="5" rx="1.25"/></clipPath>`,
    corps: `
    <rect width="5" height="5" rx="1.25" fill="url(#${id}-face)"/>
    <rect width="5" height="5" rx="1.25" fill="url(#${id}-voile)"/>
    <g clip-path="url(#${id}-corps)">
      <rect x="0.245" y="0.245" width="4.51" height="4.51" rx="1.005" fill="none" stroke="${BATI.glow}" stroke-width="0.3" opacity="0.55" filter="url(#${id}-lueur)"/>
    </g>
    <rect x="0.245" y="0.245" width="4.51" height="4.51" rx="1.005" fill="none" stroke="${BATI.rim}" stroke-width="0.09"/>
    <g transform="translate(2.5 2.55) scale(${visage}) translate(-2.5 -2.55)">
      <g fill="${BATI.glow}" opacity="0.8" filter="url(#${id}-lueur)">${yeux}</g>
      <g fill="${BATI.eyec}">${yeux}</g>
      <path d="${bouche}" fill="none" stroke="${BATI.eyec}" stroke-width="0.1" stroke-linecap="round"/>
    </g>`,
  };
}

/** Pour 16, 24 et 32 px : la même mascotte simplifiée, dessinée sur la grille de 16 px (traits nets, gros yeux). */
const PETITE = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
  <defs><linearGradient id="p" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0" stop-color="${BATI.face[0]}"/><stop offset="1" stop-color="${BATI.face[2]}"/>
  </linearGradient></defs>
  <rect width="16" height="16" rx="4" fill="url(#p)"/>
  <rect x="1.5" y="1.5" width="13" height="13" rx="2.8" fill="none" stroke="${BATI.rim}" stroke-width="1"/>
  <rect x="4" y="6" width="2" height="2.5" rx="0.8" fill="#cfe3ff"/>
  <rect x="10" y="6" width="2" height="2.5" rx="0.8" fill="#cfe3ff"/>
  <path d="M6.3 10.6 Q8 12.4 9.7 10.6" fill="none" stroke="#cfe3ff" stroke-width="1" stroke-linecap="round"/>
</svg>`;

/** En 16 px exactement : chaque trait tombe sur un pixel (yeux 2 × 2, sourire de 4 pixels). */
const PETITE_16 = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16" shape-rendering="crispEdges">
  <defs><linearGradient id="p" x1="0" y1="0" x2="0" y2="1">
    <stop offset="0" stop-color="${BATI.face[0]}"/><stop offset="1" stop-color="${BATI.face[2]}"/>
  </linearGradient></defs>
  <rect width="16" height="16" rx="4" fill="url(#p)" shape-rendering="geometricPrecision"/>
  <rect x="1.5" y="1.5" width="13" height="13" rx="2.8" fill="none" stroke="${BATI.rim}" stroke-width="1" shape-rendering="geometricPrecision"/>
  <g fill="#cfe3ff"><rect x="4" y="6" width="2" height="2"/><rect x="10" y="6" width="2" height="2"/>
  <rect x="5" y="10" width="1" height="1"/><rect x="10" y="10" width="1" height="1"/><rect x="6" y="11" width="4" height="1"/></g>
</svg>`;

/** Windows : la mascotte remplit tout le carré, coins transparents. */
const icone = () => {
  const m = mascotte("w");
  return `<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 5 5"><defs>${m.defs}</defs>${m.corps}</svg>`;
};

/** Mac : grille Apple, la forme fait 824 px dans 1024, centrée, avec une légère ombre portée. */
const iconeMac = () => {
  const m = mascotte("m");
  const k = 824 / 5;
  return `<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="0 0 1024 1024">
  <defs>${m.defs}<filter id="ombre" x="-10%" y="-10%" width="120%" height="130%"><feDropShadow dx="0" dy="12" stdDeviation="14" flood-color="#000" flood-opacity="0.3"/></filter></defs>
  <g filter="url(#ombre)" transform="translate(100 100) scale(${k})">${m.corps}</g>
</svg>`;
};

// ───────────────────────── Formats de fichiers ─────────────────────────

/** Rendu net : la densité suit la taille voulue (les SVG font 16 ou 1024 px de base). */
const png = (svg, taille) => sharp(Buffer.from(svg), { density: Math.min(72 * 4, (72 * taille) / (svg === PETITE || svg === PETITE_16 ? 16 : 1024) * 4) }).resize(taille, taille).png().toBuffer();

/** .ico : une entrée PNG par taille (Windows Vista et suivants). */
function ico(images) {
  const tete = Buffer.alloc(6 + 16 * images.length);
  tete.writeUInt16LE(0, 0); tete.writeUInt16LE(1, 2); tete.writeUInt16LE(images.length, 4);
  let decalage = tete.length;
  images.forEach(({ taille, donnees }, i) => {
    const o = 6 + 16 * i;
    tete.writeUInt8(taille >= 256 ? 0 : taille, o); tete.writeUInt8(taille >= 256 ? 0 : taille, o + 1);
    tete.writeUInt16LE(1, o + 4); tete.writeUInt16LE(32, o + 6);
    tete.writeUInt32LE(donnees.length, o + 8); tete.writeUInt32LE(decalage, o + 12);
    decalage += donnees.length;
  });
  return Buffer.concat([tete, ...images.map((x) => x.donnees)]);
}

/** .icns : entrées PNG (icp4 16, icp5 32, ic07 128, ic08 256, ic09 512, ic10 1024, et les @2x). */
function icns(entrees) {
  const blocs = entrees.map(([type, donnees]) => {
    const t = Buffer.alloc(8); t.write(type, 0, "ascii"); t.writeUInt32BE(donnees.length + 8, 4);
    return Buffer.concat([t, donnees]);
  });
  const total = 8 + blocs.reduce((s, b) => s + b.length, 0);
  const t = Buffer.alloc(8); t.write("icns", 0, "ascii"); t.writeUInt32BE(total, 4);
  return Buffer.concat([t, ...blocs]);
}

/** BMP 24 bits, lignes de bas en haut : le seul format d'image accepté par l'installeur NSIS. */
async function bmp(image) {
  const { data, info } = await image.flatten({ background: NUIT }).removeAlpha().raw().toBuffer({ resolveWithObject: true });
  const { width: w, height: h } = info;
  const ligne = Math.ceil((w * 3) / 4) * 4;
  const f = Buffer.alloc(54 + ligne * h);
  f.write("BM", 0, "ascii"); f.writeUInt32LE(f.length, 2); f.writeUInt32LE(54, 10);
  f.writeUInt32LE(40, 14); f.writeInt32LE(w, 18); f.writeInt32LE(h, 22); f.writeUInt16LE(1, 26); f.writeUInt16LE(24, 28);
  f.writeUInt32LE(ligne * h, 34); f.writeInt32LE(2835, 38); f.writeInt32LE(2835, 42);
  for (let y = 0; y < h; y++)
    for (let x = 0; x < w; x++) {
      const s = (y * w + x) * 3, d = 54 + (h - 1 - y) * ligne + x * 3;
      f[d] = data[s + 2]; f[d + 1] = data[s + 1]; f[d + 2] = data[s];
    }
  return f;
}

/** Un texte en Be Vietnam Pro, rendu en PNG transparent. */
const texte = (t, { taille, graisse = "Bold", couleur = "#ffffff" }) =>
  sharp({ text: { text: `<span foreground="${couleur}">${t}</span>`, font: `Be Vietnam Pro ${graisse} ${taille}`, fontfile: join(POLICES, `BeVietnamPro-${graisse}.ttf`), rgba: true, dpi: 72 } }).png().toBuffer();

/** Place une image centrée sur x (ou à gauche si x n'est pas donné en centre). */
async function centre(buffer, cx, top) {
  const { width } = await sharp(buffer).metadata();
  return { input: buffer, left: Math.round(cx - width / 2), top: Math.round(top) };
}

/** La mascotte seule, en PNG transparent de la largeur voulue (avec son ombre au sol). */
const mascottePng = (largeur) => {
  const m = mascotte("s");
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="-0.3 -0.3 5.6 6.1"><defs>${m.defs}
    <radialGradient id="sol"><stop offset="0" stop-color="#000" stop-opacity="0.45"/><stop offset="1" stop-color="#000" stop-opacity="0"/></radialGradient>
    <filter id="halo" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="0.35"/></filter></defs>
    <ellipse cx="2.5" cy="5.45" rx="2.4" ry="0.3" fill="url(#sol)"/>
    <rect width="5" height="5" rx="1.25" fill="${BATI.glow}" opacity="0.35" filter="url(#halo)"/>
    ${m.corps}</svg>`;
  return sharp(Buffer.from(svg), { density: (72 * largeur * 4) / 5.6 }).resize(largeur).png().toBuffer();
};

/** Un fond nuit avec une lueur bleue (le fond de l'espace Bati). */
const fond = (w, h, cx, cy, r) => Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}">
  <defs><radialGradient id="l" cx="${cx}" cy="${cy}" r="${r}" gradientUnits="userSpaceOnUse">
    <stop offset="0" stop-color="${BLEU}" stop-opacity="0.28"/><stop offset="1" stop-color="${BLEU}" stop-opacity="0"/>
  </radialGradient></defs>
  <rect width="${w}" height="${h}" fill="${NUIT}"/><rect width="${w}" height="${h}" fill="url(#l)"/></svg>`);

// ───────────────────────── Fabrication ─────────────────────────

const ecrire = (chemin, donnees) => { const p = join(RACINE, chemin); mkdirSync(dirname(p), { recursive: true }); writeFileSync(p, donnees); };
const apercu = (nom, donnees) => APERCUS && (mkdirSync(APERCUS, { recursive: true }), writeFileSync(join(APERCUS, nom), donnees));

// Sources vectorielles
const SVG_WIN = icone(), SVG_MAC = iconeMac();
ecrire("images/icone.svg", SVG_WIN);
ecrire("images/icone-mac.svg", SVG_MAC);
ecrire("images/icone-petite.svg", PETITE);

// Icônes
const win = async (t) => (t <= 16 ? png(PETITE_16, t) : t <= 32 ? png(PETITE, t) : png(SVG_WIN, t));
const mac = async (t) => {
  if (t > 32) return png(SVG_MAC, t);
  // Petites tailles Mac : la petite mascotte dans la grille (824/1024 du cadre).
  const cote = Math.round((t * 824) / 1024);
  return sharp({ create: { width: t, height: t, channels: 4, background: { r: 0, g: 0, b: 0, alpha: 0 } } })
    .composite([{ input: await png(PETITE, cote), left: Math.round((t - cote) / 2), top: Math.round((t - cote) / 2) }]).png().toBuffer();
};

const tailles = {};
for (const t of [16, 24, 32, 48, 64, 128, 256, 512, 1024]) tailles[t] = await win(t);
ecrire("src-tauri/icons/icon.ico", ico([16, 24, 32, 48, 64, 256].map((t) => ({ taille: t, donnees: tailles[t] }))));
ecrire("src-tauri/icons/32x32.png", tailles[32]);
ecrire("src-tauri/icons/128x128.png", tailles[128]);
ecrire("src-tauri/icons/128x128@2x.png", tailles[256]);
ecrire("src-tauri/icons/icon.png", tailles[1024]);
ecrire("dist/bati.png", tailles[256]);

const m = {};
for (const t of [16, 32, 64, 128, 256, 512, 1024]) m[t] = await mac(t);
ecrire("src-tauri/icons/icon.icns", icns([["icp4", m[16]], ["icp5", m[32]], ["ic11", m[32]], ["ic12", m[64]], ["ic07", m[128]], ["ic13", m[256]], ["ic08", m[256]], ["ic14", m[512]], ["ic09", m[512]], ["ic10", m[1024]]]));

for (const t of [1024, 256, 32, 16]) { apercu(`icone-windows-${t}.png`, tailles[t]); apercu(`icone-mac-${t}.png`, m[t]); }

// Installeur Windows : image latérale 164 × 314 (accueil et fin)
{
  const W = 164, H = 314;
  const cote = await sharp(fond(W, H, 82, 120, 120)).composite([
    await centre(await mascottePng(96), 82, 62),
    await centre(await texte("Bati", { taille: 34 }), 82, 182),
    await centre(await texte("par CodeN'Co", { taille: 11, graisse: "SemiBold", couleur: "#8a90a0" }), 82, 228),
  ]).png().toBuffer();
  ecrire("src-tauri/nsis/cote.bmp", await bmp(sharp(cote)));
  apercu("nsis-cote-164x314.png", cote);
}

// Installeur Windows : bandeau 150 × 57 (pages intermédiaires, à droite de l'en-tête)
{
  const W = 150, H = 57;
  const nom = await texte("Bati", { taille: 22 });
  const { width: lt } = await sharp(nom).metadata();
  const mas = await mascottePng(40);
  const total = 40 + 10 + lt, x = Math.round((W - total) / 2);
  const bandeau = await sharp(fond(W, H, 40, 28, 70)).composite([
    { input: mas, left: x, top: 7 },
    { input: nom, left: x + 50, top: 14 },
  ]).png().toBuffer();
  ecrire("src-tauri/nsis/bandeau.bmp", await bmp(sharp(bandeau)));
  apercu("nsis-bandeau-150x57.png", bandeau);
}

// Fenêtre du .dmg : 660 × 360, Bati à gauche (165, 195), Applications à droite (495, 195), icônes de 128 px.
// Les noms sous les icônes sont écrits par le Finder, en noir ou en blanc selon le mode clair ou sombre :
// chacun repose sur un socle gris-bleu (#66759b) où les deux restent lisibles (contraste 4,5 environ).
{
  const W = 660, H = 360, Y = 195;
  const socle = (cx) => `<rect x="${cx - 62}" y="${Y + 72}" width="124" height="26" rx="13" fill="#66759b"/>`;
  const svg = Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" width="${W}" height="${H}">
    <defs>
      <radialGradient id="l" cx="330" cy="195" r="340" gradientUnits="userSpaceOnUse"><stop offset="0" stop-color="${BLEU}" stop-opacity="0.22"/><stop offset="1" stop-color="${BLEU}" stop-opacity="0"/></radialGradient>
      <filter id="g" x="-20%" y="-50%" width="140%" height="200%"><feGaussianBlur stdDeviation="4"/></filter>
    </defs>
    <rect width="${W}" height="${H}" fill="${NUIT}"/><rect width="${W}" height="${H}" fill="url(#l)"/>
    ${socle(165)}${socle(495)}
    <g fill="none" stroke="${BLEU}" stroke-width="6" stroke-linecap="round" stroke-linejoin="round">
      <g filter="url(#g)" opacity="0.7"><path d="M272 ${Y} H384 M364 ${Y - 18} L386 ${Y} L364 ${Y + 18}"/></g>
      <path d="M272 ${Y} H384 M364 ${Y - 18} L386 ${Y} L364 ${Y + 18}"/>
    </g></svg>`);
  const fondDmg = await sharp(svg).composite([
    await centre(await texte("Glissez Bati dans Applications", { taille: 24 }), W / 2, 40),
    await centre(await texte("puis ouvrez-le depuis le dossier Applications", { taille: 13, graisse: "Medium", couleur: "#a3a9b8" }), W / 2, 78),
  ]).png().toBuffer();
  ecrire("src-tauri/dmg/fond.png", fondDmg);
  apercu("dmg-fond-660x360.png", fondDmg);

  // Aperçu seulement : la fenêtre telle qu'elle apparaîtra, avec les deux icônes posées.
  const app = await mac(128);
  // Le nom en noir (mode clair) sous Bati, en blanc (mode sombre) sous Applications, pour juger les deux.
  const nomFinder = (t, couleur) => sharp({ text: { text: `<span foreground="${couleur}">${t}</span>`, font: "sans 13", rgba: true, dpi: 72 } }).png().toBuffer();
  apercu("dmg-simulation.png", await sharp(fondDmg).composite([
    { input: app, left: 165 - 64, top: Y - 64 },
    await centre(await nomFinder("Bati", "#000000"), 165, Y + 77),
    await centre(await nomFinder("Applications", "#ffffff"), 495, Y + 77),
  ]).png().toBuffer());
}

console.log("Images fabriquées.");
