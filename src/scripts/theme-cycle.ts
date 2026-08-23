/**
 * theme-cycle.ts
 *
 * Time-of-day theming. The site's primary color (and page background) fades
 * through a daily cycle anchored to the real position of the sun for the
 * user's location: night -> sunrise -> noon -> sunset -> night.
 *
 * The cycle is parameterized by `t` in [0, 1):
 *   0.00 = solar midnight, 0.25 = sunrise, 0.50 = solar noon, 0.75 = sunset.
 *
 * PERSONALIZATION: tweak the HSL values in THEME_KEYFRAMES below to make the
 * theme your own. `light`/`lightest` variants are derived automatically.
 */

export type Hsl = [number, number, number]; // h: 0-360, s: 0-100, l: 0-100

export interface ThemeKeyframe {
	/** Position in the daily cycle, [0, 1). */
	t: number;
	/** Name shown in the theme picker. */
	name: string;
	/** Primary accent color. */
	primary: Hsl;
	/** Page background color (kept dark, subtly tinted). */
	bg: Hsl;
	/** Main text color: white-ish by day, dimmer at night. */
	fg: Hsl;
	/** Secondary text color (dates, durations, captions). */
	fgMuted: Hsl;
	/** Card/surface background. */
	cardBg: Hsl;
	/** Borders and separators. */
	border: Hsl;
}

/**
 * Colors for each phase of the day. Hue journeys between neighbors are chosen
 * to stay pleasant: night -> sunrise passes violet/magenta/red, morning passes
 * purple, noon -> sunset passes violet/magenta, dusk collapses back to indigo.
 * Foregrounds brighten towards noon and dim towards the night.
 */
export const THEME_KEYFRAMES: ThemeKeyframe[] = [
	{
		t: 0,
		name: "Night",
		primary: [252, 76, 64],
		bg: [244, 34, 7],
		fg: [238, 26, 92],
		fgMuted: [238, 14, 63],
		cardBg: [242, 23, 11],
		border: [242, 19, 23],
	},
	{
		t: 0.25,
		name: "Sunrise",
		primary: [28, 92, 63],
		bg: [24, 42, 7],
		fg: [30, 34, 96],
		fgMuted: [25, 18, 69],
		cardBg: [22, 25, 11],
		border: [22, 21, 22],
	},
	{
		t: 0.5,
		name: "Noon",
		primary: [195, 88, 48],
		bg: [205, 34, 7],
		fg: [205, 31, 97],
		fgMuted: [205, 15, 66],
		cardBg: [205, 27, 11],
		border: [205, 21, 22],
	},
	{
		t: 0.75,
		name: "Sunset",
		primary: [340, 88, 64],
		bg: [330, 36, 7],
		fg: [332, 29, 96],
		fgMuted: [330, 14, 67],
		cardBg: [330, 24, 11],
		border: [330, 20, 23],
	},
];

const STORAGE_KEYS = {
	mode: "spectre-theme-mode",
	phase: "spectre-theme-phase",
	vars: "spectre-theme-vars",
	coords: "spectre-theme-coords",
} as const;

export type ThemeMode = "auto" | "manual";

/* -------------------------------------------------------------------------- */
/* Color math                                                                  */
/* -------------------------------------------------------------------------- */

/** Shortest-path interpolation between two hues. */
function lerpHue(a: number, b: number, f: number): number {
	const delta = ((b - a + 540) % 360) - 180;
	return (a + delta * f + 360) % 360;
}

function lerpHsl(a: Hsl, b: Hsl, f: number): Hsl {
	return [
		lerpHue(a[0], b[0], f),
		a[1] + (b[1] - a[1]) * f,
		a[2] + (b[2] - a[2]) * f,
	];
}

export interface Palette {
	primary: Hsl;
	bg: Hsl;
	fg: Hsl;
	fgMuted: Hsl;
	cardBg: Hsl;
	border: Hsl;
}

/** Blends the keyframes at cycle position `t`. */
export function paletteAt(t: number): Palette {
	const frames = THEME_KEYFRAMES;
	const wrapped = ((t % 1) + 1) % 1;

	let next = 0;
	while (next < frames.length && frames[next].t <= wrapped) next++;

	const b = frames[next % frames.length];
	const a = frames[(next - 1 + frames.length) % frames.length];

	const span = (b.t - a.t + 1) % 1 || 1;
	const f = Math.min(1, Math.max(0, ((wrapped - a.t + 1) % 1) / span));

	return {
		primary: lerpHsl(a.primary, b.primary, f),
		bg: lerpHsl(a.bg, b.bg, f),
		fg: lerpHsl(a.fg, b.fg, f),
		fgMuted: lerpHsl(a.fgMuted, b.fgMuted, f),
		cardBg: lerpHsl(a.cardBg, b.cardBg, f),
		border: lerpHsl(a.border, b.border, f),
	};
}

