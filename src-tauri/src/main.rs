// Bati pour ordinateur : une fenêtre qui ouvre l'espace en ligne. Le site distant n'a accès à aucune commande :
// tout ce que l'appli fait (ouvrir un lien dehors, enregistrer un fichier, mémoriser la fenêtre, se mettre à jour)
// se décide ici, en Rust.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::webview::{DownloadEvent, NewWindowResponse};
use tauri::window::Color;
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_window_state::StateFlags;

/// Le fond de l'écran de démarrage (--bg de dist/index.html) : la fenêtre ne flashe jamais en blanc.
const FOND: Color = Color(10, 13, 21, 255);

/// La page locale « Pas de connexion », servie depuis dist/ (adresse différente selon le système).
#[cfg(windows)]
const PAGE_LOCALE: &str = "http://tauri.localhost/index.html";
#[cfg(not(windows))]
const PAGE_LOCALE: &str = "tauri://localhost/index.html";

/// Injecté dans les pages de codenco.io : hors ligne, un clic sur un lien interne mène à la page locale
/// au lieu de l'erreur brute du navigateur. Elle y revient d'elle-même dès que la connexion revient.
const SCRIPT_HORS_LIGNE: &str = r#"(function () {
  if (location.hostname !== "codenco.io" && location.hostname !== "www.codenco.io") return;
  document.addEventListener("click", function (e) {
    if (navigator.onLine) return;
    var a = e.target && e.target.closest ? e.target.closest("a[href]") : null;
    if (!a || a.target === "_blank" || a.origin !== location.origin) return;
    e.preventDefault();
    e.stopPropagation();
    location.href = "__PAGE_LOCALE__?retour=" + encodeURIComponent(a.href);
  }, true);
})();"#;

/// Mac seulement : WebKit ignore un lien de téléchargement vers un blob: (<a download href="blob:…">, la story PNG,
/// le QR, les exports), alors qu'il télécharge bien un data:. Ce script garde chaque blob créé par la page et,
/// au clic sur un tel lien, le convertit en data: avec le même nom : le fichier passe alors par on_download.
#[cfg(target_os = "macos")]
const SCRIPT_BLOB_MAC: &str = r#"(function () {
  if (location.hostname !== "codenco.io" && location.hostname !== "www.codenco.io") return;
  var blobs = new Map();
  var creer = URL.createObjectURL, revoquer = URL.revokeObjectURL, cliquer = HTMLAnchorElement.prototype.click;
  URL.createObjectURL = function (objet) {
    var url = creer.call(URL, objet);
    if (objet instanceof Blob) blobs.set(url, objet);
    return url;
  };
  URL.revokeObjectURL = function (url) {
    blobs.delete(url);
    return revoquer.call(URL, url);
  };
  function telecharger(a) {
    var blob = a.hasAttribute("download") ? blobs.get(a.href) : null;
    if (!blob) return false;
    var nom = a.getAttribute("download") || "telechargement";
    var lecteur = new FileReader();
    lecteur.onload = function () {
      var lien = document.createElement("a");
      lien.href = lecteur.result;
      lien.download = nom;
      document.documentElement.append(lien);
      cliquer.call(lien);
      lien.remove();
    };
    lecteur.readAsDataURL(blob);
    return true;
  }
  // a.click() depuis le code du site, que le lien soit dans la page ou non.
  HTMLAnchorElement.prototype.click = function () {
    if (!telecharger(this)) cliquer.call(this);
  };
  // Un vrai clic sur un tel lien.
  document.addEventListener("click", function (e) {
    var a = e.target && e.target.closest ? e.target.closest("a[download][href^='blob:']") : null;
    if (a && telecharger(a)) e.preventDefault();
  }, true);
})();"#;

fn chez_codenco(url: &Url) -> bool {
    url.scheme() == "https" && matches!(url.host_str(), Some("codenco.io" | "www.codenco.io"))
}

/// Le cadre anti-robot de Cloudflare (Turnstile) de la connexion et des formulaires. Ce domaine seulement, en https.
fn chez_cloudflare(url: &Url) -> bool {
    url.scheme() == "https" && url.host_str() == Some("challenges.cloudflare.com")
}

fn page_locale(url: &Url) -> bool {
    url.scheme() == "tauri" || (url.scheme() == "http" && url.host_str() == Some("tauri.localhost"))
}

