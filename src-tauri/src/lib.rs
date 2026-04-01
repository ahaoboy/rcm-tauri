use tauri::window::Color;
use tauri::{AppHandle, Emitter, Manager};
pub mod registry;
pub mod tray;

fn start_monitoring(app_handle: tauri::AppHandle) {
    use rdev::{EventType, listen};
    std::thread::spawn(move || {
        if let Err(error) = listen(move |event| match event.event_type {
            EventType::ButtonPress(_) | EventType::ButtonRelease(_) => {
                println!("My callback {:?}", event);
                let _ = app_handle.emit("input-event", event);
            }
            _ => {}
        }) {
            println!("Error: {error:?}")
        }
    });
}

#[tauri::command]
async fn create_window(app: tauri::AppHandle, label: String) {
    if app.get_webview_window(&label).is_some() {
        return;
    }

    let url = format!("index.html#{label}");
    let builder = tauri::WebviewWindowBuilder::new(
        &app,
        label.as_str(),
        tauri::WebviewUrl::App(url.as_str().into()),
    )
    .title("input-viz-key")
    .decorations(false)
    .background_color(Color(0, 0, 0, 0))
    .position(0., 0.)
    .inner_size(1., 1.)
    .always_on_top(true)
    .skip_taskbar(true)
    .fullscreen(false)
    .visible(false)
    .closable(false)
    .resizable(false)
    .minimizable(false)
    .maximizable(false)
    .focused(false)
    .shadow(false);

    #[cfg(not(target_os = "macos"))]
    let builder = builder.transparent(true);

    builder.build().unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .setup(|app| {
            tray::setup_tray(app)?;
            start_monitoring(app.app_handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![create_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
