use chumsky::{prelude::*, text::Char};

use crate::ast::{Item, Line, Pattern, Template};

pub fn parser<'src>()
-> impl Parser<'src, &'src str, Vec<Template<'src>>, extra::Err<Rich<'src, char>>> {
    let variable = just('$').ignore_then(choice((
        just("*"),
        none_of("$_()\n")
            .filter(|c: &char| c.is_ident_continue())
            .repeated()
            .at_least(1)
            .to_slice(),
        none_of("}\n")
            .repeated()
            .at_least(1)
            .delimited_by(just('{'), just('}'))
            .to_slice(),
    )));

    let comment = just('#')
        .padded_by(just(' ').repeated())
        .ignore_then(none_of('\n').repeated().to_slice());

    let text = none_of("$()#\n").repeated().at_least(1).to_slice();

    let pattern = choice((variable.map(Pattern::Variable), text.map(Pattern::Text)))
        .repeated()
        .collect::<Vec<_>>()
        .delimited_by(just('('), just(')'));

    let expr = recursive(|expr| {
        let sub = expr.delimited_by(just('('), just(')')).map(Item::Sub);

        choice((sub, variable.map(Item::Variable), text.map(Item::Text)))
            .repeated()
            .collect()
    });

    let between = choice((comment.ignored(), empty()))
        .ignore_then(just('\n'))
        .repeated();

    let indent = choice((just("\t").ignored(), just("    ").ignored()));

    let decl = pattern
        .then(comment.or_not())
        .then_ignore(just('\n'))
        .then(
            indent
                .ignore_then(expr)
                .then_ignore(just('\n').or_not())
                .map(|expr| Line { items: expr })
                .separated_by(
                    choice((
                        indent.then_ignore(comment).then_ignore(just('\n')),
                        indent.then_ignore(just('\n')),
                    ))
                    .repeated()
                )
                .allow_leading()
                .collect(),
        )
        .map(|((pattern, name), body)| Template {
            pattern,
            name,
            body,
        });

    decl.separated_by(between)
        .allow_leading()
        .allow_trailing()
        .collect()
        .then_ignore(end())
}
