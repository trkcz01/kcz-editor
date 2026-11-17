package com.kcz.client;

import java.util.ArrayList;
import java.util.List;

import net.minecraft.client.DeltaTracker;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.client.multiplayer.PlayerInfo;

/** Ekran köşesindeki FPS / ping kutuları ve tuş göstergesi. */
public final class Hud {
	private Hud() {
	}

	// FPS'i kendimiz sayıyoruz: her çizim bir kare
	private static int kare = 0;
	private static int fps = 0;
	private static long saniyeBasi = 0L;

	public static void ciz(GuiGraphicsExtractor g, DeltaTracker delta) {
		long simdi = System.nanoTime();
		kare++;
		if (saniyeBasi == 0L) {
			saniyeBasi = simdi;
		} else if (simdi - saniyeBasi >= 1_000_000_000L) {
			fps = Math.round(kare * 1.0e9f / (simdi - saniyeBasi));
			kare = 0;
			saniyeBasi = simdi;
		}

		Minecraft mc = Minecraft.getInstance();
		if (mc.player == null) return;
		Ayar a = Ayar.get();
		Font font = mc.font;

		int zemin = Ayar.renk(a.tema.zemin2, 0xB0, 0xB00D0D10);
		int vurgu = Ayar.renk(a.tema.vurgu, 0xFF, 0xFFA1A1AA);
		int yazi = 0xFFF1F5F9;
		int soluk = 0xFFB8BEC6;

		List<String[]> satirlar = new ArrayList<>();
		if (a.fps) {
			satirlar.add(new String[] {"FPS", String.valueOf(fps)});
		}
		if (a.ping) {
			int p = ping(mc);
			if (p >= 0) satirlar.add(new String[] {"Ping", p + " ms"});
		}

		int ekranGenislik = mc.getWindow().getGuiScaledWidth();
		boolean sag = "sag-ust".equals(a.konum);
		int y = 4;
		for (String[] s : satirlar) {
			int w = font.width(s[0]) + font.width(s[1]) + 14;
			int x = sag ? ekranGenislik - w - 4 : 4;
			g.fill(x, y, x + w, y + 13, zemin);
			g.fill(x, y, x + 2, y + 13, vurgu);
			g.text(font, s[0], x + 6, y + 3, soluk, false);
			g.text(font, s[1], x + 10 + font.width(s[0]), y + 3, yazi, false);
			y += 15;
		}

		if (a.tuslar) {
			int boy = 15, ara = 2;
			int genis = boy * 3 + ara * 2;
			int x = sag ? ekranGenislik - genis - 4 : 4;
			y += 2;
			tus(g, font, "W", mc.options.keyUp, x + boy + ara, y, boy, boy, zemin, vurgu, yazi);
			y += boy + ara;
			tus(g, font, "A", mc.options.keyLeft, x, y, boy, boy, zemin, vurgu, yazi);
			tus(g, font, "S", mc.options.keyDown, x + boy + ara, y, boy, boy, zemin, vurgu, yazi);
			tus(g, font, "D", mc.options.keyRight, x + (boy + ara) * 2, y, boy, boy, zemin, vurgu, yazi);
			y += boy + ara;
			int yarim = (genis - ara) / 2;
			tus(g, font, "LMB", mc.options.keyAttack, x, y, yarim, boy, zemin, vurgu, yazi);
			tus(g, font, "RMB", mc.options.keyUse, x + yarim + ara, y, genis - yarim - ara, boy, zemin, vurgu, yazi);
			y += boy + ara;
			tus(g, font, "—", mc.options.keyJump, x, y, genis, 9, zemin, vurgu, yazi);
		}
	}

	private static void tus(GuiGraphicsExtractor g, Font font, String ad, KeyMapping k, int x, int y, int w, int h,
			int zemin, int vurgu, int yazi) {
		boolean basili = k.isDown();
		g.fill(x, y, x + w, y + h, basili ? vurgu : zemin);
		int renk = basili ? Ayar.ustYazi(vurgu) : yazi;
		g.text(font, ad, x + (w - font.width(ad)) / 2, y + (h - 8) / 2, renk, false);
	}

	private static int ping(Minecraft mc) {
		try {
			ClientPacketListener baglanti = mc.getConnection();
			if (baglanti == null || mc.isLocalServer()) return -1;
			PlayerInfo bilgi = baglanti.getPlayerInfo(mc.player.getUUID());
			return bilgi == null ? -1 : bilgi.getLatency();
		} catch (Throwable t) {
			return -1;
		}
	}
}
