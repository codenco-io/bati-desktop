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

## Publier une nouvelle version

1. Changer `version` dans `src-tauri/tauri.conf.json` (par exemple `0.2.0`).
2. Committer, puis pousser un tag identique : `git tag v0.2.0 && git push origin v0.2.0`.
3. GitHub Actions compile l'installeur Windows (`.exe`) et le Mac universel (`.dmg`, puces Apple et Intel)
   et les dépose dans une Release **brouillon** (onglet Releases).
4. Vérifier les deux fichiers, puis cliquer « Publish release ».

La compilation peut aussi se lancer à la main : onglet Actions > Compiler Bati > Run workflow.
Aucun secret à configurer.

## À l'installation (appli non signée)

- **Windows** : SmartScreen affiche « Windows a protégé votre ordinateur ». Cliquer
  « Informations complémentaires », puis « Exécuter quand même ».
- **Mac** : au premier lancement, macOS refuse d'ouvrir l'appli. Ouvrir Réglages Système >
  Confidentialité et sécurité, descendre jusqu'au message sur Bati et cliquer « Ouvrir quand même »,
  puis confirmer.
