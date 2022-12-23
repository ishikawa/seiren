use svg;

fn main() {
    let rect = svg::node::element::Rectangle::new()
        .set("x", 50)
        .set("y", 80)
        .set("width", 300)
        .set("height", 36);

    let clip_path = svg::node::element::ClipPath::new()
        .set("id", "cut-off-table-header")
        .add(rect);

    let defs = svg::node::element::Definitions::new().add(clip_path);

    let doc = svg::Document::new().set("version", "1.1").add(defs);

    println!("{}", doc);
}
