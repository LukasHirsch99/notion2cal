use crate::notion_api::types::page::PageProperty;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Display, Write},
};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(transparent)]
pub struct Template {
    segments: Vec<Segment>,
}

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

impl From<Vec<Segment>> for Template {
    fn from(segments: Vec<Segment>) -> Self {
        Template { segments }
    }
}

impl Display for Template {
    /// Formats a template back into the source syntax accepted by [`parse`], so it
    /// can be shown in an edit form.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in &self.segments {
            match segment {
                Segment::Text(text) => {
                    for c in text.chars() {
                        if c == '{' || c == '}' {
                            f.write_char(c)?;
                        }
                        f.write_char(c)?;
                    }
                }
                Segment::Field(name) => {
                    f.write_char('{')?;
                    f.write_str(name)?;
                    f.write_char('}')?;
                }
            }
        }
        Ok(())
    }
}

impl Template {
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn parse(src: &str, props: &Vec<String>) -> Result<Self, ParseError> {
        if src.trim().is_empty() {
            return Ok(Template::default());
        }
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
                    if name != PAGE_URL_FIELD && !props.contains(&name) {
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

        Ok(Template { segments })
    }

    pub fn render(
        &self,
        props: &HashMap<String, PageProperty>,
        page_url: &str,
    ) -> Result<String, String> {
        let mut out = String::new();
        for seg in &self.segments {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_an_optional_template() {
        let mut columns = Vec::new();
        columns.push("Name".to_owned());

        assert_eq!(
            Template::parse("{Name}", &columns).unwrap().segments,
            vec![Segment::Field("Name".to_owned())]
        );
    }

    #[test]
    fn treats_a_blank_optional_template_as_empty() {
        assert!(
            Template::parse("  ", &Vec::new())
                .unwrap()
                .segments
                .is_empty()
        );
    }

    #[test]
    fn rejects_unknown_columns_in_an_optional_template() {
        assert_eq!(
            Template::parse("{Nope}", &Vec::new())
                .unwrap_err()
                .to_string(),
            "unknown column `Nope` at position 0."
        );
    }

    #[test]
    fn accepts_the_page_url_field() {
        let template = Template::parse("Open {Page URL}", &Vec::new()).unwrap();

        assert_eq!(
            template.segments,
            vec![
                Segment::Text("Open ".to_owned()),
                Segment::Field(PAGE_URL_FIELD.to_owned()),
            ]
        );
    }

    #[test]
    fn renders_the_page_url() {
        let template = Template::parse("{Page URL}", &Vec::new()).unwrap();

        assert_eq!(
            template
                .render(&HashMap::new(), "https://notion.so/page")
                .unwrap(),
            "https://notion.so/page"
        );
    }

    #[test]
    fn deserializes_from_a_segment_array() {
        let template: Template =
            serde_json::from_str(r#"[{"Text":"Event"},{"Field":"Name"}]"#).unwrap();

        assert_eq!(
            template.segments,
            vec![
                Segment::Text("Event".to_owned()),
                Segment::Field("Name".to_owned())
            ]
        );
    }

    #[test]
    fn serializes_to_a_segment_array() {
        let template: Template = vec![Segment::Text("Event".to_owned())].into();

        assert_eq!(
            serde_json::to_string(&template).unwrap(),
            r#"[{"Text":"Event"}]"#
        );
    }

    #[test]
    fn formats_a_template_back_into_source_text() {
        let template = Template {
            segments: vec![
                Segment::Text("Meet {".to_owned()),
                Segment::Field("Name".to_owned()),
                Segment::Text("} at {x}".to_owned()),
            ],
        };

        assert_eq!(template.to_string(), "Meet {{{Name}}} at {{x}}");
    }
}
