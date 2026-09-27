mod clienttask;
mod easymoneymanager;
mod api;
mod import;

#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

pub use easymoneymanager::EasyMoneyManager;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };

    eframe::run_native(
        "Easy Money Manager",
        options,
        Box::new(|cc| {
            Ok(Box::new(EasyMoneyManager::new(cc)))
        }),
    )
    .expect("Failed to start EMM on Android");
}
