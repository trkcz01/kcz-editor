package com.kcz.client;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

import net.fabricmc.loader.api.FabricLoader;

/**
 * KÇZ Client ayarları: config/kcz-client.json
 * Launcher oyunu açmadan önce bu dosyayı (tema renkleriyle) yazar,
 * oyun kapanınca da oyunda yapılan değişiklikleri geri okur.
 */
public class Ayar {
	// Oyun içi özellikler (hemen uygulanır)
	public boolean zoom = true;
	public float zoomKat = 4f;
	public boolean fps = true;
	public boolean ping = true;
	public boolean tuslar = false;
	public String konum = "sol-ust";
	// KÇZ optimizasyonu: pencere arkadayken FPS'i düşür
	public boolean arkaplanFps = true;

	// Eklentiler (launcher bir sonraki açılışta uygular)
	public boolean optimizasyon = true;
	public boolean shader = false;
	public boolean sesliSohbet = false;
	public boolean kayit = false;

	public Tema tema = new Tema();

	public static class Tema {
		public String zemin = "#232328";
		public String zemin2 = "#0d0d10";
		public String vurgu = "#a1a1aa";
	}

	private static final Gson GSON = new GsonBuilder().setPrettyPrinting().create();
	private static Ayar mevcut = new Ayar();

	public static Ayar get() {
		return mevcut;
	}

	private static Path dosya() {
		return FabricLoader.getInstance().getConfigDir().resolve("kcz-client.json");
	}

	public static void yukle() {
		try {
			Path d = dosya();
			if (Files.exists(d)) {
				Ayar a = GSON.fromJson(Files.readString(d, StandardCharsets.UTF_8), Ayar.class);
				if (a != null) {
					if (a.tema == null) a.tema = new Tema();
					if (a.konum == null) a.konum = "sol-ust";
					if (a.zoomKat < 1.5f || a.zoomKat > 10f) a.zoomKat = 4f;
					mevcut = a;
				}
			}
		} catch (Exception e) {
			KczClient.LOG.warn("KÇZ ayarları okunamadı: {}", e.toString());
		}
	}

	public static void kaydet() {
		try {
			Files.createDirectories(dosya().getParent());
			Files.writeString(dosya(), GSON.toJson(mevcut), StandardCharsets.UTF_8);
		} catch (Exception e) {
			KczClient.LOG.warn("KÇZ ayarları yazılamadı: {}", e.toString());
		}
	}

	/** "#rrggbb" -> 0xAARRGGBB */
	public static int renk(String hex, int alfa, int yedek) {
		try {
			String h = hex.startsWith("#") ? hex.substring(1) : hex;
			int rgb = Integer.parseInt(h, 16) & 0xFFFFFF;
			return (alfa & 0xFF) << 24 | rgb;
		} catch (Exception e) {
			return yedek;
		}
	}

	/** Rengi beyaza doğru açar (0..1) */
	public static int ac(int argb, float oran) {
		int a = argb >>> 24, r = argb >> 16 & 0xFF, g = argb >> 8 & 0xFF, b = argb & 0xFF;
		r = (int) (r + (255 - r) * oran);
		g = (int) (g + (255 - g) * oran);
		b = (int) (b + (255 - b) * oran);
		return a << 24 | r << 16 | g << 8 | b;
	}

	/** Vurgu açık renkse üstüne koyu yazı */
	public static int ustYazi(int argb) {
		int r = argb >> 16 & 0xFF, g = argb >> 8 & 0xFF, b = argb & 0xFF;
		return 0.299 * r + 0.587 * g + 0.114 * b > 150 ? 0xFF0B0D10 : 0xFFFFFFFF;
	}
}
