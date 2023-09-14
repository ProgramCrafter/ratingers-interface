use serde::{Deserialize, Serialize};
use eframe::{App, APP_KEY, Frame};
use egui::{RichText, Color32, CentralPanel, TopBottomPanel, ScrollArea, menu};
// use std::time::Duration;
use std::sync::{RwLock, OnceLock};

extern crate dlopen;
#[macro_use]
extern crate dlopen_derive;
use dlopen::wrapper::{Container, WrapperApi};



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
macro_rules! disable_enable {
    (TEXT, $name:literal, $cond:expr) => {
        if $cond {concat!("Disable ", $name)} else {concat!("Enable ", $name)}
    };
    (BUTTON at $ui:expr, $name:literal, $cond:expr) => {
        $ui.button(disable_enable!(TEXT, $name, $cond))
    };
    (CLICK at $ui:expr, $name:literal, $cond:expr) => {
        disable_enable!(BUTTON at $ui, $name, $cond).clicked()
    };
}


#[repr(C)] pub struct Color(u8, u8, u8);
#[repr(C)] pub struct Message {
    text: *mut String,
    color: Color,
}
#[derive(WrapperApi)]
struct Api {
  version: extern "C" fn() -> u64,
  
  start:              extern "C" fn(callback: extern "C" fn(Message) -> ()) -> bool,
  stop:               extern "C" fn() -> (),
  deallocate_message: extern "C" fn(msg: Message) -> (),
}
struct Reloadable {dll: Container<Api>, ver: u64, ticks: usize}
impl Reloadable {
    fn new() -> Reloadable {
        let c: Container<Api> =
            unsafe { Container::load("D:\\Rust\\ratingers-notifier\\target\\release\\rust_dl.dll") }
            .expect("Could not open library or load symbols");
        let cv = c.version();
        Reloadable {dll: c, ver: u64::MAX, ticks: 0}
    }
    fn reload(&mut self) {
        if self.ticks > 0 {self.ticks -= 1; return;}
        self.ticks = 20;
        
        let c: Container<Api> =
            unsafe { Container::load("D:\\Rust\\ratingers-notifier\\target\\release\\rust_dl.dll") }
            .expect("Could not open library or load symbols");
        let cv = c.version();
        if cv != self.ver {
            self.dll.stop();
            (self.dll, self.ver) = (c, cv);
            self.dll.start(handle_message);
        }
    }
}
impl Default for Reloadable {
    fn default() -> Self {Self::new()}
}

#[derive(Deserialize, Serialize)]
pub struct ManyCommentsApp {
    save_storage: bool,
    comments: RwLock<Vec<String>>,
    #[serde(skip)] dll: Reloadable,
}

impl Default for ManyCommentsApp {
    fn default() -> Self {
        Self {
            save_storage: false,
            comments: Vec::with_capacity(256).into(),
            dll: Reloadable::new(),
        }
    }
}

static APP_PTR: OnceLock<usize> = OnceLock::new();

impl ManyCommentsApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        match cc.storage {
            Some(storage) => eframe::get_value(storage, APP_KEY).unwrap_or_default(),
            None          => Default::default()
        }
    }
    
    fn do_save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
    }
}


extern "C" fn handle_message(msg: Message) {
    // SAFETY: app cannot be captured via &mut self reference
    // loading it from singleton
    let app_ptr = *APP_PTR.get().unwrap() as *mut ManyCommentsApp;
    
    // SAFETY: mutable reference doesn't drop contents
    let app: &mut ManyCommentsApp = unsafe {&mut *app_ptr};
    
    // SAFETY: mutable reference doesn't drop contents
    let msg_text: &mut String = unsafe {&mut *msg.text};
    
    // SAFETY: protected by RwLock
    // can deadlock if notifier attempts to create comment during app initialization
    let mut comments_lock = app.comments.write().unwrap();
    comments_lock.push(msg_text.clone());
    
    // SAFETY:
    // 1. comments_lock is still active so `update` cannot be executed at the same moment
    // 1.1. `update` does not access DLL
    // 2. DLL is marked by #[serde(skip)] so there is nothing other accessing it
    app.dll.dll.deallocate_message(msg);
    
    drop(comments_lock);
}


impl App for ManyCommentsApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.save_storage { self.do_save(storage); }
    }
    
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        let comments = self.comments.read().unwrap();
        let comments_len = comments.len();
        
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.set_min_height(30.0);
                ui.menu_button(text!("File"), |ui| {
                    if ui.button("Quit").clicked() { frame.close(); }
                    
                    // if ui.button("Clear").clicked() { self.comments.write().unwrap().clear(); }
                    // it's impossible to clear comments since
                    // 1. `self.comments` is immutably borrowed by `comments`
                    // 0. this violates invariant that different parts of UI have consistent data
                    
                    if disable_enable!(CLICK at ui, "saves", self.save_storage) {
                        self.save_storage = !self.save_storage;
                        // self.do_save(frame.storage_mut().expect("No storage available"));
                    }
                });
                ui.separator();
                label!(to ui, format ("{} comments", comments_len));
                ui.separator();
                label!(to ui, format ("DLL ver: {}", self.dll.ver));
                label!(to ui, format ("ticks: {}", self.dll.ticks));
            });
        });
        
        CentralPanel::default().frame(frame!(margin: 8.0, fill: 24 24 24)).show(ctx, |ui| {
            ScrollArea::vertical().show_rows(ui, 68.0, comments_len, |ui, row_range| {
                ui.vertical(|ui| {
                    for row in row_range {
                        sized_button!(to ui, height 60.0,
                          text text!(&comments[row], color Color32::WHITE),
                          fill color!(48, 48, 48));
                        ui.add_space(8.0);
                    }
                });
            });
        });
        
        self.dll.reload();
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
        Box::new(|cc| {
            let app_box = Box::new(ManyCommentsApp::new(cc));
            APP_PTR.set(&*app_box as *const ManyCommentsApp as usize)
                   .expect("APP_PTR was already set?");
            app_box
        }),
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
                Box::new(|cc| {
                    let app_box = Box::new(ManyCommentsApp::new(cc));
                    APP_PTR.set(&*app_box as *const ManyCommentsApp as usize)
                           .expect("APP_PTR was already set?");
                    app_box
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}
