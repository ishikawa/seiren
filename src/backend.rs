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
        let text_baseline = 22f32;
        let border_radius = 6f32;
        let record_clip_path_id_prefix = "record-clip-path-";
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

        // -- Generate clip paths for record shapes.
        for (record_index, child_id) in doc.body().children().enumerate() {
            let Some(record_node) = doc.get_node(&child_id) else { continue };
            let NodeKind::Record(_) = record_node.kind() else  { continue };

            let Some(record_origin) = record_node.origin else { return Err(BackendError::InvalidLayout(child_id)) };
            let Some(record_size) = record_node.size else { return Err(BackendError::InvalidLayout(child_id)) };

            let clip_path_rect = element::Rectangle::new()
                .set("x", record_origin.x)
                .set("y", record_origin.y)
                .set("width", record_size.width)
                .set("height", record_size.height)
                .set("rx", border_radius)
                .set("ry", border_radius);

            let id = format!("{}{}", record_clip_path_id_prefix, record_index);
            let clip_path = element::ClipPath::new().set("id", id).add(clip_path_rect);

            svg_defs = svg_defs.add(clip_path);
        }
        svg_doc = svg_doc.add(svg_defs);

        // -- Draw shapes
        for (record_index, child_id) in doc.body().children().enumerate() {
            let Some(record_node) = doc.get_node(&child_id) else { continue };
            let NodeKind::Record(record) = record_node.kind() else  { continue };
            let Some(record_origin) = record_node.origin else { return Err(BackendError::InvalidLayout(child_id)) };
            let Some(record_size) = record_node.size else { return Err(BackendError::InvalidLayout(child_id)) };

            // background
            let mut table_bg = element::Rectangle::new()
                .set("x", record_origin.x)
                .set("y", record_origin.y)
                .set("width", record_size.width)
                .set("height", record_size.height)
                .set("rx", border_radius)
                .set("ry", border_radius);
            if let Some(border_color) = &record.border_color {
                table_bg = table_bg.set("stroke", border_color.to_string());
            }
            if let Some(bg_color) = &record.bg_color {
                table_bg = table_bg.set("fill", bg_color.to_string());
            }
            svg_doc = svg_doc.add(table_bg);

            // children
            let record_clip_path_id = format!("{}{}", record_clip_path_id_prefix, record_index);

            for (field_index, field_node_id) in record_node.children().enumerate() {
                let Some(field_node) = doc.get_node(&field_node_id) else { continue };
                let NodeKind::Field(field) = field_node.kind() else  { continue };
                let Some(field_origin) = field_node.origin else { return Err(BackendError::InvalidLayout(field_node_id)) };
                let Some(field_size) = field_node.size else { return Err(BackendError::InvalidLayout(field_node_id)) };

                let x = field_origin.x;
                let y = field_origin.y;

                // background color: we use a clip path to adjust border radius.
                if let Some(bg_color) = &field.bg_color {
                    let field_bg = element::Rectangle::new()
                        .set("x", x)
                        .set("y", y)
                        .set("width", field_size.width)
                        .set("height", field_size.height)
                        .set("fill", bg_color.to_string())
                        .set("clip-path", format!("url(#{})", record_clip_path_id));
                    svg_doc = svg_doc.add(field_bg);
                }

                // border
                if field_index > 0 {
                    let mut line = element::Line::new()
                        .set("x1", x)
                        .set("x2", x + field_size.width)
                        .set("y1", y)
                        .set("y2", y);
                    if let Some(border_color) = &field.border_color {
                        line = line
                            .set("stroke", border_color.to_string())
                            .set("stroke-width", 1);
                    }
                    svg_doc = svg_doc.add(line);
                }

                // text
                let span = &field.name;
                let mut label = element::Text::new()
                    .set("x", x + px)
                    .set("y", y + text_baseline)
                    .add(svg::node::Text::new(span.text.clone()));

                if let Some(text_color) = &span.color {
                    label = label.set("fill", text_color.to_string());
                }
                if let Some(font_family) = &span.font_family {
                    label = label.set("font-family", font_family.to_string());
                }
                if let Some(font_weight) = &span.font_weight {
                    label = label.set("font-weight", font_weight.to_string());
                }

                svg_doc = svg_doc.add(label);
            }
        }

        // -- Draw edges
        for edge in doc.edges() {
            let Some(start_node) = doc.get_node(&edge.start_node_id) else { continue };
            let Some(end_node) = doc.get_node(&edge.end_node_id) else { continue };
            let start_circle = self.edge_end_circle(start_node, end_node)?;
            let end_circle = self.edge_end_circle(end_node, start_node)?;

            svg_doc = svg_doc.add(start_circle).add(end_circle);
        }

        writer.write_all(svg_doc.to_string().as_bytes())?;
        Ok(())
    }
}

impl SVGBackend {
    fn edge_end_circle(
        &self,
        node: &mir::Node,
        target_node: &mir::Node,
    ) -> Result<element::Circle, BackendError> {
        let r = 4;
        let stroke_width = 2;
        let color = WebColor::RGB(RGBColor {
            red: 136,
            green: 136,
            blue: 136,
        });
        let background_color = WebColor::RGB(RGBColor::new(28, 28, 28));

        let (Some(origin), Some(size)) = (node.origin, node.size) else {
            return Err(BackendError::InvalidLayout(node.id))
        };
        let (Some(min_x), Some(max_x), Some(target_min_x), Some(_target_max_x)) = (
            node.min_x(),
            node.max_x(),
            target_node.min_x(),
            target_node.max_x()) else { return Err(BackendError::InvalidLayout(node.id)) };

        let is_left_side = target_min_x < min_x;

        let cx = if is_left_side { min_x } else { max_x };
        let cy = origin.y + size.height / 2.0;

        Ok(element::Circle::new()
            .set("cx", cx)
            .set("cy", cy)
            .set("r", r)
            .set("stroke", color.to_string())
            .set("stroke-width", stroke_width)
            .set("fill", background_color.to_string()))
    }
}
