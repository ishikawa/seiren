use std::io;

use seiren::{
    backend::{Backend, SVGBackend},
    erd::{Column, ColumnType, ERDiagram, Relation, RelationPath, Table},
    layout::{LayoutEngine, SimpleLayoutEngine},
};

fn main() {
    let mut diagram = ERDiagram::new();
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

    diagram.tables.push(users_table);
    diagram.tables.push(posts_table);
    diagram.relations.push(Relation::new(
        RelationPath::Column("posts".into(), "created_by".into()),
        RelationPath::Column("users".into(), "id".into()),
    ));

    let mut doc = diagram.into_mir();
    let engine = SimpleLayoutEngine::new();

    engine.execute_node_layout(&mut doc);

    let backend = SVGBackend::new();
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    backend
        .generate(&doc, &mut handle)
        .expect("cannot generate SVG");
}
