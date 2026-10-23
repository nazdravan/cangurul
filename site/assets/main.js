// Butonul principal propune fișierul potrivit sistemului; fără JavaScript
// duce la secțiunea „Descarcă”. Telefoanele, tabletele și Chromebook-urile
// nu primesc nimic: Cangurul e un program pentru calculator.

export function pickDownload(ua, platform, maxTouchPoints = 0) {
  if (/Android|iPhone|iPad|iPod|CrOS/i.test(ua)) return null;
  if (/Win/i.test(platform) || /Windows/i.test(ua)) return "windows";
  if (/Mac/i.test(platform)) return maxTouchPoints > 1 ? null : "mac";
  if (/Linux/i.test(platform) || /X11/.test(ua)) return "linux";
  return null;
}

function setupDownload() {
  const button = document.getElementById("download-main");
  if (!button) return;
  const system = pickDownload(navigator.userAgent, navigator.platform || "", navigator.maxTouchPoints || 0);
  if (!system) return;
  const url = JSON.parse(button.dataset.kinds || "{}")[system];
  const label = button.dataset[`label${system[0].toUpperCase()}${system.slice(1)}`];
  if (!url || !label) return;
  button.href = url;
  button.textContent = label;
}

// Secțiunile apar lin când ajung pe ecran; conținutul e vizibil și fără script.
function setupReveal() {
  const items = document.querySelectorAll("[data-reveal]");
  if (!("IntersectionObserver" in window) || matchMedia("(prefers-reduced-motion: reduce)").matches) return;
  document.documentElement.classList.add("reveal-ready");
  const io = new IntersectionObserver((entries) => {
    for (const e of entries) {
      if (!e.isIntersecting) continue;
      e.target.classList.add("is-in");
      io.unobserve(e.target);
    }
  }, { rootMargin: "0px 0px -10% 0px" });
  items.forEach((el) => io.observe(el));
}

if (typeof document !== "undefined") {
  setupDownload();
  setupReveal();
}
