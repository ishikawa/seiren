use std::io;

use seiren::{
    erd::{Column, ColumnKey, ColumnType, ERDiagram, Relation, RelationPath, Table},
    layout::{LayoutEngine, SimpleLayoutEngine},
    mir::Document,
    renderer::{Renderer, SVGRenderer},
};

fn main() {
    let doc = demo_erd();
    let backend = SVGRenderer::new();
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    backend
        .render(&doc, &mut handle)
        .expect("cannot generate SVG");
}

fn demo_erd() -> Document {
    let mut diagram = ERDiagram::new();
    let mut users_table = Table::new("users".into());
    let mut posts_table = Table::new("posts".into());

    // users
    users_table.add_column(Column::new(
        "id".into(),
        ColumnType::Int,
        Some(ColumnKey::PrimaryKey),
    ));
    users_table.add_column(Column::new("uuid".into(), ColumnType::UUID, None));
    users_table.add_column(Column::new("email".into(), ColumnType::Text, None));
    users_table.add_column(Column::new("about_html".into(), ColumnType::Text, None));
    users_table.add_column(Column::new(
        "created_at".into(),
        ColumnType::Timestamp,
        None,
    ));

    // posts
    posts_table.add_column(Column::new(
        "id".into(),
        ColumnType::Int,
        Some(ColumnKey::PrimaryKey),
    ));
    posts_table.add_column(Column::new("uuid".into(), ColumnType::UUID, None));
    posts_table.add_column(Column::new("title".into(), ColumnType::Text, None));
    posts_table.add_column(Column::new("content".into(), ColumnType::Text, None));
    posts_table.add_column(Column::new(
        "created_at".into(),
        ColumnType::Timestamp,
        None,
    ));
    posts_table.add_column(Column::new(
        "created_by".into(),
        ColumnType::Int,
        Some(ColumnKey::ForeginKey),
    ));

    diagram.add_table(users_table);
    diagram.add_table(posts_table);
    diagram.add_relation(Relation::new(
        RelationPath::Column("posts".into(), "created_by".into()),
        RelationPath::Column("users".into(), "id".into()),
    ));

    let mut doc = diagram.into_mir();
    let engine = SimpleLayoutEngine::new();

    engine.place_nodes(&mut doc);
    engine.place_connection_points(&mut doc);
    engine.draw_edge_path(&mut doc);

    doc
}

#[cfg(test)]
mod tests {
    use crate::demo_erd;
    use difference::assert_diff;
    use seiren::renderer::{Renderer, SVGRenderer};

