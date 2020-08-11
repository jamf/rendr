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
    Var(&'a str),
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
    let pest_template = Parser::parse(Rule::template, file)?
        .next()
        .unwrap()
        .into_inner();

    println!("{:?}", pest_template);

    fn parse_element(pair: Pair<Rule>) -> Result<Element, Error<Rule>> {
        match pair.as_rule() {
            Rule::raw_text => Ok(Element::RawText(pair.as_str())),
            Rule::variable => Ok(Element::Var(pair.into_inner().as_str())),
            _              => unreachable!(),
        }
    }

    let elements: Vec<_> = pest_template
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

#[test]
fn parse_raw_text_and_tags() {
    let text = "and the mome raths outgrabe {{ foo }} and {{ bar }}";

    let template = parse_template_file(text)
        .unwrap();

    assert_eq!(template.elements, [
        Element::RawText("and the mome raths outgrabe "),
        Element::Var("foo"),
        Element::RawText(" and "),
        Element::Var("bar"),
    ]);
}

#[test]
fn parse_vars_regardless_of_whitespace() {
    let text = "and the mome raths outgrabe {{    foo    }} and {{bar}}";

    let template = parse_template_file(text)
        .unwrap();

    assert_eq!(template.elements, [
        Element::RawText("and the mome raths outgrabe "),
        Element::Var("foo"),
        Element::RawText(" and "),
        Element::Var("bar"),
    ]);
}

#[test]
fn parse_consecutive_vars() {
    let text = "and the mome raths outgrabe {{ foo }}{{ bar }}";

    let template = parse_template_file(text)
        .unwrap();

    assert_eq!(template.elements, [
        Element::RawText("and the mome raths outgrabe "),
        Element::Var("foo"),
        Element::Var("bar"),
    ]);
}

#[test]
fn attempt_parsing_invalid_template_fails() {
    let text = "and the mome raths {{ outgrabe";

    assert!(parse_template_file(text).is_err());
}