// Skin işlemleri: hesaptaki skinler, Minecraft Launcher'ın skin kütüphanesi, oyuncu adından skin,
// bilgisayardan skin ekleme ve skini Mojang'ın resmi API'siyle hesaba giydirme.
use crate::ayarlar::{self, Skin};
use crate::ortak::*;
use base64::Engine;
use serde_json::{json, Value};
use std::path::PathBuf;

pub const HAZIR: [(&str, &str, &[u8]); 5] = [
    ("kcz", "KÇZ Kapüşonlu", include_bytes!("../../ui/skinler/kcz.png")),
    ("ninja", "Gece Ninja", include_bytes!("../../ui/skinler/ninja.png")),
    ("kar", "Kar Kaşifi", include_bytes!("../../ui/skinler/kar.png")),
    ("lav", "Lav Savaşçısı", include_bytes!("../../ui/skinler/lav.png")),
    ("altin", "Altın Şövalye", include_bytes!("../../ui/skinler/altin.png")),
];

fn b64(v: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(v)
}

fn https(u: &str) -> String {
    u.replacen("http://", "https://", 1)
}

fn klasor() -> PathBuf {
    let k = kullanici_klasoru().join("skinler");
    let _ = std::fs::create_dir_all(&k);
    k
}

/// PNG skin dosyası mı (64x64 ya da eski 64x32)
fn png_kontrol(v: &[u8]) -> Sonuc<()> {
    if v.len() < 24 || v[..4] != [0x89, 0x50, 0x4e, 0x47] {
        return Err("Skin bir PNG dosyası olmalı.".into());
    }
    let g = u32::from_be_bytes(v[16..20].try_into().unwrap());
    let y = u32::from_be_bytes(v[20..24].try_into().unwrap());
    if g != 64 || (y != 64 && y != 32) {
        return Err("Skin 64x64 (ya da eski 64x32) boyutunda olmalı.".into());
    }
    if v.len() > 1024 * 1024 {
        return Err("Dosya çok büyük.".into());
    }
    Ok(())
}

/* ---------- Hesaptaki skinler ---------- */
pub async fn hesap_skinleri(erisim: &str) -> Option<(String, Vec<(String, bool, bool)>)> {
    let p = json_al_bos("https://api.minecraftservices.com/minecraft/profile", Some(erisim))
        .await
        .ok()??;
    let mut s: Vec<(String, bool, bool)> = p["skins"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|x| {
            (
                https(&metin(x, "url")),
                metin(x, "variant").eq_ignore_ascii_case("slim"),
                metin(x, "state") == "ACTIVE",
            )
        })
        .collect();
    s.sort_by_key(|x| !x.2);
    Some((metin(&p, "name"), s))
}

/* ---------- Minecraft Launcher kütüphanesi ---------- */
pub struct KutuphaneSkini {
    pub id: String,
    pub ad: String,
    pub ince: bool,
    pub doku: String,
    pub tarih: i64,
    pub veri: Vec<u8>,
}

