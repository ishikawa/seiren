//! Backends translate MIR into graphics format.
use crate::{
    color::{RGBColor, WebColor},
    error::BackendError,
    geometry::{PathCommand, Point},
    layout::RouteGraph,
    mir,
};
use std::io::Write;
use svg::node::element;

pub trait Renderer {
    fn render(&self, doc: &mir::Document, writer: &mut impl Write) -> Result<(), BackendError>;
}

#[derive(Debug)]
pub struct SVGRenderer<'g> {
    // for debug
    pub edge_route_graph: Option<&'g RouteGraph>,
}

impl SVGRenderer<'_> {
    pub fn new() -> Self {
        Self {
            edge_route_graph: None,
        }
    }
}

impl Renderer for SVGRenderer<'_> {
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
                let Some(field_rect) = field_node.rect() else { return Err(BackendError::InvalidLayout(field_node_id)) };

                let x = field_rect.min_x();
                let y = field_rect.min_y();

                // background color: we use a clip path to adjust border radius.
                if let Some(bg_color) = &field.bg_color {
                    let field_bg = element::Rectangle::new()
                        .set("x", x)
                        .set("y", y)
                        .set("width", field_rect.width())
                        .set("height", field_rect.height())
                        .set("fill", bg_color.to_string())
                        .set("clip-path", format!("url(#{})", record_clip_path_id));
                    svg_doc = svg_doc.add(field_bg);
                }

                // border
                if field_index > 0 {
                    let mut line = element::Line::new()
                        .set("x1", x)
                        .set("x2", field_rect.max_x())
                        .set("y1", y)
                        .set("y2", y);
                    if let Some(border_color) = &field.border_color {
                        line = line
                            .set("stroke", border_color.to_string())
                            .set("stroke-width", 1);
                    }
                    svg_doc = svg_doc.add(line);
                }

                // Renders text elements
                //
                // ```svgbob
                // +-------------+-------------+---------+
                // |<---- 2 ---->|<---- 2 ---->|<-- 1 -->|
                // | title       |    subtitle |  badge  |
                // +-------------+-------------+---------+
                // ```
                let column_width = field_rect.width() / 5.0;

                // title
                let text_element = self.draw_text(
                    &field.title,
                    Point::new(x + px, field_rect.mid_y()),
                    Some(SVGAnchor::Start),
                );
                svg_doc = svg_doc.add(text_element);

                // subtitle
                if let Some(subtitle) = &field.subtitle {
                    let text_element = self.draw_text(
                        subtitle,
                        Point::new(x + column_width * 4.0, field_rect.mid_y()),
                        Some(SVGAnchor::End),
                    );
                    svg_doc = svg_doc.add(text_element);
                }

                // badge
                if let Some(badge) = &field.badge {
                    let rx = field_rect.max_x() - px;
                    let cy = field_rect.mid_y();
                    let bg_radius = (field_rect.height() / 2.0) - 6.0;

                    if let Some(bg_color) = &badge.bg_color {
                        let bg_element = element::Circle::new()
                            .set("cx", rx - bg_radius)
                            .set("cy", cy)
                            .set("r", bg_radius)
                            .set("fill", bg_color.to_string());
                        svg_doc = svg_doc.add(bg_element);
                    }

                    let text_element = self.draw_text(
                        &badge.into_text_span(),
                        Point::new(rx - bg_radius, cy),
                        Some(SVGAnchor::Middle),
                    );
                    svg_doc = svg_doc.add(text_element);
                }
            }
        }

        // -- Draw edges
        for edge in doc.edges() {
            let (edge_path, start_circle, end_circle) = self.draw_edge_connection(edge)?;
            svg_doc = svg_doc.add(edge_path).add(start_circle).add(end_circle);
        }

        // -- Draw debug info
        if let Some(edge_route_graph) = self.edge_route_graph {
            // Draw route edges
            for junction in edge_route_graph.nodes() {
                if let Some(edges) = edge_route_graph.edges(&junction.id()) {
                    for edge in edges {
                        let Some(dest) = edge_route_graph.get_node(edge.dest()) else { continue };

                        let line = element::Line::new()
                            .set("x1", junction.location().x)
                            .set("y1", junction.location().y)
                            .set("x2", dest.location().x)
                            .set("y2", dest.location().y)
                            .set("stroke", "red")
                            .set("stroke-width", 1);

                        svg_doc = svg_doc.add(line);
                    }
                }
            }

            // Draw junction nodes
            let circle_radius = 4.0;

            for junction in edge_route_graph.nodes() {
                let circle = element::Circle::new()
                    .set("cx", junction.location().x)
                    .set("cy", junction.location().y)
                    .set("r", circle_radius)
                    .set("stroke", "white")
                    .set("stroke-width", 1)
                    .set("fill", "red");

                svg_doc = svg_doc.add(circle);
            }
        }

        writer.write_all(svg_doc.to_string().as_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SVGAnchor {
    Start,
    #[allow(dead_code)]
    Middle,
    End,
}

impl SVGAnchor {
    pub fn text_anchor(&self) -> String {
        match self {
            SVGAnchor::Start => "start".into(),
            SVGAnchor::Middle => "middle".into(),
            SVGAnchor::End => "end".into(),
        }
    }
}

impl SVGRenderer<'_> {
    fn draw_text(
        &self,
        span: &mir::TextSpan,
        origin: Point,
        text_anchor: Option<SVGAnchor>,
    ) -> element::Text {
        let mut label = element::Text::new()
            .set("x", origin.x)
            .set("y", origin.y)
            .set("dominant-baseline", "middle")
            .add(svg::node::Text::new(span.text.clone()));

        if let Some(text_anchor) = text_anchor {
            label = label.set("text-anchor", text_anchor.text_anchor());
        }
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
            label = label.set("font-size", font_size.to_string());
        }

        label
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
