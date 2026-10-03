use easy_money_manager::EasyMoneyManager;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    eframe::run_native("Easy Money Manager", options, Box::new(|cc| Ok(Box::new(EasyMoneyManager::new(cc)))),)
}

#[cfg(target_os = "android")]
fn main() {}

#[cfg(target_arch = "wasm32")]
fn main() -> eframe::Result {
    let options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .unwrap()
            .document()
            .unwrap();

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        eframe::WebRunner::new()
            .start(
                canvas,
                options,
                Box::new(|cc| {
                    Ok(Box::new(EasyMoneyManager::new(cc)))
                }),
            )
            .await
            .expect("failed to start eframe");
    });

    Ok(())
}