pub fn kutuphane() -> Vec<KutuphaneSkini> {
    let kok = PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join(".minecraft");
    let mut sonuc: Vec<KutuphaneSkini> = vec![];
    for d in ["launcher_custom_skins.json", "launcher_skins.json"] {
        let Ok(m) = std::fs::read_to_string(kok.join(d)) else { continue };
        let Ok(j) = serde_json::from_str::<Value>(&m) else { continue };
        let kayitlar = j.get("customSkins").or_else(|| j.get("skins")).cloned().unwrap_or(j.clone());
        let Some(o) = kayitlar.as_object() else { continue };
        for k in o.values() {
            let resim = k
                .get("skinImage")
                .or_else(|| k.get("textureImage"))
                .and_then(|x| x.as_str())
                .unwrap_or("");
            let Some(b) = resim.strip_prefix("data:image/png;base64,") else {
                continue;
            };
            let Ok(veri) = base64::engine::general_purpose::STANDARD.decode(b) else {
                continue;
            };
            if png_kontrol(&veri).is_err() {
                continue;
            }
            let anahtar = sha1_hex(&veri);
            let ad: String = metin(k, "name").trim().chars().take(24).collect();
            if let Some(ayni) = sonuc.iter_mut().find(|x| x.id == anahtar[..16]) {
                if ayni.ad.is_empty() && !ad.is_empty() {
                    ayni.ad = ad;
                }
                continue;
            }
            let tarih = ["updated", "created"]
                .iter()
                .filter_map(|a| k.get(*a).and_then(|x| x.as_str()))
                .filter_map(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                .map(|t| t.timestamp_millis())
                .next()
                .unwrap_or(0);
            sonuc.push(KutuphaneSkini {
                id: anahtar[..16].to_string(),
                ad,
                ince: k.get("slim").and_then(|x| x.as_bool()).unwrap_or(false) || metin(k, "model") == "slim",
                doku: metin(k, "textureId"),
                tarih,
                veri,
            });
        }
    }
    sonuc.sort_by(|a, b| b.tarih.cmp(&a.tarih));
    sonuc
}

/* ---------- Vitrin listesi ---------- */
pub async fn liste() -> Value {
    let a = ayarlar::getir();
    let mut liste: Vec<Value> = vec![];
    let mut urller: Vec<String> = vec![];
    if let Some(o) = crate::hesap::yetki().await {
        if let Some((ad, skinler)) = hesap_skinleri(&o.erisim).await {
            for (i, (url, ince, aktif)) in skinler.iter().enumerate() {
                urller.push(url.clone());
                let kaynak = json!({ "url": url, "ince": ince });
                liste.push(if *aktif {
                    json!({ "tur": "benim", "ad": "Senin skinin", "alt": ad, "doku": url, "ince": ince, "kaynak": kaynak })
                } else {
                    json!({ "tur": "hesap", "ad": format!("Hesabındaki skin {i}"), "alt": "Minecraft hesabında", "doku": url, "ince": ince, "kaynak": kaynak })
                });
            }
        }
    }
    let hashler: Vec<String> = urller.iter().filter_map(|u| u.rsplit('/').next().map(String::from)).collect();
    let mut n = 0;
    for k in kutuphane() {
        if !k.doku.is_empty() && hashler.contains(&k.doku) {
            continue; // şu an giydiğin skin zaten başta
        }
        n += 1;
        let tarih = if k.tarih > 0 {
            chrono::DateTime::from_timestamp_millis(k.tarih)
                .map(|t| t.format("%d.%m.%Y").to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        liste.push(json!({
            "tur": "kutuphane",
            "ad": if k.ad.is_empty() { format!("Skinim {n}") } else { k.ad.clone() },
            "alt": if tarih.is_empty() { "Minecraft Launcher kütüphanen".to_string() } else { format!("Minecraft Launcher kütüphanen · {tarih}") },
            "doku": format!("data:image/png;base64,{}", b64(&k.veri)),
            "ince": k.ince,
            "kaynak": { "kutuphane": k.id, "ince": k.ince }
        }));
    }
    for (i, s) in a.skinler.iter().enumerate() {
        if let Some(u) = &s.url {
            if urller.contains(u) {
                continue;
            }
        }
        let doku = if let Some(u) = &s.url {
            Some(https(u))
        } else if let Some(d) = &s.dosya {
            std::fs::read(klasor().join(d))
                .ok()
                .map(|v| format!("data:image/png;base64,{}", b64(&v)))
        } else {
            None
        };
        if let Some(doku) = doku {
            liste.push(json!({
                "tur": "ekli", "sira": i, "ad": s.ad, "alt": if s.url.is_some() { "Oyuncu skini" } else { "Senin dosyan" },
                "doku": doku, "ince": s.ince, "kaynak": s
            }));
        }
    }
    for (id, ad, _) in HAZIR {
        liste.push(json!({
            "tur": "hazir", "ad": ad, "alt": "KÇZ skini", "doku": format!("skinler/{id}.png"), "ince": false,
            "kaynak": { "hazir": id, "ince": false }
        }));
    }
    Value::Array(liste)
}

/* ---------- Oyuncu adından skin ---------- */
pub async fn isimden_ekle(isim: &str) -> Sonuc<()> {
    let isim = isim.trim();
    if !(2..=16).contains(&isim.len()) || !isim.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Geçerli bir Minecraft kullanıcı adı yaz.".into());
    }
    let o = json_al_bos(&format!("https://api.mojang.com/users/profiles/minecraft/{isim}"), None)
        .await?
        .ok_or("Bu isimde bir oyuncu bulunamadı.")?;
    let id = metin(&o, "id");
    let p = json_al_bos(&format!("https://sessionserver.mojang.com/session/minecraft/profile/{id}"), None)
        .await?
        .ok_or("Bu oyuncunun skini alınamadı.")?;
    let doku = p["properties"]
        .as_array()
        .and_then(|d| d.iter().find(|x| metin(x, "name") == "textures"))
        .map(|x| metin(x, "value"))
        .ok_or("Bu oyuncunun skini alınamadı.")?;
    let t: Value = serde_json::from_slice(&base64::engine::general_purpose::STANDARD.decode(doku).h()?).h()?;
    let s = &t["textures"]["SKIN"];
    let url = https(&metin(s, "url"));
    if url.is_empty() {
        return Err("Bu oyuncu varsayılan skini kullanıyor.".into());
    }
    let ince = metin(&s["metadata"], "model") == "slim";
    let mut a = ayarlar::getir();
    if !a.skinler.iter().any(|x| x.url.as_deref() == Some(&url)) {
        a.skinler.push(Skin {
            ad: metin(&o, "name"),
            url: Some(url),
            dosya: None,
            ince,
        });
    }
    ayarlar::kaydet(&json!({ "skinler": a.skinler }));
    Ok(())
}

/* ---------- Bilgisayardan skin ---------- */
pub fn dosyadan_ekle(kaynak: &std::path::Path, ince: bool) -> Sonuc<()> {
    let veri = std::fs::read(kaynak).h()?;
    png_kontrol(&veri)?;
    let ad = format!("{}.png", &sha1_hex(&veri)[..16]);
    std::fs::write(klasor().join(&ad), &veri).h()?;
    let gorunen: String = kaynak
        .file_stem()
        .map(|x| x.to_string_lossy().to_string())
        .unwrap_or_default()
        .chars()
        .take(24)
        .collect();
    let mut a = ayarlar::getir();
    if !a.skinler.iter().any(|x| x.dosya.as_deref() == Some(&ad)) {
        a.skinler.push(Skin {
            ad: gorunen,
            url: None,
            dosya: Some(ad),
            ince,
        });
    }
    ayarlar::kaydet(&json!({ "skinler": a.skinler }));
    Ok(())
}

pub fn kaldir(sira: usize) {
    let mut a = ayarlar::getir();
    if sira < a.skinler.len() {
        a.skinler.remove(sira);
    }
    ayarlar::kaydet(&json!({ "skinler": a.skinler }));
}

/* ---------- Skini hesaba giydir ---------- */
pub async fn giy(kaynak: &Value) -> Sonuc<()> {
    let o = crate::hesap::yetki().await.ok_or("Önce Microsoft hesabınla giriş yapmalısın.")?;
    let ince = kaynak.get("ince").and_then(|x| x.as_bool()).unwrap_or(false);
    let variant = if ince { "slim" } else { "classic" };
    let adres = "https://api.minecraftservices.com/minecraft/profile/skins";

    // Gönderilecek dosya (varsa)
    let mut veri: Option<Vec<u8>> = None;
    if let Some(h) = kaynak.get("hazir").and_then(|x| x.as_str()) {
        veri = HAZIR.iter().find(|x| x.0 == h).map(|x| x.2.to_vec());
        if veri.is_none() {
            return Err("Bilinmeyen skin.".into());
        }
    } else if let Some(d) = kaynak.get("dosya").and_then(|x| x.as_str()) {
        let ad = std::path::Path::new(d)
            .file_name()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or_default();
        veri = Some(std::fs::read(klasor().join(ad)).map_err(|_| "Skin dosyası bulunamadı.".to_string())?);
    } else if let Some(k) = kaynak.get("kutuphane").and_then(|x| x.as_str()) {
        veri = Some(
            kutuphane()
                .into_iter()
                .find(|x| x.id == k)
                .map(|x| x.veri)
                .ok_or("Bu skin artık Minecraft Launcher kütüphanesinde yok.")?,
        );
    }

    let r = if let Some(v) = veri {
        png_kontrol(&v)?;
        let parca = reqwest::multipart::Part::bytes(v).file_name("skin.png").mime_str("image/png").h()?;
        let form = reqwest::multipart::Form::new().text("variant", variant).part("file", parca);
        ISTEMCI.post(adres).bearer_auth(&o.erisim).multipart(form).send().await.h()?
    } else {
        let url = https(kaynak.get("url").and_then(|x| x.as_str()).unwrap_or(""));
        if !url.starts_with("https://textures.minecraft.net/") {
            return Err("Bu skin giyilemiyor.".into());
        }
        ISTEMCI
            .post(adres)
            .bearer_auth(&o.erisim)
            .json(&json!({ "variant": variant, "url": url }))
            .send()
            .await
            .h()?
    };
    match r.status().as_u16() {
        200..=299 => Ok(()),
        429 => Err("Çok sık skin değiştirdin, biraz bekleyip tekrar dene.".into()),
        k => Err(format!("Skin değiştirilemedi ({k}).")),
    }
}
