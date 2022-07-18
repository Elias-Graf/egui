use std::{
    sync::{Arc, RwLock},
    time::{Duration, SystemTime},
};

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
            let dbg_text_man = self.text_man.read().unwrap();

            ui.collapsing("Allocated textures", |ui| {
                ScrollArea::new([false, true])
                    .auto_shrink([false, false])
                    .max_height(ui.available_height() - 100.0)
                    .show(ui, |ui| {
                        for ((url, text_size), text) in dbg_text_man.cached_text_ids() {
                            let text_bytes_used =
                                get_bytes_used(low_lvl_text_man, text.text_id.clone());
                            let access_delta = SystemTime::now()
                                .duration_since(text.last_used)
                                .unwrap_or(Duration::default());

                            ui.label(format!(
                                "{}, last accessed: {:#?}, size: {:?} allocated (bytes): {} bytes",
                                url, access_delta, text_size, text_bytes_used
                            ));
                        }
                    });
            });

            ui.label(format!(
                "total allocated bytes {}",
                dbg_text_man.cached_text_id_size()
            ));
            if ui.button("📋").on_hover_text("Click to copy").clicked() {
                ui.output().copied_text = dbg_text_man.cached_text_id_size().to_string();
            }
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
