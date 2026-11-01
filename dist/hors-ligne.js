// Première page de l'appli : on part tout de suite vers l'espace, sans test préalable.
// « Pas de connexion » s'affiche si l'ordinateur est hors ligne, si l'appli revient ici après un échec
// de chargement (?echec=1, renvoyé par l'appli sur Windows) ou si rien n'a répondu au bout de 10 secondes (Mac).
// Revient aussi ici quand un lien est cliqué hors ligne (?retour=<adresse de codenco.io>).

var ACCUEIL = "https://codenco.io/bati/espace/";
var params = new URLSearchParams(location.search);
var carte = document.getElementById("carte");
var demarrage = document.getElementById("demarrage");
var bouton = document.getElementById("reessayer");
var minuterie;

function cible() {
  try {
    var url = new URL(params.get("retour"));
    if (url.protocol === "https:" && (url.hostname === "codenco.io" || url.hostname === "www.codenco.io")) return url.href;
  } catch (e) {}
  return ACCUEIL;
}

function horsLigne() {
  clearTimeout(minuterie);
  demarrage.hidden = true;
  carte.classList.add("visible");
  bouton.disabled = false;
  bouton.textContent = "Réessayer";
}

function partir() {
  bouton.disabled = true;
  bouton.textContent = "Connexion…";
  demarrage.hidden = false;
  carte.classList.remove("visible");
  clearTimeout(minuterie);
  // Sur Mac, un chargement qui échoue laisse cette page affichée : on montre alors « Pas de connexion ».
  minuterie = setTimeout(horsLigne, 10000);
  location.replace(cible());
}

bouton.addEventListener("click", partir);
window.addEventListener("online", partir);
if (params.has("echec") || !navigator.onLine) horsLigne();
else partir();
