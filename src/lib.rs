pub mod color;
pub mod erd;
pub mod error;
pub mod geometry;
pub mod layout;
pub mod mir;
pub mod parser;
pub mod renderer;

#[cfg(test)]
mod tests {
    use crate::{
        layout::{LayoutEngine, SimpleLayoutEngine},
        mir::Document,
        erd::{Module, EntityDefinition, EntityPath, EntityField, EntityFieldType, EntityFieldKey, EntityRelation},
        renderer::{Renderer, SVGRenderer},
    };
    use difference::assert_diff;

    #[test]
    fn empty_doc() {
        let diagram = Module::new(None);
        let mut doc = diagram.into_mir();

        let engine = SimpleLayoutEngine::new();

        engine.place_nodes(&mut doc);

        let backend = SVGRenderer::new();
        let mut bytes: Vec<u8> = vec![];

        backend.render(&doc, &mut bytes).expect("generate SVG");

        let svg = String::from_utf8(bytes).unwrap();

        assert_diff!(
            svg.as_str(), 
            "<svg version=\"1.1\" xmlns=\"http://www.w3.org/2000/svg\">\n<rect fill=\"#1C1C1C\" height=\"100%\" width=\"100%\"/>\n<defs/>\n</svg>", 
            "\n",
            0);
    }

