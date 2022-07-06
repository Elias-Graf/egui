use egui::ColorImage;

use super::TextSize;

// TODO: the bytes parser should return [`Result`].

pub trait BytesParser {
    fn parse(&self, bytes: &[u8], size: &TextSize) -> ColorImage;
}

impl<T> BytesParser for T
where
    T: Fn(&[u8], &TextSize) -> ColorImage,
{
    fn parse(&self, bytes: &[u8], size: &TextSize) -> ColorImage {
        self(bytes, size)
    }
}

#[cfg(feature = "svg")]
pub fn svg_bytes_parser(bytes: &[u8], size: &TextSize) -> ColorImage {
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(&bytes, &options.to_ref()).unwrap();
    let (width, height) = *size;

    // TODO: Make size optional, and read it from the svg if possible.
    // let (width, height) = match size {
    //     Some(size) => *size,
    //     None => {
    //         let svg_node_size = tree.svg_node().size;

    //         (
    //             svg_node_size.width() as usize,
    //             svg_node_size.height() as usize,
    //         )
    //     }
    // };

    let mut pixmap = tiny_skia::Pixmap::new(width as u32, height as u32).unwrap();

    resvg::render(
        &tree,
        usvg::FitTo::Size(width as u32, height as u32),
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();

    egui::ColorImage::from_rgba_unmultiplied([width as _, height as _], pixmap.data())
}
