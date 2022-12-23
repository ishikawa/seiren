use svg;
use svg::node::element;

fn main() {
    let width = 300;
    let x = 50;
    let y = 80;
    let table_height = 247;
    let header_height = 36;
    let border_radius = 6;
    let light_gray_color = "#494949";
    let text_color = "white";
    let header_text = "users";
    let rows = ["id", "uuid"];

    // clip path
    let rect = element::Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", width)
        .set("height", header_height);

    let header_clip_path_id = "cut-off-table-header";
    let clip_path = element::ClipPath::new()
        .set("id", header_clip_path_id)
        .add(rect);

    let header_bg = element::Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", width)
        .set("height", table_height)
        .set("rx", border_radius)
        .set("ry", border_radius)
        .set("stroke", light_gray_color)
        .set("fill", light_gray_color)
        .set("clip-path", format!("url(#{})", header_clip_path_id));

    let header_text = element::Text::new()
        .set("x", x + 12)
        .set("y", y + 23)
        .set("fill", text_color)
        .set("font-weight", "bold")
        .set("font-family", "Monaco,monospace")
        .add(svg::node::Text::new(header_text));

    // Background
    let background = element::Rectangle::new()
        .set("width", "100%")
        .set("height", "100%")
        .set("fill", "#1c1c1c");

    // Table
    let table_bg = element::Rectangle::new()
        .set("x", x)
        .set("y", y)
        .set("width", width)
        .set("height", 247)
        .set("rx", border_radius)
        .set("ry", border_radius)
        .set("stroke", light_gray_color)
        .set("fill", "#212121");

    // doc
    let defs = element::Definitions::new();
    let mut doc = svg::Document::new()
        .set("version", "1.1")
        .add(defs.add(clip_path))
        .add(background)
        .add(table_bg)
        .add(header_bg)
        .add(header_text);

    // rows
    let base = y + header_height;

    for (i, row) in rows.iter().enumerate() {
        if i > 0 {
            let y = base + 34 * i;
            let line = element::Line::new()
                .set("x1", x)
                .set("x2", x + width)
                .set("y1", y)
                .set("y2", y)
                .set("stroke", light_gray_color)
                .set("stroke-width", 1);
            doc = doc.add(line);
        }

        let label = element::Text::new()
            .set("x", x + 12)
            .set("y", (base + 22) + 35 * i)
            .set("fill", text_color)
            .set("font-weight", "lighter")
            .set("font-family", "Courier New,monospace")
            .add(svg::node::Text::new(*row));
        doc = doc.add(label);
    }

    println!("{}", doc);
}
