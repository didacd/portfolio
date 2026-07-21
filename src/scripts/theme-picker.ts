/**
 * theme-picker.ts
 *
 * Wires up the theme picker in the navbar: a button that opens a blurred
 * popover with an interactive earth/sun/moon dial. The sun and moon travel
 * along an arc in the background (bright above the horizon, faint below),
 * while grabbing and spinning the earth scrubs through the day, fading the
 * theme live. The user can pin a fixed position (manual mode) or follow the
 * real sun (auto mode).
 */

import {
	applyPhase,
	approximateLocation,
	currentAutoPhase,
	getManualPhase,
	getThemeMode,
	initThemeCycle,
	phaseFromTime,
	phaseNameAt,
	resolveLocation,
	setManualPhase,
	setThemeMode,
	solarDisplayTimes,
	wallClockFromPhase,
} from "./theme-cycle";

const toggle = document.querySelector<HTMLButtonElement>("#theme-toggle");
const popover = document.querySelector<HTMLDivElement>("#theme-popover");
const dial = document.querySelector<SVGSVGElement>("#theme-dial");
const sun = document.querySelector<SVGGElement>("#dial-sun");
const moon = document.querySelector<SVGGElement>("#dial-moon");
const land = document.querySelector<SVGGElement>("#dial-land");
const terminator =
	document.querySelector<SVGLinearGradientElement>("#dial-terminator");
const label = document.querySelector<HTMLSpanElement>("#theme-phase-label");
const autoCheck = document.querySelector<HTMLInputElement>("#theme-auto-check");
const presets = document.querySelectorAll<HTMLButtonElement>(".theme-preset");
const hourMarkers = {
	sunrise: document.querySelector<SVGTextElement>("#dial-hour-sunrise"),
	noon: document.querySelector<SVGTextElement>("#dial-hour-noon"),
	sunset: document.querySelector<SVGTextElement>("#dial-hour-sunset"),
	midnight: document.querySelector<SVGTextElement>("#dial-hour-midnight"),
};

