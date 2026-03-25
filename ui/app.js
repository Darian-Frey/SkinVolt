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
        if (row) row.dataset.lastUpdated = data.timestamp.toString();
        renderOfflineModeBanner(false); // Hide banner on success
    } catch (err) {
        console.error("Fetch failed:", err);
        if (targetCell) targetCell.innerText = "❌ Error (See Logs)";
        renderOfflineModeBanner(true); // Show banner on failure
    }
}

function renderOfflineModeBanner(show) {
    const banner = document.getElementById("offlineBanner");
    if (show) {
        banner.classList.add("active");
    } else {
        banner.classList.remove("active");
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



let _mainChart = null;
let _currentTimeframe = 30;
let _currentSelectedItem = null;

async function changeTimeframe(days) {
    _currentTimeframe = days;
    document.querySelectorAll('.btn-timeframe').forEach(btn => {
        btn.classList.toggle('active', parseInt(btn.getAttribute('onclick').match(/\d+/)[0]) === days);
    });
    updateDashboard(_currentSelectedItem);
}

async function updateDashboard(itemName) {
    if (!itemName && !_currentSelectedItem) {
        if (_inventoryData.length > 0) {
            itemName = _inventoryData[0].market_hash_name;
        } else {
            return;
        }
    }
    
    if (itemName) _currentSelectedItem = itemName;
    else itemName = _currentSelectedItem;

    try {
        const analytics = await window.invoke("get_item_analytics", { marketHashName: itemName });
        let history = await window.invoke("get_item_history_full", { marketHashName: itemName });

        // Clip history to timeframe
        if (history.length > _currentTimeframe) {
            history = history.slice(-_currentTimeframe);
        }

        // Update Text Elements
        document.getElementById("dashVolatility").innerText = analytics.volatility.toFixed(4);
        document.getElementById("dashSMA30").innerText = `$${analytics.sma_30.toFixed(2)}`;
        document.getElementById("dashTrend").innerText = analytics.trend.toUpperCase();
        document.getElementById("marketTrend").innerText = `Target: ${itemName} | Signal: ${analytics.trend.toUpperCase()}`;

        renderPriceChart(history, itemName);
    } catch (err) {
        console.error("Dashboard update failed:", err);
    }
}

function renderPriceChart(history, name) {
    const ctx = document.getElementById('mainChart').getContext('2d');
    const isPro = ['pro', 'elite'].includes(_currentTier);
    
    const labels = history.map(p => new Date(p.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', day: '2-digit', month: '2-digit' }));
    
    const priceData = history.map(p => p.price);
    const smaData = isPro ? history.map(p => p.sma) : [];
    const upperBand = isPro ? history.map(p => p.upper_band) : [];
    const lowerBand = isPro ? history.map(p => p.lower_band) : [];

    if (_mainChart) {
        _mainChart.destroy();
    }

    const datasets = [
        {
            label: 'Price',
            data: priceData,
            borderColor: '#4a7aff',
            backgroundColor: 'rgba(74, 122, 255, 0.1)',
            borderWidth: 3,
            tension: 0.4,
            pointRadius: 1,
            fill: true,
            order: 2
        }
    ];

    if (isPro && history.length > 5) {
        // Simple Moving Average
        datasets.push({
            label: 'SMA (20)',
            data: smaData,
            borderColor: 'rgba(255, 165, 0, 0.6)',
            borderWidth: 2,
            borderDash: [5, 5],
            pointRadius: 0,
            fill: false,
            tension: 0.4,
            order: 1
        });

        // Bollinger Bands (Upper)
        datasets.push({
            label: 'Volatility Band',
            data: upperBand,
            borderColor: 'rgba(255, 255, 255, 0.1)',
            pointRadius: 0,
            fill: '+1', // Fill to lower band
            backgroundColor: 'rgba(74, 122, 255, 0.05)',
            tension: 0.4,
            order: 3
        });

        datasets.push({
            label: 'Lower Band',
            data: lowerBand,
            borderColor: 'rgba(255, 255, 255, 0.1)',
            pointRadius: 0,
            fill: false,
            tension: 0.4,
            order: 3
        });
    }

    _mainChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: labels,
            datasets: datasets
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            interaction: {
                intersect: false,
                mode: 'index',
            },
            plugins: {
                legend: { display: isPro, position: 'bottom', labels: { color: '#888', font: { size: 10 } } },
                tooltip: { backgroundColor: '#1e1e1e', titleColor: '#4a7aff' }
            },
            scales: {
                x: {
                    grid: { display: false },
                    ticks: { color: '#888', maxTicksLimit: 8 }
                },
                y: {
                    grid: { color: 'rgba(255,255,255,0.05)' },
                    ticks: { 
                        color: '#888',
                        callback: (value) => '$' + value.toFixed(2)
                    }
                }
            }
        }
    });
}

// Update the navigation switcher to trigger dashboard update
document.querySelectorAll("#navbar .nav-links li").forEach(link => {
    link.addEventListener("click", () => {
        const viewId = link.getAttribute("data-view");
        if (viewId === "dashboard") {
            updateDashboard();
        }
    });
});

// Update renderInventory to include a "Show Chart" link or click
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
                    <td style="cursor: pointer; color: var(--accent);" onclick="showItemDetails('${safeName}')">
                        ${item.market_hash_name}
                    </td>
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

async function showItemDetails(name) {
    // Switch to dashboard and show this item
    document.querySelector('[data-view="dashboard"]').click();
    updateDashboard(name);
}

async function testTier(newTier) {
    await window.invoke("dev_set_tier", { tier: newTier });
    await updateTierUI(); 
    await renderInventory(); 
    console.log(`Testing Mode: ${newTier.toUpperCase()} Active`);
}

async function initializeSettings() {
    try {
        const darkMode = await window.invoke("get_setting", { key: "dark_mode" });
        if (darkMode === "true") {
            document.documentElement.setAttribute("data-theme", "dark");
            document.getElementById("darkModeToggle").checked = true;
        }
        const currency = await window.invoke("get_setting", { key: "currency" });
        document.getElementById("currencySelect").value = currency;
        const interval = await window.invoke("get_setting", { key: "refresh_interval" });
        document.getElementById("refreshInterval").value = interval;
    } catch (e) {
        console.error("Failed to load settings from DB:", e);
    }
}

// Initialize listeners on boot
setupEventListeners();
updateTierUI();
initializeSettings();
// Initial dashboard load
setTimeout(updateDashboard, 500); 