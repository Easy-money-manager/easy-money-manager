#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
mod database;
mod record;
mod sheet;
mod sheetcollection;
mod requests;
#[cfg(feature = "client")]
mod easymoneymanager;
#[cfg(feature = "client")]
mod api;
#[cfg(feature = "client")]
use easymoneymanager::EasyMoneyManager;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

#[cfg(feature = "client")]
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Easy Money Manager",
        options,
        Box::new(|_cc| Ok(Box::new(EasyMoneyManager::default()))),
    )
}

#[cfg(feature = "client")]
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

#[cfg(feature = "server")]
#[tokio::main]
async fn main() {
    if let Err(error) = server::run().await {
        eprintln!("Server failed to start: {}", error);
    }
}
