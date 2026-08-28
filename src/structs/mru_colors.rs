// mruColors
use std::io::Cursor;

use quick_xml::{
    Reader,
    Writer,
    events::{
        BytesStart,
        Event,
    },
};

use super::Color;
use crate::{
    reader::driver::xml_read_loop,
    writer::driver::{
        write_end_tag,
        write_start_tag,
    },
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Debug)]
pub(crate) struct MruColors {
    color: Vec<Color>,
}

impl MruColors {
    #[inline]
    pub(crate) fn color(&self) -> &[Color] {
        &self.color
    }

    #[inline]
    #[deprecated(since = "3.0.0", note = "Use color()")]
    pub(crate) fn get_color(&self) -> &[Color] {
        self.color()
    }

    #[inline]
    pub(crate) fn color_mut(&mut self) -> &mut Vec<Color> {
        &mut self.color
    }

    #[inline]
    #[deprecated(since = "3.0.0", note = "Use color_mut()")]
    pub(crate) fn get_color_mut(&mut self) -> &mut Vec<Color> {
        self.color_mut()
    }

    #[inline]
    pub(crate) fn set_color(&mut self, value: Color) -> &mut Self {
        self.color.push(value);
        self
    }

    pub(crate) fn set_attributes<R: std::io::BufRead>(
        &mut self,
        reader: &mut Reader<R>,
        _e: &BytesStart,
    ) {
        xml_read_loop!(
            reader,
            Event::Start(ref e) => {
                if e.name().local_name().into_inner() == b"color" {
                    let mut obj = Color::default();
                    obj.set_attributes(reader, e, true);
                    self.set_color(obj);
                }
            },
            // `<color rgb="…"/>` is the NORMAL spelling — a colour carries its
            // value in attributes and has no children — so without this arm the
            // MRU colour list parses as empty every time. `empty_flg = true`
            // makes `Color::set_attributes` read the attributes and return
            // without hunting for an end tag.
            Event::Empty(ref e) => {
                if e.name().local_name().into_inner() == b"color" {
                    let mut obj = Color::default();
                    obj.set_attributes(reader, e, true);
                    self.set_color(obj);
                }
            },
            Event::End(ref e) => {
                if e.name().local_name().into_inner() == b"mruColors" {
                    return
                }
            },
            Event::Eof => panic!("Error: Could not find {} end element", "mruColors")
        );
    }

    pub(crate) fn write_to(&self, writer: &mut Writer<Cursor<Vec<u8>>>) {
        if !self.color.is_empty() {
            // mruColors
            write_start_tag(writer, "mruColors", vec![], false);

            // color
            for color in &self.color {
                color.write_to_color(writer);
            }

            write_end_tag(writer, "mruColors");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `<color rgb="…"/>` is self-closing, which is the ONLY spelling Excel
    /// writes here — so before the `Event::Empty` arm existed this list parsed
    /// as empty every single time and the MRU colours were silently lost.
    #[test]
    fn self_closing_colors_are_parsed() {
        let xml = concat!(
            "<colors><mruColors>",
            r#"<color rgb="FFFF0000"/><color rgb="FF00FF00"/>"#,
            "</mruColors></colors>",
        );
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(false);
        let mut buf = Vec::new();
        let obj = loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().local_name().into_inner() == b"mruColors" => {
                    let start = e.clone();
                    let mut obj = MruColors::default();
                    obj.set_attributes(&mut reader, &start);
                    break obj;
                }
                Ok(Event::Eof) => panic!("no <mruColors> in fixture"),
                Ok(_) => {}
                Err(e) => panic!("xml error: {e:?}"),
            }
        };
        assert_eq!(obj.color().len(), 2, "both self-closing colours must parse");
        let c0 = obj.color()[0].argb();
        let c1 = obj.color()[1].argb();
        assert_eq!((c0.a, c0.r, c0.g, c0.b), (0xFF, 0xFF, 0x00, 0x00), "first is red");
        assert_eq!((c1.a, c1.r, c1.g, c1.b), (0xFF, 0x00, 0xFF, 0x00), "second is green");
    }
}