/// Ce qui reste dans la fenêtre. Tout le reste part dans le navigateur du système.
fn reste_dans_la_fenetre(url: &Url) -> bool {
    if page_locale(url) {
        return true;
    }
    if chez_codenco(url) {
        // Seul l'espace Bati reste dans la fenêtre : le site, /p/…, /l/… s'ouvrent dans le navigateur.
        let chemin = url.path().to_ascii_lowercase();
        return (chemin == "/bati" || chemin.starts_with("/bati/") || chemin.starts_with("/api/bati/")) && !chemin.ends_with(".pdf");
    }
    // Sur Mac, ce filtre voit aussi les cadres intégrés (sur Windows, seulement la page) : sans cette ligne, le cadre
    // anti-robot de la connexion part dans Safari et l'écran reste blanc.
    if chez_cloudflare(url) {
        return true;
    }
    match url.scheme() {
        "about" | "blob" | "data" => true,
        // Sur Mac, ce filtre voit aussi les cadres intégrés : les lecteurs vidéo (YouTube, Instagram) se chargent sur place.
        "https" => url.path().contains("/embed"),
        _ => false,
    }
}

/// Après un chargement de l'espace qui échoue faute de connexion : la page locale « Pas de connexion »,
/// qui pourra y revenir (?retour=). Rien pour une autre adresse.
#[cfg_attr(not(windows), allow(dead_code))]
fn page_hors_ligne(adresse: &str) -> Option<Url> {
    let url = Url::parse(adresse).ok().filter(chez_codenco)?;
    Url::parse_with_params(PAGE_LOCALE, [("echec", "1"), ("retour", url.as_str())]).ok()
}

/// Windows : quand l'espace ne se charge pas (pas d'internet, serveur injoignable), WebView2 afficherait sa page
/// d'erreur brute. On renvoie à la place sur « Pas de connexion ». Sur Mac, la page locale reste affichée et
/// montre elle-même ce message au bout de 10 secondes.
#[cfg(windows)]
fn hors_ligne_si_echec(fenetre: &tauri::WebviewWindow) -> tauri::Result<()> {
    use webview2_com::Microsoft::Web::WebView2::Win32::*;
    use webview2_com::{take_pwstr, NavigationCompletedEventHandler};
    use windows_core::{BOOL, HSTRING, PCWSTR, PWSTR};

    fenetre.with_webview(|vue| unsafe {
        let Ok(coeur) = vue.controller().CoreWebView2() else { return };
        let gestionnaire = NavigationCompletedEventHandler::create(Box::new(|coeur, details| {
            let (Some(coeur), Some(details)) = (coeur, details) else { return Ok(()) };
            let mut reussi = BOOL::default();
            details.IsSuccess(&mut reussi)?;
            let mut erreur = COREWEBVIEW2_WEB_ERROR_STATUS::default();
            details.WebErrorStatus(&mut erreur)?;
            // Seulement les erreurs de connexion : une page 404 ou un chargement interrompu restent tels quels.
            let connexion = matches!(
                erreur,
                COREWEBVIEW2_WEB_ERROR_STATUS_SERVER_UNREACHABLE
                    | COREWEBVIEW2_WEB_ERROR_STATUS_TIMEOUT
                    | COREWEBVIEW2_WEB_ERROR_STATUS_CONNECTION_ABORTED
                    | COREWEBVIEW2_WEB_ERROR_STATUS_CONNECTION_RESET
                    | COREWEBVIEW2_WEB_ERROR_STATUS_DISCONNECTED
                    | COREWEBVIEW2_WEB_ERROR_STATUS_CANNOT_CONNECT
                    | COREWEBVIEW2_WEB_ERROR_STATUS_HOST_NAME_NOT_RESOLVED
            );
            if reussi.as_bool() || !connexion {
                return Ok(());
            }
            let mut source = PWSTR::null();
            coeur.Source(&mut source)?;
            if let Some(page) = page_hors_ligne(&take_pwstr(source)) {
                let page = HSTRING::from(page.as_str());
                coeur.Navigate(PCWSTR(page.as_ptr()))?;
            }
            Ok(())
        }));
        let mut jeton = 0i64;
        let _ = coeur.add_NavigationCompleted(&gestionnaire, &mut jeton);
    })
}

/// Au lancement, en silence : s'il existe une version plus récente (la dernière Release publiée, jamais un brouillon),
/// on propose de l'installer. Rien sans accord ; une erreur (pas de réseau, fichier absent) ne dit rien.
fn proposer_mise_a_jour(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let Ok(verif) = app.updater() else { return };
        let Ok(Some(maj)) = verif.check().await else { return };
        let mut dialogue = app
            .dialog()
            .message("Une nouvelle version de Bati est disponible.")
            .title("Bati")
            .buttons(MessageDialogButtons::OkCancelCustom("Mettre à jour".into(), "Plus tard".into()));
        if let Some(principale) = app.get_webview_window("main") {
            dialogue = dialogue.parent(&principale);
        }
        dialogue.show(move |accord| {
            if !accord {
                return;
            }
            tauri::async_runtime::spawn(async move {
                // Windows : l'installeur ferme Bati et le relance lui-même. Mac : on relance une fois l'appli remplacée.
                if maj.download_and_install(|_, _| {}, || {}).await.is_ok() {
                    app.restart();
                }
            });
        });
    });
}

