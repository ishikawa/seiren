//! Backends translate MIR into graphics format.
use crate::{
    color::{RGBColor, WebColor},
    error::BackendError,
    geometry::{Direction, Point},
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
        let circle_radius = 4.0;

        if let Some(edge_route_graph) = self.edge_route_graph {
            // Draw route edges with direction
            for junction in edge_route_graph.nodes() {
                let Some(edges) = edge_route_graph.edges(junction.id()) else { continue };
                let from_pt = junction.location();

                for edge in edges {
                    let Some(dest) = edge_route_graph.get_node(edge.dest()) else { continue };
                    let to_pt = dest.location();

                    let line = element::Line::new()
                        .set("x1", from_pt.x)
                        .set("y1", from_pt.y)
                        .set("x2", to_pt.x)
                        .set("y2", to_pt.y)
                        .set("stroke", "red")
                        .set("stroke-width", 1);

                    // arrow
                    let (x, y) = (to_pt.x, to_pt.y);
                    let width = 5.0 / 2.0;
                    let height = 7.0;
                    let points = match from_pt.vh_direction(to_pt) {
                        Direction::Up => [
                            (x, y + circle_radius),
                            (x - width, y + height + circle_radius),
                            (x + width, y + height + circle_radius),
                        ],
                        Direction::Down => [
                            (x, y - circle_radius),
                            (x - width, y - height - circle_radius),
                            (x + width, y - height - circle_radius),
                        ],
                        Direction::Left => [
                            (x + circle_radius, y),
                            (x + height + circle_radius, y + width),
                            (x + height + circle_radius, y - width),
                        ],
                        Direction::Right => [
                            (x - circle_radius, y),
                            (x - height - circle_radius, y + width),
                            (x - height - circle_radius, y - width),
                        ],
                    };

                    points
                        .iter()
                        .map(|p| format!("{}, {}", p.0, p.1))
                        .collect::<Vec<_>>()
                        .join(" ");

                    let arrow = element::Polygon::new().set("fill", "red").set(
                        "points",
                        points
                            .iter()
                            .map(|p| format!("{}, {}", p.0, p.1))
                            .collect::<Vec<_>>()
                            .join(" "),
                    );

                    svg_doc = svg_doc.add(line).add(arrow);
                }
            }

            // Draw junction nodes
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

            // Draw shortest paths
            for edge in doc.edges() {
                let Some(path_points) = &edge.path_points else { continue };

                for p in path_points {
                    let circle = element::Circle::new()
                        .set("cx", p.x)
                        .set("cy", p.y)
                        .set("r", circle_radius)
                        .set("stroke", "white")
                        .set("stroke-width", 1)
                        .set("fill", "orange");
                    svg_doc = svg_doc.add(circle);
                }
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
        let path_radius = 6.0;
        let stroke_width = 1.5;
        let stroke_color = WebColor::RGB(RGBColor {
            red: 136,
            green: 136,
            blue: 136,
        });
        let background_color = WebColor::RGB(RGBColor::new(28, 28, 28));

        let Some(path_points) = &edge.path_points else {
            return Err(BackendError::InvalidLayout(edge.start_node_id))
        };
        assert!(path_points.len() >= 2);

        // Draw circles at both ends of the edge.
        let start_point = path_points[0];
        let end_point = path_points.last().unwrap();

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

        // When you draw the line, trace edge's `path_points` and look at the points before and
        // after to determine the path to draw.
        //
        // ```svgbob
        // 0 - - - - - - - - - - - - - - - - - - - - - ->
        // ! -------+
        // !        |       (1)
        // !    (0) o--------*--o
        // !        |           |
        // !        |           * (1)
        // !        |           |
        // !        |           |
        // !        |           |
        // !        |       (2) *                +------
        // !        |           | (2)    (3)     |
        // !        |           o--*------o------o (4)
        // v        |                            |
        // ```

        let mut d = vec![];

        for i in 0..path_points.len() {
            let pt = path_points[i];

            if i == 0 {
                d.push(format!("M{} {}", pt.x, pt.y));
            } else if i == path_points.len() - 1 {
                d.push(format!("L{} {}", pt.x, pt.y));
            } else {
                let bp = path_points[i - 1]; // backward
                let fp = path_points[i + 1]; // forward

                let d1 = bp.vh_direction(&pt);
                let d2 = pt.vh_direction(&fp);

                match (d1, d2) {
                    (Direction::Up, Direction::Up)
                    | (Direction::Down, Direction::Down)
                    | (Direction::Left, Direction::Left)
                    | (Direction::Right, Direction::Right) => {
                        // same direction
                        d.push(format!("L{} {}", pt.x, pt.y));
                    }
                    (Direction::Up, Direction::Down)
                    | (Direction::Down, Direction::Up)
                    | (Direction::Left, Direction::Right)
                    | (Direction::Right, Direction::Left) => {
                        // A turnaround line is invalid
                        panic!("turnaround line is detected at #{}", i);
                    }
                    (Direction::Up, Direction::Left) => {
                        // ```svgbob
                        //  o<--------*--o (pt)
                        // (fp)          |
                        //               *
                        //               |
                        //               |
                        //               o (bp)
                        // ```
                        d.push(format!("L{} {}", pt.x, pt.y + path_radius));
                        d.push(format!(
                            "Q{} {} {} {}",
                            pt.x,
                            pt.y,
                            pt.x - path_radius,
                            pt.y
                        ));
                    }
                    (Direction::Right, Direction::Down) => {
                        // ```svgbob
                        //  o---------*--o (pt)
                        // (bp)          |
                        //               *
                        //               |
                        //               v
                        //               o (fp)
                        // ```
                        d.push(format!("L{} {}", pt.x - path_radius, pt.y));
                        d.push(format!(
                            "Q{} {} {} {}",
                            pt.x,
                            pt.y,
                            pt.x,
                            pt.y + path_radius
                        ));
                    }
                    (Direction::Up, Direction::Right) => {
                        // ```svgbob
                        //  o--*------->o (fp)
                        //  | (pt)
                        //  *
                        //  |
                        //  |
                        //  o (bp)
                        // ```
                        d.push(format!("L{} {}", pt.x, pt.y + path_radius));
                        d.push(format!(
                            "Q{} {} {} {}",
                            pt.x,
                            pt.y,
                            pt.x + path_radius,
                            pt.y
                        ));
                    }
                    (Direction::Down, Direction::Left) => {
                        // ```svgbob
                        //              o (bp)
                        //              |
                        //              |
                        //              *
                        //              |
                        //  o<-------*--o (pt)
                        // (fp)
                        // ```
                        d.push(format!("L{} {}", pt.x, pt.y - path_radius));
                        d.push(format!(
                            "Q{} {} {} {}",
                            pt.x,
                            pt.y,
                            pt.x - path_radius,
                            pt.y
                        ));
                    }
                    (Direction::Down, Direction::Right) => {
                        // ```svgbob
                        // (bp)
                        //  o
                        //  |
                        //  |
                        //  *
                        //  |
                        //  o---*------->o (fp)
                        // (pt)
                        // ```
                        d.push(format!("L{} {}", pt.x, pt.y - path_radius));
                        d.push(format!(
                            "Q{} {} {} {}",
                            pt.x,
                            pt.y,
                            pt.x + path_radius,
                            pt.y
                        ));
                    }
                    (Direction::Left, Direction::Up) => {
                        // ```svgbob
                        // (fp)
                        //  o
                        //  ^
                        //  |
                        //  *
                        //  |
                        //  o---*--------o (bp)
                        // (pt)
                        // ```
                        d.push(format!("L{} {}", pt.x + path_radius, pt.y));
                        d.push(format!(
                            "Q{} {} {} {}",
                            pt.x,
                            pt.y,
                            pt.x,
                            pt.y - path_radius
                        ));
                    }
                    (Direction::Left, Direction::Down) => {
                        // ```svgbob
                        //  o<-*--------o (bp)
                        //  | (pt)
                        //  *
                        //  |
                        //  v
                        //  o (fp)
                        // ```
                        d.push(format!("L{} {}", pt.x + path_radius, pt.y));
                        d.push(format!(
                            "Q{} {} {} {}",
                            pt.x,
                            pt.y,
                            pt.x,
                            pt.y + path_radius
                        ));
                    }
                    (Direction::Right, Direction::Up) => {
                        // ```svgbob
                        //              o (fp)
                        //              ^
                        //              |
                        //              *
                        //              |
                        //  o--------*--o (pt)
                        // (bp)
                        // ```
                        d.push(format!("L{} {}", pt.x - path_radius, pt.y));
                        d.push(format!(
                            "Q{} {} {} {}",
                            pt.x,
                            pt.y,
                            pt.x,
                            pt.y - path_radius
                        ));
                    }
                };
            }
        }

        let svg_path = element::Path::new()
            .set("stroke", stroke_color.to_string())
            .set("stroke-width", stroke_width)
            .set("fill", "transparent")
            .set("d", d.join(" "));

        Ok((svg_path, start_circle, end_circle))
    }
}
