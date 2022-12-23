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
    let doc = svg::Document::new()
        .set("version", "1.1")
        .add(defs.add(clip_path))
        .add(background)
        .add(table_bg)
        .add(header_bg);

    println!("{}", doc);
}
