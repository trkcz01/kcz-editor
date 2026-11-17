// KÇZ Client: launcher'a gömülü oyun içi menü modu, ayarlarının oyunla eşitlenmesi
// ve eklentiler (shader, sesli sohbet, oyun kaydı).
use crate::ayarlar::{self, Ayarlar};
use crate::ortak::*;
use crate::performans;
use serde_json::{json, Value};
use std::path::PathBuf;

// Derleme sırasında istemci-mod/cikti/kcz-client-<sürüm>.jar dosyalarından oluşturulur
include!(concat!(env!("OUT_DIR"), "/istemci_jarlar.rs"));

/// Bu Minecraft sürümü için gömülü KÇZ Client var mı? (26.3 -> 26.3, 26.3.1 ...)
pub fn jar_bul(mc: &str) -> Option<&'static [u8]> {
    JARLAR
        .iter()
        .filter(|(v, _)| mc == *v || mc.starts_with(&format!("{v}.")))
        .max_by_key(|(v, _)| v.len())
        .map(|(_, b)| *b)
}

pub fn desteklenen_surumler() -> Vec<&'static str> {
    JARLAR.iter().map(|(v, _)| *v).collect()
}

fn mod_yukleyici_fabric(tur: &str) -> bool {
    tur == "fabric" || tur == "quilt"
}

/// Vanilla seçiliyken bile oyunun Fabric ile açılmasını gerektiren bir özellik açık mı?
pub fn fabric_gerekli(a: &Ayarlar, mc: &str) -> bool {
    let i = &a.istemci;
    (i.acik && jar_bul(mc).is_some()) || i.shader || i.sesli_sohbet || i.kayit
}

/// Launcher temasının oyundaki menü renkleri
fn tema_renkleri(tema: &str) -> (&'static str, &'static str) {
    match tema {
        "siyah" => ("#0a0a0a", "#000000"),
        "end" => ("#22143f", "#07050f"),
        "nether" => ("#43100c", "#120404"),
        "dunya" => ("#123a31", "#0a1620"),
        "performans" => ("#141416", "#0a0a0b"),
        _ => ("#232328", "#0d0d10"),
    }
}

fn ayar_dosyasi() -> PathBuf {
    oyun_klasoru().join("config").join("kcz-client.json")
}

/// Oyunu açmadan önce: launcher ayarlarını ve tema renklerini oyundaki menüye yaz
pub fn ayar_yaz(a: &Ayarlar) {
    let yol = ayar_dosyasi();
    let mut j: Value = std::fs::read_to_string(&yol)
        .ok()
        .and_then(|m| serde_json::from_str(&m).ok())
        .filter(|v: &Value| v.is_object())
        .unwrap_or_else(|| json!({}));
    let i = &a.istemci;
    let (zemin, zemin2) = tema_renkleri(&a.tema);
    for (k, v) in [
        ("zoom", json!(i.zoom)),
        ("zoomKat", json!(i.zoom_kat)),
        ("fps", json!(i.fps)),
        ("ping", json!(i.ping)),
        ("tuslar", json!(i.tuslar)),
        ("konum", json!(i.konum)),
        ("optimizasyon", json!(a.hafif_mod)),
        ("shader", json!(i.shader)),
        ("sesliSohbet", json!(i.sesli_sohbet)),
        ("kayit", json!(i.kayit)),
        ("arkaplanFps", json!(i.arkaplan_fps)),
        ("tema", json!({ "zemin": zemin, "zemin2": zemin2, "vurgu": a.vurgu })),
    ] {
        j[k] = v;
    }
    if let Some(k) = yol.parent() {
        let _ = std::fs::create_dir_all(k);
    }
    if let Ok(m) = serde_json::to_string_pretty(&j) {
        let _ = std::fs::write(&yol, m);
    }
}

