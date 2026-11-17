package com.kcz.client;

import java.util.ArrayList;
import java.util.List;
import java.util.function.BooleanSupplier;
import java.util.function.Supplier;

import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.client.input.KeyEvent;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.network.chat.Component;

/**
 * KÇZ Client menüsü (ESC ekranındaki "KÇZ Menü" düğmesi). Renkleri launcher'da seçili temadan gelir.
 * Sol sütun hemen uygulanır; sağ sütundaki eklentileri launcher bir sonraki açılışta uygular.
 */
public class KczEkrani extends Screen {
	private final Screen onceki;

	private record Alan(int x, int y, int w, int h, Runnable tik) {
		boolean icinde(double mx, double my) {
			return mx >= x && mx < x + w && my >= y && my < y + h;
		}
	}

	private final List<Alan> alanlar = new ArrayList<>();
	private boolean degisti = false;

	private static final float[] ZOOM_KATLARI = {2f, 3f, 4f, 6f, 8f};

	public KczEkrani(Screen onceki) {
		super(Component.literal("KÇZ Client"));
		this.onceki = onceki;
	}

	@Override
	public void onClose() {
		if (degisti) Ayar.kaydet();
		this.minecraft.gui.setScreen(this.onceki);
	}

	@Override
	public boolean keyPressed(KeyEvent olay) {
		if (KczClient.MENU.matches(olay)) {
			onClose();
			return true;
		}
		return super.keyPressed(olay);
	}

	@Override
	public boolean mouseClicked(MouseButtonEvent olay, boolean ciftTik) {
		if (olay.button() == 0) {
			for (Alan a : alanlar) {
				if (a.icinde(olay.x(), olay.y())) {
					a.tik().run();
					degisti = true;
					Ayar.kaydet();
					return true;
				}
			}
		}
		return super.mouseClicked(olay, ciftTik);
	}

	@Override
	public void extractRenderState(GuiGraphicsExtractor g, int mx, int my, float delta) {
		super.extractRenderState(g, mx, my, delta);
		alanlar.clear();
		Ayar a = Ayar.get();

		int zemin = Ayar.renk(a.tema.zemin, 0xF2, 0xF2232328);
		int zemin2 = Ayar.renk(a.tema.zemin2, 0xF2, 0xF20D0D10);
		int vurgu = Ayar.renk(a.tema.vurgu, 0xFF, 0xFFA1A1AA);
		int kart = Ayar.ac(zemin, 0.06f);
		int kartUstu = Ayar.ac(zemin, 0.13f);
		int yazi = 0xFFF1F5F9;
		int soluk = 0xFF9CA3AF;

		int pw = Math.min(440, this.width - 16);
		int satirH = 20, araH = 3;
		int ph = 34 + 14 + 6 * (satirH + araH) + 26;
		int px = (this.width - pw) / 2;
		int py = Math.max(6, (this.height - ph) / 2);

		// Panel
		g.fill(px, py, px + pw, py + ph, zemin2);
		g.fill(px + 1, py + 1, px + pw - 1, py + ph - 1, zemin);
		g.fill(px, py, px + pw, py + 2, vurgu);

		// Başlık: K logosu + ad
		int lx = px + 10, ly = py + 10;
		g.fill(lx, ly, lx + 16, ly + 16, vurgu);
		g.text(this.font, "K", lx + 5, ly + 4, Ayar.ustYazi(vurgu), false);
		g.text(this.font, "KÇZ Client", lx + 22, ly + 4, yazi, true);
		String ipucu = "Esc ile geri dön";
		g.text(this.font, ipucu, px + pw - 10 - this.font.width(ipucu), ly + 4, soluk, false);

		int pad = 10;
		int sw = (pw - pad * 3) / 2;
		int solX = px + pad, sagX = px + pad * 2 + sw;
		int y0 = py + 34;

		g.text(this.font, "OYUN İÇİ", solX, y0, soluk, false);
		g.text(this.font, "EKLENTİLER · sonraki açılışta", sagX, y0, soluk, false);
		int y = y0 + 14;

		// Sol sütun: hemen uygulanır
		int sy = y;
		sy = anahtar(g, solX, sy, sw, satirH, "Yakınlaştırma (C + tekerlek)", () -> a.zoom, () -> a.zoom = !a.zoom, mx, my, kart, kartUstu, vurgu, yazi, soluk) + araH;
		sy = secim(g, solX, sy, sw, satirH, "Zoom gücü", () -> kat(a.zoomKat), () -> {
			int i = 0;
			while (i < ZOOM_KATLARI.length && ZOOM_KATLARI[i] <= a.zoomKat + 0.01f) i++;
			a.zoomKat = ZOOM_KATLARI[i % ZOOM_KATLARI.length];
		}, mx, my, kart, kartUstu, vurgu, yazi, soluk) + araH;
		sy = anahtar(g, solX, sy, sw, satirH, "FPS göstergesi", () -> a.fps, () -> a.fps = !a.fps, mx, my, kart, kartUstu, vurgu, yazi, soluk) + araH;
		sy = anahtar(g, solX, sy, sw, satirH, "Ping göstergesi", () -> a.ping, () -> a.ping = !a.ping, mx, my, kart, kartUstu, vurgu, yazi, soluk) + araH;
		sy = anahtar(g, solX, sy, sw, satirH, "Tuş göstergesi", () -> a.tuslar, () -> a.tuslar = !a.tuslar, mx, my, kart, kartUstu, vurgu, yazi, soluk) + araH;
		secim(g, solX, sy, sw, satirH, "Konum", () -> "sag-ust".equals(a.konum) ? "Sağ üst" : "Sol üst",
				() -> a.konum = "sag-ust".equals(a.konum) ? "sol-ust" : "sag-ust", mx, my, kart, kartUstu, vurgu, yazi, soluk);

		// Sağ sütun: launcher bir sonraki açılışta uygular
		int ry = y;
		ry = anahtar(g, sagX, ry, sw, satirH, "KÇZ Optimizasyon", () -> a.optimizasyon, () -> a.optimizasyon = !a.optimizasyon, mx, my, kart, kartUstu, vurgu, yazi, soluk) + araH;
		ry = anahtar(g, sagX, ry, sw, satirH, "Shader (gölge ve ışık)", () -> a.shader, () -> a.shader = !a.shader, mx, my, kart, kartUstu, vurgu, yazi, soluk) + araH;
		ry = anahtar(g, sagX, ry, sw, satirH, "Sesli sohbet", () -> a.sesliSohbet, () -> a.sesliSohbet = !a.sesliSohbet, mx, my, kart, kartUstu, vurgu, yazi, soluk) + araH;
		ry = anahtar(g, sagX, ry, sw, satirH, "Oyun kaydı (Flashback)", () -> a.kayit, () -> a.kayit = !a.kayit, mx, my, kart, kartUstu, vurgu, yazi, soluk) + araH;
		g.text(this.font, "Bu değişiklikler oyunu", sagX + 2, ry + 4, soluk, false);
		g.text(this.font, "kapatıp açınca uygulanır.", sagX + 2, ry + 14, soluk, false);

		String alt = "Ayarlar launcher ile eşitlenir";
		g.text(this.font, alt, px + (pw - this.font.width(alt)) / 2, py + ph - 14, soluk, false);
	}

