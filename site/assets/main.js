// Butonul principal propune fișierul potrivit sistemului; fără JavaScript
// duce la secțiunea „Descarcă”. Telefoanele, tabletele și Chromebook-urile
// nu primesc nimic: Cangurul e un program pentru calculator.

export function pickDownload(ua, platform, maxTouchPoints = 0) {
  if (/Android|iPhone|iPad|iPod|CrOS/i.test(ua)) return null;
  // Windows 7 și 8 (NT 6.x): .msi cere Windows 10+, deci arhiva pentru Windows 7.
  if (/Windows NT 6\./.test(ua)) return "win7";
  if (/Win/i.test(platform) || /Windows/i.test(ua)) return "windows";
  if (/Mac/i.test(platform)) return maxTouchPoints > 1 ? null : "mac";
  if (/Linux/i.test(platform) || /X11/.test(ua)) return "linux";
  return null;
}

export function withHash(href, hash) {
  return href.split("#")[0] + (hash || "");
}

function setupLanguages() {
  for (const a of document.querySelectorAll(".langs a")) {
    a.addEventListener("click", () => { a.href = withHash(a.getAttribute("href"), location.hash); });
  }
}

function setupDownload() {
  const button = document.getElementById("download-main");
  if (!button) return;
  const system = pickDownload(navigator.userAgent, navigator.platform || "", navigator.maxTouchPoints || 0);
  if (!system) return;
  const url = JSON.parse(button.dataset.kinds || "{}")[system];
  const label = button.getAttribute(`data-label-${system}`);
  if (!url || !label) return;
  button.href = url;
  button.textContent = label;
}

// Videoclipul: până la click e doar imaginea, fără nicio cerere spre YouTube;
// fără JavaScript linkul duce la YouTube.
export function embedUrl(id) {
  return `https://www.youtube-nocookie.com/embed/${encodeURIComponent(id)}?autoplay=1&rel=0`;
}

function setupVideo() {
  for (const a of document.querySelectorAll("a.video[data-video]")) {
    a.addEventListener("click", (e) => {
      if (e.metaKey || e.ctrlKey || e.shiftKey || e.button !== 0) return;
      e.preventDefault();
      const f = document.createElement("iframe");
      f.className = "video";
      f.src = embedUrl(a.dataset.video);
      f.title = a.dataset.title || "";
      f.allow = "autoplay; encrypted-media; picture-in-picture; fullscreen";
      f.allowFullscreen = true;
      f.referrerPolicy = "strict-origin-when-cross-origin";
      a.replaceWith(f);
      f.focus();
    });
  }
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
  setupLanguages();
  setupReveal();
  setupVideo();
}
