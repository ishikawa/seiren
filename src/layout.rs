//! Layout engine
use crate::{
    geometry::{Point, Size},
    mir::{self, NodeKind},
};

pub trait LayoutEngine {
    fn layout_nodes(&self, doc: &mut mir::Document);
}

#[derive(Debug)]
pub struct SimpleLayoutEngine {}

impl SimpleLayoutEngine {
    pub fn new() -> Self {
        Self {}
    }
}

impl LayoutEngine for SimpleLayoutEngine {
    fn layout_nodes(&self, doc: &mut mir::Document) {
        let x = 50f32;
        let y = 80f32;
        let line_height = 35f32;
        let header_height = line_height;
        let record_width = 300f32;
        let record_space = 80f32;

        // Iterate records
        let child_id_vec = doc.body().children().collect::<Vec<_>>();

        for (record_index, child_id) in child_id_vec.iter().enumerate() {
            let Some(record_node) = doc.get_node_mut(child_id) else { continue };
            let NodeKind::Record(record) = record_node.kind() else  { continue };

            let n_records = record_node.children().len() as f32;
            let has_header = record.header.is_some();
            let x = x + (record_width + record_space) * record_index as f32;

            // TODO: We can remove header and teat it as a field that has background color.
            // +1 for header
            let record_height = if has_header {
                line_height * (n_records + 1.0)
            } else {
                line_height * n_records
            };

            record_node.origin = Some(Point::new(x, y));
            record_node.size = Some(Size::new(record_width.into(), record_height.into()));

            // children
            let field_id_vec = record_node.children().collect::<Vec<_>>();
            let base = if has_header { header_height } else { 0.0 };

            for (field_index, field_node_id) in field_id_vec.iter().enumerate() {
                let y = base + line_height * field_index as f32;
                let Some(field_node) = doc.get_node_mut(field_node_id) else { continue };
                let NodeKind::Field(_) = field_node.kind() else  { continue };

                field_node.origin = Some(Point::new(0.0, y));
                field_node.size = Some(Size::new(record_width, line_height));
            }
        }
    }
}