    #[test]
    fn demo_svg() {
        let doc = demo_erd();

        let backend = SVGRenderer::new();
        let mut bytes: Vec<u8> = vec![];

        backend
            .render(&doc, &mut bytes)
            .expect("cannot generate SVG");

        let svg = String::from_utf8(bytes).unwrap();
        assert_diff!(svg.as_str(), "<svg version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\">
<rect fill=\"#1C1C1C\" height=\"100%\" width=\"100%\"/>
<defs>
<clipPath id=\"record-clip-path-0\">
<rect height=\"210\" rx=\"6\" ry=\"6\" width=\"300\" x=\"50\" y=\"80\"/>
</clipPath>
<clipPath id=\"record-clip-path-1\">
<rect height=\"245\" rx=\"6\" ry=\"6\" width=\"300\" x=\"430\" y=\"80\"/>
</clipPath>
</defs>
<rect fill=\"#212121\" height=\"210\" rx=\"6\" ry=\"6\" stroke=\"#494949\" width=\"300\" x=\"50\" y=\"80\"/>
<rect clip-path=\"url(#record-clip-path-0)\" fill=\"#494949\" height=\"35\" width=\"300\" x=\"50\" y=\"80\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Monaco,Lucida Console,monospace\" font-weight=\"bold\" x=\"62\" y=\"97.5\">
users
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"50\" x2=\"350\" y1=\"115\" y2=\"115\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" x=\"62\" y=\"132.5\">
id
</text>
<text dominant-baseline=\"middle\" fill=\"#ECC700\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" x=\"200\" y=\"132.5\">
int
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"50\" x2=\"350\" y1=\"150\" y2=\"150\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" x=\"62\" y=\"167.5\">
uuid
</text>
<text dominant-baseline=\"middle\" fill=\"#ECC700\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" x=\"200\" y=\"167.5\">
uuid
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"50\" x2=\"350\" y1=\"185\" y2=\"185\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" x=\"62\" y=\"202.5\">
email
</text>
<text dominant-baseline=\"middle\" fill=\"#D66905\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" x=\"200\" y=\"202.5\">
text
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"50\" x2=\"350\" y1=\"220\" y2=\"220\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" x=\"62\" y=\"237.5\">
about_html
</text>
<text dominant-baseline=\"middle\" fill=\"#D66905\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" x=\"200\" y=\"237.5\">
text
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"50\" x2=\"350\" y1=\"255\" y2=\"255\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" x=\"62\" y=\"272.5\">
created_at
</text>
<text dominant-baseline=\"middle\" fill=\"#06B697\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" x=\"200\" y=\"272.5\">
timestamp
</text>
<rect fill=\"#212121\" height=\"245\" rx=\"6\" ry=\"6\" stroke=\"#494949\" width=\"300\" x=\"430\" y=\"80\"/>
<rect clip-path=\"url(#record-clip-path-1)\" fill=\"#494949\" height=\"35\" width=\"300\" x=\"430\" y=\"80\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Monaco,Lucida Console,monospace\" font-weight=\"bold\" x=\"442\" y=\"97.5\">
posts
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"430\" x2=\"730\" y1=\"115\" y2=\"115\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" x=\"442\" y=\"132.5\">
id
</text>
<text dominant-baseline=\"middle\" fill=\"#ECC700\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" x=\"580\" y=\"132.5\">
int
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"430\" x2=\"730\" y1=\"150\" y2=\"150\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" x=\"442\" y=\"167.5\">
uuid
</text>
<text dominant-baseline=\"middle\" fill=\"#ECC700\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" x=\"580\" y=\"167.5\">
uuid
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"430\" x2=\"730\" y1=\"185\" y2=\"185\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" x=\"442\" y=\"202.5\">
title
</text>
<text dominant-baseline=\"middle\" fill=\"#D66905\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" x=\"580\" y=\"202.5\">
text
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"430\" x2=\"730\" y1=\"220\" y2=\"220\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" x=\"442\" y=\"237.5\">
content
</text>
<text dominant-baseline=\"middle\" fill=\"#D66905\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" x=\"580\" y=\"237.5\">
text
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"430\" x2=\"730\" y1=\"255\" y2=\"255\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" x=\"442\" y=\"272.5\">
created_at
</text>
<text dominant-baseline=\"middle\" fill=\"#06B697\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" x=\"580\" y=\"272.5\">
timestamp
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"430\" x2=\"730\" y1=\"290\" y2=\"290\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" x=\"442\" y=\"307.5\">
created_by
</text>
<text dominant-baseline=\"middle\" fill=\"#ECC700\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" x=\"580\" y=\"307.5\">
int
</text>
<path d=\"M430 307.5 L396 307.5 Q390 307.5 390 301.5 L390 138.5 Q390 132.5 384 132.5 L350 132.5\" fill=\"transparent\" stroke=\"#888888\" stroke-width=\"1.5\"/>
<circle cx=\"430\" cy=\"307.5\" fill=\"#1C1C1C\" r=\"4\" stroke=\"#888888\" stroke-width=\"1.5\"/>
<circle cx=\"350\" cy=\"132.5\" fill=\"#1C1C1C\" r=\"4\" stroke=\"#888888\" stroke-width=\"1.5\"/>
</svg>", "\n", 0);
    }
}
