use egui::ColorImage;

use super::TextSize;

pub trait BytesParser {
    fn parse(&self, bytes: &[u8], size: &TextSize) -> ColorImage;
}

impl BytesParser for fn(bytes: &[u8], size: &TextSize) -> ColorImage {
    fn parse(&self, bytes: &[u8], size: &TextSize) -> ColorImage {
        self(bytes, size)
    }
}
