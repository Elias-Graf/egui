use std::sync::{Arc, RwLock};

use egui::{vec2, ScrollArea, Sense, TextureId, Widget, Window};

use super::DbgTextMan;

pub struct TextManDebugWidget {
    // TODO: Figure out why egui used the RwLock from [`parking_lot`]
    // [`egui::mutex::RwLock`].
    // The issue is that this one doesn't seem to be compatible with unsized types,
    // (the dyn traits).
    pub text_man: Arc<RwLock<dyn DbgTextMan>>,
}

impl Widget for TextManDebugWidget {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let low_lvl_text_man = ui.ctx().tex_manager();
        let low_lvl_text_man = &low_lvl_text_man.read();

        Window::new("TextManDebugWidget").show(ui.ctx(), |ui| {
            let mut total_bytes_used = 0;

            ui.collapsing("Allocated textures", |ui| {
                ScrollArea::new([false, true]).show(ui, |ui| {
                    for ((url, text_size), text_id) in
                        self.text_man.read().unwrap().cached_text_ids()
                    {
                        let text_bytes_used = get_bytes_used(low_lvl_text_man, text_id.clone());
                        total_bytes_used += text_bytes_used;

                        ui.label(format!("{} {:?} {} bytes", url, text_size, text_bytes_used));
                    }
                });
            });
            ui.label(format!("total allocated bytes {}", total_bytes_used));
        });

        // TODO: Figure out a better solution than this
        ui.allocate_response(vec2(0f32, 0f32), Sense::click())
    }
}

fn get_bytes_used(low_lvl_text_man: &egui::epaint::TextureManager, text_id: TextureId) -> usize {
    let meta = low_lvl_text_man.meta(text_id).unwrap_or_else(|| {
        panic!(
            "our texture does not seem to exist inside the low level\
        texture manager, that shouldn't happen"
        )
    });

    meta.bytes_used()
}