/** Display name of the nearest phase (Night, Sunrise, Noon or Sunset). */
export function phaseNameAt(t: number): string {
	const wrapped = ((t % 1) + 1) % 1;
	let best = THEME_KEYFRAMES[0];
	let bestDist = Number.POSITIVE_INFINITY;
	for (const frame of THEME_KEYFRAMES) {
		const dist = Math.min(
			Math.abs(wrapped - frame.t),
			1 - Math.abs(wrapped - frame.t),
		);
		if (dist < bestDist) {
			bestDist = dist;
			best = frame;
		}
	}
	return best.name;
}

export function hslToRgb([h, s, l]: Hsl): [number, number, number] {
	const sn = s / 100;
	const ln = l / 100;
	const c = (1 - Math.abs(2 * ln - 1)) * sn;
	const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
	const m = ln - c / 2;

	let rgb: [number, number, number];
	if (h < 60) rgb = [c, x, 0];
	else if (h < 120) rgb = [x, c, 0];
	else if (h < 180) rgb = [0, c, x];
	else if (h < 240) rgb = [0, x, c];
	else if (h < 300) rgb = [x, 0, c];
	else rgb = [c, 0, x];

	return [
		Math.round((rgb[0] + m) * 255),
		Math.round((rgb[1] + m) * 255),
		Math.round((rgb[2] + m) * 255),
	];
}

export function rgbToHex([r, g, b]: [number, number, number]): string {
	const toHex = (v: number) => v.toString(16).padStart(2, "0");
	return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}

/* -------------------------------------------------------------------------- */
/* Solar math (compact NOAA approximation, ~2 minute accuracy)                 */
/* -------------------------------------------------------------------------- */

export interface SunEvents {
	sunrise: number; // local fractional hours
	solarNoon: number;
	sunset: number;
}

/** Sunrise, solar noon and sunset in local fractional hours for a date/place. */
export function sunEvents(
	date: Date,
	lat: number,
	lon: number,
): SunEvents | null {
	const rad = Math.PI / 180;
	const start = new Date(date.getFullYear(), 0, 0);
	const day = Math.floor((date.getTime() - start.getTime()) / 86400000);

	// Fractional year (radians)
	const gamma = ((2 * Math.PI) / 365) * (day - 0.5);

	const eqTime =
		229.18 *
		(0.000075 +
			0.001868 * Math.cos(gamma) -
			0.032077 * Math.sin(gamma) -
			0.014615 * Math.cos(2 * gamma) -
			0.040849 * Math.sin(2 * gamma)); // minutes

	const decl =
		0.006918 -
		0.399912 * Math.cos(gamma) +
		0.070257 * Math.sin(gamma) -
		0.006758 * Math.cos(2 * gamma) +
		0.000907 * Math.sin(2 * gamma) -
		0.002697 * Math.cos(3 * gamma) +
		0.00148 * Math.sin(3 * gamma); // radians

	const latRad = lat * rad;
	const cosHourAngle =
		Math.cos(90.833 * rad) / (Math.cos(latRad) * Math.cos(decl)) -
		Math.tan(latRad) * Math.tan(decl);

	// Polar day/night: no sunrise or sunset today.
	if (cosHourAngle < -1 || cosHourAngle > 1) return null;

	const hourAngle = Math.acos(cosHourAngle) / rad; // degrees
	const tzOffset = -date.getTimezoneOffset(); // minutes east of UTC

	const solarNoon = (720 - 4 * lon - eqTime + tzOffset) / 60;
	const halfDay = (4 * hourAngle) / 60;

	return {
		sunrise: solarNoon - halfDay,
		solarNoon,
		sunset: solarNoon + halfDay,
	};
}

