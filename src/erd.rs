use svg;
use svg::node::element;

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

    pub fn into_svg(&self) -> svg::Document {
        let x = 50;
        let y = 80;
        let px = 12;
        let line_height = 35;
        let text_baseline = 22;
        let header_height = line_height;
        let border_radius = 6;
        let light_gray_color = "#494949";
        let text_color = "white";
        let table_width = 300;
        let table_space = 80;
        let header_clip_path_id_prefix = "header-clip-path-";

        // Build a SVG document
        let mut doc = svg::Document::new().set("version", "1.1");
        let mut defs = element::Definitions::new();

        // Background
        let background = element::Rectangle::new()
            .set("width", "100%")
            .set("height", "100%")
            .set("fill", "#1c1c1c");

        doc = doc.add(background);

        // defs
        for (table_index, _) in self.tables.iter().enumerate() {
            let x = x + (table_width + table_space) * table_index;

            // header clip path
            let rect = element::Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", table_width)
                .set("height", header_height);

            let header_clip_path_id = format!("{}{}", header_clip_path_id_prefix, table_index);
            let clip_path = element::ClipPath::new()
                .set("id", header_clip_path_id)
                .add(rect);

            defs = defs.add(clip_path);
        }

        doc = doc.add(defs);

        // shapes
        for (table_index, table) in self.tables.iter().enumerate() {
            let x = x + ((table_width + table_space) * table_index);
            // +1 for header
            let table_height = line_height * (table.columns.len() + 1);

            let header_clip_path_id = format!("{}{}", header_clip_path_id_prefix, table_index);
            let header_bg = element::Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", table_width)
                .set("height", table_height)
                .set("rx", border_radius)
                .set("ry", border_radius)
                .set("stroke", light_gray_color)
                .set("fill", light_gray_color)
                .set("clip-path", format!("url(#{})", header_clip_path_id));

            let header_text = element::Text::new()
                .set("x", x + px)
                .set("y", y + text_baseline)
                .set("fill", text_color)
                .set("font-weight", "bold")
                .set("font-family", "Monaco,Lucida Console,monospace")
                .add(svg::node::Text::new(table.name.clone()));

            // Table
            let table_bg = element::Rectangle::new()
                .set("x", x)
                .set("y", y)
                .set("width", table_width)
                .set("height", table_height)
                .set("rx", border_radius)
                .set("ry", border_radius)
                .set("stroke", light_gray_color)
                .set("fill", "#212121");

            doc = doc.add(table_bg).add(header_bg).add(header_text);

            // columns
            let base = y + header_height;

            for (column_index, column) in table.columns.iter().enumerate() {
                if column_index > 0 {
                    let y = base + line_height * column_index;
                    let line = element::Line::new()
                        .set("x1", x)
                        .set("x2", x + table_width)
                        .set("y1", y)
                        .set("y2", y)
                        .set("stroke", light_gray_color)
                        .set("stroke-width", 1);
                    doc = doc.add(line);
                }

                let label = element::Text::new()
                    .set("x", x + px)
                    .set("y", (base + text_baseline) + line_height * column_index)
                    .set("fill", text_color)
                    .set("font-weight", "lighter")
                    .set("font-family", "Courier New,monospace")
                    .add(svg::node::Text::new(column.name.clone()));
                doc = doc.add(label);
            }
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

    #[test]
    fn empty_doc() {
        let doc = ERDiagram::new();
        let svg = doc.into_svg();

        assert_eq!(svg.get_name(), "svg");
        assert_eq!(
            svg.get_attributes().get("version").map(|x| x.to_string()),
            Some("1.1".into())
        );
        assert_eq!(
            svg.get_attributes().get("xmlns").map(|x| x.to_string()),
            Some("http://www.w3.org/2000/svg".into())
        );
    }
}
