// Modlar: Modrinth'te arama, indirme (gerekli yan modlarla), aç/kapat, silme, onarım,
// mod ayar dosyaları (config). Her yükleyici + sürüm (+ sunucu/profil) için ayrı mod klasörü.
use crate::ortak::*;
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

const API: &str = "https://api.modrinth.com/v2";
// Fabric/Quilt modlarının neredeyse hepsinin istediği temel kütüphaneler
fn temel(yukleyici: &str) -> Option<&'static str> {
    match yukleyici {
        "fabric" => Some("P7dR8mSH"), // Fabric API
        "quilt" => Some("qvIfYCYJ"),  // Quilted Fabric API
        _ => None,
    }
}

fn mod_yukleyici(y: &str) -> Option<&str> {
    match y {
        "fabric" | "quilt" | "forge" | "neoforge" => Some(y),
        _ => None,
    }
}

fn proje_turu(tur: &str) -> &str {
    match tur {
        "shader" => "shader",
        "paket" => "resourcepack",
        _ => "mod",
    }
}

fn temizle_ad(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() || "._-".contains(c) { c } else { '_' })
        .collect()
}

pub fn profil_klasoru(yukleyici: &str, mc: &str, sunucu: Option<&str>) -> PathBuf {
    let ad = format!("{yukleyici}-{mc}");
    let ad = match sunucu {
        Some(s) => temizle_ad(&format!("sunucu_{}_{ad}", s.to_lowercase())).chars().take(120).collect(),
        None => temizle_ad(&ad),
    };
    let k = ana_klasor().join("modlar").join(ad);
    let _ = std::fs::create_dir_all(&k);
    k
}

pub fn hedef_klasor(tur: &str, yukleyici: &str, mc: &str, sunucu: Option<&str>) -> PathBuf {
    let k = match tur {
        "shader" => oyun_klasoru().join("shaderpacks"),
        "paket" => oyun_klasoru().join("resourcepacks"),
        _ => return profil_klasoru(yukleyici, mc, sunucu),
    };
    let _ = std::fs::create_dir_all(&k);
    k
}