/** Maps a local time to the cycle position `t` for a given location. */
export function phaseFromTime(now: Date, lat: number, lon: number): number {
	const h = now.getHours() + now.getMinutes() / 60 + now.getSeconds() / 3600;

	const events = sunEvents(now, lat, lon);
	if (!events) return h / 24; // Polar fallback: plain 24h clock cycle.

	const { sunrise, solarNoon, sunset } = events;

	if (h >= sunrise && h < solarNoon) {
		return 0.25 + (0.25 * (h - sunrise)) / (solarNoon - sunrise);
	}
	if (h >= solarNoon && h < sunset) {
		return 0.5 + (0.25 * (h - solarNoon)) / (sunset - solarNoon);
	}

	// Night: spans sunset -> next sunrise, wrapping midnight.
	const nightLength = 24 - (sunset - sunrise);
	const hourIntoNight = (h < sunrise ? h + 24 : h) - sunset;
	return (0.75 + (0.5 * hourIntoNight) / nightLength) % 1;
}

/**
 * Inverse of phaseFromTime: the local wall-clock time (fractional hours) that
 * a cycle position represents, for a given date and location. Used to show
 * real clock times on the theme dial instead of solar-relative ones.
 */
export function wallClockFromPhase(
	t: number,
	now: Date,
	lat: number,
	lon: number,
): number {
	const wrapped = ((t % 1) + 1) % 1;

	const events = sunEvents(now, lat, lon);
	if (!events) return wrapped * 24; // Polar fallback: plain 24h clock cycle.

	const { sunrise, solarNoon, sunset } = events;

	if (wrapped >= 0.25 && wrapped < 0.5) {
		return sunrise + ((wrapped - 0.25) / 0.25) * (solarNoon - sunrise);
	}
	if (wrapped >= 0.5 && wrapped < 0.75) {
		return solarNoon + ((wrapped - 0.5) / 0.25) * (sunset - solarNoon);
	}

	// Night: spans sunset -> next sunrise, wrapping midnight.
	const nightLength = 24 - (sunset - sunrise);
	const f = ((wrapped - 0.75 + 1) % 1) / 0.5;
	return (sunset + f * nightLength) % 24;
}

export interface SolarDisplayTimes {
	sunrise: number; // local fractional hours
	solarNoon: number;
	sunset: number;
	midnight: number;
}

/** Local fractional hours of today's four solar quarter events. */
export function solarDisplayTimes(
	now: Date,
	lat: number,
	lon: number,
): SolarDisplayTimes {
	const events = sunEvents(now, lat, lon);
	if (!events) {
		return { sunrise: 6, solarNoon: 12, sunset: 18, midnight: 0 };
	}
	return {
		sunrise: events.sunrise,
		solarNoon: events.solarNoon,
		sunset: events.sunset,
		midnight: ((events.sunset + events.sunrise + 24) / 2) % 24,
	};
}

/* -------------------------------------------------------------------------- */
/* Location                                                                    */
/* -------------------------------------------------------------------------- */

/** Representative coordinates for common IANA timezones. */
const ZONE_COORDS: Record<string, [number, number]> = {
	"Europe/Madrid": [40.42, -3.7],
	"Europe/London": [51.51, -0.13],
	"Europe/Paris": [48.86, 2.35],
	"Europe/Berlin": [52.52, 13.4],
	"Europe/Rome": [41.9, 12.5],
	"Europe/Amsterdam": [52.37, 4.9],
	"Europe/Lisbon": [38.72, -9.14],
	"Europe/Athens": [37.98, 23.73],
	"Europe/Stockholm": [59.33, 18.07],
	"Europe/Oslo": [59.91, 10.75],
	"Europe/Helsinki": [60.17, 24.94],
	"Europe/Warsaw": [52.23, 21.01],
	"Europe/Prague": [50.08, 14.44],
	"Europe/Vienna": [48.21, 16.37],
	"Europe/Zurich": [47.38, 8.54],
	"Europe/Dublin": [53.35, -6.26],
	"Europe/Moscow": [55.76, 37.62],
	"Europe/Istanbul": [41.01, 28.98],
	"America/New_York": [40.71, -74.01],
	"America/Chicago": [41.88, -87.63],
	"America/Denver": [39.74, -104.99],
	"America/Los_Angeles": [34.05, -118.24],
	"America/Toronto": [43.65, -79.38],
	"America/Vancouver": [49.28, -123.12],
	"America/Mexico_City": [19.43, -99.13],
	"America/Sao_Paulo": [-23.55, -46.63],
	"America/Argentina/Buenos_Aires": [-34.6, -58.38],
	"America/Santiago": [-33.45, -70.67],
	"America/Bogota": [4.71, -74.07],
	"Asia/Tokyo": [35.68, 139.69],
	"Asia/Shanghai": [31.23, 121.47],
	"Asia/Singapore": [1.35, 103.82],
	"Asia/Seoul": [37.57, 126.98],
	"Asia/Dubai": [25.2, 55.27],
	"Asia/Kolkata": [28.61, 77.21],
	"Asia/Bangkok": [13.76, 100.5],
	"Asia/Jakarta": [-6.21, 106.85],
	"Australia/Sydney": [-33.87, 151.21],
	"Australia/Melbourne": [-37.81, 144.96],
	"Australia/Perth": [-31.95, 115.86],
	"Pacific/Auckland": [-36.85, 174.76],
	"Africa/Cairo": [30.04, 31.24],
	"Africa/Johannesburg": [-26.2, 28.05],
	"Africa/Lagos": [6.52, 3.38],
	"Atlantic/Reykjavik": [64.15, -21.94],
};

