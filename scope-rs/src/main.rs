#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{cell::RefCell, rc::Rc};
// // When compiling natively:
// #[tokio::main]
// #[cfg(not(target_arch = "wasm32"))]
// async fn main() -> eframe::Result<()> {
//     env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
//                         // let backend = Backend::new();
//     let native_options = eframe::NativeOptions::default();
//     eframe::run_native(
//         "eframe template",
//         native_options,
//         Box::new(|cc| Box::new(eth_toolkit::Frontend::new(cc))),
//     )
// }

// // When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // error messages

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        console_error_panic_hook::set_once();
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(eth_toolkit::Frontend::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });

    // wasm_bindgen_futures::spawn_local(async {
    //     console_error_panic_hook::set_once();
    //     // eframe::WebRunner::new()

    //     eframe::WebRunner::new()
    //         .start(
    //             "the_canvas_id", // hardcode it
    //             web_options,
    //             Box::new(|cc| Box::new(DeleteFrontend::new(cc))),
    //         )
    //         .await
    //         .expect("failed to start eframe");
    // });
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct DeleteFrontend {
    text: String,
}

impl DeleteFrontend {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            ..Default::default()
        }
    }
    // pub fn init(&self) {
    //     self.backend
    //         .as_ref()
    //         .expect("no backend set")
    //         .query_for_open_files();
    // }
}

impl eframe::App for DeleteFrontend {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut panel_frame = egui::containers::Frame::none();
        panel_frame.inner_margin = egui::style::Margin {
            left: 10.,
            right: 10.,
            top: 10.,
            bottom: 10.,
        };
        // log!("del_one");
        // frame.fill = egui::Color32::GRAY; // bunch of other configs we could put here
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| ui.text_edit_singleline(&mut self.text));
    }
}
