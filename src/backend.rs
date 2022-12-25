//! Backends translate MIR into graphics format.
use std::io::Write;

use svg::node::element;

use crate::{
    color::{RGBColor, WebColor},
    error::BackendError,
    mir::{self, NodeKind},
};

pub trait Backend {
    fn generate(&self, doc: &mir::Document, writer: &mut impl Write) -> Result<(), BackendError>;
}

#[derive(Debug)]
pub struct SVGBackend {}

impl SVGBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl Backend for SVGBackend {
    fn generate(&self, doc: &mir::Document, writer: &mut impl Write) -> Result<(), BackendError> {
        let px = 12f32;
        let line_height = 35f32;
        let text_baseline = 22f32;
        let border_radius = 6f32;
        let header_clip_path_id_prefix = "header-clip-path-";
        let background_color = WebColor::RGB(RGBColor::new(28, 28, 28));

        // -- Build a SVG document
        let mut svg_doc = svg::Document::new().set("version", "1.1");
        let mut svg_defs = element::Definitions::new();

        // -- Background
        let background_rect = element::Rectangle::new()
            .set("width", "100%")
            .set("height", "100%")
            .set("fill", background_color.to_string());

        svg_doc = svg_doc.add(background_rect);

        // -- Generate clip paths in defs
        for (record_index, child_id) in doc.body().children().enumerate() {
            let Some(record_node) = doc.get_node(&child_id) else { continue };
            let NodeKind::Record(record) = record_node.kind() else  { continue };

            if record.header.is_none() {
                continue;
            }

            let Some(origin) = record_node.origin else { return Err(BackendError::InvalidLayout(child_id)) };
            let Some(size) = record_node.size else { return Err(BackendError::InvalidLayout(child_id)) };

            // header clip path
            let rect = element::Rectangle::new()
                .set("x", origin.x.to_string())
                .set("y", origin.y.to_string())
                .set("width", size.width.to_string())
                .set("height", line_height);

            let header_clip_path_id = format!("{}{}", header_clip_path_id_prefix, record_index);
            let clip_path = element::ClipPath::new()
                .set("id", header_clip_path_id)
                .add(rect);

            svg_defs = svg_defs.add(clip_path);
        }
        svg_doc = svg_doc.add(svg_defs);

        // -- Generate shapes
        for (record_index, child_id) in doc.body().children().enumerate() {
            let Some(record_node) = doc.get_node(&child_id) else { continue };
            let NodeKind::Record(record) = record_node.kind() else  { continue };
            let Some(origin) = record_node.origin else { return Err(BackendError::InvalidLayout(child_id)) };
            let Some(size) = record_node.size else { return Err(BackendError::InvalidLayout(child_id)) };

            // background
            let table_bg = element::Rectangle::new()
                .set("x", origin.x.to_string())
                .set("y", origin.y.to_string())
                .set("width", size.width.to_string())
                .set("height", size.height.to_string())
                .set("rx", border_radius)
                .set("ry", border_radius)
                .set("stroke", record.border_color.to_string())
                .set("fill", record.bg_color.to_string());
            svg_doc = svg_doc.add(table_bg);

            // header
            if let Some(header) = &record.header {
                let header_clip_path_id = format!("{}{}", header_clip_path_id_prefix, record_index);
                let header_bg = element::Rectangle::new()
                    .set("x", origin.x.to_string())
                    .set("y", origin.y.to_string())
                    .set("width", size.width.to_string())
                    .set("height", size.height.to_string())
                    .set("rx", border_radius)
                    .set("ry", border_radius)
                    .set("stroke", header.bg_color.to_string())
                    .set("fill", header.bg_color.to_string())
                    .set("clip-path", format!("url(#{})", header_clip_path_id));
                let header_text = element::Text::new()
                    .set("x", origin.x + px)
                    .set("y", origin.y + text_baseline)
                    .set("fill", header.text_color.to_string())
                    .set("font-weight", "bold")
                    .set("font-family", "Monaco,Lucida Console,monospace")
                    .add(svg::node::Text::new(header.title.clone()));
                svg_doc = svg_doc.add(header_bg).add(header_text);
            }

            // children
            for (field_index, field_node_id) in record_node.children().enumerate() {
                let Some(field_node) = doc.get_node(&field_node_id) else { continue };
                let NodeKind::Field(field) = field_node.kind() else  { continue };
                let Some(field_origin) = field_node.origin else { return Err(BackendError::InvalidLayout(field_node_id)) };
                let Some(field_size) = field_node.size else { return Err(BackendError::InvalidLayout(field_node_id)) };

                let x = origin.x + field_origin.x;
                let y = origin.y + field_origin.y;

                // border
                if field_index > 0 {
                    let line = element::Line::new()
                        .set("x1", x)
                        .set("x2", x + field_size.width)
                        .set("y1", y)
                        .set("y2", y)
                        .set("stroke", record.border_color.to_string())
                        .set("stroke-width", 1);
                    svg_doc = svg_doc.add(line);
                }

                let label = element::Text::new()
                    .set("x", x + px)
                    .set("y", y + text_baseline)
                    .set("fill", field.text_color.to_string())
                    .set("font-weight", "lighter")
                    .set("font-family", "Courier New,monospace")
                    .add(svg::node::Text::new(field.name.clone()));
                svg_doc = svg_doc.add(label);
            }
        }

        writer.write_all(svg_doc.to_string().as_bytes())?;
        Ok(())
    }
}
