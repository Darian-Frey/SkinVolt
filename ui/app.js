window.invoke = window.__TAURI__.core.invoke;
const { listen } = window.__TAURI__.event;

// Module-level state for cooldown ticker
let _inventoryData = [];
let _currentTier = 'free';

function _cooldownSecs(tier) {
    if (tier === 'free') return 3600;
    if (tier === 'basic') return 600;
    if (tier === 'pro') return 60;
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
    document.querySelectorAll('#inventoryBody .item-card[data-item]').forEach(card => {
        const lastUpdated = parseInt(card.dataset.lastUpdated || '0', 10);
        const btn = card.querySelector('.btn-refresh');
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

function setupNavigation() {
    document.querySelectorAll("#navbar .nav-links li").forEach(link => {
        link.addEventListener("click", () => {
            const viewId = link.getAttribute("data-view");

            document.querySelectorAll("#navbar .nav-links li").forEach(l => l.classList.remove("active"));
            link.classList.add("active");
            document.querySelectorAll(".view").forEach(v => { v.style.display = "none"; v.classList.remove("active"); });

            const activeView = document.getElementById(viewId + "-view");
            if (activeView) {
                activeView.style.display = "block";
                activeView.classList.add("active");
                if (viewId === "inventory") renderInventory();
                if (viewId === "dashboard") updateDashboard();
            }
        });
    });
}

async function setupEventListeners() {
    try {
        await listen('inventory-updated', () => {
            console.log("⚡ [SkinVolt] Background sync complete. Refreshing UI...");
            renderInventory();
        });
    } catch (e) {
        console.error("Failed to set up inventory-updated listener:", e);
    }
}

// Navigation switching logic removed here and consolidated below

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

        const cardPromises = _inventoryData.map(async item => {
            const metadata = await window.invoke("get_item_metadata", { marketHashName: item.market_hash_name });
            return renderItemCard(item, metadata);
        });

        const cards = await Promise.all(cardPromises);
        tbody.innerHTML = cards.join('');
    } catch (err) {
        console.error("Inventory render failed:", err);
    }
}

function renderItemCard(item, metadata) {
    const priceText = item.price != null ? `$${item.price.toFixed(2)}` : 'Pending...';
    const lastUpdated = item.last_updated || 0;
    const remaining = _remainingSecs(lastUpdated, _currentTier);
    const btnDisabled = remaining > 0 ? 'disabled' : '';
    const btnClass = remaining > 0 ? 'btn-locked btn-refresh' : 'btn-refresh-active btn-refresh';
    const btnText = remaining > 0 ? `⏳ ${_fmtCountdown(remaining)}` : '⚡ Refresh';
    const safeName = item.market_hash_name.replace(/'/g, "\\'");

    const rarityClass = _getRarityClass(metadata?.rarity);
    const iconUrl = metadata?.icon_url || 'branding/logo_placeholder.png';
    const collection = metadata?.collection || 'Standard Steam Item';

    return `
        <div class="item-card ${rarityClass}" data-item="${item.market_hash_name}" data-last-updated="${lastUpdated}">
            <div class="icon-container" onclick="showItemDetails('${safeName}')" style="cursor: pointer;">
                <img src="${iconUrl}" alt="${item.market_hash_name}" onerror="this.src='branding/logo_placeholder.png'">
            </div>
            <div class="item-name" title="${item.market_hash_name}">${item.market_hash_name}</div>
            <div class="item-collection">${collection}</div>
            <div class="card-footer">
                <div class="price">${priceText}</div>
                <div class="qty">x${item.quantity}</div>
            </div>
            <div style="margin-top: 12px;">
                <button class="${btnClass} btn-refresh" style="width: 100%; border-radius: 8px;" ${btnDisabled}
                        onclick="refreshPrice('${safeName}')">
                    ${btnText}
                </button>
            </div>
        </div>
    `;
}

function _getRarityClass(rarity) {
    if (!rarity) return '';
    const r = rarity.toLowerCase();
    if (r.includes('covert') || r.includes('extraordinary')) return 'rarity-covert';
    if (r.includes('classified')) return 'rarity-classified';
    if (r.includes('restricted')) return 'rarity-restricted';
    if (r.includes('mil-spec')) return 'rarity-mil-spec';
    if (r.includes('industrial')) return 'rarity-industrial';
    if (r.includes('consumer')) return 'rarity-consumer';
    if (r.includes('gold') || r.includes('contraband')) return 'rarity-gold';
    return '';
}

async function refreshPrice(name) {
    const card = document.querySelector(`.item-card[data-item="${CSS.escape(name)}"]`);
    const priceEl = card ? card.querySelector('.price') : null;

    if (priceEl) priceEl.innerText = "⏳...";

    try {
        const data = await window.invoke("refresh_steam_data", { args: { item_name: name } });
        if (priceEl) priceEl.innerText = `$${data.price.toFixed(2)}`;
        if (card) {
            card.dataset.lastUpdated = data.timestamp.toString();
            // Re-render card to update metadata if it was missing 
            const metadata = await window.invoke("get_item_metadata", { marketHashName: name });
            const newHtml = renderItemCard({ market_hash_name: name, price: data.price, quantity: parseInt(card.querySelector('.qty').innerText.replace('x', '')), last_updated: data.timestamp }, metadata);
            card.outerHTML = newHtml;
        }
        renderOfflineModeBanner(false);
    } catch (err) {
        console.error("Fetch failed:", err);
        if (priceEl) priceEl.innerText = "❌ Error";
        renderOfflineModeBanner(true);
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
        const navDisplay = document.getElementById("navTier");

        // Reset classes
        if (display) display.className = "tier-badge";
        if (navDisplay) navDisplay.className = "tier-badge";

        const setDisplay = (text, cls) => {
            if (display) { display.innerText = text; display.classList.add(cls); }
            if (navDisplay) { navDisplay.innerText = text; navDisplay.classList.add(cls); }
        };

        // Apply specific branding [cite: 146]
        switch (tier.toLowerCase()) {
            case 'pro':
                setDisplay("⚡ PRO", "badge-pro");
                break;
            case 'elite':
                setDisplay("💎 ELITE", "badge-elite");
                break;
            case 'basic':
                setDisplay("✓ BASIC", "badge-basic");
                break;
            default:
                setDisplay("FREE", "badge-free");
        }
    } catch (e) {
        console.error("Tier UI update failed:", e);
    }
}



let _mainChart = null;
let _currentTimeframe = 30;
let _currentSelectedItem = null;

async function changeTimeframe(days) {
    const isBasic = ['free', 'basic'].includes(_currentTier);
    if (isBasic && days > 7) {
        alert("30D and 90D timeframes are for Pro & Elite tiers. Upgrade to unlock full historical depth!");
        return;
    }
    _currentTimeframe = days;
    document.querySelectorAll('.btn-timeframe').forEach(btn => {
        btn.classList.toggle('active', parseInt(btn.getAttribute('onclick').match(/\d+/)[0]) === days);
    });
    updateDashboard();
}

async function updateTopMovers() {
    const listEl = document.getElementById("topMoversList");
    const sortVal = document.getElementById("moverSort") ? document.getElementById("moverSort").value : "change";
    const isElite = _currentTier === 'elite';

    // Gating check for sorting
    const effectiveSort = (sortVal === "volatility" && !isElite) ? "change" : sortVal;

    try {
        const movers = await window.invoke("get_top_movers", { limit: 5, sortBy: effectiveSort });
        listEl.innerHTML = movers.map(m => `
            <div class="mover-item">
                <span class="mover-name">${m.market_hash_name}</span>
                <span class="mover-price monospace">$${m.current_price.toFixed(2)}</span>
                <span class="mover-change monospace ${m.change_pct >= 0 ? "up" : "down"}">
                    ${m.change_pct >= 0 ? "▲" : "▼"}${Math.abs(m.change_pct).toFixed(1)}%
                </span>
                ${effectiveSort === "volatility" ? `<span class="mover-vol monospace" style="font-size: 0.7rem; color: #888;"> (⚡${m.volatility_pct.toFixed(1)}%)</span>` : ""}
            </div>
        `).join("");
    } catch (err) {
        console.error("Top movers failed:", err);
    }
}

let _searchTimeout = null;
function handleSearch() {
    clearTimeout(_searchTimeout);
    _searchTimeout = setTimeout(async () => {
        const query = document.getElementById("marketSearch").value;
        if (!query) {
            renderInventory();
            return;
        }

        try {
            const results = await window.invoke("search_market_items", { query });
            renderInventoryResults(results);
        } catch (err) {
            console.error("Search failed:", err);
        }
    }, 300);
}

async function renderInventoryResults(data) {
    const tbody = document.getElementById("inventoryBody");
    const cards = await Promise.all(data.map(async item => {
        const metadata = await window.invoke("get_item_metadata", { marketHashName: item.market_hash_name });
        return renderItemCard(item, metadata);
    }));
    tbody.innerHTML = cards.join('');
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

    // Trigger Top Movers independently
    updateTopMovers();

    try {
        const analytics = await window.invoke("get_item_analytics", { marketHashName: itemName });
        let history = await window.invoke("get_item_history_full", { marketHashName: itemName });

        // Filter and Pad history to timeframe (Backend handles 7d limit for Basic/Free)
        const now = Math.floor(Date.now() / 1000);
        const cutoff = now - (_currentTimeframe * 24 * 3600);
        history = history.filter(p => p.timestamp > cutoff);

        // Padding: If data doesn't start at the timeframe edge, prepend an average baseline [cite: UI requirement]
        if (history.length > 0 && history[0].timestamp > (cutoff + 3600)) {
            const prices = history.map(h => h.price);
            const avg = (Math.max(...prices) + Math.min(...prices)) / 2;
            history.unshift({
                timestamp: cutoff,
                price: avg,
                sma: avg,
                upper_band: avg,
                lower_band: avg,
                rsi: 50
            });
        }

        // Tier Checks
        const isPro = ['pro', 'elite'].includes(_currentTier);
        const isElite = _currentTier === 'elite';

        // Update Analytics Cards
        if (document.getElementById("dashVolatility")) document.getElementById("dashVolatility").innerText = analytics.volatility.toFixed(4);
        if (document.getElementById("dashSMA30")) document.getElementById("dashSMA30").innerText = `$${analytics.sma_30.toFixed(2)}`;

        // Trend: Elite Only
        const trendEl = document.getElementById("dashTrend");
        if (trendEl) {
            const trend = analytics.trend.toUpperCase();
            trendEl.innerText = isElite ? trend : "ELITE ONLY";
            trendEl.parentElement.setAttribute("data-locked", !isElite);
            trendEl.parentElement.setAttribute("data-tier", "ELITE");

            // Dynamic Coloring
            trendEl.className = "metric-value"; // Reset
            if (isElite) {
                if (trend === "UPTREND") trendEl.classList.add("metric-trend-up");
                else if (trend === "DOWNTREND") trendEl.classList.add("metric-trend-down");
                else trendEl.classList.add("metric-trend-stable");
            }
        }

        // RSI/Momentum: Pro+
        const rsiEl = document.getElementById("dashRSI");
        if (rsiEl) {
            rsiEl.innerText = isPro ? analytics.rsi.toFixed(2) : "--";
            rsiEl.parentElement.setAttribute("data-locked", !isPro);
            rsiEl.parentElement.setAttribute("data-tier", "PRO");
        }
        const momEl = document.getElementById("dashMomentum");
        if (momEl) {
            momEl.innerText = isPro ? analytics.momentum.toFixed(2) : "--";
            momEl.parentElement.setAttribute("data-locked", !isPro);
            momEl.parentElement.setAttribute("data-tier", "PRO");
        }

        const marketTrendEl = document.getElementById("marketTrend");
        if (marketTrendEl) {
            marketTrendEl.innerText = isElite ? `Signal: ${analytics.trend.toUpperCase()}` : "Market Intelligence: ELITE";
        }

        renderPriceChart(history);
    } catch (err) {
        console.error("Dashboard update failed:", err);
    }
}

function renderPriceChart(history) {
    const ctx = document.getElementById('mainChart').getContext('2d');
    const isPro = ['pro', 'elite'].includes(_currentTier);

    // Smarter Labels based on timeframe
    const labels = history.map(p => {
        const d = new Date(p.timestamp * 1000);
        const timeStr = d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        if (_currentTimeframe <= 24) {
            return timeStr;
        } else {
            return d.toLocaleDateString([], { month: 'short', day: 'numeric' }) + ' ' + timeStr;
        }
    });

    const priceData = history.map(p => p.price);
    const smaData = isPro ? history.map(p => p.sma) : [];
    const upperBand = isPro ? history.map(p => p.upper_band) : [];
    const lowerBand = isPro ? history.map(p => p.lower_band) : [];

    if (_mainChart) {
        _mainChart.destroy();
    }

    const datasets = [
        {
            label: 'Market Price',
            data: priceData,
            borderColor: '#4a7aff',
            backgroundColor: 'rgba(74, 122, 255, 0.15)',
            borderWidth: 3,
            tension: 0.4,
            pointRadius: history.length > 50 ? 0 : 2,
            pointHoverRadius: 6,
            fill: true,
            order: 2
        }
    ];

    if (isPro && history.some(p => p.rsi)) {
        datasets.push({
            label: 'RSI (14)',
            data: history.map(p => p.rsi),
            borderColor: '#ff007f', // Vibrant Pink
            borderWidth: 2,
            yAxisID: 'yRsi',
            pointRadius: 0,
            fill: false,
            tension: 0.4,
            order: 4
        });
    }

    if (isPro && history.length > 5) {
        datasets.push({
            label: 'SMA (20)',
            data: smaData,
            borderColor: '#ffaa00', // Amber
            borderWidth: 2,
            borderDash: [5, 5],
            pointRadius: 0,
            fill: false,
            tension: 0.4,
            order: 1
        });

        datasets.push({
            label: 'Volatility Band',
            data: upperBand,
            borderColor: 'rgba(74, 122, 255, 0.4)', // Distinct border for boundaries
            borderWidth: 1,
            pointRadius: 0,
            fill: '+1',
            backgroundColor: 'rgba(74, 122, 255, 0.1)', // Slightly more opaque fill
            tension: 0.4,
            order: 3
        });

        datasets.push({
            label: 'Lower Band',
            data: lowerBand,
            borderColor: 'rgba(74, 122, 255, 0.8)', // Stronger visibility for lower line
            borderWidth: 2,
            pointRadius: 0,
            fill: false,
            tension: 0.4,
            order: 3
        });
    }

    _mainChart = new Chart(ctx, {
        type: 'line',
        data: { labels, datasets },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            padding: { top: 10, bottom: 10 },
            interaction: {
                intersect: false,
                mode: 'index',
            },
            plugins: {
                legend: {
                    display: isPro,
                    position: 'bottom',
                    labels: {
                        color: 'var(--chart-tick)',
                        font: { family: "'Inter', sans-serif", size: 12, weight: '500' },
                        usePointStyle: true,
                        padding: 20
                    }
                },
                tooltip: {
                    backgroundColor: 'rgba(26, 29, 46, 0.95)',
                    titleColor: '#4a7aff',
                    titleFont: { family: "'Inter', sans-serif", size: 14, weight: 'bold' },
                    bodyFont: { family: "'Inter', sans-serif", size: 13 },
                    backgroundColor: 'rgba(20, 20, 20, 0.9)',
                    titleFont: { family: "'Inter', sans-serif", size: 13 },
                    bodyFont: { family: "'Inter', sans-serif", size: 12 },
                    padding: 12,
                    borderColor: 'rgba(255,255,255,0.1)',
                    borderWidth: 1
                }
            },
            scales: {
                x: {
                    grid: { display: false },
                    ticks: { color: 'var(--chart-tick)', font: { size: 10 } }
                },
                y: {
                    position: 'left',
                    grid: { color: 'var(--chart-grid)' },
                    ticks: {
                        color: 'var(--chart-tick)',
                        font: { family: "'Inter', sans-serif", size: 11 },
                        callback: (value) => '$' + value.toLocaleString()
                    }
                },
                yRsi: {
                    position: 'right',
                    min: 0,
                    max: 100,
                    display: false, // Hidden per user request for cleaner UI
                    grid: { display: false },
                    ticks: {
                        color: '#ff007f',
                        font: { size: 10, weight: 'bold' },
                        callback: (value) => value + '%' // Add % for RSI
                    }
                }
            }
        }
    });
}

// Duplicate listeners removed

// Update renderInventory to include a "Show Chart" link or click
async function renderInventory() {
    const tbody = document.getElementById("inventoryBody");
    if (!tbody) return;

    try {
        _currentTier = await window.invoke("get_setting", { key: "tier_level" });
        _inventoryData = await window.invoke("get_inventory_full");

        const cardPromises = _inventoryData.map(async item => {
            try {
                const metadata = await window.invoke("get_item_metadata", { marketHashName: item.market_hash_name });
                return renderItemCard(item, metadata);
            } catch (e) {
                console.warn("Metadata fetch failed for", item.market_hash_name, e);
                return renderItemCard(item, null);
            }
        });

        const cards = await Promise.all(cardPromises);
        tbody.innerHTML = cards.length > 0 ? cards.join('') : '<div class="placeholder">Your inventory is empty. Add some skins to start tracking!</div>';
    } catch (err) {
        console.error("Inventory render failed:", err);
        tbody.innerHTML = `<div class="placeholder error">Failed to load inventory: ${err}</div>`;
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
setupNavigation();
setupEventListeners();
updateTierUI();
initializeSettings();
// Initial dashboard load
setTimeout(updateDashboard, 500); 