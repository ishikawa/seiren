use svg;
use svg::node::element;

#[derive(Debug, Clone)]
pub struct Document {
    pub tables: Vec<Table>,
}

impl Document {
    pub fn new() -> Self {
        Self { tables: vec![] }
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

impl From<Document> for svg::Document {
    fn from(value: Document) -> Self {
        value.into_svg()
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
}

impl Table {
    pub fn new(name: String) -> Self {
        Self {
            name,
            columns: vec![],
        }
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
    pub name: String,
    pub r#type: ColumnType,
}

impl Column {
    pub fn new(name: String, r#type: ColumnType) -> Self {
        Self { name, r#type }
    }
}

fn main() {
    let mut doc = Document::new();
    let mut users_table = Table::new("users".into());
    let mut posts_table = Table::new("posts".into());

    // users
    users_table
        .columns
        .push(Column::new("id".into(), ColumnType::Int));
    users_table
        .columns
        .push(Column::new("uuid".into(), ColumnType::Uuid));
    users_table
        .columns
        .push(Column::new("email".into(), ColumnType::Text));
    users_table
        .columns
        .push(Column::new("about_html".into(), ColumnType::Text));
    users_table
        .columns
        .push(Column::new("created_at".into(), ColumnType::Timestamp));

    // posts
    posts_table
        .columns
        .push(Column::new("id".into(), ColumnType::Int));
    posts_table
        .columns
        .push(Column::new("uuid".into(), ColumnType::Uuid));
    posts_table
        .columns
        .push(Column::new("title".into(), ColumnType::Text));
    posts_table
        .columns
        .push(Column::new("content".into(), ColumnType::Text));
    posts_table
        .columns
        .push(Column::new("created_at".into(), ColumnType::Timestamp));
    posts_table
        .columns
        .push(Column::new("created_by".into(), ColumnType::Int));

    doc.tables.push(users_table);
    doc.tables.push(posts_table);

    println!("{}", doc.into_svg());
}
