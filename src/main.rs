// No console window next to the app in Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod formations;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([900.0, 900.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Onze",
        options,
        Box::new(|cc| Ok(Box::new(app::Onze::new(cc)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast;
    wasm_bindgen_futures::spawn_local(async {
        let canvas = eframe::web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("onze"))
            .and_then(|e| e.dyn_into::<eframe::web_sys::HtmlCanvasElement>().ok())
            .expect("index.html needs <canvas id=\"onze\">");
        let started = eframe::WebRunner::new()
            .start(
                canvas.clone(),
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(app::Onze::new(cc)))),
            )
            .await;
        // Show the failure on the page instead of leaving a blank canvas.
        if let Err(e) = started
            && let Some(parent) = canvas.parent_element()
        {
            parent.set_text_content(Some(&format!(
                "Onze couldn't start in this browser (WebGL2 is required): {e:?}"
            )));
        }
    });
}
