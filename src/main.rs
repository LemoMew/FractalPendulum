#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

fn setup_custom_fonts(ctx: &egui::Context) {
    // 开始设置字体
    let mut fonts = egui::FontDefinitions::default();

    // 添加中文字体
    fonts.font_data.insert(
        "MaoKenZhuYuanTi".to_owned(),
        #[expect(clippy::large_include_file)]
        std::sync::Arc::from(egui::FontData::from_static(include_bytes!(
            "../assets/fonts/MaoKenZhuYuanTi-MaokenZhuyuanTi-2.ttf"
        ))),
    );

    // 设置优先级
    fonts.families.insert(
        egui::FontFamily::Proportional,
        vec![
            "MaoKenZhuYuanTi".to_owned(), // 首选字体
            "Ubuntu-Light".to_owned(),    // 回退到默认字体
            "NotoEmoji-Regular".to_owned(),
            "emoji-icon-font".to_owned(),
        ],
    );

    // 应用字体设置
    ctx.set_fonts(fonts);
}

fn setup_custom_style(ctx: &egui::Context) {
    let mut style = egui::Style::default();
    style.spacing.slider_width = 100.0;
    style.spacing.window_margin = egui::Margin::ZERO;
    ctx.set_style(style);
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([500.0, 300.0])
            .with_decorations(true)
            .with_resizable(true)
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "分形摆",
        native_options,
        Box::new(|cc| {
            setup_custom_fonts(&cc.egui_ctx);
            setup_custom_style(&cc.egui_ctx);
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(fractal_pendulum::TemplateApp::new(cc)))
        }),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    setup_custom_fonts(&cc.egui_ctx);
                    setup_custom_style(&cc.egui_ctx);
                    cc.egui_ctx.set_visuals(egui::Visuals::dark());
                    Ok(Box::new(fractal_pendulum::TemplateApp::new(cc)))
                }),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
