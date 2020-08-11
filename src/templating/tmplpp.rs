use super::{TemplatingEngine, RenderError};

use pest::{
    error::Error,
    Parser as PestParser,
    iterators::Pair,
};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "templating/tmplpp.pest"]
struct Parser;

#[derive(Debug, PartialEq)]
enum Element<'a> {
    RawText(&'a str),
}

#[derive(Debug)]
struct Template<'a> {
    elements: Vec<Element<'a>>,
}

impl<'a> Template<'a> {
    fn new(elements: Vec<Element<'a>>) -> Self {
        Self {
            elements,
        }
    }
}

#[derive(Debug)]
pub struct Tmplpp {

}

fn parse_template_file(file: &str) -> Result<Template, Error<Rule>> {
    let pest_template = Parser::parse(Rule::template, file)?.next().unwrap();

    fn parse_element(pair: Pair<Rule>) -> Result<Element, Error<Rule>> {
        match pair.as_rule() {
            Rule::raw_text => Ok(Element::RawText(pair.as_str())),
            _              => unreachable!(),
        }
    }

    let elements: Vec<_> = pest_template.into_inner()
        .map(parse_element)
        .collect::<Result<_, _>>()?;

    let template = Template::new(elements);

    Ok(template)
}

#[test]
fn parse_raw_text() {
    let text = "and the mome raths outgrabe";

    let template = parse_template_file(text)
        .unwrap();

    assert_eq!(template.elements, [Element::RawText(text)]);
}