use crate::notion_api::types::page::PageProperty;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

pub type Template = Vec<Segment>;

/// Reserved field for the Notion page URL. It resolves to the page's URL rather
/// than a property value.
pub const PAGE_URL_FIELD: &str = "Page URL";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Segment {
    Text(String),
    Field(String),
}

#[derive(Debug)]
pub enum ParseError {
    UnclosedBrace(usize),
    StrayClosingBrace(usize),
    InvalidField(usize, String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnclosedBrace(position) => {
                write!(f, "unclosed `{{` at position {position}")
            }
            Self::StrayClosingBrace(position) => {
                write!(f, "unexpected `}}` at position {position}")
            }
            Self::InvalidField(position, field) => {
                write!(f, "unknown column `{field}` at position {position}")
            }
        }
    }
}

pub fn parse<T>(src: &str, props: HashMap<String, T>) -> Result<Template, ParseError> {
    let mut segments = Vec::new();
    let mut text = String::new();
    let mut chars = src.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '{' if matches!(chars.peek(), Some((_, '{'))) => {
                chars.next();
                text.push('{');
            }
            '{' => {
                let mut name = String::new();
                let mut closed = false;
                for (_, c) in chars.by_ref() {
                    if c == '}' {
                        closed = true;
                        break;
                    }
                    name.push(c);
                }
                if !closed {
                    return Err(ParseError::UnclosedBrace(i));
                }
                name = name.trim().to_owned();
                if name != PAGE_URL_FIELD && !props.contains_key(&name) {
                    return Err(ParseError::InvalidField(i, name));
                }
                if !text.is_empty() {
                    segments.push(Segment::Text(std::mem::take(&mut text)));
                }
                segments.push(Segment::Field(name));
            }
            '}' if matches!(chars.peek(), Some((_, '}'))) => {
                chars.next();
                text.push('}');
            }
            '}' => return Err(ParseError::StrayClosingBrace(i)),
            _ => text.push(c),
        }
    }
    if !text.is_empty() {
        segments.push(Segment::Text(text));
    }

    Ok(segments)
}

/// Formats a template back into the source syntax accepted by [`parse`], so it
/// can be shown in an edit form.
pub fn format_source(template: &Template) -> String {
    let mut out = String::new();
    for segment in template {
        match segment {
            Segment::Text(text) => {
                for c in text.chars() {
                    if c == '{' || c == '}' {
                        out.push(c);
                    }
                    out.push(c);
                }
            }
            Segment::Field(name) => {
                out.push('{');
                out.push_str(name);
                out.push('}');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_page_url_field() {
        let template = parse("Open {Page URL}", HashMap::<String, ()>::new()).unwrap();

        assert_eq!(
            template,
            vec![
                Segment::Text("Open ".to_owned()),
                Segment::Field(PAGE_URL_FIELD.to_owned()),
            ]
        );
    }

    #[test]
    fn renders_the_page_url() {
        let template = parse("{Page URL}", HashMap::<String, ()>::new()).unwrap();

        assert_eq!(
            render(&template, &HashMap::new(), "https://notion.so/page").unwrap(),
            "https://notion.so/page"
        );
    }

    #[test]
    fn formats_a_template_back_into_source_text() {
        let template = vec![
            Segment::Text("Meet {".to_owned()),
            Segment::Field("Name".to_owned()),
            Segment::Text("} at {x}".to_owned()),
        ];

        assert_eq!(format_source(&template), "Meet {{{Name}}} at {{x}}");
    }
}

pub fn render(
    segments: &Template,
    props: &HashMap<String, PageProperty>,
    page_url: &str,
) -> Result<String, String> {
    let mut out = String::new();
    for seg in segments {
        match seg {
            Segment::Text(text) => out.push_str(text),
            Segment::Field(field) if field == PAGE_URL_FIELD => out.push_str(page_url),
            Segment::Field(field) => match props.get(field) {
                Some(prop) => out.push_str(&prop.value.to_string()),
                None => return Err(format!("unknown field: {}", field)),
            },
        }
    }
    Ok(out)
}
