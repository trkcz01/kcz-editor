package com.kcz.client;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import com.mojang.blaze3d.platform.InputConstants;

import net.minecraft.client.KeyMapping;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.PauseScreen;
import net.minecraft.network.chat.Component;
import net.minecraft.resources.Identifier;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.keymapping.v1.KeyMappingHelper;
import net.fabricmc.fabric.api.client.rendering.v1.hud.HudElementRegistry;
import net.fabricmc.fabric.api.client.rendering.v1.hud.VanillaHudElements;
import net.fabricmc.fabric.api.client.screen.v1.ScreenEvents;
import net.fabricmc.fabric.api.client.screen.v1.Screens;

public class KczClient implements ClientModInitializer {
	public static final String ID = "kcz-client";
	public static final Logger LOG = LoggerFactory.getLogger("KCZ Client");

	public static Identifier id(String yol) {
		return Identifier.fromNamespaceAndPath(ID, yol);
	}

	private static final KeyMapping.Category KATEGORI = KeyMapping.Category.register(id("kategori"));
	// Menü ESC ekranındaki KÇZ düğmesiyle açılır; istersen Kontroller'den bir tuş da atanabilir (varsayılan: yok)
	public static final KeyMapping MENU = new KeyMapping("key.kcz-client.menu", InputConstants.UNKNOWN.getValue(), KATEGORI);
	public static final KeyMapping ZOOM = new KeyMapping("key.kcz-client.zoom", InputConstants.KEY_C, KATEGORI);

	private static volatile boolean zoomda = false;
	private static float zoomDegeri = 1f;
	private static long sonZaman = 0L;
	// Tekerlekle ayarlanan ek çarpan: her yeni zoomda ayardaki güçten başlar
	private static float tekerlekCarpani = 1f;
	private static final float EN_AZ_ZOOM = 1.5f;
	private static final float EN_COK_ZOOM = 50f;

	@Override
	public void onInitializeClient() {
		Ayar.yukle();
		KeyMappingHelper.registerKeyMapping(MENU);
		KeyMappingHelper.registerKeyMapping(ZOOM);
		HudElementRegistry.attachElementBefore(VanillaHudElements.CHAT, id("hud"), Hud::ciz);

		// ESC (duraklatma) ekranının sol üstüne KÇZ düğmesi
		ScreenEvents.AFTER_INIT.register((mc, ekran, genislik, yukseklik) -> {
			if (ekran instanceof PauseScreen) {
				Screens.getWidgets(ekran).add(Button.builder(Component.literal("KÇZ Menü"), b -> mc.gui.setScreen(new KczEkrani(ekran)))
						.bounds(6, 6, 78, 20)
						.build());
			}
		});

		ClientTickEvents.END_CLIENT_TICK.register(mc -> {
			while (MENU.consumeClick()) {
				if (mc.gui.screen() == null) {
					mc.gui.setScreen(new KczEkrani(null));
				}
			}
			boolean simdi = Ayar.get().zoom && mc.player != null && mc.gui.screen() == null && ZOOM.isDown();
			if (!simdi) {
				tekerlekCarpani = 1f;
			}
			zoomda = simdi;
		});
		LOG.info("KÇZ Client hazır.");
	}

	public static boolean zoomdaMi() {
		return zoomda;
	}

	/** Şu anki (yumuşatılmış) zoom katı */
	public static float anlikZoom() {
		return Math.max(1f, zoomDegeri);
	}

	private static float hedefZoom() {
		float k = Ayar.get().zoomKat * tekerlekCarpani;
		return Math.max(EN_AZ_ZOOM, Math.min(EN_COK_ZOOM, k));
	}

	/** Tekerlek: yukarı yakınlaştırır, aşağı uzaklaştırır (her adım %25) */
	public static void tekerlek(int adim) {
		float yeni = tekerlekCarpani * (float) Math.pow(1.25, adim);
		float k = Ayar.get().zoomKat;
		tekerlekCarpani = Math.max(EN_AZ_ZOOM / k, Math.min(EN_COK_ZOOM / k, yeni));
	}

	/** Kameranın görüş açısı bu sayıya bölünür (1 = zoom yok). Kare hızından bağımsız yumuşak geçiş. */
	public static float zoomBoleni() {
		long simdi = System.nanoTime();
		float dt = sonZaman == 0L ? 0f : Math.min((simdi - sonZaman) / 1.0e9f, 0.1f);
		sonZaman = simdi;
		float hedef = zoomda ? hedefZoom() : 1f;
		zoomDegeri += (hedef - zoomDegeri) * (1f - (float) Math.exp(-dt * 14f));
		if (Math.abs(hedef - zoomDegeri) < 0.002f) {
			zoomDegeri = hedef;
		}
		return Math.max(1f, zoomDegeri);
	}
}
