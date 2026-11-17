// Launcher ayarları: kullanıcı klasöründe ayarlar.json (eski sürümle aynı dosya ve biçim).
use crate::ortak::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::{LazyLock, Mutex};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Skin {
    #[serde(default)]
    pub ad: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dosya: Option<String>,
    #[serde(default)]
    pub ince: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct SunucuProfili {
    #[serde(default)]
    pub surum: String,
    #[serde(default)]
    pub yukleyici: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Profil {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub ad: String,
    #[serde(default)]
    pub ikon: String,
    #[serde(default)]
    pub surum: String,
    #[serde(default)]
    pub yukleyici: String,
}

/// KÇZ Client (oyun içi menü) ayarları. Oyundaki config/kcz-client.json ile eşitlenir.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Istemci {
    pub acik: bool,
    pub zoom: bool,
    pub zoom_kat: f32,
    pub fps: bool,
    pub ping: bool,
    pub tuslar: bool,
    pub konum: String,
    pub shader: bool,
    pub sesli_sohbet: bool,
    pub kayit: bool,
    pub arkaplan_fps: bool,
}

impl Default for Istemci {
    fn default() -> Self {
        Istemci {
            acik: true,
            zoom: true,
            zoom_kat: 4.0,
            fps: true,
            ping: true,
            tuslar: false,
            konum: "sol-ust".into(),
            shader: false,
            sesli_sohbet: false,
            kayit: false,
            arkaplan_fps: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Pencere {
    pub w: u32,
    pub h: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Ayarlar {
    pub mc_surum: String,
    pub ram: u32,
    pub yukleyici: String,
    pub oyun_acilinca_gizle: bool,
    pub tema: String,
    pub vurgu: String,
    pub saydamlik: u32,
    pub secili_sunucu: String,
    pub skinler: Vec<Skin>,
    pub sunucu_profilleri: BTreeMap<String, SunucuProfili>,
    pub pencere: Pencere,
    pub profiller: Vec<Profil>,
    pub secili_profil: String,
    pub aktif_secim: String,
    pub sol_sekme: String,
    // Performans
    pub hafif_mod: bool,
    pub ekran_karti: bool,
    pub guc_plani: bool,
    pub java_ayar: bool,
    pub oncelik: bool,
    pub performans_seviye: String,
    // KÇZ Client ve kolaylıklar
    pub istemci: Istemci,
    pub tepsiye_kucult: bool,
    pub dunya_yedek: bool,
}

impl Default for Ayarlar {
    fn default() -> Self {
        Ayarlar {
            mc_surum: "26.2".into(),
            ram: 4,
            yukleyici: "fabric".into(),
            oyun_acilinca_gizle: true,
            tema: "gri".into(),
            vurgu: "#a1a1aa".into(),
            saydamlik: 1,
            secili_sunucu: String::new(),
            skinler: vec![],
            sunucu_profilleri: BTreeMap::new(),
            pencere: Pencere { w: 1100, h: 680 },
            profiller: vec![],
            secili_profil: String::new(),
            aktif_secim: "sunucu".into(),
            sol_sekme: "sunucu".into(),
            hafif_mod: true,
            ekran_karti: true,
            guc_plani: false,
            java_ayar: true,
            oncelik: false,
            performans_seviye: String::new(),
            istemci: Istemci::default(),
            tepsiye_kucult: true,
            dunya_yedek: true,
        }
    }
}

const YUKLEYICILER: [&str; 5] = ["vanilla", "fabric", "quilt", "forge", "neoforge"];
const IKONLAR: [&str; 18] = [
    "yildiz", "simsek", "kalp", "alev", "kalkan", "tac", "kristal", "roket", "yaprak", "ay", "dag", "kup", "kupa", "kar", "gezegen",
    "kilic", "oyun", "portal",
];

static SURUM_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[A-Za-z0-9._-]{1,40}$").unwrap());
static RENK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^#[0-9a-fA-F]{6}$").unwrap());
static SKIN_URL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^https?://textures\.minecraft\.net/texture/[0-9a-f]+$").unwrap());
static SKIN_DOSYA_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9a-f]{16}\.png$").unwrap());

fn kes(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

pub fn surum_gecerli(s: &str) -> bool {
    SURUM_RE.is_match(s)
}

fn duzelt(mut t: Ayarlar) -> Ayarlar {
    let v = Ayarlar::default();
    t.mc_surum = t.mc_surum.trim().to_string();
    if t.mc_surum.is_empty() {
        t.mc_surum = v.mc_surum.clone();
    }
    t.ram = t.ram.clamp(2, 16);
    if !YUKLEYICILER.contains(&t.yukleyici.as_str()) {
        t.yukleyici = "fabric".into();
    }
    if !["gri", "siyah", "end", "nether", "dunya", "performans"].contains(&t.tema.as_str()) {
        t.tema = "gri".into();
    }
    t.vurgu = if RENK_RE.is_match(&t.vurgu) {
        t.vurgu.to_lowercase()
    } else {
        v.vurgu.clone()
    };
    if t.saydamlik > 3 {
        t.saydamlik = 1;
    }
    t.secili_sunucu = kes(&t.secili_sunucu, 300);
    t.skinler = t
        .skinler
        .into_iter()
        .filter_map(|s| {
            let ad = kes(if s.ad.is_empty() { "Skin" } else { &s.ad }, 24);
            if let Some(u) = s.url.as_ref().filter(|u| SKIN_URL_RE.is_match(u)) {
                return Some(Skin {
                    ad,
                    url: Some(u.replacen("http:", "https:", 1)),
                    dosya: None,
                    ince: s.ince,
                });
            }
            if let Some(d) = s.dosya.as_ref().filter(|d| SKIN_DOSYA_RE.is_match(d)) {
                return Some(Skin {
                    ad,
                    url: None,
                    dosya: Some(d.clone()),
                    ince: s.ince,
                });
            }
            None
        })
        .take(30)
        .collect();
    t.sunucu_profilleri = t
        .sunucu_profilleri
        .into_iter()
        .filter(|(adres, p)| adres.len() <= 300 && SURUM_RE.is_match(p.surum.trim()))
        .map(|(adres, p)| {
            let y = if ["fabric", "quilt", "forge", "neoforge"].contains(&p.yukleyici.as_str()) {
                p.yukleyici
            } else {
                "fabric".into()
            };
            (
                adres,
                SunucuProfili {
                    surum: p.surum.trim().to_string(),
                    yukleyici: y,
                },
            )
        })
        .collect();
    t.profiller = t
        .profiller
        .into_iter()
        .filter_map(|p| {
            let id: String = p.id.chars().filter(|c| c.is_ascii_alphanumeric()).take(16).collect();
            let surum = p.surum.trim().to_string();
            if id.is_empty() || !SURUM_RE.is_match(&surum) {
                return None;
            }
            let ad = kes(p.ad.trim(), 32);
            Some(Profil {
                id,
                ad: if ad.is_empty() { "Profil".into() } else { ad },
                ikon: if IKONLAR.contains(&p.ikon.as_str()) { p.ikon } else { "kup".into() },
                surum,
                yukleyici: if YUKLEYICILER.contains(&p.yukleyici.as_str()) {
                    p.yukleyici
                } else {
                    "vanilla".into()
                },
            })
        })
        .take(50)
        .collect();
    t.secili_profil = kes(&t.secili_profil, 16);
    if t.aktif_secim != "profil" {
        t.aktif_secim = "sunucu".into();
    }
    if t.sol_sekme != "profil" {
        t.sol_sekme = "sunucu".into();
    }
    if !["dusuk", "orta", "yuksek"].contains(&t.performans_seviye.as_str()) {
        t.performans_seviye = String::new();
    }
    if !t.istemci.zoom_kat.is_finite() {
        t.istemci.zoom_kat = 4.0;
    }
    t.istemci.zoom_kat = t.istemci.zoom_kat.clamp(1.5, 10.0);
    if t.istemci.konum != "sag-ust" {
        t.istemci.konum = "sol-ust".into();
    }
    t.pencere.w = t.pencere.w.clamp(1000, 4000);
    t.pencere.h = t.pencere.h.clamp(640, 3000);
    t
}

static MEVCUT: LazyLock<Mutex<Ayarlar>> = LazyLock::new(|| Mutex::new(Ayarlar::default()));

fn dosya() -> std::path::PathBuf {
    kullanici_klasoru().join("ayarlar.json")
}

/// Değerleri tek tek okuyup ayarlara yerleştirir (biri bozuksa diğerleri kaybolmasın)
fn birlestir(taban: &Ayarlar, yeni: &Value) -> Ayarlar {
    let mut tam = serde_json::to_value(taban).unwrap_or(Value::Null);
    if let (Some(hedef), Some(kaynak)) = (tam.as_object_mut(), yeni.as_object()) {
        for (k, v) in kaynak {
            if hedef.contains_key(k) {
                // Tek bir alan bozuksa onu atla
                let mut deneme = hedef.clone();
                deneme.insert(k.clone(), v.clone());
                if serde_json::from_value::<Ayarlar>(Value::Object(deneme)).is_ok() {
                    hedef.insert(k.clone(), v.clone());
                } else if k == "ram" {
                    // Arayüz RAM'i metin olarak gönderebilir ("6")
                    if let Some(n) = v.as_str().and_then(|x| x.parse::<u32>().ok()) {
                        hedef.insert(k.clone(), Value::from(n));
                    }
                }
            }
        }
    }
    serde_json::from_value(tam).unwrap_or_else(|_| taban.clone())
}

pub fn yukle() {
    let mut a = Ayarlar::default();
    if let Ok(metin) = std::fs::read_to_string(dosya()) {
        if let Ok(mut okunan) = serde_json::from_str::<Value>(&metin) {
            // Eski sürümden geçiş: "fabric: false" ise vanilla
            if okunan.get("yukleyici").is_none() && okunan.get("fabric") == Some(&Value::Bool(false)) {
                okunan["yukleyici"] = Value::from("vanilla");
            }
            a = birlestir(&a, &okunan);
        }
    }
    *MEVCUT.lock().unwrap() = duzelt(a);
}

pub fn getir() -> Ayarlar {
    MEVCUT.lock().unwrap().clone()
}

pub fn kaydet(yeni: &Value) -> Ayarlar {
    let mut kilit = MEVCUT.lock().unwrap();
    let a = duzelt(birlestir(&kilit, yeni));
    *kilit = a.clone();
    drop(kilit);
    if let Ok(metin) = serde_json::to_string_pretty(&a) {
        let _ = std::fs::write(dosya(), metin);
    }
    a
}
