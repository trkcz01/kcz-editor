# KÇZ Launcher (Tauri sürümü)

Aynı launcher, Electron yerine **Tauri** ile: arka taraf **Rust**, arayüz aynı.
Kurulum dosyası çok daha küçük (~5–10 MB), launcher daha az RAM kullanır.

## İlk kurulum
1. **`TAURI-KUR.bat`**'a çift tıkla.
   - İlk seferde **Rust** ve **Visual Studio C++ araçları** kurulur (bir kere, 10–30 dk).
   - Güncelleme imzası oluşturulur (`imza` klasörü — **silme, kimseyle paylaşma**).
   - Launcher derlenir ve kurulum ekranı açılır.
2. Hata çıkarsa bu klasördeki **`derleme-kaydi.txt`** dosyasının içini Claude'a gönder.

## Eski (Electron) launcher'dan geçiş
- Sürümler, Java, modlar, sunucular, ayarlar, profiller **aynı yerde** (`%APPDATA%\KÇZ Launcher`) — hepsi korunur.
- Microsoft girişi bir kere tekrar istenir (giriş artık Windows Kimlik Bilgisi Yöneticisi'nde saklanıyor).
- Her şey çalışınca eski launcher'ı Denetim Masası > Programlar'dan kaldırabilirsin.

## Güncelleme yayınlama
- **`YAYINLA.bat`**: sürümü bir artırır (0.0.1 → 0.0.2), imzalı derler, GitHub'a (`trkcz01/kcz-launcher`) yükler.
- İlk seferde GitHub anahtarı ister. `gh-token.txt` ve `imza` klasörünü kimseyle paylaşma.

## KÇZ Client (oyun içi menü)
- Oyunda **Esc** menüsünün sol üstündeki **KÇZ Menü** düğmesi: KÇZ menüsü (launcher temasıyla aynı renkte). **C** basılı: yakınlaştırma.
- FPS, ping ve tuş göstergesi; shader, sesli sohbet ve oyun kaydı açma/kapama. Aynı ayarlar Ayarlar > KÇZ Client'ta da var, ikisi eşitlenir.
- Kaynak kodu `istemci-mod/` klasöründe. `TAURI-KUR.bat` her seferinde Minecraft 26.2 ve 26.3 için derleyip launcher'ın içine gömer
  (ilk seferde Java 25 JDK kurulur). Bu adımda hata olursa launcher yine derlenir; hata yazısı `mod-derleme-kaydi.txt` dosyasındadır.

## Kısayollar
- `Enter` Oyna · `Ctrl+1–4` sayfalar · `Ctrl+,` Ayarlar · `F5` sunucuları yenile · `Esc` açık pencereyi kapat
- Launcher kapatılınca saatin yanındaki simgeye küçülür (Ayarlar'dan kapatılabilir). Tamamen kapatmak için simgeye sağ tık > Çıkış.

## Klasörler
- `ui/` — arayüz (HTML, CSS, JS). `kopru.js` arayüzü Rust tarafına bağlar.
- `src-tauri/src/` — Rust: giriş (`hesap.rs`), sürümler (`surumler.rs`), Java (`java.rs`),
  oyunu başlatma (`oyun.rs`), Fabric/Quilt/Forge/NeoForge (`yukleyici.rs`), modlar (`modlar.rs`),
  sunucular (`sunucular.rs`, `ping.rs`), skinler (`skin.rs`), süre (`sure.rs`), ayarlar (`ayarlar.rs`).
