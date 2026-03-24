use tauri::ipc::Invoke;
use tauri::Runtime;

pub fn register<R: Runtime>() -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
    move |invoke: Invoke<R>| {
        let cmd = invoke.message.command();

        match cmd {
            "ping" => {
                invoke.resolver.resolve("pong");
                true
            }

            "get_inventory" => {
                match crate::db::get_inventory() {
                    Ok(items) => invoke.resolver.resolve(serde_json::to_string(&items).unwrap()),
                    Err(e) => invoke.resolver.reject(e.to_string()),
                }
                true
            }

            _ => false,
        }
    }
}



