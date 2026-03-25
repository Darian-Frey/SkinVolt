window.invoke = window.__TAURI__.core.invoke;


// Navigation switching
document.querySelectorAll("#navbar .nav-links li").forEach(link => {
    link.addEventListener("click", () => {
        // Remove active class from all nav items 
        document.querySelectorAll("#navbar .nav-links li").forEach(l => l.classList.remove("active"));
        // Add active class to clicked item 
        link.classList.add("active");

        const viewId = link.getAttribute("data-view");

        // Hide all views 
        document.querySelectorAll(".view").forEach(v => {
            v.classList.remove("active");
            v.style.display = "none"; // Explicitly hide to ensure override 
        });

        // Show the selected view 
        const activeView = document.getElementById(viewId + "-view");
        if (activeView) {
            activeView.classList.add("active");
            activeView.style.display = "block"; // Explicitly show 

            // Trigger specific view logic [cite: 16]
            if (viewId === "inventory") {
                renderInventory(); // Refresh the list when clicking the tab [cite: 16]
            }
        }
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

// Add Item Logic
document.getElementById("addItemBtn").addEventListener("click", async () => {
    const name = document.getElementById("itemNameInput").value;
    const qty = parseInt(document.getElementById("itemQtyInput").value);

    if (!name) return alert("Please enter an item name.");

    try {
        // This calls the add_item command in commands.rs
        await window.invoke("add_item", { args: { name, quantity: qty } });

        // Clear inputs and refresh the table
        document.getElementById("itemNameInput").value = "";
        renderInventory();
        console.log(`Added ${qty}x ${name} to local inventory.`);
    } catch (err) {
        console.error("Failed to add item:", err);
        alert(err);
    }
});

async function renderInventory() {
    const tbody = document.getElementById("inventoryBody");
    const currentTier = await window.invoke("get_setting", { key: "tier_level" });

    try {
        const raw = await window.invoke("get_inventory");
        const items = JSON.parse(raw);

        tbody.innerHTML = items.map(item => {
            // Strategic Logic: Lock the button for Free users 
            const isLocked = currentTier === "free";
            const btnClass = isLocked ? "btn-locked" : "btn-refresh-active";
            const btnText = isLocked ? "🔒 1hr Cooldown" : "⚡ Refresh";

            return `
                <tr>
                    <td>${item.market_hash_name}</td>
                    <td>${item.quantity}</td>
                    <td class="price-cell">Pending...</td>
                    <td>
                        <button class="${btnClass}" 
                                ${isLocked ? 'disabled title="Upgrade to Basic for 10min refreshes"' : ''} 
                                onclick="refreshPrice('${item.market_hash_name}')">
                            ${btnText}
                        </button>
                    </td>
                </tr>
            `;
        }).join('');
    } catch (err) {
        console.error("Inventory render failed:", err);
    }
}

async function refreshPrice(name) {
    try {
        const raw = await window.__TAURI__.core.invoke("refresh_steam_data", name);
        const data = JSON.parse(raw);

        console.log("Updated price:", data);

        // TODO: update UI row
    } catch (err) {
        console.error("Steam refresh error:", err);
    }

    window.__TAURI__.core.invoke("get_inventory")
        .then(raw => {
            const items = JSON.parse(raw);
            console.log("Inventory:", items);
        })
        .catch(err => console.error("Inventory error:", err));



    // Initialize UI
    loadTheme();

}

async function updateTierUI() {
    try {
        const tier = await window.invoke("get_setting", { key: "tier_level" });
        const display = document.getElementById("tierDisplay");

        // Reset classes
        display.className = "tier-badge";

        // Apply specific branding [cite: 146]
        switch (tier.toLowerCase()) {
            case 'pro':
                display.classList.add("badge-pro");
                display.innerText = "⚡ PRO";
                break;
            case 'elite':
                display.classList.add("badge-elite");
                display.innerText = "💎 ELITE";
                break;
            case 'basic':
                display.classList.add("badge-basic");
                display.innerText = "✓ BASIC";
                break;
            default:
                display.classList.add("badge-free");
                display.innerText = "FREE";
        }
    } catch (e) {
        console.error("Tier UI update failed:", e);
    }
}

// Call this during your app initialization
updateTierUI();

async function testTier(newTier) {
    await window.invoke("dev_set_tier", { tier: newTier });
    await updateTierUI(); // Refresh the badge [cite: 38]
    await renderInventory(); // Refresh the "Lock" icons
    alert(`Testing Mode: ${newTier.toUpperCase()} Active`);
}

async function initializeSettings() {
    try {
        // 1. Load and Apply Theme
        const darkMode = await window.__TAURI__.core.invoke("get_setting", { key: "dark_mode" });
        if (darkMode === "true") {
            document.documentElement.setAttribute("data-theme", "dark");
            document.getElementById("darkModeToggle").checked = true;
        }

        // 2. Load and Apply Currency
        const currency = await window.__TAURI__.core.invoke("get_setting", { key: "currency" });
        document.getElementById("currencySelect").value = currency;

        // 3. Load Refresh Interval
        const interval = await window.__TAURI__.core.invoke("get_setting", { key: "refresh_interval" });
        document.getElementById("refreshInterval").value = interval;

        // 4. Update Tier Badge (Optional UI Polish)
        const tier = await window.__TAURI__.core.invoke("get_setting", { key: "tier_level" });
        console.log("SkinVolt initialized with tier:", tier);

    } catch (e) {
        console.error("Failed to load settings from DB:", e);
    }
}

// Ensure this runs when the script loads
initializeSettings();