use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tauri::{Manager, Emitter};
use rdev::{listen, EventType};
use mouse_position::mouse_position::Mouse;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[derive(Clone, serde::Serialize)]
struct MouseMoveEvent {
    x: i32,
    y: i32,
}

#[derive(Clone, serde::Serialize)]
struct ColorEvent {
    x: i32,
    y: i32,
    hex: String,
    grid: Vec<String>,
}

// Helper to get color at specific coordinates, including a small grid around it
fn get_color_grid(x: i32, y: i32, radius: i32) -> Option<(String, Vec<String>)> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;
        use windows::Win32::Graphics::Gdi::{
            GetDC, ReleaseDC, CreateCompatibleDC, CreateCompatibleBitmap, 
            SelectObject, BitBlt, DeleteDC, DeleteObject, SRCCOPY,
            GetDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS
        };
        
        unsafe {
            let hwnd = GetDesktopWindow();
            let hdc_screen = GetDC(Some(hwnd));
            if hdc_screen.is_invalid() {
                return None;
            }

            let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
            let size = (radius * 2 + 1) as i32;
            let hbm = CreateCompatibleBitmap(hdc_screen, size, size);
            
            let h_old = SelectObject(hdc_mem, hbm.into());

            // Copy the region from screen to memory DC
            let start_x = x - radius;
            let start_y = y - radius;
            let _ = BitBlt(hdc_mem, 0, 0, size, size, Some(hdc_screen), start_x, start_y, SRCCOPY);

            // Prepare to read the bits
            let mut bmi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: size,
                    biHeight: -size, // Top-down DIB
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0 as u32,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [windows::Win32::Graphics::Gdi::RGBQUAD::default(); 1],
            };

            let mut pixels = vec![0u32; (size * size) as usize];
            GetDIBits(
                hdc_mem,
                hbm,
                0,
                size as u32,
                Some(pixels.as_mut_ptr() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            );

            // Cleanup GDI objects
            SelectObject(hdc_mem, h_old);
            let _ = DeleteObject(hbm.into());
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(Some(hwnd), hdc_screen);

            let mut grid = Vec::with_capacity((size * size) as usize);
            let mut center_hex = String::from("#FFFFFF");
            let center_idx = (radius * size + radius) as usize;

            for (i, &color_val) in pixels.iter().enumerate() {
                // GetDIBits with 32-bit returns BGRA, we need RGB
                let b = color_val & 0xFF;
                let g = (color_val >> 8) & 0xFF;
                let r = (color_val >> 16) & 0xFF;
                
                let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
                if i == center_idx {
                    center_hex = hex.clone();
                }
                grid.push(hex);
            }
            
            return Some((center_hex, grid));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let grid = vec![String::from("#FFFFFF"); ((radius * 2 + 1) * (radius * 2 + 1)) as usize];
        Some((String::from("#FFFFFF"), grid))
    }
}

struct AppState {
    picking: Arc<AtomicBool>,
}

#[tauri::command]
fn start_picking(state: tauri::State<'_, AppState>) {
    state.picking.store(true, Ordering::SeqCst);
}

#[tauri::command]
fn stop_picking(state: tauri::State<'_, AppState>) {
    state.picking.store(false, Ordering::SeqCst);
}