	private static String kat(float k) {
		return (k == Math.round(k) ? String.valueOf(Math.round(k)) : String.valueOf(k)) + "x";
	}

	private int anahtar(GuiGraphicsExtractor g, int x, int y, int w, int h, String ad, BooleanSupplier deger, Runnable tik,
			int mx, int my, int kart, int kartUstu, int vurgu, int yazi, int soluk) {
		boolean ust = mx >= x && mx < x + w && my >= y && my < y + h;
		g.fill(x, y, x + w, y + h, ust ? kartUstu : kart);
		g.text(this.font, ad, x + 7, y + (h - 8) / 2, yazi, false);
		boolean acik = deger.getAsBoolean();
		int tw = 22, th = 10;
		int tx = x + w - tw - 7, ty = y + (h - th) / 2;
		g.fill(tx, ty, tx + tw, ty + th, acik ? vurgu : 0xFF4B5563);
		int dx = acik ? tx + tw - 9 : tx + 1;
		g.fill(dx, ty + 1, dx + 8, ty + th - 1, 0xFFFFFFFF);
		alanlar.add(new Alan(x, y, w, h, tik));
		return y + h;
	}

	private int secim(GuiGraphicsExtractor g, int x, int y, int w, int h, String ad, Supplier<String> deger, Runnable tik,
			int mx, int my, int kart, int kartUstu, int vurgu, int yazi, int soluk) {
		boolean ust = mx >= x && mx < x + w && my >= y && my < y + h;
		g.fill(x, y, x + w, y + h, ust ? kartUstu : kart);
		g.text(this.font, ad, x + 7, y + (h - 8) / 2, yazi, false);
		String d = "‹ " + deger.get() + " ›";
		g.text(this.font, d, x + w - 7 - this.font.width(d), y + (h - 8) / 2, vurgu, false);
		alanlar.add(new Alan(x, y, w, h, tik));
		return y + h;
	}
}
