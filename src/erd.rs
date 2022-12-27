//! ER diagram AST
use crate::color::{NamedColor, RGBColor, WebColor};
use crate::mir;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ERDiagram {
    pub tables: Vec<Table>,
    pub relations: Vec<Relation>,
}

impl ERDiagram {
    pub fn new() -> Self {
        Self {
            tables: vec![],
            relations: vec![],
        }
    }

    pub fn find_table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name() == name)
    }

    pub fn into_mir(&self) -> mir::Document {
        let light_gray_color = WebColor::RGB(RGBColor::new(73, 73, 73));
        let table_border_color = light_gray_color.clone();
        let table_bg_color = WebColor::RGB(RGBColor::new(33, 33, 33));
        let text_color = WebColor::Named(NamedColor::White);
        let mut doc = mir::Document::new();

        // node path (e.g. ["users", "id"]) -> node ID
        let mut node_paths: HashMap<RelationPath, mir::NodeId> = HashMap::new();

        for table in self.tables.iter() {
            let header_node_id = {
                let name = mir::TextSpanBuilder::default()
                    .text(table.name())
                    .color(text_color.clone())
                    .font_family(mir::FontFamily::Monospace1)
                    .font_weight(mir::FontWeight::Bold)
                    .build()
                    .unwrap();
                let field = mir::FieldNodeBuilder::default()
                    .name(name)
                    .bg_color(light_gray_color.clone())
                    .build()
                    .unwrap();

                doc.create_field(field)
            };

            let record = mir::RecordNodeBuilder::default()
                .rounded(true)
                .bg_color(table_bg_color.clone())
                .border_color(table_border_color.clone())
                .build()
                .unwrap();

            let field_ids: Vec<_> = table
                .columns
                .iter()
                .map(|column| {
                    let name = mir::TextSpanBuilder::default()
                        .text(column.name())
                        .color(text_color.clone())
                        .font_family(mir::FontFamily::Monospace2)
                        .font_weight(mir::FontWeight::Lighter)
                        .build()
                        .unwrap();

                    let field = mir::FieldNodeBuilder::default()
                        .name(name)
                        .border_color(table_border_color.clone())
                        .build()
                        .unwrap();

                    let node_id = doc.create_field(field);

                    node_paths.insert(
                        RelationPath::Column(table.name().into(), column.name().into()),
                        node_id,
                    );
                    node_id
                })
                .collect();

            let record_id = doc.create_record(record);
            node_paths.insert(RelationPath::Table(table.name().into()), record_id);

            let record_node = doc.get_node_mut(&record_id).unwrap();

            record_node.append_child(header_node_id);
            for field_id in field_ids {
                record_node.append_child(field_id);
            }

            doc.body_mut().append_child(record_id);
        }

        // Translates relations to edges.
        for relation in self.relations.iter() {
            let Some(start_node_id) = node_paths.get(relation.start_path()) else { continue };
            let Some(end_node_id) = node_paths.get(relation.end_path()) else { continue };

            doc.append_edge(mir::Edge::new(*start_node_id, *end_node_id));
        }

        doc
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    name: String,
    pub columns: Vec<Column>,
}

impl Table {
    pub fn new(name: String) -> Self {
        Self {
            name,
            columns: vec![],
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn find_column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|c| c.name() == name)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ColumnType {
    Int,
    Uuid,
    Text,
    Timestamp,
}

#[derive(Debug, Clone)]
pub struct Column {
    name: String,
    pub r#type: ColumnType,
}

impl Column {
    pub fn new(name: String, r#type: ColumnType) -> Self {
        Self { name, r#type }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum RelationPath {
    Table(String),
    Column(String, String),
}

#[derive(Debug, Clone)]
pub struct Relation {
    start_path: RelationPath,
    end_path: RelationPath,
}

impl Relation {
    pub fn new(from: RelationPath, to: RelationPath) -> Self {
        Self {
            start_path: from,
            end_path: to,
        }
    }

    pub fn start_path(&self) -> &RelationPath {
        &self.start_path
    }

    pub fn end_path(&self) -> &RelationPath {
        &self.end_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        layout::{LayoutEngine, SimpleLayoutEngine},
        renderer::{Renderer, SVGRenderer},
    };

    #[test]
    fn empty_doc() {
        let diagram = ERDiagram::new();
        let mut doc = diagram.into_mir();

        let engine = SimpleLayoutEngine::new();

        engine.place_nodes(&mut doc);

        let backend = SVGRenderer::new();
        let mut bytes: Vec<u8> = vec![];

        backend.render(&doc, &mut bytes).expect("generate SVG");

        let svg = String::from_utf8(bytes).unwrap();

        assert_eq!(svg, "<svg version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\">\n<rect fill=\"#1C1C1C\" height=\"100%\" width=\"100%\"/>\n<defs/>\n</svg>");
    }
}
