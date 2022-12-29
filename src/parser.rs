/*!
ERD module EBNF
---------------

TODO: Follow UAX31 Default Identifier <https://www.unicode.org/reports/tr31/tr31-37.html#Default_Identifier_Syntax>

```ebnf
program = SP, relation, SP ;
relation = entity, { SP, edge, SP, entity };
entity = identifier, [ ".", identifier ] ;
edge = left_connector, edge_path, right_connector ;
left_connector = "o" ;
right_connector = "o" ;
edge_path = "--" ;
identifier = identifier_start, { identifier_continue } ;
identifier_start = "_" | letter ;
identifier_continue = "_" | letter | digit ;
letter = ? a-zA-Z ? ;
digit = ? 0-9 ? ;
whitespace = ? whitespace ? ;
SP = { whitespace } ;
```
*/

use chumsky::prelude::*;
use derive_more::Display;

#[derive(Debug, Display)]
pub enum Expr {
    #[display(fmt = "({} o--o {})", _0, _1)]
    Relation(Box<Expr>, Box<Expr>),
    #[display(fmt = "{}", _0)]
    Entity(EntityPath),
}

#[derive(Debug, Display)]
pub enum EntityPath {
    #[display(fmt = "{}", _0)]
    Table(String),
    #[display(fmt = "{}.{}", _0, _1)]
    Column(String, String),
}

pub fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    relation().padded().then_ignore(end())
}

fn relation() -> impl Parser<char, Expr, Error = Simple<char>> {
    entity()
        .then(just("o--o").padded().ignore_then(entity()).repeated())
        .foldl(|a, b| Expr::Relation(Box::new(a), Box::new(b)))
}

fn entity() -> impl Parser<char, Expr, Error = Simple<char>> {
    text::ident()
        .then(just(".").ignore_then(text::ident()).or_not())
        .map(|(table, column)| {
            if let Some(column) = column {
                Expr::Entity(EntityPath::Column(table, column))
            } else {
                Expr::Entity(EntityPath::Table(table))
            }
        })
}
