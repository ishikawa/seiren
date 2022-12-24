use seiren::erd::{Column, ColumnType, ERDiagram, Relation, RelationItem, Table};

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
    diagram.edges.push(Relation::new(
        RelationItem::Column("posts".into(), "created_by".into()),
        RelationItem::Column("users".into(), "id".into()),
    ));

    println!("{}", diagram.into_svg());
}
