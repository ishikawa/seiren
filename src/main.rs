use seiren::parser::parse;
use std::io;
use std::{fs, io::Read};

fn main() -> Result<(), io::Error> {
    let mut args = std::env::args();

    // Read the contents of a specified file or from stdio.
    let src = if args.len() >= 2 {
        let path = args.nth(1).unwrap();
        fs::read_to_string(path)?
    } else {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s)?;
        s
    };

    let (ast, errs, parse_errs) = parse(&src);

    println!("{:?} - {:?}", errs, parse_errs);
    if let Some(ast) = ast {
        println!("{}", ast);
    }

    Ok(())
}
