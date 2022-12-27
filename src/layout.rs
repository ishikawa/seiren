//! Layout engine
use crate::{
    geometry::{Point, Size},
    mir::{self, NodeKind},
};

pub trait LayoutEngine {
    /// Place all nodes on 2D coordination.
    ///
    /// The engine must assign `origin` and `size` of all nodes.
    fn execute_node_layout(&self, doc: &mut mir::Document);

    /// Place all connection points for every node.
    ///
    /// The engine must add all possible connection points to `connection_points` of nodes.
    fn execute_connection_point_layout(&self, doc: &mut mir::Document);
}

#[derive(Debug)]
pub struct SimpleLayoutEngine {}

impl SimpleLayoutEngine {
    pub fn new() -> Self {
        Self {}
    }
}

impl LayoutEngine for SimpleLayoutEngine {
    fn execute_node_layout(&self, doc: &mut mir::Document) {
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

    fn execute_connection_point_layout(&self, doc: &mut mir::Document) {
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
}
