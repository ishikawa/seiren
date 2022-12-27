//! Layout engine
use std::collections::VecDeque;

use crate::{
    geometry::{Path, Point, Size},
    mir::{self, NodeKind},
};

pub trait LayoutEngine {
    /// Place all nodes on 2D coordination.
    ///
    /// The engine must assign `origin` and `size` of all nodes.
    fn place_nodes(&self, doc: &mut mir::Document);

    /// Place all connection points for every node.
    ///
    /// The engine must add all possible connection points to `connection_points` of nodes.
    fn place_connection_points(&self, doc: &mut mir::Document);

    /// Draw path between both ends (connection points) of each edge.
    ///
    /// The engine must build a `path` of edges.
    fn draw_edge_path(&self, doc: &mut mir::Document);
}

#[derive(Debug)]
pub struct SimpleLayoutEngine {}

impl SimpleLayoutEngine {
    pub fn new() -> Self {
        Self {}
    }
}

impl LayoutEngine for SimpleLayoutEngine {
    fn place_nodes(&self, doc: &mut mir::Document) {
        let x = 50f32;
        let y = 80f32;
        let line_height = 35f32;
        let record_width = 300f32;
        let record_space = 80f32;

        // Iterate records
        let child_id_vec = doc.body().children().collect::<Vec<_>>();

        for (record_index, child_id) in child_id_vec.iter().enumerate() {
            let Some(record_node) = doc.get_node_mut(child_id) else { continue };
            let NodeKind::Record(_) = record_node.kind() else  { continue };

            let n_records = record_node.children().len() as f32;
            let x = x + (record_width + record_space) * record_index as f32;

            let record_height = line_height * n_records;

            record_node.origin = Some(Point::new(x, y));
            record_node.size = Some(Size::new(record_width.into(), record_height.into()));

            // children
            let base_y = y;
            let field_id_vec = record_node.children().collect::<Vec<_>>();

            for (field_index, field_node_id) in field_id_vec.iter().enumerate() {
                let y = base_y + line_height * field_index as f32;
                let Some(field_node) = doc.get_node_mut(field_node_id) else { continue };
                let NodeKind::Field(_) = field_node.kind() else  { continue };

                field_node.origin = Some(Point::new(x, y));
                field_node.size = Some(Size::new(record_width, line_height));
            }
        }
    }

    fn place_connection_points(&self, doc: &mut mir::Document) {
        let child_id_vec = doc.body().children().collect::<Vec<_>>();

        for (_, child_id) in child_id_vec.iter().enumerate() {
            let Some(record_node) = doc.get_node_mut(child_id) else { continue };
            let Some(record_rect) = record_node.rect() else { continue };

            // In the case of a rectangle, connection points are placed in
            // the center of each of the four edges.
            for pt in [
                Point::new(record_rect.mid_x(), record_rect.min_y()), // top
                Point::new(record_rect.max_x(), record_rect.mid_y()), // right
                Point::new(record_rect.mid_x(), record_rect.max_y()), // bottom
                Point::new(record_rect.min_x(), record_rect.mid_y()), // left
            ] {
                record_node.append_connection_point(pt);
            }

            // For each field in a rectangle, connection points are placed
            // the center of:
            // - each of the four edges - if the number of fields is `1`.
            // - top, left and right - for the top field
            // - bottom, left and right - for the bottom field
            // - left and right - for the rest
            let field_id_vec = record_node.children().collect::<Vec<_>>();

            for (field_index, field_node_id) in field_id_vec.iter().enumerate() {
                let Some(field_node) = doc.get_node_mut(field_node_id) else { continue };
                let Some(field_rect) = field_node.rect() else { continue };

                if field_id_vec.len() == 1 {
                    for pt in [
                        Point::new(field_rect.mid_x(), field_rect.min_y()), // top
                        Point::new(field_rect.max_x(), field_rect.mid_y()), // right
                        Point::new(field_rect.mid_x(), field_rect.max_y()), // bottom
                        Point::new(field_rect.min_x(), field_rect.mid_y()), // left
                    ] {
                        field_node.append_connection_point(pt);
                    }
                } else if field_index == 0 {
                    for pt in [
                        Point::new(field_rect.mid_x(), field_rect.min_y()), // top
                        Point::new(field_rect.max_x(), field_rect.mid_y()), // right
                        Point::new(field_rect.min_x(), field_rect.mid_y()), // left
                    ] {
                        field_node.append_connection_point(pt);
                    }
                } else if field_index == (field_id_vec.len() - 1) {
                    for pt in [
                        Point::new(field_rect.max_x(), field_rect.mid_y()), // right
                        Point::new(field_rect.mid_x(), field_rect.max_y()), // bottom
                        Point::new(field_rect.min_x(), field_rect.mid_y()), // left
                    ] {
                        field_node.append_connection_point(pt);
                    }
                } else {
                    for pt in [
                        Point::new(field_rect.max_x(), field_rect.mid_y()), // right
                        Point::new(field_rect.min_x(), field_rect.mid_y()), // left
                    ] {
                        field_node.append_connection_point(pt);
                    }
                }
            }
        }
    }

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
    fn draw_edge_path(&self, doc: &mut mir::Document) {
        let path_radius = 6.0;

        let mut paths: VecDeque<Path> = VecDeque::with_capacity(doc.edges().len());

        for edge in doc.edges() {
            let Some(start_node) = doc.get_node(&edge.start_node_id) else { continue };
            let Some(end_node) = doc.get_node(&edge.end_node_id) else { continue };

            // Choose the combination with the shortest distance between two connection points.
            let mut connection_points = (
                // start point
                Point::default(),
                // end point
                Point::default(),
                // distance
                f32::MAX,
            );

            for pt1 in start_node.connection_points() {
                for pt2 in end_node.connection_points() {
                    let d = pt1.distance(pt2);
                    if d < connection_points.2 {
                        connection_points = (pt1.clone(), pt2.clone(), d);
                    }
                }
            }

            // Build a path.
            let start_cx = connection_points.0.x;
            let end_cx = connection_points.1.x;
            let start_cy = connection_points.0.y;
            let end_cy = connection_points.1.y;

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

            let mut path = Path::new(connection_points.0);

            path.line_to(Point::new(ctrl1_x, start_cy));
            path.quad_to(Point::new(mid_x, start_cy), Point::new(mid_x, ctrl1_y));
            path.line_to(Point::new(mid_x, ctrl2_y));
            path.quad_to(Point::new(mid_x, end_cy), Point::new(ctrl2_x, end_cy));
            path.line_to(Point::new(end_cx, end_cy));

            paths.push_back(path);
        }

        for edge in doc.edges_mut() {
            edge.path = Some(paths.pop_front().unwrap());
        }
    }
}
