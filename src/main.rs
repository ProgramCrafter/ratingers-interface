use serde::{Deserialize, Serialize};
use eframe::{App, APP_KEY, Frame};
use egui::{RichText, Color32, CentralPanel, TopBottomPanel, ScrollArea, menu};



macro_rules! color {
    ($r:expr, $g:expr, $b:expr) => {
        Color32::from_rgb($r, $g, $b)
    }
}
macro_rules! frame {
    (margin: $margin:expr, fill: $r:literal $g:literal $b:literal) => {
        egui::Frame::none().inner_margin($margin).fill(Color32::from_rgb($r, $g, $b))
    }
}
macro_rules! text {
    ($text:expr) => {
        RichText::new($text).size(15.0)
    };
    ($text:expr, size $size:expr) => {
        RichText::new($text).size($size)
    };
    ($text:expr, color $c:expr) => {
        RichText::new($text).size(15.0).color($c)
    };
    ($text:expr, size $size:expr, color $c:expr) => {
        RichText::new($text).size($size).color($c)
    };
}
macro_rules! sized_button {
    (to $ui:expr, size $width:expr, $height:expr, text $comment:expr) => {
        $ui.add_sized([$width, $height], egui::Button::new($comment))
    };
    (to $ui:expr, size $width:expr, $height:expr, text $comment:expr, fill $c:expr) => {
        $ui.add_sized([$width, $height], egui::Button::new($comment).fill($c))
    };
    (to $ui:expr, height $height:expr, $($etc:tt)+) => {
        sized_button!(to $ui, size $ui.available_width(), $height, $($etc)+)
    };
}
macro_rules! label {
    (to $ui:expr, format ($($etc:tt)+)) => {
        $ui.label(text!(
          format!($($etc)+)
        ));
    };
    (to $ui:expr, $comment:expr) => {
        $ui.label($comment);
    };
}



#[derive(Deserialize, Serialize)]
pub struct ManyCommentsApp {
    save_storage: bool,
    comments: Vec<String>,
}

impl Default for ManyCommentsApp {
    fn default() -> Self {
        let mut v = Vec::with_capacity(131072);
        for i in 0..131072 {
            v.push(format!("{}", i));
        }
        
        Self {
            save_storage: false,
            comments: v,
        }
    }
}

impl ManyCommentsApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        match cc.storage {
            Some(storage) => eframe::get_value(storage, APP_KEY).unwrap_or_default(),
            None          => Default::default()
        }
    }
    
    fn do_save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
    }
}

impl App for ManyCommentsApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.save_storage { self.do_save(storage); }
    }
    
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.set_min_height(30.0);
                ui.menu_button(text!("File"), |ui| {
                    if ui.button("Quit").clicked() { frame.close(); }
                    
                    let saves_text = if self.save_storage {"Disable saves"} else {"Enable saves"};
                    if ui.button(saves_text).clicked() {
                        self.save_storage = !self.save_storage;
                        self.do_save(frame.storage_mut().expect("No storage available"));
                    }
                });
                ui.separator();
                label!(to ui, format ("{} comments", self.comments.len()));
            });
        });
        
        CentralPanel::default().frame(frame!(margin: 8.0, fill: 24 24 24)).show(ctx, |ui| {
            ScrollArea::vertical().show_rows(ui, 68.0, self.comments.len(), |ui, row_range| {
                ui.vertical(|ui| {
                    for row in row_range {
                        sized_button!(to ui, height 60.0,
                          text text!(&self.comments[row], color Color32::WHITE),
                          fill color!(48, 48, 48));
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
