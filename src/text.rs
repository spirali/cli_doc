use askama::filters::{Escaper, Html};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RichText {
    pub parts: Vec<RichTextPart>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum RichTextPart {
    Text(String),
    Ul(Vec<String>),
    Config { key: String, value: String },
}

impl RichTextPart {
    pub fn to_html_into(&self, out: &mut String) {
        let html = Html;
        match self {
            RichTextPart::Text(s) => {
                out.push_str("<p>");
                html.write_escaped_str(&mut *out, s).unwrap();
                out.push_str("</p>");
            }
            RichTextPart::Ul(items) => {
                out.push_str("<ul>");
                for item in items {
                    out.push_str("<li>");
                    html.write_escaped_str(&mut *out, item).unwrap();
                    out.push_str("</li>");
                }
                out.push_str("</ul>");
            }
            RichTextPart::Config { key, value } => {
                out.push_str("<p class=\"rt-config\">[");
                html.write_escaped_str(&mut *out, key).unwrap();
                out.push_str(": <span class=\"rt-config-value\">");
                html.write_escaped_str(&mut *out, value).unwrap();
                out.push_str("</span>]</p>");
            }
        }
    }
}

impl RichText {
    pub fn new() -> Self {
        RichText { parts: Vec::new() }
    }

    pub fn from_single_line(line: &str) -> RichText {
        let mut text = RichText::new();
        text.add_lines(&[line]);
        text
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    fn push_part(&mut self, item: Option<RichTextPart>) {
        if let Some(item) = item {
            if let RichTextPart::Text(text) = &item {
                if text.is_empty() {
                    return;
                }
            }
            self.parts.push(item);
        }
    }

    pub fn add_lines(&mut self, lines: &[&str]) {
        let mut current = None;
        'main: for line in lines {
            for pattern in ["* ", "- "] {
                if let Some(t) = line.strip_prefix(pattern) {
                    if let Some(RichTextPart::Ul(ref mut items)) = current {
                        items.push(t.to_string())
                    } else {
                        self.push_part(current);
                        current = Some(RichTextPart::Ul(vec![t.to_string()]));
                    }
                    continue 'main;
                }
            }
            if line.starts_with('[') {
                let mut s: &str = line;
                while let Some((first, rest)) = s.split_once(']') {
                    if let Some(t) = first.strip_prefix('[') {
                        if let Some((left, right)) = t.split_once(':') {
                            self.push_part(current);
                            current = Some(RichTextPart::Config {
                                key: left.trim().to_string(),
                                value: right.trim().to_string(),
                            });
                        }
                    }
                    s = rest.trim();
                }
                continue 'main;
            }
            if let Some(RichTextPart::Text(ref mut text)) = current {
                text.push(' ');
                text.push_str(line);
            } else {
                self.push_part(current);
                current = Some(RichTextPart::Text(line.to_string()));
            }
        }
        self.push_part(current);
    }

    pub fn to_html(&self) -> String {
        let mut s = String::new();
        s.push_str("<div class=\"rich-text\">");
        for part in &self.parts {
            part.to_html_into(&mut s);
        }
        s.push_str("</div>");
        s
    }
}
