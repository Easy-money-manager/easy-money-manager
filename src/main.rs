mod easymoneymanager;
mod record;
mod sheet;
mod sheetcollection;
use easymoneymanager::EasyMoneyManager;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Easy Money Manager",
        options,
        Box::new(|_cc| Ok(Box::new(EasyMoneyManager::default()))),
    )
}

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
                    Ok(Box::new(EasyMoneyManager::default()))
                }),
            )
            .await
            .expect("failed to start eframe");
    });

    Ok(())
}
