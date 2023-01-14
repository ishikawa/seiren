//! Evcxr extension
use crate::{
    geometry::Rect,
    mir,
    renderer::{Renderer, SVGRenderer},
};

/// See https://github.com/google/evcxr/blob/main/evcxr_jupyter/README.md#custom-output
pub trait CustomDisplay {
    fn evcxr_display(&self);
}

pub struct SVGOutput {
    svg: String,
}

impl SVGOutput {
    pub fn new(svg: String) -> Self {
        Self { svg }
    }
}

impl CustomDisplay for SVGOutput {
    fn evcxr_display(&self) {
        println!(
            "EVCXR_BEGIN_CONTENT image/svg+xml\n{}\nEVCXR_END_CONTENT",
            self.svg
        );
    }
}

pub fn draw_erd(doc: &mir::Document, view_box: Option<Rect>) -> impl CustomDisplay {
    let mut backend: SVGRenderer = SVGRenderer::new();
    let mut bytes: Vec<u8> = vec![];

    backend.view_box = view_box;
    backend
        .render(&doc, &mut bytes)
        .expect("cannot generate SVG");

    let svg = String::from_utf8(bytes).unwrap();
    SVGOutput::new(svg)
}
