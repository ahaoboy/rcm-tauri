use tauri::window::Color;
use tauri::{AppHandle, Emitter, Manager};
pub mod rcm;
pub mod registry;
pub mod tray;
pub mod pipe;

fn start_monitoring(app_handle: tauri::AppHandle) {
    use rdev::{Button, EventType, listen};

    std::thread::spawn(move || {
        if let Err(error) = listen(move |event| {
            let (event_name, button_name) = match event.event_type {
                EventType::ButtonPress(Button::Left) => ("ButtonPress", "Left"),
                EventType::ButtonRelease(Button::Right) => ("ButtonRelease", "Right"),
                _ => return,
            };

            let menu = rcm::rcm().ok();

            let payload = serde_json::json!({
                "event": event_name,
                "button": button_name,
                "menu": menu,
            });

            let _ = app_handle.emit("input-event", payload);
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
    if pipe::check_client_cli() {
        return; // Execute as an IPC client CLI utility and exit immediately directly saving memory
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .setup(|app| {
            tray::setup_tray(app)?;
            pipe::start_pipe_server(app.app_handle().clone());
            start_monitoring(app.app_handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![create_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
