use std::borrow::Cow;
use std::collections::HashMap;
use std::str::FromStr;

use super::{TemplatingEngine, RenderError};

use pest::{
    error::Error,
    Parser as PestParser,
    iterators::{Pair, Pairs},
};
use pest_derive::Parser;
use regex::Regex;

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

        // TODO: Is there a DRYer way to write this?
        for element in self.elements.iter() {
            match element {
                Element::RawText(text)        => result.push_str(text),
                Element::Editable(_, content) => {
                    for element in content {
                        match element {
                            Element::RawText(text)  => result.push_str(text),
                            Element::Editable(_, _) => panic!("nested editables are illegal"), // TODO: Proper error handling here.
                            Element::Var(var_name)  => if let Some(value) = values.get(var_name) {
                                result.push_str(value);
                            },
                        }
                    }
                },
                Element::Var(var_name)        => if let Some(value) = values.get(var_name) {
                    result.push_str(value);
                },
            }
        }

        Ok(result)
    }
    
    // TODO: Maybe memoize this somehow? Or create a separate TemplateWithValues
    // struct that holds the values and a pre-generated regex validator.
    fn regex(&self, values: &HashMap<&str, &str>) -> Regex {
        let len = self.elements.len();
        
        let mut regex_str = String::from("^");

        fn sanitize_string_literal(s: &str) ->  String {
            let mut result = s.replace("\\", "\\\\");
            result = s.replace("(", "\\(");
            result.replace(")", "\\)")
        }

        for el in &self.elements {
            match el {
                Element::RawText(text)           => regex_str.push_str(&sanitize_string_literal(text)),
                Element::Var(var_name)           => if let Some(value) = values.get(var_name) {
                    regex_str.push_str(&sanitize_string_literal(value));
                },
                Element::Editable(name, content) => regex_str.push_str(&format!("(?P<{}>(.|\\n)*)", name)),
            }
        }

        regex_str.push_str("$");
        println!("{}", regex_str);

        Regex::from_str(&regex_str).unwrap()
    }

    fn validate_generated_output(&self, values: &HashMap<&str, &str>, output: &str) -> bool {
        let regex = self.regex(values);

        regex.is_match(output)
    }

    fn upgrade_to(&self, new_template: &Template, values: &HashMap<&str, &str>, output: &str) -> String {
        let regex = self.regex(values);

        let caps = regex.captures(output).unwrap();

        let mut result = String::new();

        // TODO: Is there a DRYer way to write this?
        for element in new_template.elements.iter() {
            match element {
                Element::RawText(text)     => result.push_str(text),
                Element::Editable(name, _) => result.push_str(caps.name(name).unwrap().as_str()),
                Element::Var(var_name)     => if let Some(value) = values.get(var_name) {
                    result.push_str(value);
                },
            }
        }

        result
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

// Render tests

#[test]
fn render_empty_template() {
    let template = Template::from_str("").unwrap();

    assert_eq!(template.render_to_string(&HashMap::new()).unwrap(), "");
}

#[test]
fn render_raw_text() {
    let template = Template::from_str("All mimsy were the borogoves.").unwrap();

    assert_eq!(template.render_to_string(&HashMap::new()).unwrap(), "All mimsy were the borogoves.");
}

#[test]
fn render_empty_var() {
    let template = Template::from_str("All mimsy were {{ foo }} borogoves.").unwrap();

    assert_eq!(template.render_to_string(&HashMap::new()).unwrap(), "All mimsy were  borogoves.");
}

#[test]
fn render_single_var() {
    let template = Template::from_str("All mimsy were {{ foo }} borogoves.").unwrap();

    assert_eq!(
        template.render_to_string(&[("foo", "the")].iter()
            .cloned()
            .collect()
        ).unwrap(),
        "All mimsy were the borogoves.",
    );
}

#[test]
fn render_editable_block() {
    let template = Template::from_str("All mimsy were {{@ foo }}the{{@ / }} borogoves.").unwrap();

    assert_eq!(
        template.render_to_string(&HashMap::new()).unwrap(),
        "All mimsy were the borogoves.",
    );
}

#[test]
fn render_editable_block_with_vars_inside() {
    let template = Template::from_str("All {{@ foo }}mimsy {{ bar }} the{{@ / }} borogoves.").unwrap();

    assert_eq!(
        template.render_to_string(&[("bar", "were")].iter()
            .cloned()
            .collect()
        ).unwrap(),
        "All mimsy were the borogoves.",
    );
}

// Validator tests

#[test]
fn validate_simple_text() {
    let template = Template::from_str("All mimsy were the borogoves.").unwrap();

    assert!(template.validate_generated_output(&HashMap::new(), "All mimsy were the borogoves."));
    assert!(!template.validate_generated_output(&HashMap::new(), "All mimsy were the borogoves. "));
}

#[test]
fn validate_simple_text_with_newlines() {
    let template = Template::from_str("All mimsy\nwere the borogoves.").unwrap();

    assert!(template.validate_generated_output(&HashMap::new(), "All mimsy\nwere the borogoves."));
    assert!(!template.validate_generated_output(&HashMap::new(), "All mimsy\nwere the borogoves. "));
    assert!(!template.validate_generated_output(&HashMap::new(), "All mimsy\nwere the borogoves.\nAll mimsy\nwere the borogoves."));
}

#[test]
fn validate_output_with_vars() {
    let template = Template::from_str("All mimsy {{ foo }} the borogoves.").unwrap();

    let values = [("foo", "were")].iter()
        .cloned()
        .collect();

    assert!(template.validate_generated_output(&values, "All mimsy were the borogoves."));
    assert!(!template.validate_generated_output(&values, "All mimsy was the borogoves."));
}

#[test]
fn validate_output_with_an_editable() {
    let template = Template::from_str("All mimsy {{@ foo }}were{{@/}} the borogoves.").unwrap();

    let values = HashMap::new();

    assert!(template.validate_generated_output(&values, "All mimsy were the borogoves."));
    // We're allowed to edit the text inside the editable...
    assert!(template.validate_generated_output(&values, "All mimsy was the borogoves."));
    assert!(template.validate_generated_output(&values, "All mimsy asd fsd sdf the borogoves."));
    // ...but we shouldn't edit the text outside of the editable.
    assert!(!template.validate_generated_output(&values, "All mimsy were the borogoves. Stuff."));
}

// Upgrade tests

#[test]
fn upgrade_output_with_simple_text() {
    let v1 = Template::from_str("All mimsy were the borogoves.").unwrap();
    let v2 = Template::from_str("All mimsy were my borogoves.").unwrap();

    let values = HashMap::new();

    let output = v1.render_to_string(&values).unwrap();

    assert_eq!(output, "All mimsy were the borogoves.");

    let new_output = v1.upgrade_to(&v2, &values, &output);

    assert_eq!(new_output, "All mimsy were my borogoves.");
}

#[test]
fn upgrade_output_with_vars() {
    let v1 = Template::from_str("All mimsy {{ foo }} the borogoves.").unwrap();
    let v2 = Template::from_str("All mimsy {{ foo }} my borogoves.").unwrap();

    let values = [("foo", "were")].iter()
        .cloned()
        .collect();

    let output = v1.render_to_string(&values).unwrap();

    assert_eq!(output, "All mimsy were the borogoves.");

    let new_output = v1.upgrade_to(&v2, &values, &output);

    assert_eq!(new_output, "All mimsy were my borogoves.");
}

#[test]
fn upgrade_output_with_an_editable() {
    let v1 = Template::from_str("All mimsy {{@ foo }}were{{@/}} the borogoves.").unwrap();
    let v2 = Template::from_str("All mimsy {{@ foo }}were{{@/}} my borogoves.").unwrap();

    let values = HashMap::new();

    let modified_output = "All mimsy bla bla bla the borogoves.";

    let new_output = v1.upgrade_to(&v2, &values, modified_output);

    assert_eq!(new_output, "All mimsy bla bla bla my borogoves.");
}