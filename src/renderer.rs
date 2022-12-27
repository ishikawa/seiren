//! Backends translate MIR into graphics format.
use crate::{
    color::{RGBColor, WebColor},
    error::BackendError,
    geometry::{PathCommand, Point},
    mir,
};
use std::io::Write;
use svg::node::element;

pub trait Renderer {
    fn render(&self, doc: &mir::Document, writer: &mut impl Write) -> Result<(), BackendError>;
}

#[derive(Debug)]
pub struct SVGRenderer {}

impl SVGRenderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for SVGRenderer {
    fn render(&self, doc: &mir::Document, writer: &mut impl Write) -> Result<(), BackendError> {
        let px = 12f32;
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
            let mir::NodeKind::Record(_) = record_node.kind() else  { continue };

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
            let mir::NodeKind::Record(record) = record_node.kind() else  { continue };
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
                let mir::NodeKind::Field(field) = field_node.kind() else  { continue };
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
                let label = self.draw_text(&field.name, Point::new(x + px, y));
                svg_doc = svg_doc.add(label);
            }
        }

        // -- Draw edges
        for edge in doc.edges() {
            let (edge_path, start_circle, end_circle) = self.draw_edge_connection(edge)?;
            svg_doc = svg_doc.add(edge_path).add(start_circle).add(end_circle);
        }

        writer.write_all(svg_doc.to_string().as_bytes())?;
        Ok(())
    }
}

impl SVGRenderer {
    /// Returns the distance between text origin (top-left) and baseline.
    ///
    /// ```svgbob
    ///  o
    ///  !
    ///  !
    ///  v
    ///  +-------------
    /// ```
    ///
    fn text_height(size: &mir::FontSize) -> f32 {
        match size {
            mir::FontSize::XXSmall => todo!(),
            mir::FontSize::XSmall => todo!(),
            mir::FontSize::Small => todo!(),
            mir::FontSize::Medium => 22.0,
            mir::FontSize::Large => todo!(),
            mir::FontSize::XLarge => todo!(),
            mir::FontSize::XXLarge => todo!(),
            mir::FontSize::XXXLarge => todo!(),
        }
    }

    fn draw_text(&self, span: &mir::TextSpan, origin: Point) -> element::Text {
        let mut text_baseline = SVGRenderer::text_height(&mir::FontSize::default());

        let mut label = element::Text::new().add(svg::node::Text::new(span.text.clone()));

        if let Some(text_color) = &span.color {
            label = label.set("fill", text_color.to_string());
        }
        if let Some(font_family) = &span.font_family {
            label = label.set("font-family", font_family.to_string());
        }
        if let Some(font_weight) = &span.font_weight {
            label = label.set("font-weight", font_weight.to_string());
        }

        // position
        if let Some(font_size) = &span.font_size {
            text_baseline = SVGRenderer::text_height(font_size);
            label = label.set("font-size", font_size.to_string());
        }

        label.set("x", origin.x).set("y", origin.y + text_baseline)
    }

    fn draw_edge_connection(
        &self,
        edge: &mir::Edge,
    ) -> Result<(element::Path, element::Circle, element::Circle), BackendError> {
        let circle_radius = 4.0;
        let stroke_width = 1.5;
        let stroke_color = WebColor::RGB(RGBColor {
            red: 136,
            green: 136,
            blue: 136,
        });
        let background_color = WebColor::RGB(RGBColor::new(28, 28, 28));

        let Some(path) = &edge.path else {
            return Err(BackendError::InvalidLayout(edge.start_node_id))
        };

        // Draw circles at both ends of the edge.
        let start_point = path.start_point();
        let end_point = path.end_point();

        let start_circle = element::Circle::new()
            .set("cx", start_point.x)
            .set("cy", start_point.y)
            .set("r", circle_radius)
            .set("stroke", stroke_color.to_string())
            .set("stroke-width", stroke_width)
            .set("fill", background_color.to_string());
        let end_circle = element::Circle::new()
            .set("cx", end_point.x)
            .set("cy", end_point.y)
            .set("r", circle_radius)
            .set("stroke", stroke_color.to_string())
            .set("stroke-width", stroke_width)
            .set("fill", background_color.to_string());

        let mut d = vec![];

        for command in path.commands() {
            let c = match command {
                PathCommand::MoveTo(pt) => format!("M{} {}", pt.x, pt.y),
                PathCommand::LineTo(pt) => format!("L{} {}", pt.x, pt.y),
                PathCommand::QuadTo(ctrl, to) => {
                    format!("Q{} {} {} {}", ctrl.x, ctrl.y, to.x, to.y)
                }
            };

            d.push(c);
        }

        let svg_path = element::Path::new()
            .set("stroke", stroke_color.to_string())
            .set("stroke-width", stroke_width)
            .set("fill", "transparent")
            .set("d", d.join(" "));

        Ok((svg_path, start_circle, end_circle))
    }
}
