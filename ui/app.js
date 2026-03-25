window.invoke = window.__TAURI__.core.invoke;
const { listen } = window.__TAURI__.event;

// Module-level state for cooldown ticker
let _inventoryData = [];
let _currentTier = 'free';

function _cooldownSecs(tier) {
    if (tier === 'free')  return 3600;
    if (tier === 'basic') return 600;
    if (tier === 'pro')   return 60;
    return 0; // elite: no cooldown
}

function _remainingSecs(lastUpdated, tier) {
    const limit = _cooldownSecs(tier);
    if (!lastUpdated || limit === 0) return 0;
    const now = Math.floor(Date.now() / 1000);
    return Math.max(0, limit - (now - lastUpdated));
}

function _fmtCountdown(secs) {
    const m = Math.floor(secs / 60).toString().padStart(2, '0');
    const s = (secs % 60).toString().padStart(2, '0');
    return `${m}:${s}`;
}

// Updates button text/state every second without re-rendering the table
function _tickCooldowns() {
    document.querySelectorAll('#inventoryBody tr[data-item]').forEach(row => {
        const lastUpdated = parseInt(row.dataset.lastUpdated || '0', 10);
        const btn = row.querySelector('.btn-refresh');
        if (!btn) return;
        const remaining = _remainingSecs(lastUpdated, _currentTier);
        if (remaining > 0) {
            btn.disabled = true;
            btn.textContent = `⏳ ${_fmtCountdown(remaining)}`;
            btn.className = 'btn-locked btn-refresh';
        } else {
            btn.disabled = false;
            btn.textContent = '⚡ Refresh';
            btn.className = 'btn-refresh-active btn-refresh';
        }
    });
}

setInterval(_tickCooldowns, 1000);

async function setupEventListeners() {
    await listen('inventory-updated', () => {
        console.log("⚡ [SkinVolt] Background sync complete. Refreshing UI...");
        renderInventory();
    });
}

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

    try {
        _currentTier = await window.invoke("get_setting", { key: "tier_level" });
        _inventoryData = await window.invoke("get_inventory_full");

        tbody.innerHTML = _inventoryData.map(item => {
            const priceText = item.price != null ? `$${item.price.toFixed(2)}` : 'Pending...';
            const lastUpdated = item.last_updated || 0;
            const remaining = _remainingSecs(lastUpdated, _currentTier);
            const btnDisabled = remaining > 0 ? 'disabled' : '';
            const btnClass = remaining > 0 ? 'btn-locked btn-refresh' : 'btn-refresh-active btn-refresh';
            const btnText = remaining > 0 ? `⏳ ${_fmtCountdown(remaining)}` : '⚡ Refresh';
            const safeName = item.market_hash_name.replace(/'/g, "\\'");

            return `
                <tr data-item="${item.market_hash_name}" data-last-updated="${lastUpdated}">
                    <td>${item.market_hash_name}</td>
                    <td>${item.quantity}</td>
                    <td class="price-cell">${priceText}</td>
                    <td>
                        <button class="${btnClass} btn-refresh" ${btnDisabled}
                                onclick="refreshPrice('${safeName}')">
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
    const row = document.querySelector(`#inventoryBody tr[data-item="${CSS.escape(name)}"]`);
    const targetCell = row ? row.cells[2] : null;

    if (targetCell) targetCell.innerText = "⏳ Fetching...";

    try {
        const data = await window.invoke("refresh_steam_data", { args: { item_name: name } });
        if (targetCell) targetCell.innerText = `$${data.price.toFixed(2)}`;
        // Stamp the row with the fresh timestamp so _tickCooldowns starts counting down immediately
        if (row) row.dataset.lastUpdated = data.timestamp.toString();
    } catch (err) {
        console.error("Fetch failed:", err);
        if (targetCell) targetCell.innerText = "❌ Error (See Logs)";
    }
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

// Initialize listeners on boot
setupEventListeners();
// Call this during your app initialization
updateTierUI();
// Ensure this runs when the script loads
initializeSettings();