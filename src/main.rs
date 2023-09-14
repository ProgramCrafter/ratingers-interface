use serde::{Deserialize, Serialize};
use eframe::{App, APP_KEY, Frame};


#[derive(Deserialize, Serialize)]
pub struct ManyCommentsApp {
    enable_quit_option: bool,
    comments: Vec<String>,
}

impl Default for ManyCommentsApp {
    fn default() -> Self {
        let mut v = Vec::with_capacity(131072);
        for i in 0..131072 {
            v.push(format!("{}", i));
        }
        
        Self {
            enable_quit_option: true,
            comments: v
        }
    }
}

impl ManyCommentsApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any). `persistence` feature is enabled.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl App for ManyCommentsApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
    }
    
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        if self.enable_quit_option {
            #[cfg(not(target_arch = "wasm32"))]   // no File->Quit on web pages!
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() { frame.close(); }
                    });
                });
            });
        }
        
        egui::CentralPanel::default()
        .frame(egui::Frame::none().inner_margin(8.0).fill(egui::Color32::from_rgb(24, 24, 24)))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show_rows(ui, 68.0, self.comments.len(), |ui, row_range| {
                ui.vertical(|ui| {
                    for row in row_range {
                        let comment = &self.comments[row];
                        ui.add_sized([ui.available_width(), 60.0],
                            egui::Button::new(egui::RichText::new(comment).size(17.0).color(egui::Color32::WHITE).weak())
                              .fill(egui::Color32::from_rgb(48, 48, 48))
                        );
                        ui.add_space(8.0);
                    }
                });
            });
        });
    }
}


//------------------------------------------------------------------------------

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "many comments",
        native_options,
        Box::new(|cc| Box::new(ManyCommentsApp::new(cc))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| Box::new(ManyCommentsApp::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}