/// Ouvre dans le navigateur ou l'appli par défaut, seulement pour des adresses sûres (jamais file: ni un protocole inconnu).
fn ouvrir_dehors(app: &AppHandle, url: &Url) {
    if matches!(url.scheme(), "https" | "http" | "mailto" | "tel" | "sms" | "whatsapp") {
        let _ = app.opener().open_url(url.as_str(), None::<&str>);
    }
}

/// Celui du navigateur intégré (Edge WebView2 sur Windows, WebKit sur Mac), suivi de « BatiDesktop/<version> ».
fn user_agent(version: &str) -> String {
    #[cfg(windows)]
    let base = {
        let moteur = tauri::webview_version().unwrap_or_default();
        let majeure = moteur.split('.').next().filter(|m| !m.is_empty()).unwrap_or("130").to_string();
        let complete = if moteur.is_empty() { format!("{majeure}.0.0.0") } else { moteur.clone() };
        format!("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{majeure}.0.0.0 Safari/537.36 Edg/{complete}")
    };
    #[cfg(not(windows))]
    let base = String::from("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)");
    format!("{base} BatiDesktop/{version}")
}

/// « export.csv », puis « export (2).csv » si le nom est déjà pris.
fn place_libre(dossier: &Path, nom: &str) -> PathBuf {
    let chemin = dossier.join(nom);
    if !chemin.exists() {
        return chemin;
    }
    let p = Path::new(nom);
    let base = p.file_stem().and_then(|s| s.to_str()).unwrap_or("fichier");
    let ext = p.extension().and_then(|s| s.to_str()).map(|e| format!(".{e}")).unwrap_or_default();
    (2..).map(|n| dossier.join(format!("{base} ({n}){ext}"))).find(|c| !c.exists()).unwrap()
}

