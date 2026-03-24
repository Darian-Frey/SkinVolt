// Navigation switching
document.querySelectorAll("#navbar .nav-links li").forEach(link => {
    link.addEventListener("click", () => {
        document.querySelectorAll("#navbar .nav-links li").forEach(l => l.classList.remove("active"));
        link.classList.add("active");

        const view = link.getAttribute("data-view");
        document.querySelectorAll(".view").forEach(v => v.classList.remove("active"));
        document.getElementById(view + "-view").classList.add("active");
    });
});

// Theme loader
async function loadTheme() {
    const { invoke } = window.__TAURI__.tauri;

    try {
        const darkMode = await invoke("load_cached_setting", { key: "dark_mode" });
        if (darkMode === "true") {
            document.documentElement.setAttribute("data-theme", "dark");
            document.getElementById("darkModeToggle").checked = true;
        }
    } catch (e) {
        console.error("Theme load failed:", e);
    }
}

// Save settings
document.getElementById("saveSettings").addEventListener("click", async () => {
    const { invoke } = window.__TAURI__.tauri;

    const refresh = document.getElementById("refreshInterval").value;
    const currency = document.getElementById("currencySelect").value;
    const dark = document.getElementById("darkModeToggle").checked;

    await invoke("set_refresh_interval", { seconds: Number(refresh) });
    await invoke("set_currency_preference", { currency });
    await invoke("toggle_dark_mode", { enabled: dark });

    if (dark) {
        document.documentElement.setAttribute("data-theme", "dark");
    } else {
        document.documentElement.removeAttribute("data-theme");
    }

    alert("Settings saved");
});

window.__TAURI__.core.invoke("get_inventory")
    .then(raw => {
        const items = JSON.parse(raw);
        console.log("Inventory:", items);
    })
    .catch(err => console.error("Inventory error:", err));



// Initialize UI
loadTheme();
