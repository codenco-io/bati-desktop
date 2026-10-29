// Bati pour ordinateur : une fenêtre qui ouvre l'espace en ligne. Le site distant n'a accès à aucune commande :
// tout ce que l'appli fait (ouvrir un lien dehors, enregistrer un fichier, mémoriser la fenêtre) se décide ici, en Rust.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::webview::{DownloadEvent, NewWindowResponse};
use tauri::{AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_window_state::StateFlags;

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

fn chez_codenco(url: &Url) -> bool {
    url.scheme() == "https" && matches!(url.host_str(), Some("codenco.io" | "www.codenco.io"))
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
    match url.scheme() {
        "about" | "blob" | "data" => true,
        // Sur Mac, ce filtre voit aussi les cadres intégrés : les lecteurs vidéo (YouTube, Instagram) se chargent sur place.
        "https" => url.path().contains("/embed"),
        _ => false,
    }
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
        .plugin(tauri_plugin_window_state::Builder::default().with_state_flags(StateFlags::all() & !StateFlags::VISIBLE).build())
        .setup(|app| {
            let version = app.package_info().version.to_string();
            let (pour_nav, pour_fenetres, pour_fichiers) = (app.handle().clone(), app.handle().clone(), app.handle().clone());
            let dernier_fichier: Mutex<Option<PathBuf>> = Mutex::new(None);

            WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("Bati")
                .inner_size(1280.0, 820.0)
                .min_inner_size(900.0, 600.0)
                .center()
                .user_agent(&user_agent(&version))
                .initialization_script(SCRIPT_HORS_LIGNE.replace("__PAGE_LOCALE__", PAGE_LOCALE))
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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Bati n'a pas pu démarrer");
}