fn main() {
    tauri::Builder::default()
        // En premier : relancer l'appli remet la fenêtre existante devant.
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(fenetre) = app.get_webview_window("main") {
                let _ = fenetre.unminimize();
                let _ = fenetre.show();
                let _ = fenetre.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::default().with_state_flags(StateFlags::all() & !StateFlags::VISIBLE).build())
        .setup(|app| {
            let version = app.package_info().version.to_string();
            let (pour_nav, pour_fenetres, pour_fichiers) = (app.handle().clone(), app.handle().clone(), app.handle().clone());
            let dernier_fichier: Mutex<Option<PathBuf>> = Mutex::new(None);

            // Créée cachée, le temps de remettre sa taille et sa place mémorisées, puis montrée aussitôt
            // (sans attendre la page) : pas de saut à l'ouverture.
            let fenetre = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Bati")
                .inner_size(1280.0, 820.0)
                .min_inner_size(900.0, 600.0)
                .center()
                .visible(false)
                .background_color(FOND)
                .user_agent(&user_agent(&version))
                .initialization_script(SCRIPT_HORS_LIGNE.replace("__PAGE_LOCALE__", PAGE_LOCALE));
            #[cfg(target_os = "macos")]
            let fenetre = fenetre.initialization_script(SCRIPT_BLOB_MAC);
            let fenetre = fenetre
                .on_navigation(move |url| {
                    if reste_dans_la_fenetre(url) {
                        return true;
                    }
                    ouvrir_dehors(&pour_nav, url);
                    false
                })
                // Le titre de la page (« Bati · Mes demandes », posé par le site) devient celui de la fenêtre.
                .on_document_title_changed(|fenetre, titre| {
                    let _ = fenetre.set_title(&titre);
                })
                // target=_blank et window.open : toujours dans le navigateur, jamais de seconde fenêtre.
                .on_new_window(move |url, _| {
                    ouvrir_dehors(&pour_fenetres, &url);
                    NewWindowResponse::Deny
                })
                .on_download(move |_, evenement| match evenement {
                    // Un fichier en ligne : le navigateur s'en charge.
                    DownloadEvent::Requested { url, .. } if matches!(url.scheme(), "http" | "https") => {
                        ouvrir_dehors(&pour_fichiers, &url);
                        false
                    }
                    // Un fichier fabriqué par la page (export, QR, story, PDF) : rangé dans Téléchargements.
                    DownloadEvent::Requested { destination, .. } => {
                        let Ok(dossier) = pour_fichiers.path().download_dir() else { return false };
                        let nom = destination.file_name().and_then(|n| n.to_str()).unwrap_or("telechargement").to_string();
                        *destination = place_libre(&dossier, &nom);
                        *dernier_fichier.lock().unwrap() = Some(destination.clone());
                        true
                    }
                    // Fini : on le montre dans l'Explorateur ou le Finder (sur Mac, le chemin n'est pas fourni).
                    DownloadEvent::Finished { path, success, .. } => {
                        let chemin = path.or_else(|| dernier_fichier.lock().unwrap().take());
                        if let (true, Some(chemin)) = (success, chemin) {
                            let _ = pour_fichiers.opener().reveal_item_in_dir(chemin);
                        }
                        true
                    }
                    _ => true,
                })
                .build()?;
            fenetre.show()?;
            #[cfg(windows)]
            hors_ligne_si_echec(&fenetre)?;
            proposer_mise_a_jour(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Bati n'a pas pu démarrer");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dedans(adresse: &str) -> bool {
        reste_dans_la_fenetre(&Url::parse(adresse).expect("adresse valide"))
    }

    #[test]
    fn le_cadre_anti_robot_de_cloudflare_reste_dans_la_fenetre() {
        assert!(dedans("https://challenges.cloudflare.com/cdn-cgi/challenge-platform/h/b/turnstile/if/ov2/av0/rcv/abc/0x4AAAA/light/fbE/new/normal/auto/"));
        assert!(dedans("https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit"));
    }

    #[test]
    fn seul_ce_domaine_est_ajoute() {
        assert!(!dedans("http://challenges.cloudflare.com/cdn-cgi/challenge-platform/x"));
        assert!(!dedans("https://cloudflare.com/"));
        assert!(!dedans("https://www.cloudflare.com/"));
        assert!(!dedans("https://dash.cloudflare.com/"));
        assert!(!dedans("https://evilchallenges.cloudflare.com/"));
        assert!(!dedans("https://challenges.cloudflare.com.evil.example/"));
        assert!(!dedans("https://challenges.cloudflare.com@evil.example/"));
        assert!(!dedans("https://evil.example/challenges.cloudflare.com/"));
    }

    #[test]
    fn un_echec_de_chargement_de_l_espace_mene_a_pas_de_connexion() {
        let page = page_hors_ligne("https://codenco.io/bati/espace/demandes/?filtre=a&b=1").expect("page locale");
        assert!(page_locale(&page));
        assert!(dedans(page.as_str()));
        let attendus = [("echec", "1"), ("retour", "https://codenco.io/bati/espace/demandes/?filtre=a&b=1")];
        assert!(page.query_pairs().map(|(cle, valeur)| (cle.into_owned(), valeur.into_owned())).eq(attendus.map(|(c, v)| (c.to_string(), v.to_string()))));
        // Une autre adresse ne renvoie jamais sur la page locale.
        assert!(page_hors_ligne("https://wa.me/33600000000").is_none());
        assert!(page_hors_ligne("http://codenco.io/bati/espace/").is_none());
        assert!(page_hors_ligne("pas une adresse").is_none());
    }

    #[test]
    fn le_reste_ne_change_pas() {
        // L'espace Bati et la page locale restent dans la fenêtre.
        assert!(dedans("https://codenco.io/bati/espace/"));
        assert!(dedans("https://www.codenco.io/bati"));
        assert!(dedans("https://codenco.io/api/bati/code/send"));
        assert!(dedans("tauri://localhost/index.html"));
        assert!(dedans("http://tauri.localhost/index.html"));
        assert!(dedans("about:blank"));
        assert!(dedans("data:text/html,bonjour"));
        assert!(dedans("blob:https://codenco.io/0b9c5f1e-0000-4000-8000-000000000000"));
        // Les lecteurs vidéo (/embed) aussi.
        assert!(dedans("https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ"));
        assert!(dedans("https://www.instagram.com/p/abc/embed/"));
        // Le site, les pages publiques et les PDF s'ouvrent dans le navigateur.
        assert!(!dedans("https://codenco.io/"));
        assert!(!dedans("https://codenco.io/mentions-legales/"));
        assert!(!dedans("https://codenco.io/p/camille/"));
        assert!(!dedans("https://codenco.io/l/camille/"));
        assert!(!dedans("https://codenco.io/bati/export.PDF"));
        assert!(!dedans("http://codenco.io/bati/espace/"));
        // Les liens externes aussi.
        assert!(!dedans("https://wa.me/33600000000"));
        assert!(!dedans("https://checkout.stripe.com/c/pay/cs_test_abc"));
        assert!(!dedans("https://cal.com/pro/rendez-vous"));
        assert!(!dedans("mailto:contact@exemple.fr"));
        assert!(!dedans("tel:+33600000000"));
        assert!(!dedans("file:///etc/passwd"));
    }
}