/// Oyun kapandıktan sonra: oyunun içinden yapılan değişiklikleri launcher ayarlarına al
pub fn ayar_geri_oku() {
    let Some(j) = std::fs::read_to_string(ayar_dosyasi())
        .ok()
        .and_then(|m| serde_json::from_str::<Value>(&m).ok())
    else {
        return;
    };
    let a = ayarlar::getir();
    let mut i = serde_json::to_value(&a.istemci).unwrap_or(json!({}));
    let mut degisti = false;
    for k in [
        "zoom",
        "zoomKat",
        "fps",
        "ping",
        "tuslar",
        "konum",
        "shader",
        "sesliSohbet",
        "kayit",
        "arkaplanFps",
    ] {
        if let Some(v) = j.get(k) {
            if i.get(k) != Some(v) && (v.is_boolean() || v.is_number() || v.is_string()) {
                i[k] = v.clone();
                degisti = true;
            }
        }
    }
    let mut yeni = json!({ "istemci": i });
    if let Some(o) = j["optimizasyon"].as_bool() {
        if o != a.hafif_mod {
            yeni["hafifMod"] = json!(o);
            degisti = true;
        }
    }
    if degisti {
        ayarlar::kaydet(&yeni);
        log("KÇZ Client ayarları oyundan alındı.");
    }
}

/* ---------- Eklentiler ---------- */
const SHADER_PAKETI: &str = "complementary-reimagined";

/// Shader paketini kur (yoksa) ve Iris'e "bunu kullan" de
async fn shader_hazirla(mc: &str, yukleyici: &str) {
    let klasor = oyun_klasoru().join("shaderpacks");
    let bul = || -> Option<String> {
        std::fs::read_dir(&klasor)
            .ok()?
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .find(|d| {
                let d = d.to_lowercase();
                d.contains("complementary") && (d.ends_with(".zip") || !d.contains('.'))
            })
    };
    if bul().is_none() {
        if let Err(e) = crate::modlar::yukle(SHADER_PAKETI, "shader", mc, yukleyici, None).await {
            log(format!("Shader paketi kurulamadı: {e}"));
        }
    }
    iris_ayari(bul());
}

/// config/iris.properties: shader açık/kapalı ve hangi paket
fn iris_ayari(paket: Option<String>) {
    let yol = oyun_klasoru().join("config").join("iris.properties");
    let eski = std::fs::read_to_string(&yol).unwrap_or_default();
    let mut satirlar: Vec<String> = eski
        .lines()
        .filter(|s| !s.starts_with("enableShaders=") && !(paket.is_some() && s.starts_with("shaderPack=")))
        .map(String::from)
        .collect();
    satirlar.push(format!("enableShaders={}", paket.is_some()));
    if let Some(p) = paket {
        satirlar.push(format!("shaderPack={p}"));
    }
    if let Some(k) = yol.parent() {
        let _ = std::fs::create_dir_all(k);
    }
    let _ = std::fs::write(&yol, satirlar.join("\n") + "\n");
}

/// Oyunu açmadan önce: KÇZ Client ve açık eklentilerin klasörlerini hazırlar
pub async fn klasorler_hazirla(mc: &str, tur: &str, a: &Ayarlar) -> Vec<PathBuf> {
    let mut k: Vec<PathBuf> = vec![];
    let i = &a.istemci;

    // Gömülü KÇZ Client (yalnızca Fabric/Quilt) + istediği Fabric API
    if i.acik && mod_yukleyici_fabric(tur) {
        if let Some(jar) = jar_bul(mc) {
            let temel = if tur == "quilt" { "qvIfYCYJ" } else { "P7dR8mSH" };
            let klasor = performans::klasor_hazirla("kcz-istemci", vec![vec![temel]], mc, tur, Some("KÇZ Client hazırlanıyor…")).await;
            let hedef = klasor.join("kcz-client.jar");
            let ayni = std::fs::read(&hedef).map(|b| b == jar).unwrap_or(false);
            if !ayni && std::fs::write(&hedef, jar).is_err() {
                log("KÇZ Client yerleştirilemedi.");
            }
            k.push(klasor);
        }
    }

    if i.shader {
        k.push(performans::klasor_hazirla("kcz-eklenti-shader", vec![vec!["iris"]], mc, tur, Some("Shader hazırlanıyor…")).await);
        shader_hazirla(mc, tur).await;
    } else if std::fs::metadata(oyun_klasoru().join("config").join("iris.properties")).is_ok() {
        iris_ayari(None);
    }
    if i.sesli_sohbet {
        k.push(
            performans::klasor_hazirla(
                "kcz-eklenti-sesli",
                vec![vec!["simple-voice-chat"]],
                mc,
                tur,
                Some("Sesli sohbet hazırlanıyor…"),
            )
            .await,
        );
    }
    if i.kayit {
        k.push(
            performans::klasor_hazirla(
                "kcz-eklenti-kayit",
                vec![vec!["flashback"]],
                mc,
                tur,
                Some("Oyun kaydı hazırlanıyor…"),
            )
            .await,
        );
    }
    k
}
