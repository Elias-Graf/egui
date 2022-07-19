use std::{
    error::Error,
    fmt::{Debug, Display},
};

use egui::ColorImage;

use super::TextSize;

pub trait BytesParser {
    fn parse(&self, bytes: &[u8], size: &TextSize) -> Result<ColorImage, BytesParserErr>;
}

impl<T> BytesParser for T
where
    T: Fn(&[u8], &TextSize) -> Result<ColorImage, BytesParserErr>,
{
    fn parse(&self, bytes: &[u8], size: &TextSize) -> Result<ColorImage, BytesParserErr> {
        self(bytes, size)
    }
}

#[cfg(feature = "image")]
#[cfg(any(feature = "png", feature = "jpg"))]
macro_rules! new_decoder {
    ($decoder:ty, $bytes:ident) => {
        match <$decoder>::new($bytes) {
            Ok(d) => d,
            Err(e) => return Err(BytesParserErr::Unknown(format!("{}", e))),
        }
    };
}

#[cfg(feature = "image")]
#[cfg(any(feature = "png", feature = "jpg"))]
fn with_image_create_decoder<'a, D: image::ImageDecoder<'a>>(
    decoder: D,
) -> Result<ColorImage, BytesParserErr> {
    let dyn_img = image::DynamicImage::from_decoder(decoder)
        .map_err(|e| BytesParserErr::Unknown(format!("{}", e)))?;
    let size = [dyn_img.width() as _, dyn_img.height() as _];
    let img_buff = dyn_img.to_rgba8();
    let flat_buff = img_buff.as_flat_samples();

    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        flat_buff.as_slice(),
    ))
}

/// Loads the png using the [`image`] library.
///
/// Note that this parser ignores the passed size, as a png already has a fixed
/// one.
#[cfg(all(feature = "image", feature = "png"))]
pub fn png_bytes_parser(bytes: &[u8], _: &TextSize) -> Result<ColorImage, BytesParserErr> {
    with_image_create_decoder(new_decoder!(image::codecs::png::PngDecoder<_>, bytes))
}

/// Loads the jpg using the [`image`] library.
///
/// Note that this parser ignores the passed size, as a png already has a fixed
/// one.
#[cfg(all(feature = "image", feature = "jpeg"))]
pub fn jpg_bytes_parser(bytes: &[u8], _: &TextSize) -> Result<ColorImage, BytesParserErr> {
    with_image_create_decoder(new_decoder!(image::codecs::jpeg::JpegDecoder<_>, bytes))
}

#[cfg(feature = "svg")]
pub fn svg_bytes_parser(bytes: &[u8], size: &TextSize) -> Result<ColorImage, BytesParserErr> {
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(&bytes, &options.to_ref())
        .map_err(|e| BytesParserErr::Unknown(format!("{}", e)))?;
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
    );

    Ok(egui::ColorImage::from_rgba_unmultiplied([width as _, height as _], pixmap.data()))
}

#[derive(Debug)]
pub enum BytesParserErr {
    Unknown(String),
}

impl Error for BytesParserErr {}

impl Display for BytesParserErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