fn kayit_oku(k: &Path) -> Map<String, Value> {
    std::fs::read_to_string(k.join("kcz-modlar.json"))
        .ok()
        .and_then(|m| serde_json::from_str::<Value>(&m).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

fn kayit_yaz(k: &Path, kayit: &Map<String, Value>) {
    if let Ok(m) = serde_json::to_string_pretty(kayit) {
        let _ = std::fs::write(k.join("kcz-modlar.json"), m);
    }
}

/// Yarım indirmeleri sil, dosyası olmayan kayıtları çıkar
fn temizle(k: &Path) {
    if let Ok(d) = std::fs::read_dir(k) {
        for e in d.flatten() {
            if e.file_name().to_string_lossy().ends_with(".part") {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    let mut kayit = kayit_oku(k);
    let once = kayit.len();
    kayit.retain(|ad, _| k.join(ad).exists() || k.join(format!("{ad}.disabled")).exists());
    if kayit.len() != once {
        kayit_yaz(k, &kayit);
    }
}

async fn api(yol: &str) -> Sonuc<Value> {
    json_al(&format!("{API}{yol}")).await.map_err(|e| format!("Modrinth: {e}"))
}

fn kodla(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

/* ---------- Arama ---------- */
pub async fn ara(sorgu: &str, tur: &str, mc: &str, yukleyici: &str, sunucu: Option<&str>, sira: &str, sayfa: u64) -> Sonuc<Value> {
    let mut facets = vec![vec![format!("project_type:{}", proje_turu(tur))]];
    if !mc.is_empty() {
        facets.push(vec![format!("versions:{mc}")]);
    }
    if tur == "mod" {
        if let Some(y) = mod_yukleyici(yukleyici) {
            facets.push(vec![format!("categories:{y}")]);
        }
    }
    let sira = if ["relevance", "downloads", "updated", "newest", "follows"].contains(&sira) {
        sira
    } else {
        "relevance"
    };
    let sorgu: String = sorgu.chars().take(100).collect();
    let j = api(&format!(
        "/search?query={}&facets={}&index={sira}&offset={}&limit=20",
        kodla(&sorgu),
        kodla(&serde_json::to_string(&facets).h()?),
        sayfa * 20
    ))
    .await?;
    let klasor = hedef_klasor(tur, yukleyici, mc, sunucu);
    let yuklu: Vec<String> = kayit_oku(&klasor).values().map(|v| metin(v, "proje")).collect();
    let sonuclar: Vec<Value> = j["hits"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|h| {
            let id = metin(h, "project_id");
            json!({
                "id": id, "slug": metin(h, "slug"), "ad": metin(h, "title"), "yazar": metin(h, "author"),
                "aciklama": metin(h, "description"),
                "ikon": h.get("icon_url").cloned().unwrap_or(Value::Null),
                "indirme": h["downloads"].as_u64().unwrap_or(0),
                "yuklu": yuklu.contains(&id)
            })
        })
        .collect();
    Ok(json!({ "toplam": j["total_hits"].as_u64().unwrap_or(0), "sonuclar": sonuclar }))
}

/* ---------- İndirme ---------- */
async fn uygun_surum(proje: &str, tur: &str, mc: &str, yukleyici: &str) -> Sonuc<Option<Value>> {
    let mut q = vec![];
    if !mc.is_empty() {
        q.push(format!("game_versions={}", kodla(&serde_json::to_string(&[mc]).h()?)));
    }
    if tur == "mod" {
        if let Some(y) = mod_yukleyici(yukleyici) {
            q.push(format!("loaders={}", kodla(&serde_json::to_string(&[y]).h()?)));
        }
    }
    let l = api(&format!("/project/{}/version?{}", kodla(proje), q.join("&"))).await?;
    let l = l.as_array().cloned().unwrap_or_default();
    Ok(l.iter().find(|v| metin(v, "version_type") == "release").or(l.first()).cloned())
}

// Kurulumlar aynı anda çalışmasın: hepsi sıraya girer
static KUYRUK: LazyLock<tokio::sync::Mutex<()>> = LazyLock::new(|| tokio::sync::Mutex::new(()));

pub async fn yukle(proje: &str, tur: &str, mc: &str, yukleyici: &str, sunucu: Option<&str>) -> Sonuc<Vec<String>> {
    let _sira = KUYRUK.lock().await;
    if tur == "mod" && mod_yukleyici(yukleyici).is_none() {
        return Err("Mod kurmak için önce bir mod yükleyici seç (Fabric, Quilt, Forge ya da NeoForge).".into());
    }
    // Kısa ad (slug) verildiyse gerçek proje kimliğini bul
    let ana = api(&format!("/project/{}", kodla(proje))).await?;
    let proje = metin(&ana, "id");
    let klasor = hedef_klasor(tur, yukleyici, mc, sunucu);
    temizle(&klasor);
    let mut kayit = kayit_oku(&klasor);
    let mut kurulan = vec![];
    let mut bekleyen = vec![proje.clone()];
    let mut gorulen: Vec<String> = kayit.values().map(|v| metin(v, "proje")).collect();
    // Fabric API her mod kurulumunda kurulur ya da güncellenir
    let temel_id = if tur == "mod" { temel(yukleyici).map(String::from) } else { None };
    if let Some(t) = &temel_id {
        if *t != proje {
            bekleyen.push(t.clone());
            gorulen.retain(|x| x != t);
        }
    }

    while let Some(id) = (!bekleyen.is_empty()).then(|| bekleyen.remove(0)) {
        if gorulen.contains(&id) && id != proje && Some(&id) != temel_id.as_ref() {
            continue;
        }
        let Some(surum) = uygun_surum(&id, tur, mc, yukleyici).await? else {
            if id == proje {
                return Err(format!(
                    "Bu {} seçili sürüm ({mc}) için yok.",
                    if tur == "mod" { "mod" } else { "içerik" }
                ));
            }
            continue;
        };
        let dosyalar = surum["files"].as_array().cloned().unwrap_or_default();
        let Some(dosya) = dosyalar
            .iter()
            .find(|f| f["primary"].as_bool() == Some(true))
            .or(dosyalar.first())
            .cloned()
        else {
            continue;
        };
        let ad = Path::new(&metin(&dosya, "filename"))
            .file_name()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or_default();
        if ad.is_empty() {
            continue;
        }
        // Aynı sürüm zaten varsa tekrar indirme
        if id != proje && kayit.get(&ad).map(|k| metin(k, "proje")) == Some(id.clone()) && klasor.join(&ad).exists() {
            gorulen.push(id);
            continue;
        }
        let bilgi = if id == proje {
            ana.clone()
        } else {
            api(&format!("/project/{}", kodla(&id))).await.unwrap_or(json!({}))
        };
        let baslik = {
            let t = metin(&bilgi, "title");
            if t.is_empty() {
                ad.clone()
            } else {
                t
            }
        };
        gonder("mod-durum", format!("{baslik} indiriliyor…"));
        // Aynı projenin eski dosyasını kaldır
        let eskiler: Vec<String> = kayit
            .iter()
            .filter(|(_, k)| metin(k, "proje") == id)
            .map(|(a, _)| a.clone())
            .collect();
        for e in eskiler {
            let _ = std::fs::remove_file(klasor.join(&e));
            let _ = std::fs::remove_file(klasor.join(format!("{e}.disabled")));
            kayit.remove(&e);
        }
        let url = metin(&dosya, "url");
        if !url.starts_with("https://cdn.modrinth.com/") {
            return Err("Beklenmeyen indirme adresi.".into());
        }
        indir(
            &url,
            &klasor.join(&ad),
            dosya["hashes"]["sha1"].as_str(),
            dosya["size"].as_u64(),
            |_, _| {},
        )
        .await?;
        kayit.insert(
            ad.clone(),
            json!({
                "proje": id, "slug": metin(&bilgi, "slug"), "ad": baslik,
                "ikon": bilgi.get("icon_url").cloned().unwrap_or(Value::Null),
                "surum": metin(&surum, "version_number"), "yan": id != proje
            }),
        );
        kayit_yaz(&klasor, &kayit);
        kurulan.push(baslik);
        gorulen.push(id.clone());
        if tur == "mod" {
            for b in surum["dependencies"].as_array().cloned().unwrap_or_default() {
                let bid = metin(&b, "project_id");
                if metin(&b, "dependency_type") == "required" && !bid.is_empty() && !gorulen.contains(&bid) && !bekleyen.contains(&bid) {
                    bekleyen.push(bid);
                }
            }
        }
    }
    Ok(kurulan)
}

/* ---------- Yüklü içerik ---------- */
pub fn liste(tur: &str, mc: &str, yukleyici: &str, sunucu: Option<&str>) -> Vec<Value> {
    let klasor = hedef_klasor(tur, yukleyici, mc, sunucu);
    temizle(&klasor);
    let kayit = kayit_oku(&klasor);
    let mut l: Vec<Value> = std::fs::read_dir(&klasor)
        .map(|d| d.flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect::<Vec<_>>())
        .unwrap_or_default()
        .into_iter()
        .filter(|d| {
            let d = d.to_lowercase();
            let d = d.strip_suffix(".disabled").unwrap_or(&d).to_string();
            if tur == "mod" {
                d.ends_with(".jar")
            } else {
                d.ends_with(".zip") || d.ends_with(".jar")
            }
        })
        .map(|d| {
            let acik = !d.ends_with(".disabled");
            let temel_ad = d.strip_suffix(".disabled").unwrap_or(&d).to_string();
            let k = kayit.get(&temel_ad).cloned().unwrap_or(json!({}));
            let ad = {
                let a = metin(&k, "ad");
                if a.is_empty() {
                    temel_ad.clone()
                } else {
                    a
                }
            };
            json!({
                "dosya": temel_ad, "acik": acik, "ad": ad, "ikon": k.get("ikon").cloned().unwrap_or(Value::Null),
                "surum": metin(&k, "surum"), "slug": metin(&k, "slug"), "yan": k["yan"].as_bool().unwrap_or(false),
                "proje": metin(&k, "proje")
            })
        })
        .collect();
    l.sort_by_key(|x| metin(x, "ad").to_lowercase());
    l
}

fn guvenli_ad(d: &str) -> Sonuc<String> {
    let ad = Path::new(d)
        .file_name()
        .map(|x| x.to_string_lossy().to_string())
        .unwrap_or_default();
    if ad.is_empty() || ad.starts_with('.') || ad.contains("..") {
        return Err("Geçersiz dosya.".into());
    }
    Ok(ad)
}

pub fn ac_kapat(tur: &str, mc: &str, yukleyici: &str, sunucu: Option<&str>, dosya: &str, acik: bool) -> Sonuc<bool> {
    let k = hedef_klasor(tur, yukleyici, mc, sunucu);
    let ad = guvenli_ad(dosya)?;
    let (a, kapali) = (k.join(&ad), k.join(format!("{ad}.disabled")));
    if acik && kapali.exists() {
        std::fs::rename(&kapali, &a).h()?;
    }
    if !acik && a.exists() {
        std::fs::rename(&a, &kapali).h()?;
    }
    Ok(true)
}

pub fn sil(tur: &str, mc: &str, yukleyici: &str, sunucu: Option<&str>, dosya: &str) -> Sonuc<bool> {
    let k = hedef_klasor(tur, yukleyici, mc, sunucu);
    let ad = guvenli_ad(dosya)?;
    let _ = std::fs::remove_file(k.join(&ad));
    let _ = std::fs::remove_file(k.join(format!("{ad}.disabled")));
    let mut kayit = kayit_oku(&k);
    kayit.remove(&ad);
    kayit_yaz(&k, &kayit);
    Ok(true)
}

/// Oyun açılmadan önce: oyunun mods klasörünü seçili profille aynı yap.
/// `kullanici`: profilin kendi modları; `ek`: KÇZ Client, optimizasyon ve eklenti klasörleri (aynı mod iki kez konmaz).
pub fn esitle(yukleyici: &str, mc: &str, sunucu: Option<&str>, kullanici: bool, ek: &[PathBuf]) -> usize {
    let hedef = oyun_klasoru().join("mods");
    let _ = std::fs::create_dir_all(&hedef);
    if let Ok(d) = std::fs::read_dir(&hedef) {
        for e in d.flatten() {
            if e.file_name().to_string_lossy().to_lowercase().ends_with(".jar") {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    if mod_yukleyici(yukleyici).is_none() {
        return 0;
    }
    let mut n = 0;
    let mut adlar: Vec<String> = vec![];
    let mut projeler: Vec<String> = vec![];
    let mut kaynaklar: Vec<PathBuf> = vec![];
    if kullanici {
        kaynaklar.push(profil_klasoru(yukleyici, mc, sunucu));
    }
    kaynaklar.extend(ek.iter().cloned());
    for kaynak in kaynaklar {
        let kayit = kayit_oku(&kaynak);
        let Ok(d) = std::fs::read_dir(&kaynak) else { continue };
        for e in d.flatten() {
            let ad = e.file_name().to_string_lossy().to_string();
            if !ad.to_lowercase().ends_with(".jar") || adlar.contains(&ad.to_lowercase()) {
                continue;
            }
            let proje = kayit.get(&ad).map(|k| metin(k, "proje")).unwrap_or_default();
            if !proje.is_empty() && projeler.contains(&proje) {
                continue;
            }
            if std::fs::copy(e.path(), hedef.join(&ad)).is_ok() {
                n += 1;
                adlar.push(ad.to_lowercase());
                if !proje.is_empty() {
                    projeler.push(proje);
                }
            }
        }
    }
    n
}

/// Oyun açılmadan önce: yarım dosyaları temizle, mod varsa ama Fabric API yoksa kur
pub async fn onar(mc: &str, yukleyici: &str, sunucu: Option<&str>) -> Sonuc<Vec<String>> {
    let klasor = profil_klasoru(yukleyici, mc, sunucu);
    temizle(&klasor);
    let Some(t) = temel(yukleyici) else { return Ok(vec![]) };
    let jarlar: Vec<String> = std::fs::read_dir(&klasor)
        .map(|d| {
            d.flatten()
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|a| a.to_lowercase().ends_with(".jar"))
                .collect()
        })
        .unwrap_or_default();
    if jarlar.is_empty() {
        return Ok(vec![]);
    }
    let kayit = kayit_oku(&klasor);
    let var = kayit.iter().any(|(ad, k)| metin(k, "proje") == t && klasor.join(ad).exists())
        || jarlar.iter().any(|d| {
            let d = d.to_lowercase();
            d.starts_with("fabric-api") || d.starts_with("qfapi") || d.starts_with("quilted-fabric-api")
        });
    if var {
        return Ok(vec![]);
    }
    crate::surumler::ilerleme(
        mc,
        100,
        if yukleyici == "quilt" {
            "Quilted Fabric API eksik, kuruluyor…"
        } else {
            "Fabric API eksik, kuruluyor…"
        },
    );
    yukle(t, "mod", mc, yukleyici, sunucu).await
}

/// Oyunun "eksik mod" hatasındaki kimlikleri kur (ör. fabric-resource-loader-v1 -> Fabric API)
pub async fn eksikleri_kur(mc: &str, yukleyici: &str, sunucu: Option<&str>, kimlikler: &[String]) -> Sonuc<Vec<String>> {
    let mut projeler: Vec<String> = vec![];
    for k in kimlikler {
        let id = k.to_lowercase();
        if id.is_empty() || ["minecraft", "java", "fabricloader"].contains(&id.as_str()) {
            continue;
        }
        let p = if (id.starts_with("fabric-") || id == "fabric" || id == "fabric_api") && temel(yukleyici).is_some() {
            temel(yukleyici).unwrap().to_string()
        } else if id.starts_with("quilt") || id.starts_with("qsl") {
            "qvIfYCYJ".to_string()
        } else {
            id.replace('_', "-")
        };
        if !projeler.contains(&p) {
            projeler.push(p);
        }
    }
    let mut kurulan: Vec<String> = vec![];
    for p in projeler {
        match yukle(&p, "mod", mc, yukleyici, sunucu).await {
            Ok(k) => {
                for x in k {
                    if !kurulan.contains(&x) {
                        kurulan.push(x);
                    }
                }
            }
            Err(h) => gonder("mod-durum", format!("{p} kurulamadı: {h}")),
        }
    }
    Ok(kurulan)
}

pub fn mod_sayisi(yukleyici: &str, mc: &str, sunucu: Option<&str>) -> usize {
    std::fs::read_dir(profil_klasoru(yukleyici, mc, sunucu))
        .map(|d| {
            d.flatten()
                .filter(|e| e.file_name().to_string_lossy().to_lowercase().ends_with(".jar"))
                .count()
        })
        .unwrap_or(0)
}

/* ---------- Mod ayar dosyaları ---------- */
pub fn config_klasoru() -> PathBuf {
    let k = oyun_klasoru().join("config");
    let _ = std::fs::create_dir_all(&k);
    k
}

pub fn config_liste() -> Vec<Value> {
    let kok = config_klasoru();
    let mut sonuc = vec![];
    fn gez(k: &Path, on: &str, sonuc: &mut Vec<Value>) {
        let Ok(d) = std::fs::read_dir(k) else { return };
        for e in d.flatten() {
            if sonuc.len() > 500 {
                return;
            }
            let ad = e.file_name().to_string_lossy().to_string();
            let goreli = if on.is_empty() { ad.clone() } else { format!("{on}/{ad}") };
            let p = e.path();
            if p.is_dir() {
                gez(&p, &goreli, sonuc);
            } else {
                let kucuk = ad.to_lowercase();
                if [
                    ".json",
                    ".json5",
                    ".toml",
                    ".properties",
                    ".cfg",
                    ".txt",
                    ".yml",
                    ".yaml",
                    ".conf",
                    ".ini",
                    ".snbt",
                ]
                .iter()
                .any(|u| kucuk.ends_with(u))
                {
                    let boyut = e.metadata().map(|m| m.len()).unwrap_or(0);
                    sonuc.push(json!({ "yol": goreli, "boyut": boyut }));
                }
            }
        }
    }
    gez(&kok, "", &mut sonuc);
    sonuc.sort_by_key(|x| metin(x, "yol"));
    sonuc
}

fn config_yolu(goreli: &str) -> Sonuc<PathBuf> {
    let kok = config_klasoru();
    if goreli.contains("..") || goreli.starts_with('/') || goreli.starts_with('\\') || goreli.contains(':') {
        return Err("Geçersiz dosya.".into());
    }
    Ok(kok.join(goreli))
}

pub fn config_oku(goreli: &str) -> Sonuc<String> {
    let y = config_yolu(goreli)?;
    if std::fs::metadata(&y).h()?.len() > 512 * 1024 {
        return Err("Dosya çok büyük.".into());
    }
    std::fs::read_to_string(&y).h()
}

pub fn config_yaz(goreli: &str, metin_: &str) -> Sonuc<bool> {
    let y = config_yolu(goreli)?;
    if !y.exists() {
        return Err("Dosya bulunamadı.".into());
    }
    let yedek = PathBuf::from(format!("{}.kcz-yedek", y.display()));
    if !yedek.exists() {
        let _ = std::fs::copy(&y, &yedek);
    }
    std::fs::write(&y, metin_).h()?;
    Ok(true)
}
