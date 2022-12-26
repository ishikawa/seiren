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

            let (edge_path, start_circle, end_circle) = self.edge_path(start_node, end_node)?;

            svg_doc = svg_doc.add(edge_path).add(start_circle).add(end_circle);
        }

        writer.write_all(svg_doc.to_string().as_bytes())?;
        Ok(())
    }
}

impl SVGBackend {
    ///
    /// ```svgbob
    /// 0 - - - - - - - - - - - - - - - - - - - ->
    /// ! -------+
    /// !        |  ctrl1(x)  middle
    /// !  start o--------*--.
    /// !        |           |
    /// !        |           * ctrl1(y)
    /// !        |           |
    /// !        |           |
    /// !        |           |
    /// !        |  ctrl2(y) *           +-------
    /// !        |           | ctrl2(x)  |
    /// !        |           `--*--------o end
    /// !        |                       |
    /// ```
    fn edge_path(
        &self,
        start_node: &mir::Node,
        end_node: &mir::Node,
    ) -> Result<(element::Path, element::Circle, element::Circle), BackendError> {
        let circle_radius = 4.0;
        let path_radius = 6.0;
        let stroke_width = 1.5;
        let stroke_color = WebColor::RGB(RGBColor {
            red: 136,
            green: 136,
            blue: 136,
        });
        let background_color = WebColor::RGB(RGBColor::new(28, 28, 28));

        let (Some(start_origin), Some(start_size)) = (start_node.origin, start_node.size) else {
            return Err(BackendError::InvalidLayout(start_node.id))
        };
        let (Some(end_origin), Some(end_size)) = (end_node.origin, end_node.size) else {
            return Err(BackendError::InvalidLayout(end_node.id))
        };
        let (Some(start_min_x), Some(start_max_x), Some(end_min_x), Some(end_max_x)) = (
            start_node.min_x(),
            start_node.max_x(),
            end_node.min_x(),
            end_node.max_x()) else { return Err(BackendError::InvalidLayout(end_node.id)) };

        // Choose the combination with the shortest distance between two points in x-axis.
        let x1 = (start_min_x - end_min_x).abs(); // left:left
        let x2 = (start_min_x - end_max_x).abs(); // left:right
        let x3 = (start_max_x - end_min_x).abs(); // right:left
        let x4 = (start_max_x - end_max_x).abs(); // right:right

        let start_cy = start_origin.y + start_size.height / 2.0;
        let end_cy = end_origin.y + end_size.height / 2.0;

        let (start_cx, end_cx) = if x1 <= x2 && x1 <= x3 && x1 <= x4 {
            // left:left
            (start_min_x, end_min_x)
        } else if x2 <= x1 && x2 <= x3 && x2 <= x4 {
            // left:right
            (start_min_x, end_max_x)
        } else if x3 <= x1 && x3 <= x2 && x3 <= x4 {
            // right:left
            (start_max_x, end_min_x)
        } else {
            // right:right
            (start_max_x, end_max_x)
        };

        // Draw circles at both ends of the edge.
        let start_circle = element::Circle::new()
            .set("cx", start_cx)
            .set("cy", start_cy)
            .set("r", circle_radius)
            .set("stroke", stroke_color.to_string())
            .set("stroke-width", stroke_width)
            .set("fill", background_color.to_string());
        let end_circle = element::Circle::new()
            .set("cx", end_cx)
            .set("cy", end_cy)
            .set("r", circle_radius)
            .set("stroke", stroke_color.to_string())
            .set("stroke-width", stroke_width)
            .set("fill", background_color.to_string());

        // Draw path
        let mid_x = start_cx.min(end_cx) + (start_cx - end_cx).abs() / 2.0;

        let (ctrl1_x, ctrl2_x) = if start_cx < end_cx {
            (mid_x - path_radius, mid_x + path_radius)
        } else {
            (mid_x + path_radius, mid_x - path_radius)
        };
        let (ctrl1_y, ctrl2_y) = if start_cy < end_cy {
            (start_cy + path_radius, end_cy - path_radius)
        } else {
            (start_cy - path_radius, end_cy + path_radius)
        };

        let path = element::Path::new()
            .set("stroke", stroke_color.to_string())
            .set("stroke-width", stroke_width)
            .set("fill", "transparent")
            .set(
                "d",
                vec![
                    format!("M{} {}", start_cx, start_cy),
                    format!("L{} {}", ctrl1_x, start_cy),
                    format!("Q{} {} {} {}", mid_x, start_cy, mid_x, ctrl1_y),
                    format!("L{} {}", mid_x, ctrl2_y),
                    format!("Q{} {} {} {}", mid_x, end_cy, ctrl2_x, end_cy),
                    format!("L{} {}", end_cx, end_cy),
                ]
                .join(" "),
            );

        Ok((path, start_circle, end_circle))
    }
}
