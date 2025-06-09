mod steps;
mod app;
mod components;
mod font_util;
use app::App;
use eframe::egui;
use eframe::CreationContext;
use std::fs;
use winit::event_loop::EventLoop;
use winit::monitor::MonitorHandle;

fn main() {
    // ロギングとパニックハンドラーを設定
    env_logger::init();
    
    // パニックハンドラーを設定して詳細なエラー情報を取得
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("🚨 PANIC OCCURRED:");
        eprintln!("Location: {}", panic_info.location().unwrap_or(&std::panic::Location::caller()));
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("Message: {}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            eprintln!("Message: {}", s);
        } else {
            eprintln!("Message: Unknown panic payload");
        }
        
        // ファイルに書き出し
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("crash_log.txt") {
            use std::io::Write;
            writeln!(&mut file, "🚨 PANIC OCCURRED at: {:?}", std::time::SystemTime::now()).ok();
            writeln!(&mut file, "Location: {}", panic_info.location().unwrap_or(&std::panic::Location::caller())).ok();
            if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                writeln!(&mut file, "Message: {}", s).ok();
            }
        }
    }));
    
    println!("🚀 Starting Magic Merge Excel 2.0...");
    let app_name = "Magic Merge Excel 2.0";

    // winitでプライマリモニターのサイズ取得
    let event_loop = EventLoop::new();
    let monitor: Option<MonitorHandle> = event_loop.primary_monitor();
    let (width, height) = if let Some(monitor) = monitor {
        let size = monitor.size();
        (size.width as f32 / 4.0, size.height as f32 / 4.0) // 1/4サイズ（25%）
    } else {
        (480.0, 270.0) // fallback（25%相当）
    };

    let mut options = eframe::NativeOptions::default();
    
    // エミュレーション環境での互換性向上
    options.renderer = eframe::Renderer::Glow;
    
    // 環境変数でソフトウェアレンダリングを強制可能
    if std::env::var("MAGIC_MERGE_SOFTWARE_RENDER").is_ok() {
        println!("🖥️ Software rendering mode enabled");
        options.hardware_acceleration = eframe::HardwareAcceleration::Off;
    } else {
        options.hardware_acceleration = eframe::HardwareAcceleration::Preferred;
    }
    
    options.window_builder = Some(Box::new(move |builder| {
        builder
            .with_inner_size((width, height))
            .with_position((0.0, 0.0))
            .clone()
    }));

    match eframe::run_native(
        app_name,
        options,
        Box::new(|cc: &CreationContext| {
            font_util::set_japanese_font(&cc.egui_ctx);
            Box::new(App::new())
        })
    ) {
        Ok(_) => println!("App exited normally"),
        Err(e) => {
            eprintln!("Failed to run app: {}", e);
            std::process::exit(1);
        }
    }
}
