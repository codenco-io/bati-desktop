# Bati pour ordinateur

L'application de bureau de Bati (Windows et Mac). Une fenêtre qui ouvre l'espace en ligne,
https://codenco.io/bati/espace/ : le contenu vient du site, il se met donc à jour tout seul.
L'appli elle-même ne se met pas à jour automatiquement.

- Une seule fenêtre, rouverte à la même taille et au même endroit. Relancer l'appli la remet devant.
- Les liens qui sortent de codenco.io (WhatsApp, e-mail, téléphone, Stripe, Instagram…), les liens
  « nouvel onglet » et les PDF en ligne s'ouvrent dans le navigateur.
- Les fichiers fabriqués par la page (exports, QR, story) vont dans Téléchargements.
- Sans internet : un écran « Pas de connexion » avec un bouton « Réessayer ».
- Le site est reconnu grâce à « BatiDesktop/<version> » à la fin du user agent.
- Le site n'a accès à aucune commande de l'appli ni aux fichiers de l'ordinateur.

## Comment installer

L'appli n'est pas encore signée. Windows et Mac affichent donc un avertissement la première fois.
C'est normal, et cela ne se produit qu'une fois.

### Sur Windows

1. Téléchargez le fichier de Bati qui finit par `x64-setup.exe` et ouvrez-le.
2. Windows affiche « Windows a protégé votre ordinateur ».
3. Cliquez sur « Informations complémentaires ».
4. Cliquez sur « Exécuter quand même ».
5. Suivez l'installation. À la fin, laissez « Lancer Bati » coché et cliquez sur « Terminer ».

### Sur Mac

1. Téléchargez le fichier de Bati qui finit par `universal.dmg` et ouvrez-le.
2. Glissez Bati dans le dossier Applications.
3. Ouvrez Bati depuis le dossier Applications. Le Mac refuse de l'ouvrir : fermez le message (« OK » ou « Terminé »).
4. Ouvrez Réglages Système, puis Confidentialité et sécurité.
5. Descendez jusqu'au message sur Bati et cliquez sur « Ouvrir quand même ».
6. Confirmez avec votre mot de passe ou Touch ID, puis cliquez sur « Ouvrir ».

## Publier une nouvelle version

1. Changer la version dans `src-tauri/tauri.conf.json` (par exemple `0.2.0`), ainsi que dans
   `src-tauri/Cargo.toml` et `package.json` pour rester alignés.
2. Committer, puis pousser un tag identique : `git tag v0.2.0 && git push origin v0.2.0`.
3. GitHub Actions compile l'installeur Windows (`.exe`) et le Mac universel (`.dmg`, puces Apple et Intel)
   et les dépose dans une Release **brouillon** (onglet Releases).
4. Vérifier les deux fichiers, puis cliquer « Publish release ».

La compilation peut aussi se lancer à la main : onglet Actions > Compiler Bati > Run workflow.
Aucun secret à configurer.

## Refaire les images

Toutes les images sont versionnées : la compilation ne les régénère pas. Pour les refaire (après une
retouche de la mascotte dans `images/fabriquer.mjs`) :

1. Avoir le dépôt du site à côté de celui-ci, avec ses modules installés (`npm install` dans le site) :

   ```
   codenco/
   ├── bati-desktop/   (ce dépôt)
   └── site-codenco/   (le générateur y prend l'outil d'image sharp, en lecture seule)
   ```

2. Télécharger la police Be Vietnam Pro en TTF (Google Fonts, licence OFL) dans un dossier hors du dépôt,
   avec ces trois fichiers : `BeVietnamPro-Bold.ttf`, `BeVietnamPro-SemiBold.ttf`, `BeVietnamPro-Medium.ttf`
   (https://github.com/google/fonts/tree/main/ofl/bevietnampro). Le site n'a que du WOFF2, que l'outil ne lit pas.
3. Depuis ce dépôt : `node images/fabriquer.mjs <dossier des polices> [dossier des aperçus]`.

Ce que le générateur écrit :

- `images/icone.svg`, `icone-mac.svg`, `icone-petite.svg` : les sources vectorielles de l'icône.
- `src-tauri/icons/` : `icon.ico` (Windows, mascotte pleine), `icon.icns` (Mac, grille Apple) et les PNG.
- `src-tauri/nsis/cote.bmp` (164 × 314) et `bandeau.bmp` (150 × 57) : l'installeur Windows.
- `src-tauri/dmg/fond.png` (660 × 360) : la fenêtre du .dmg. Les positions des icônes sont dans
  `tauri.conf.json` (Bati en 165, 195 ; Applications en 495, 195).
- `dist/bati.png` : l'image de l'écran « Pas de connexion ».
- Le dossier des aperçus, s'il est donné : des PNG pour vérifier le résultat à l'œil.
