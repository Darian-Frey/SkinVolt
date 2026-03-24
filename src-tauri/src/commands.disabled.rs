use tauri::ipc::InvokeHandler;


pub fn register_commands() -> InvokeHandler {
    tauri::generate_handler![
        // Phase 1 commands
        crate::steam::cache::cache_price_data,
        crate::steam::cache::load_cached_price,
        crate::steam::cache::cache_price_history,
        crate::steam::cache::prune_old_cache_entries,
        crate::settings::settings::set_refresh_interval,
        crate::settings::settings::set_currency_preference,
        crate::settings::settings::toggle_dark_mode,
        crate::utils::logging::log_error,
        crate::steam::validate::validate_steam_response
    ]
}
