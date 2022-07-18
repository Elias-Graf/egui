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

/// Loads the png using the [`image`] library.
///
/// Note that this parser ignores the passed size, as a png already has a fixed
/// one.
#[cfg(all(feature = "image", feature = "png"))]
pub fn png_bytes_parser(bytes: &[u8], _: &TextSize) -> ColorImage {
    // TODO: Remove unwrap (the parser should return a result in the future anyway)
    image_create_parser(image::codecs::png::PngDecoder::new(bytes).unwrap())
}

/// Loads the jpg using the [`image`] library.
///
/// Note that this parser ignores the passed size, as a png already has a fixed
/// one.
#[cfg(all(feature = "image", feature = "jpeg"))]
pub fn jpg_bytes_parser(bytes: &[u8], _: &TextSize) -> ColorImage {
    // TODO: Remove unwrap (the parser should return a result in the future anyway)
    image_create_parser(image::codecs::jpeg::JpegDecoder::new(bytes).unwrap())
}

#[cfg(feature = "image")]
#[cfg(any(feature = "png", feature = "jpg"))]
fn image_create_parser<'a, D: image::ImageDecoder<'a>>(decoder: D) -> ColorImage {
    // TODO: Remove unwrap (the parser should return a result in the future anyway);
    let dyn_img = image::DynamicImage::from_decoder(decoder).unwrap();
    let size = [dyn_img.width() as _, dyn_img.height() as _];
    let img_buff = dyn_img.to_rgba8();
    let flat_buff = img_buff.as_flat_samples();

    egui::ColorImage::from_rgba_unmultiplied(size, flat_buff.as_slice())
}

pub fn svg_bytes_parser(bytes: &[u8], size: &TextSize) -> ColorImage {
    let options = usvg::Options::default();
    // TODO: remove unwrap (the parser should return a result in the future)
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