/** Coarse location from the browser timezone (no permission prompts). */
export function approximateLocation(now = new Date()): [number, number] {
	try {
		const zone = Intl.DateTimeFormat().resolvedOptions().timeZone;
		if (zone && ZONE_COORDS[zone]) return ZONE_COORDS[zone];
	} catch {
		// Fall through to offset-based guess.
	}

	// Rough guess: longitude from UTC offset, temperate latitude.
	const lon = Math.max(
		-180,
		Math.min(180, (-now.getTimezoneOffset() / 60) * 15),
	);
	return [40, lon];
}

/**
 * Precise location, but only if the user has already granted geolocation
 * permission to this site. Never triggers a permission prompt.
 */
async function preciseLocation(): Promise<[number, number] | null> {
	try {
		if (!navigator.geolocation || !navigator.permissions?.query) return null;
		const status = await navigator.permissions.query({
			name: "geolocation" as PermissionName,
		});
		if (status.state !== "granted") return null;

		return await new Promise((resolve) => {
			navigator.geolocation.getCurrentPosition(
				(pos) => resolve([pos.coords.latitude, pos.coords.longitude]),
				() => resolve(null),
				{ timeout: 5000, maximumAge: 86400000 },
			);
		});
	} catch {
		return null;
	}
}

/* -------------------------------------------------------------------------- */
/* Persistence (localStorage may be unavailable; fail soft)                    */
/* -------------------------------------------------------------------------- */

function storageGet(key: string): string | null {
	try {
		return localStorage.getItem(key);
	} catch {
		return null;
	}
}

function storageSet(key: string, value: string): void {
	try {
		localStorage.setItem(key, value);
	} catch {
		// Private browsing etc. — theme just won't persist.
	}
}

export function getThemeMode(): ThemeMode {
	return storageGet(STORAGE_KEYS.mode) === "manual" ? "manual" : "auto";
}

export function setThemeMode(mode: ThemeMode): void {
	storageSet(STORAGE_KEYS.mode, mode);
}

export function getManualPhase(): number {
	const raw = Number.parseFloat(storageGet(STORAGE_KEYS.phase) ?? "");
	return Number.isFinite(raw) ? ((raw % 1) + 1) % 1 : 0.5;
}

export function setManualPhase(t: number): void {
	storageSet(STORAGE_KEYS.phase, String(((t % 1) + 1) % 1));
}

/* -------------------------------------------------------------------------- */
/* Applying the theme                                                          */
/* -------------------------------------------------------------------------- */

const clamp = (v: number, min: number, max: number) =>
	Math.min(max, Math.max(min, v));