#[tauri::command]
fn get_current_color(state: tauri::State<'_, AppState>, radius: i32, x: i32, y: i32) -> Option<ColorEvent> {
    let is_picking = state.picking.load(Ordering::SeqCst);
    if !is_picking {
        return None;
    }

    get_color_grid(x, y, radius).map(|(hex, grid)| ColorEvent { x, y, hex, grid })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let picking_state = Arc::new(AtomicBool::new(false));
    let picking_state_clone = picking_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState { picking: picking_state.clone() })
        .setup(move |app| {
            let app_handle = app.handle().clone();
            
            // Start hidden
            if let Some(window) = app.get_webview_window("main") {
                // We do NOT ignore cursor events so that CSS `cursor: none` can hide the system cursor
                let _ = window.hide();
            }
            
            // Spawn rdev listener
            std::thread::spawn(move || {
                let mut last_move = std::time::Instant::now();
                if let Err(error) = listen(move |event| {
                    let is_picking = picking_state_clone.load(Ordering::SeqCst);
                    if !is_picking {
                        return;
                    }
                    
                    match event.event_type {
                        EventType::MouseMove { x, y } => {
                            // Throttle mouse-move events to ~60fps (16ms)
                            if last_move.elapsed().as_millis() > 16 {
                                let _ = app_handle.emit("mouse-move", MouseMoveEvent { x: x as i32, y: y as i32 });
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    if let Ok(size) = window.outer_size() {
                                        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { 
                                            x: (x as i32) - (size.width as i32 / 2), 
                                            y: (y as i32) - (size.height as i32 / 2) 
                                        }));
                                    }
                                }
                                last_move = std::time::Instant::now();
                            }
                        }
                        EventType::ButtonPress(button) => {
                            let btn_str = match button {
                                rdev::Button::Left => "left",
                                rdev::Button::Right => "right",
                                _ => "other",
                            };
                            let _ = app_handle.emit("mouse-click", btn_str);
                            
                            // If left click, copy color directly and emit event
                            if matches!(button, rdev::Button::Left) {
                                let position = Mouse::get_mouse_position();
                                if let Mouse::Position { x, y } = position {
                                    if let Some((hex, _)) = get_color_grid(x, y, 0) {
                                        let _ = app_handle.clipboard().write_text(hex.clone());
                                        let _ = app_handle.emit("color-copied", hex);
                                    }
                                }
                                
                                picking_state_clone.store(false, Ordering::SeqCst);
                                if let Some(window) = app_handle.get_webview_window("main") {
                                    let _ = window.hide();
                                }
                            }
                        }
                        EventType::KeyPress(rdev::Key::Escape) => {
                            let _ = app_handle.emit("exit-picking", ());
                            picking_state_clone.store(false, Ordering::SeqCst);
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                        _ => {}
                    }
                }) {
                    println!("Error: {:?}", error);
                }
            });

            // Set up system tray
            let quit_i = tauri::menu::MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let toggle_i = tauri::menu::MenuItem::with_id(app, "toggle", "显示/隐藏悬浮窗", true, None::<&str>)?;
            let menu = tauri::menu::Menu::with_items(app, &[&toggle_i, &quit_i])?;

            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app: &tauri::AppHandle, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            std::process::exit(0);
                        }
                        "toggle" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let is_visible = window.is_visible().unwrap_or(false);
                                if is_visible {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                }
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Register global shortcut Ctrl+Shift+C
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
            let shortcut = Shortcut::new(Some(tauri_plugin_global_shortcut::Modifiers::ALT | tauri_plugin_global_shortcut::Modifiers::SHIFT), tauri_plugin_global_shortcut::Code::KeyC);
            let picking_state_shortcut = picking_state.clone();
            app.handle().plugin(
                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, req_shortcut, event| {
                        if req_shortcut == &shortcut && event.state == ShortcutState::Pressed {
                            if let Some(window) = app.get_webview_window("main") {
                                let is_visible = window.is_visible().unwrap_or(false);
                                if is_visible {
                                    let _ = window.hide();
                                    picking_state_shortcut.store(false, Ordering::SeqCst);
                                    let _ = app.emit("exit-picking", ());
                                } else {
                                    let _ = window.show();
                                    picking_state_shortcut.store(true, Ordering::SeqCst);
                                    let position = Mouse::get_mouse_position();
                                    if let Mouse::Position { x, y } = position {
                                        if let Ok(size) = window.outer_size() {
                                            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { 
                                                x: (x as i32) - (size.width as i32 / 2), 
                                                y: (y as i32) - (size.height as i32 / 2) 
                                            }));
                                        }
                                        let _ = app.emit("start-picking", MouseMoveEvent { x, y });
                                        // Also trigger an immediate mouse-move to force color update
                                        let _ = app.emit("mouse-move", MouseMoveEvent { x, y });
                                    } else {
                                        let _ = app.emit("start-picking", MouseMoveEvent { x: -1, y: -1 });
                                    }
                                }
                            }
                        }
                    })
                    .build(),
            )?;
            
            if let Err(e) = app.handle().global_shortcut().register(shortcut) {
                println!("Warning: Failed to register global shortcut: {}", e);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_picking,
            stop_picking,
            get_current_color
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
