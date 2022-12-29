/*!
ERD module EBNF
---------------

TODO: Follow UAX31 Default Identifier <https://www.unicode.org/reports/tr31/tr31-37.html#Default_Identifier_Syntax>

```ebnf
program = SP, relation, SP ;
relation = entity, SP, edge, SP, entity ;
entity = identifier, [ ".", identifier ] ;
edge = left_connector, path, right_connector ;
left_connector = "o" ;
right_connector = "o" ;
path = "--" ;
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

#[derive(Debug)]
pub enum Expr {
    Relation(RelationPath),
}

#[derive(Debug)]
pub enum RelationPath {
    Table(String),
    Column(String, String),
}

pub fn relation() -> impl Parser<char, Expr, Error = Simple<char>> {
    text::ident()
        .then(just(".").ignore_then(text::ident()).or_not())
        .map(|(table, column)| {
            if let Some(column) = column {
                Expr::Relation(RelationPath::Column(table, column))
            } else {
                Expr::Relation(RelationPath::Table(table))
            }
        })
}
