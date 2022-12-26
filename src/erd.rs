use crate::color::{NamedColor, RGBColor, WebColor};
use crate::mir;

#[derive(Debug, Clone)]
pub struct ERDiagram {
    pub tables: Vec<Table>,
    pub edges: Vec<Relation>,
}

impl ERDiagram {
    pub fn new() -> Self {
        Self {
            tables: vec![],
            edges: vec![],
        }
    }

    pub fn find_table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name() == name)
    }

    pub fn into_mir(&self) -> mir::Document {
        let light_gray_color = WebColor::RGB(RGBColor::new(73, 73, 73));
        let table_bg_color = WebColor::RGB(RGBColor::new(33, 33, 33));
        let text_color = WebColor::Named(NamedColor::White);
        let mut doc = mir::Document::new();

        for table in self.tables.iter() {
            let header = mir::RecordNodeHeaderBuilder::default()
                .title(table.name())
                .text_color(text_color.clone())
                .bg_color(light_gray_color.clone())
                .build()
                .unwrap();

            let record = mir::RecordNodeBuilder::default()
                .header(header)
                .rounded(true)
                .bg_color(table_bg_color.clone())
                .border_color(light_gray_color.clone())
                .build()
                .unwrap();

            let field_ids: Vec<_> = table
                .columns
                .iter()
                .map(|column| {
                    let field = mir::FieldNodeBuilder::default()
                        .name(column.name())
                        .text_color(text_color.clone())
                        .build()
                        .unwrap();

                    doc.create_field(field)
                })
                .collect();

            let record_id = doc.create_record(record);
            let record_node = doc.get_node_mut(&record_id).unwrap();

            for field_id in field_ids {
                record_node.append_child(field_id);
            }

            doc.body_mut().append_child(record_id);
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Hash)]
pub enum RelationItem {
    Table(String),
    Column(String, String),
}

#[derive(Debug, Clone)]
pub struct Relation {
    start_node: RelationItem,
    end_node: RelationItem,
}

impl Relation {
    pub fn new(from: RelationItem, to: RelationItem) -> Self {
        Self {
            start_node: from,
            end_node: to,
        }
    }

    pub fn start_node(&self) -> &RelationItem {
        &self.start_node
    }

    pub fn end_node(&self) -> &RelationItem {
        &self.end_node
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        backend::{Backend, SVGBackend},
        layout::{LayoutEngine, SimpleLayoutEngine},
    };

    #[test]
    fn empty_doc() {
        let diagram = ERDiagram::new();
        let mut doc = diagram.into_mir();

        let engine = SimpleLayoutEngine::new();

        engine.layout_nodes(&mut doc);

        let backend = SVGBackend::new();
        let mut bytes: Vec<u8> = vec![];

        backend.generate(&doc, &mut bytes).expect("generate SVG");

        let svg = String::from_utf8(bytes).unwrap();

        assert_eq!(svg, "<svg version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\">\n<rect fill=\"#1C1C1C\" height=\"100%\" width=\"100%\"/>\n<defs/>\n</svg>");
    }
}
