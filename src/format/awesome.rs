extern crate xml_writer;

use self::xml_writer::XmlWriter;
use super::data::*;
use std::io;

pub struct AwesomeFormatter {}

impl Formatter for AwesomeFormatter {
    fn format(&mut self, data: &Format) -> String {
        let mut xml = XmlWriter::new(Vec::new());
        xml.pretty = false;

        self.markup(&mut xml, data)
            .expect("error generating xml markup");

        let raw = xml.into_inner();
        String::from_utf8(raw).expect("invalid utf-8 in generated xml markup")
    }
}

impl AwesomeFormatter {
    pub fn new() -> AwesomeFormatter {
        AwesomeFormatter {}
    }

    fn markup<W>(&self, xml: &mut XmlWriter<W>, data: &Format) -> io::Result<()>
    where
        W: io::Write,
    {
        xml.begin_elem("markup")?;
        self.format_span(xml, data)?;
        xml.end_elem()?;
        xml.flush()
    }

    fn format_span<W>(&self, xml: &mut XmlWriter<W>, data: &Format) -> io::Result<()>
    where
        W: io::Write,
    {
        match *data {
            Format::UnescapedStr(ref s) | Format::Str(ref s) => {
                xml.begin_elem("span")?;
                xml.text(s)?;
                xml.end_elem()
            }
            Format::Concat(ref fs) => fs.iter().map(|f| self.format_span(xml, f)).collect(),
            Format::Align(ref _alignment, ref content) =>
            // Can't align with pango markup so it's ignored
            {
                self.format_span(xml, content)
            }
            Format::FgColor(ref c, ref f) => {
                xml.begin_elem("span")?;
                xml.attr_esc("foreground", c)?;
                self.format_span(xml, f)?;
                xml.end_elem()
            }
            Format::BgColor(ref c, ref f) => {
                xml.begin_elem("span")?;
                xml.attr_esc("background", c)?;
                self.format_span(xml, f)?;
                xml.end_elem()
            }
            Format::NoSeparator(ref f) => self.format_span(xml, f),
            Format::Padding(n, ref f) => {
                xml.begin_elem("span")?;
                xml.text(&" ".repeat(n as usize))?;
                xml.end_elem()?;

                self.format_span(xml, f)
            }
            Format::Clickable(ref act, ref f) => match act {
                _ => self.format_span(xml, f), // TODO
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_unescaped_str() {
        let mut formatter = AwesomeFormatter::new();
        let expected = "<markup><span>Test &amp; Thing</span></markup>".to_string();
        assert_eq!(
            formatter.format(&Format::UnescapedStr("Test & Thing".to_string())),
            expected
        )
    }

    #[test]
    fn test_format_str() {
        let mut formatter = AwesomeFormatter::new();
        let expected = "<markup><span>Test &amp; Thing</span></markup>".to_string();
        assert_eq!(
            formatter.format(&Format::Str("Test & Thing".to_string())),
            expected
        )
    }

    #[test]
    fn test_format_concat() {
        let mut formatter = AwesomeFormatter::new();
        let expected = "<markup><span>One</span><span>Two</span></markup>".to_string();
        let format = Format::Concat(vec![
            Box::new(Format::Str("One".to_string())),
            Box::new(Format::Str("Two".to_string())),
        ]);
        assert_eq!(formatter.format(&format), expected)
    }

    #[test]
    fn test_format_align() {
        let mut formatter = AwesomeFormatter::new();
        let expected = "<markup><span>Center</span></markup>".to_string();
        let format = Format::Align(
            Alignment::Center,
            Box::new(Format::Str("Center".to_string())),
        );
        assert_eq!(formatter.format(&format), expected)
    }

    #[test]
    fn test_format_fgcolor() {
        let mut formatter = AwesomeFormatter::new();
        let expected =
            "<markup><span foreground=\"#00FF00\"><span>Green</span></span></markup>".to_string();
        let format = Format::FgColor(
            "#00FF00".to_string(),
            Box::new(Format::Str("Green".to_string())),
        );
        assert_eq!(formatter.format(&format), expected)
    }

    #[test]
    fn test_format_bgcolor() {
        let mut formatter = AwesomeFormatter::new();
        let expected = "<markup><span background=\"#00FF00\"><span>Green BG</span></span></markup>"
            .to_string();
        let format = Format::BgColor(
            "#00FF00".to_string(),
            Box::new(Format::Str("Green BG".to_string())),
        );
        assert_eq!(formatter.format(&format), expected)
    }

    #[test]
    fn test_format_no_separator() {
        let mut formatter = AwesomeFormatter::new();
        let expected = "<markup><span>Test</span></markup>".to_string();
        let format = Format::NoSeparator(Box::new(Format::Str("Test".to_string())));
        assert_eq!(formatter.format(&format), expected)
    }

    #[test]
    fn test_format_padding() {
        let mut formatter = AwesomeFormatter::new();
        let expected = "<markup><span>Test</span></markup>".to_string();
        let format = Format::Padding(3, Box::new(Format::Str("Test".to_string())));
        assert_eq!(formatter.format(&format), expected)
    }

    #[test]
    fn test_format_clickable() {
        let mut formatter = AwesomeFormatter::new();
        let expected = "<markup><span>Test</span></markup>".to_string();
        let format = Format::Clickable(
            ClickAction::ShellCommand(MouseButton::Left, "echo 'hello world'".to_string()),
            Box::new(Format::Str("Test".to_string())),
        );
        assert_eq!(formatter.format(&format), expected)
    }
}
