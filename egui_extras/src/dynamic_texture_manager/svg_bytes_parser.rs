#[cfg(feature = "svg")]
pub const SVG_BYTES_PARSER: super::BytesParser = |bytes, size| {
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_data(&bytes, &options.to_ref()).unwrap();

    let (width, height) = match size {
        Some(size) => *size,
        None => {
            let svg_node_size = tree.svg_node().size;

            (
                svg_node_size.width() as usize,
                svg_node_size.height() as usize,
            )
        }
    };

    let mut pixmap = tiny_skia::Pixmap::new(width as u32, height as u32).unwrap();

    resvg::render(
        &tree,
        usvg::FitTo::Size(width as u32, height as u32),
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .unwrap();

    egui::ColorImage::from_rgba_unmultiplied([width as _, height as _], pixmap.data())
};
