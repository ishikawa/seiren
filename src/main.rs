use ariadne::{Color, Fmt, Label, Report, ReportKind, Source};
use seiren::layout::{LayoutEngine, SimpleLayoutEngine};
use seiren::parser::parse;
use seiren::renderer::{Renderer, SVGRenderer};
use std::io;
use std::{fs, io::Read};

const DEBUG: bool = true;

fn main() -> Result<(), io::Error> {
    let mut filename = "(stdin)".to_string();
    let mut args = std::env::args();

    // Read the contents of a specified file or from stdio.
    let src = if args.len() >= 2 {
        let path = args.nth(1).unwrap();

        filename = path.clone();
        fs::read_to_string(path)?
    } else {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s)?;
        s
    };

    let (ast, tokenize_errs, parse_errs) = parse(&src);

    // Convert both errors into error::Simple<String>
    let errors = tokenize_errs
        .into_iter()
        .map(|x| x.map(|c| c.to_string()))
        .chain(parse_errs.into_iter().map(|e| e.map(|tok| tok.to_string())))
        .collect::<Vec<_>>();

    // Report errors
    for e in errors {
        let filename = filename.as_str();
        let report = Report::build(ReportKind::Error, filename, e.span().start);

        let report = match e.reason() {
            chumsky::error::SimpleReason::Unclosed { span, delimiter } => report
                .with_message(format!(
                    "Unclosed delimiter {}",
                    delimiter.fg(Color::Yellow)
                ))
                .with_label(
                    Label::new((filename, span.clone()))
                        .with_message(format!(
                            "Unclosed delimiter {}",
                            delimiter.fg(Color::Yellow)
                        ))
                        .with_color(Color::Yellow),
                )
                .with_label(
                    Label::new((filename, e.span()))
                        .with_message(format!(
                            "Must be closed before this {}",
                            e.found()
                                .unwrap_or(&"end of file".to_string())
                                .fg(Color::Red)
                        ))
                        .with_color(Color::Red),
                ),
            chumsky::error::SimpleReason::Unexpected => report
                .with_message(format!(
                    "{}, expected {}",
                    if e.found().is_some() {
                        "Unexpected token in input"
                    } else {
                        "Unexpected end of input"
                    },
                    if e.expected().len() == 0 {
                        "something else".to_string()
                    } else {
                        e.expected()
                            .map(|expected| match expected {
                                Some(expected) => expected.to_string(),
                                None => "end of input".to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                ))
                .with_label(
                    Label::new((filename, e.span()))
                        .with_message(format!(
                            "Unexpected token {}",
                            e.found()
                                .unwrap_or(&"end of file".to_string())
                                .fg(Color::Red)
                        ))
                        .with_color(Color::Red),
                ),
            chumsky::error::SimpleReason::Custom(msg) => report.with_message(msg).with_label(
                Label::new((filename, e.span()))
                    .with_message(format!("{}", msg.fg(Color::Red)))
                    .with_color(Color::Red),
            ),
        };

        report
            .finish()
            .eprint((filename, Source::from(&src)))
            .unwrap();
    }

    // AST -> MIR

    if let Some(ast) = ast {
        let mut doc = ast.into_mir();
        let mut engine = SimpleLayoutEngine::new();

        engine.place_nodes(&mut doc);
        engine.place_connection_points(&mut doc);
        engine.draw_edge_path(&mut doc);

        let mut backend = SVGRenderer::new();

        if DEBUG {
            backend.edge_route_graph = Some(engine.edge_route_graph());
        }

        let stdout = io::stdout();
        let mut handle = stdout.lock();

        backend
            .render(&doc, &mut handle)
            .expect("Couldn't render as SVG.");
    }

    Ok(())
}