if (
	toggle &&
	popover &&
	dial &&
	sun &&
	moon &&
	land &&
	terminator &&
	label &&
	autoCheck
) {
	// Dial geometry (must match the SVG in Navbar.astro)
	const CX = 100;
	const CY = 110;
	const ORBIT_RADIUS = 56;
	const EARTH_RADIUS = 28;

	let currentPhase = 0.5;
	let dragging = false;
	let grabAngle = 0;
	let grabPhase = 0.5;
	let coords: [number, number] = approximateLocation();

	/* ---------------------------------- UI ---------------------------------- */

	/** Formats fractional hours as a wall-clock time, e.g. 6.58 -> "6:35". */
	const formatHours = (hours: number): string => {
		const total = Math.round(hours * 60) % 1440;
		const h = Math.floor(total / 60);
		const m = String(total % 60).padStart(2, "0");
		return `${h}:${m}`;
	};

	/** Sets the ring markers to today's real solar event times. */
	const updateHourMarkers = () => {
		const times = solarDisplayTimes(new Date(), coords[0], coords[1]);
		if (hourMarkers.sunrise)
			hourMarkers.sunrise.textContent = formatHours(times.sunrise);
		if (hourMarkers.noon)
			hourMarkers.noon.textContent = formatHours(times.solarNoon);
		if (hourMarkers.sunset)
			hourMarkers.sunset.textContent = formatHours(times.sunset);
		if (hourMarkers.midnight)
			hourMarkers.midnight.textContent = formatHours(times.midnight);
	};

	const orbitPosition = (angleDeg: number) => {
		const angle = (angleDeg * Math.PI) / 180;
		return {
			x: CX + ORBIT_RADIUS * Math.cos(angle),
			y: CY + ORBIT_RADIUS * Math.sin(angle),
			angle,
		};
	};

	/** Bright above the horizon, a soft glow at it, faint below — like the real sky. */
	const horizonOpacity = (angleDeg: number) => {
		const altitude = -Math.sin((angleDeg * Math.PI) / 180);
		if (altitude < 0) return 0.12;
		return Math.min(1, 0.35 + altitude * 2);
	};

	const renderDial = (t: number) => {
		// Sun: left horizon at sunrise (t=0.25), top at noon, right at sunset.
		const sunAngle = 180 + (t - 0.25) * 360;
		const moonAngle = sunAngle + 180;

		const sunPos = orbitPosition(sunAngle);
		sun.setAttribute(
			"transform",
			`translate(${sunPos.x.toFixed(2)} ${sunPos.y.toFixed(2)})`,
		);
		sun.setAttribute("opacity", horizonOpacity(sunAngle).toFixed(2));

		const moonPos = orbitPosition(moonAngle);
		moon.setAttribute(
			"transform",
			`translate(${moonPos.x.toFixed(2)} ${moonPos.y.toFixed(2)})`,
		);
		moon.setAttribute("opacity", horizonOpacity(moonAngle).toFixed(2));

		// Only the earth rotates: one full spin per day.
		land.setAttribute("transform", `rotate(${((t * 360) % 360).toFixed(2)})`);

		// The terminator always points towards the sun.
		terminator.setAttribute(
			"x1",
			(CX + Math.cos(sunPos.angle) * EARTH_RADIUS).toFixed(2),
		);
		terminator.setAttribute(
			"y1",
			(CY + Math.sin(sunPos.angle) * EARTH_RADIUS).toFixed(2),
		);
		terminator.setAttribute(
			"x2",
			(CX - Math.cos(sunPos.angle) * EARTH_RADIUS).toFixed(2),
		);
		terminator.setAttribute(
			"y2",
			(CY - Math.sin(sunPos.angle) * EARTH_RADIUS).toFixed(2),
		);

		dial.setAttribute("aria-valuenow", String(Math.round(t * 100)));
	};

	const render = (t: number) => {
		currentPhase = t;
		renderDial(t);
		// The dial geometry is solar (sun overhead at solar noon), but the
		// label shows the equivalent local wall-clock time.
		const hours = wallClockFromPhase(t, new Date(), coords[0], coords[1]);
		const text = `${formatHours(hours)} · ${phaseNameAt(t)}`;
		label.textContent = text;
		dial.setAttribute("aria-valuetext", text);
	};

	const pinPhase = (t: number) => {
		setThemeMode("manual");
		setManualPhase(t);
		autoCheck.checked = false;
		applyPhase(t);
		render(t);
	};

	const followSun = async () => {
		setThemeMode("auto");
		autoCheck.checked = true;
		applyPhase(await currentAutoPhase());
	};

	/* --------------------------------- Dial --------------------------------- */

	/** Pointer angle relative to the earth's center, in degrees. */
	const pointerAngle = (e: PointerEvent): number => {
		const rect = dial.getBoundingClientRect();
		const x = ((e.clientX - rect.left) / rect.width) * 200 - CX;
		const y = ((e.clientY - rect.top) / rect.height) * 200 - CY;
		return (Math.atan2(y, x) * 180) / Math.PI;
	};

	dial.addEventListener("pointerdown", (e) => {
		e.preventDefault();
		dragging = true;
		grabAngle = pointerAngle(e);
		grabPhase = currentPhase;
		dial.setPointerCapture(e.pointerId);
	});

	dial.addEventListener("pointermove", (e) => {
		if (!dragging) return;
		// Spin the earth: the grabbed point follows the pointer.
		const delta = pointerAngle(e) - grabAngle;
		const wrapped = ((delta + 540) % 360) - 180;
		pinPhase((grabPhase + wrapped / 360 + 1) % 1);
	});

	const stopDragging = () => {
		dragging = false;
	};
	dial.addEventListener("pointerup", stopDragging);
	dial.addEventListener("pointercancel", stopDragging);

	// Keyboard support: left/right arrows step the dial by 15 minutes.
	dial.addEventListener("keydown", (e) => {
		if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
		e.preventDefault();
		const step = (e.key === "ArrowRight" ? 1 : -1) / 96;
		pinPhase((currentPhase + step + 1) % 1);
	});

	/* -------------------------------- Presets ------------------------------- */

	for (const preset of presets) {
		preset.addEventListener("click", () => {
			const t = Number.parseFloat(preset.dataset.phase ?? "0.5");
			pinPhase(t);
		});
	}

	autoCheck.addEventListener("change", () => {
		if (autoCheck.checked) {
			followSun();
		} else {
			pinPhase(currentPhase);
		}
	});

	/* -------------------------------- Popover ------------------------------- */

	const closePopover = () => popover.classList.remove("active");

	toggle.addEventListener("click", (e) => {
		e.stopPropagation();
		popover.classList.toggle("active");
	});

	document.addEventListener("click", (e) => {
		if (
			!popover.contains(e.target as Node) &&
			!toggle.contains(e.target as Node)
		) {
			closePopover();
		}
	});

	document.addEventListener("keydown", (e) => {
		if (e.key === "Escape") closePopover();
	});

	/* --------------------------------- Init --------------------------------- */

	// Keep the dial in sync with theme changes (e.g. the sun moving in auto).
	window.addEventListener("themechange", (e) => {
		const t = (e as CustomEvent).detail?.t;
		if (typeof t === "number" && !dragging) render(t);
	});

	// Initial dial position: best guess synchronously, corrected by the first
	// themechange event once the precise phase is resolved.
	if (getThemeMode() === "manual") {
		render(getManualPhase());
		autoCheck.checked = false;
	} else {
		render(phaseFromTime(new Date(), coords[0], coords[1]));
	}
	updateHourMarkers();

	initThemeCycle();

	// Upgrade to the best available coordinates, then refresh the markers.
	resolveLocation().then((resolved) => {
		coords = resolved;
		updateHourMarkers();
		if (!dragging) render(currentPhase);
	});
}
