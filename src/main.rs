mod myapp;
mod record;
mod sheet;
use myapp::MyApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();


    eframe::run_native(
        "Sheets",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