    fn demo_erd() -> Document {
        let mut diagram = Module::new(None);
        let mut users_table = EntityDefinition::new("users".into());
        let mut posts_table = EntityDefinition::new("posts".into());

        // users
        users_table.add_field(EntityField::new(
            "id".into(),
            EntityFieldType::Int,
            Some(EntityFieldKey::PrimaryKey),
        ));
        users_table.add_field(EntityField::new("uuid".into(), EntityFieldType::Uuid, None));
        users_table.add_field(EntityField::new(
            "email".into(),
            EntityFieldType::Text,
            None,
        ));
        users_table.add_field(EntityField::new(
            "about_html".into(),
            EntityFieldType::Text,
            None,
        ));
        users_table.add_field(EntityField::new(
            "created_at".into(),
            EntityFieldType::Timestamp,
            None,
        ));

        // posts
        posts_table.add_field(EntityField::new(
            "id".into(),
            EntityFieldType::Int,
            Some(EntityFieldKey::PrimaryKey),
        ));
        posts_table.add_field(EntityField::new("uuid".into(), EntityFieldType::Uuid, None));
        posts_table.add_field(EntityField::new(
            "title".into(),
            EntityFieldType::Text,
            None,
        ));
        posts_table.add_field(EntityField::new(
            "content".into(),
            EntityFieldType::Text,
            None,
        ));
        posts_table.add_field(EntityField::new(
            "created_at".into(),
            EntityFieldType::Timestamp,
            None,
        ));
        posts_table.add_field(EntityField::new(
            "created_by".into(),
            EntityFieldType::Int,
            Some(EntityFieldKey::ForeginKey),
        ));

        diagram.add_entity_definition(users_table);
        diagram.add_entity_definition(posts_table);
        diagram.add_entity_relation(EntityRelation::new(
            EntityPath::Field("posts".into(), "created_by".into()),
            EntityPath::Field("users".into(), "id".into()),
        ));

        let mut doc = diagram.into_mir();
        let engine = SimpleLayoutEngine::new();

        engine.place_nodes(&mut doc);
        engine.place_connection_points(&mut doc);
        engine.draw_edge_path(&mut doc);

        doc
    }

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
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Monaco,Lucida Console,monospace\" font-weight=\"bold\" text-anchor=\"start\" x=\"62\" y=\"97.5\">
users
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"50\" x2=\"350\" y1=\"115\" y2=\"115\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" text-anchor=\"start\" x=\"62\" y=\"132.5\">
id
</text>
<text dominant-baseline=\"middle\" fill=\"#ECC700\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" text-anchor=\"end\" x=\"290\" y=\"132.5\">
int
</text>
<circle cx=\"326.5\" cy=\"132.5\" fill=\"#373737\" r=\"11.5\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Trebuchet MS,sans-serif\" font-size=\"xx-small\" text-anchor=\"middle\" x=\"326.5\" y=\"132.5\">
PK
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"50\" x2=\"350\" y1=\"150\" y2=\"150\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" text-anchor=\"start\" x=\"62\" y=\"167.5\">
uuid
</text>
<text dominant-baseline=\"middle\" fill=\"#ECC700\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" text-anchor=\"end\" x=\"290\" y=\"167.5\">
uuid
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"50\" x2=\"350\" y1=\"185\" y2=\"185\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" text-anchor=\"start\" x=\"62\" y=\"202.5\">
email
</text>
<text dominant-baseline=\"middle\" fill=\"#D66905\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" text-anchor=\"end\" x=\"290\" y=\"202.5\">
text
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"50\" x2=\"350\" y1=\"220\" y2=\"220\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" text-anchor=\"start\" x=\"62\" y=\"237.5\">
about_html
</text>
<text dominant-baseline=\"middle\" fill=\"#D66905\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" text-anchor=\"end\" x=\"290\" y=\"237.5\">
text
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"50\" x2=\"350\" y1=\"255\" y2=\"255\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" text-anchor=\"start\" x=\"62\" y=\"272.5\">
created_at
</text>
<text dominant-baseline=\"middle\" fill=\"#06B697\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" text-anchor=\"end\" x=\"290\" y=\"272.5\">
timestamp
</text>
<rect fill=\"#212121\" height=\"245\" rx=\"6\" ry=\"6\" stroke=\"#494949\" width=\"300\" x=\"430\" y=\"80\"/>
<rect clip-path=\"url(#record-clip-path-1)\" fill=\"#494949\" height=\"35\" width=\"300\" x=\"430\" y=\"80\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Monaco,Lucida Console,monospace\" font-weight=\"bold\" text-anchor=\"start\" x=\"442\" y=\"97.5\">
posts
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"430\" x2=\"730\" y1=\"115\" y2=\"115\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" text-anchor=\"start\" x=\"442\" y=\"132.5\">
id
</text>
<text dominant-baseline=\"middle\" fill=\"#ECC700\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" text-anchor=\"end\" x=\"670\" y=\"132.5\">
int
</text>
<circle cx=\"706.5\" cy=\"132.5\" fill=\"#373737\" r=\"11.5\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Trebuchet MS,sans-serif\" font-size=\"xx-small\" text-anchor=\"middle\" x=\"706.5\" y=\"132.5\">
PK
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"430\" x2=\"730\" y1=\"150\" y2=\"150\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" text-anchor=\"start\" x=\"442\" y=\"167.5\">
uuid
</text>
<text dominant-baseline=\"middle\" fill=\"#ECC700\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" text-anchor=\"end\" x=\"670\" y=\"167.5\">
uuid
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"430\" x2=\"730\" y1=\"185\" y2=\"185\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" text-anchor=\"start\" x=\"442\" y=\"202.5\">
title
</text>
<text dominant-baseline=\"middle\" fill=\"#D66905\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" text-anchor=\"end\" x=\"670\" y=\"202.5\">
text
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"430\" x2=\"730\" y1=\"220\" y2=\"220\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" text-anchor=\"start\" x=\"442\" y=\"237.5\">
content
</text>
<text dominant-baseline=\"middle\" fill=\"#D66905\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" text-anchor=\"end\" x=\"670\" y=\"237.5\">
text
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"430\" x2=\"730\" y1=\"255\" y2=\"255\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" text-anchor=\"start\" x=\"442\" y=\"272.5\">
created_at
</text>
<text dominant-baseline=\"middle\" fill=\"#06B697\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" text-anchor=\"end\" x=\"670\" y=\"272.5\">
timestamp
</text>
<line stroke=\"#494949\" stroke-width=\"1\" x1=\"430\" x2=\"730\" y1=\"290\" y2=\"290\"/>
<text dominant-baseline=\"middle\" fill=\"white\" font-family=\"Courier New,monospace\" font-weight=\"lighter\" text-anchor=\"start\" x=\"442\" y=\"307.5\">
created_by
</text>
<text dominant-baseline=\"middle\" fill=\"#ECC700\" font-family=\"Courier New,monospace\" font-size=\"small\" font-weight=\"lighter\" text-anchor=\"end\" x=\"670\" y=\"307.5\">
int
</text>
<circle cx=\"706.5\" cy=\"307.5\" fill=\"#202937\" r=\"11.5\"/>
<text dominant-baseline=\"middle\" fill=\"#1170FB\" font-family=\"Trebuchet MS,sans-serif\" font-size=\"xx-small\" text-anchor=\"middle\" x=\"706.5\" y=\"307.5\">
FK
</text>
<path d=\"M430 307.5 L396 307.5 Q390 307.5 390 301.5 L390 138.5 Q390 132.5 384 132.5 L350 132.5\" fill=\"transparent\" stroke=\"#888888\" stroke-width=\"1.5\"/>
<circle cx=\"430\" cy=\"307.5\" fill=\"#1C1C1C\" r=\"4\" stroke=\"#888888\" stroke-width=\"1.5\"/>
<circle cx=\"350\" cy=\"132.5\" fill=\"#1C1C1C\" r=\"4\" stroke=\"#888888\" stroke-width=\"1.5\"/>
</svg>", "\n", 0);
    }
}
