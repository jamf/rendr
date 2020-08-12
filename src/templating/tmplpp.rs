use std::collections::HashMap;

use super::{TemplatingEngine, RenderError};

use pest::{
    error::Error,
    Parser as PestParser,
    iterators::{Pair, Pairs},
};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "templating/tmplpp.pest"]
struct Parser;

#[derive(Debug, PartialEq)]
enum Element<'a> {
    RawText(&'a str),
    Editable(&'a str, Vec<Element<'a>>),
    Var(&'a str),
}

#[derive(Debug)]
struct Template<'a> {
    elements: Vec<Element<'a>>,
}

impl<'a> Template<'a> {
    pub fn from_str(template_str: &'a str) -> Result<Self, Error<Rule>> {
        let pest_template = Parser::parse(Rule::template, template_str)?
            .next()
            .unwrap()
            .into_inner();

        fn parse_element(pair: Pair<Rule>) -> Result<Element, Error<Rule>> {
            match pair.as_rule() {
                Rule::raw_text => Ok(Element::RawText(pair.as_str())),
                Rule::editable => {
                    let mut pairs = pair.into_inner();
                    let name = pairs.next().unwrap().into_inner().as_str();
                    let elements = parse_elements(pairs.next().unwrap().into_inner())?;
                    Ok(Element::Editable(name, elements))
                },
                Rule::variable => Ok(Element::Var(pair.into_inner().as_str())),
                _              => unreachable!(),
            }
        }

        fn parse_elements(pairs: Pairs<Rule>) -> Result<Vec<Element>, Error<Rule>> {
            pairs
                .map(parse_element)
                .collect::<Result<_, _>>()
        }

        let template = Self::from_elements(parse_elements(pest_template)?);

        Ok(template)
    }

    fn from_elements(elements: Vec<Element<'a>>) -> Self {
        Self {
            elements,
        }
    }

    fn render_to_string(&self, values: &HashMap<&str, &str>) -> Result<String, RenderError> {
        let mut result = String::new();

        for element in self.elements.iter() {
            match element {
                Element::RawText(text)         => result.push_str(text),
                Element::Editable(_, _content) => todo!(),
                Element::Var(var_name)         => if let Some(value) = values.get(var_name) {
                                                      result.push_str(value);
                                                  },
            }
        }

        Ok(result)
    }
}

#[derive(Debug)]
pub struct Tmplpp;

impl Tmplpp {
    pub fn new() -> Self {
        Tmplpp
    }
}

impl TemplatingEngine for Tmplpp {
    fn render_template(&self, template_str: &str, values: &HashMap<&str, &str>) -> Result<String, RenderError> {
        let template = Template::from_str(template_str)?;

        Ok(template.render_to_string(&values)?)
    }
}

impl From<Error<Rule>> for RenderError {
    fn from(e: Error<Rule>) -> Self {
        RenderError {
            inner: Box::new(e),
        }
    }
}

// Parser tests

#[test]
fn parse_raw_text() {
    let text = "and the mome raths outgrabe";

    let template = Template::from_str(text)
        .unwrap();

    assert_eq!(template.elements, [Element::RawText(text)]);
}

#[test]
fn parse_raw_text_and_tags() {
    let text = "and the mome raths outgrabe {{ foo }} and {{ bar }}";

    let template = Template::from_str(text)
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

    let template = Template::from_str(text)
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

    let template = Template::from_str(text)
        .unwrap();

    assert_eq!(template.elements, [
        Element::RawText("and the mome raths outgrabe "),
        Element::Var("foo"),
        Element::Var("bar"),
    ]);
}

#[test]
fn parse_a_simple_editable() {
    let text = "and the mome {{@ foo }}raths{{@ / }} outgrabe";

    let template = Template::from_str(text)
        .unwrap();

    assert_eq!(template.elements, [
        Element::RawText("and the mome "),
        Element::Editable(
            "foo",
            vec!(Element::RawText("raths")),
        ),
        Element::RawText(" outgrabe"),
    ]);
}

#[test]
fn parse_an_editable_with_vars() {
    let text = "and the mome {{@ foo }}raths {{ bar }}{{@ / }} outgrabe";

    let template = Template::from_str(text)
        .unwrap();

    assert_eq!(template.elements, [
        Element::RawText("and the mome "),
        Element::Editable(
            "foo",
            vec!(
                Element::RawText("raths "),
                Element::Var("bar"),
            ),
        ),
        Element::RawText(" outgrabe"),
    ]);
}

#[test]
fn strip_newlines_when_parsing_editables() {
    let text = "stuff\n{{@ foo }}\nstuff\n{{@ / }}";

    let template = Template::from_str(text)
        .unwrap();

    assert_eq!(template.elements, [
        Element::RawText("stuff\n"),
        Element::Editable(
            "foo",
            vec!(
                Element::RawText("stuff"),
            ),
        ),
    ]);
}

#[test]
fn strip_only_one_newline_when_parsing_editables() {
    let text = "stuff\n{{@ foo }}\n\nstuff\n\n\n{{@ / }}";

    let template = Template::from_str(text)
        .unwrap();

    assert_eq!(template.elements, [
        Element::RawText("stuff\n"),
        Element::Editable(
            "foo",
            vec!(
                Element::RawText("\nstuff\n\n"),
            ),
        ),
    ]);
}

#[test]
fn attempt_parsing_invalid_template_fails() {
    let text = "and the mome raths {{ outgrabe";

    assert!(Template::from_str(text).is_err());
}