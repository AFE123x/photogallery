use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use eframe::{egui, epaint::ColorImage};
use notify::{RecursiveMode, Result as NotifyResult, Watcher};
use rand::seq::IndexedMutRandom;

struct SlideshowApp {
    image_paths: Arc<Mutex<Vec<PathBuf>>>,
    current_image: Option<egui::TextureHandle>,
    last_switch: Instant,
}

impl SlideshowApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let image_dir = Arc::new(PathBuf::from("./images"));

        let image_paths = Arc::new(Mutex::new(scan_images(&image_dir)));
        let image_paths_clone = Arc::clone(&image_paths);
        let image_dir_clone = Arc::clone(&image_dir);
        

        thread::spawn(move || {
            let image_dir_for_watcher = Arc::clone(&image_dir_clone);
            
            let mut watcher = notify::recommended_watcher(move |res: NotifyResult<notify::Event>| {
                if let Ok(_) = res {
                    let updated = scan_images(&image_dir_for_watcher);
                    if let Ok(mut lock) = image_paths_clone.lock() {
                        *lock = updated;
                    }
                }
            }).unwrap();

            watcher
                .watch(&**image_dir_clone, RecursiveMode::NonRecursive)
                .unwrap();


            loop {
                thread::sleep(Duration::from_secs(60));
            }
        });

        let mut app = Self {
            image_paths,
            current_image: None,
            last_switch: Instant::now(),
        };

        app.load_random_image(&cc.egui_ctx);
        app
    }

    fn load_random_image(&mut self, ctx: &egui::Context) {
        let mut lock = self.image_paths.lock().unwrap();
        if lock.is_empty() {
            self.current_image = None;
            return;
        }

        let mut rng = rand::rng();
        let path = lock.choose_mut(&mut rng).unwrap();

        if let Ok(img) = image::open(path) {
            let img = img.to_rgba8();
            let size = [img.width() as _, img.height() as _];
            let pixels = img.into_vec();
            let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
            self.current_image = Some(ctx.load_texture("image", color_image, Default::default()));
        }
    }
}

impl eframe::App for SlideshowApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        if self.last_switch.elapsed() > Duration::from_secs(10) {
            self.load_random_image(ctx);
            self.last_switch = Instant::now();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = &self.current_image {
                let available_size = ui.available_size();
                let texture_size = texture.size_vec2();
                

                let scale_x = available_size.x / texture_size.x;
                let scale_y = available_size.y / texture_size.y;
                let scale = scale_x.min(scale_y);
                
                let scaled_size = texture_size * scale;


                let response = ui.allocate_response(available_size, egui::Sense::click());
                let rect = egui::Rect::from_center_size(
                    response.rect.center(),
                    scaled_size,
                );
                

                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
                    ui.add(egui::Image::from_texture(texture).fit_to_exact_size(scaled_size));
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No images found in ./images directory");
                });
            }
        });


        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn scan_images(dir: &PathBuf) -> Vec<PathBuf> {
    if let Ok(entries) = fs::read_dir(dir) {
        entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && matches!(
                        p.extension().and_then(|e| e.to_str()),
                        Some("jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp")
                    )
            })
            .collect()
    } else {
        Vec::new()
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Image Slideshow",
        options,
        Box::new(|cc| Ok(Box::new(SlideshowApp::new(cc)))),
    )
}