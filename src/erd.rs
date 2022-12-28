//! ER diagram AST
use crate::color::{NamedColor, RGBColor, WebColor};
use crate::mir;
use derive_more::Display;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ERDiagram {
    tables: Vec<Table>,
    relations: Vec<Relation>,
}

impl ERDiagram {
    pub fn new() -> Self {
        Self {
            tables: vec![],
            relations: vec![],
        }
    }

    pub fn tables(&self) -> impl ExactSizeIterator<Item = &Table> {
        self.tables.iter()
    }

    pub fn relations(&self) -> impl ExactSizeIterator<Item = &Relation> {
        self.relations.iter()
    }

    pub fn add_table(&mut self, table: Table) {
        self.tables.push(table);
    }

    pub fn add_relation(&mut self, relation: Relation) {
        self.relations.push(relation);
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
                    .title(name)
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

                    let column_type = mir::TextSpanBuilder::default()
                        .text(column.r#type.to_string())
                        .color(ERDiagram::column_type_color(&column.r#type))
                        .font_family(mir::FontFamily::Monospace2)
                        .font_weight(mir::FontWeight::Lighter)
                        .font_size(mir::FontSize::Small)
                        .build()
                        .unwrap();

                    let field = mir::FieldNodeBuilder::default()
                        .title(name)
                        .subtitle(column_type)
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

    fn column_type_color(column_type: &ColumnType) -> WebColor {
        let yellow = WebColor::RGB(RGBColor {
            red: 236,
            green: 199,
            blue: 0,
        });
        let orange = WebColor::RGB(RGBColor {
            red: 214,
            green: 105,
            blue: 5,
        });
        let green = WebColor::RGB(RGBColor {
            red: 6,
            green: 182,
            blue: 151,
        });

        match column_type {
            ColumnType::Int => yellow.clone(),
            ColumnType::UUID => yellow.clone(),
            ColumnType::Text => orange.clone(),
            ColumnType::Timestamp => green.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    name: String,
    columns: Vec<Column>,
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

    pub fn columns(&self) -> impl ExactSizeIterator<Item = &Column> {
        self.columns.iter()
    }

    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum ColumnType {
    #[display(fmt = "int")]
    Int,
    #[display(fmt = "uuid")]
    UUID,
    #[display(fmt = "text")]
    Text,
    #[display(fmt = "timestamp")]
    Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display)]
pub enum ColumnKey {
    #[display(fmt = "PK")]
    PrimaryKey,
    #[display(fmt = "FK")]
    ForeginKey,
}

#[derive(Debug, Clone)]
pub struct Column {
    name: String,
    r#type: ColumnType,
    key: Option<ColumnKey>,
}

impl Column {
    pub fn new(name: String, r#type: ColumnType, key: Option<ColumnKey>) -> Self {
        Self { name, r#type, key }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn column_type(&self) -> &ColumnType {
        &self.r#type
    }

    pub fn column_key(&self) -> Option<&ColumnKey> {
        self.key.as_ref()
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