/** Computes and applies all CSS variables for cycle position `t`. */
export function applyPhase(t: number): void {
	const palette = paletteAt(t);
	const { primary } = palette;

	const light: Hsl = [
		primary[0],
		clamp(primary[1] + 3, 0, 100),
		clamp(primary[2] + 8, 0, 80),
	];
	const lightest: Hsl = [
		primary[0],
		clamp(primary[1], 0, 100),
		clamp(primary[2] + 18, 0, 88),
	];

	const primaryRgb = hslToRgb(primary);
	const heading: Hsl = [
		primary[0],
		clamp(primary[1] - 30, 0, 100),
		clamp(primary[2] + 14, 64, 80),
	];
	const gradientStart: Hsl = [
		palette.bg[0],
		clamp(palette.bg[1] + 8, 0, 100),
		clamp(palette.bg[2] + 4, 0, 18),
	];
	const gradientEnd: Hsl = [
		lerpHue(palette.bg[0], primary[0], 0.16),
		clamp(palette.bg[1] + 12, 0, 100),
		clamp(palette.bg[2] + 2, 0, 15),
	];

	const vars: Record<string, string> = {
		"--primary": rgbToHex(primaryRgb),
		"--primary-rgb": primaryRgb.join(", "),
		"--primary-light": rgbToHex(hslToRgb(light)),
		"--primary-lightest": rgbToHex(hslToRgb(lightest)),
		"--heading": rgbToHex(hslToRgb(heading)),
		"--on-primary": palette.primary[2] > 58 ? "#0b1020" : "#ffffff",
		"--bg": rgbToHex(hslToRgb(palette.bg)),
		"--bg-start": rgbToHex(hslToRgb(gradientStart)),
		"--bg-end": rgbToHex(hslToRgb(gradientEnd)),
		"--daylight-glow": `rgb(${primaryRgb.join(" ")} / 11%)`,
		"--daylight-glow-secondary": `rgb(${primaryRgb.join(" ")} / 6%)`,
		"--fg": rgbToHex(hslToRgb(palette.fg)),
		"--fg-muted": rgbToHex(hslToRgb(palette.fgMuted)),
		"--card-bg": rgbToHex(hslToRgb(palette.cardBg)),
		"--surface-raised": `color-mix(in srgb, ${rgbToHex(hslToRgb(palette.cardBg))} 97%, ${rgbToHex(primaryRgb)})`,
		"--surface-interactive": `color-mix(in srgb, ${rgbToHex(hslToRgb(palette.cardBg))} 91%, ${rgbToHex(primaryRgb)})`,
		"--primary-subtle": `rgb(${primaryRgb.join(" ")} / 10%)`,
		"--border": rgbToHex(hslToRgb(palette.border)),
		"--focus-ring": `rgb(${primaryRgb.join(" ")} / 55%)`,
		"--shadow-soft": "0 0.5rem 1.5rem rgb(0 0 0 / 8%)",
		"--shadow-hover": "0 0.75rem 2rem rgb(0 0 0 / 12%)",
		"--interaction-glow": `0 0 1rem rgb(${primaryRgb.join(" ")} / 35%)`,
	};

	const root = document.documentElement;
	for (const name in vars) {
		root.style.setProperty(name, vars[name]);
	}

	// Keep the browser UI (tab bar etc.) in sync.
	document
		.querySelector('meta[name="theme-color"]')
		?.setAttribute("content", vars["--primary"]);

	// Cache for the no-flash bootstrap on the next page load.
	storageSet(STORAGE_KEYS.vars, JSON.stringify(vars));

	// Notify listeners (the canvas background reads its glow color from this).
	window.dispatchEvent(
		new CustomEvent("themechange", { detail: { rgb: primaryRgb, t } }),
	);
}

/**
 * Resolves the best available coordinates: cached first, then precise
 * geolocation (only if permission was already granted), then a
 * timezone-based guess. The result is cached in localStorage.
 */
export async function resolveLocation(): Promise<[number, number]> {
	const cached = storageGet(STORAGE_KEYS.coords);
	if (cached) {
		try {
			const parsed = JSON.parse(cached);
			if (
				Array.isArray(parsed) &&
				parsed.length === 2 &&
				parsed.every((v) => Number.isFinite(v))
			) {
				return parsed as [number, number];
			}
		} catch {
			// Corrupted cache — fall through and re-resolve.
		}
	}

	const coords = (await preciseLocation()) ?? approximateLocation();
	storageSet(STORAGE_KEYS.coords, JSON.stringify(coords));
	return coords;
}

/** Resolves the phase for right now in `auto` mode. */
export async function currentAutoPhase(): Promise<number> {
	const now = new Date();
	const [lat, lon] = await resolveLocation();
	return phaseFromTime(now, lat, lon);
}

/**
 * Applies the theme for the current mode and keeps `auto` mode updated.
 * Safe to call on every page; runs until the document is unloaded.
 */
export async function initThemeCycle(): Promise<void> {
	const apply = async () => {
		if (getThemeMode() === "manual") {
			applyPhase(getManualPhase());
		} else {
			applyPhase(await currentAutoPhase());
		}
	};

	await apply();

	// Re-check the sun position periodically and when the tab regains focus.
	setInterval(apply, 60 * 1000);
	document.addEventListener("visibilitychange", () => {
		if (document.visibilityState === "visible") apply();
	});
}
