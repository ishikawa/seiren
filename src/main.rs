use seiren::{Column, ColumnType, Document, Table};

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
