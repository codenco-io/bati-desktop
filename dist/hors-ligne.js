// Première page de l'appli : si codenco.io répond, on y va tout de suite ; sinon « Pas de connexion ».
// Revient aussi ici quand un lien est cliqué hors ligne (?retour=<adresse de codenco.io>).

var ACCUEIL = "https://codenco.io/bati/connexion/";
var carte = document.getElementById("carte");
var demarrage = document.getElementById("demarrage");
var bouton = document.getElementById("reessayer");

function cible() {
  var retour = new URLSearchParams(location.search).get("retour");
  try {
    var url = new URL(retour);
    if (url.protocol === "https:" && (url.hostname === "codenco.io" || url.hostname === "www.codenco.io")) return url.href;
  } catch (e) {}
  return ACCUEIL;
}

function essayer() {
  bouton.disabled = true;
  bouton.textContent = "Connexion…";
  demarrage.hidden = false;
  carte.classList.remove("visible");
  var arret = new AbortController();
  var minuterie = setTimeout(function () { arret.abort(); }, 8000);
  fetch("https://codenco.io/bati/icone/192.png", { method: "HEAD", mode: "no-cors", cache: "no-store", signal: arret.signal })
    .then(function () { location.replace(cible()); })
    .catch(function () {
      demarrage.hidden = true;
      carte.classList.add("visible");
      bouton.disabled = false;
      bouton.textContent = "Réessayer";
    })
    .finally(function () { clearTimeout(minuterie); });
}

bouton.addEventListener("click", essayer);
window.addEventListener("online", essayer);
essayer();
