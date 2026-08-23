const favicon = document.querySelector<HTMLLinkElement>("#site-favicon");

function updateFavicon() {
	if (!favicon) {
		return;
	}

	const styles = getComputedStyle(document.documentElement);
	const primary = styles.getPropertyValue("--primary").trim();
	const background = styles.getPropertyValue("--bg").trim();
	if (!primary || !background) {
		return;
	}

	const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="${primary}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><ellipse cx="12" cy="12" rx="10" ry="3.5" transform="rotate(-25 12 12)"/><circle cx="12" cy="12" r="4" fill="${background}"/><circle cx="12" cy="12" r="4"/></svg>`;
	favicon.href = `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

updateFavicon();
window.addEventListener("themechange", updateFavicon);
