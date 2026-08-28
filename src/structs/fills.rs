// fills
use std::io::Cursor;

use quick_xml::{
    Reader,
    Writer,
    events::{
        BytesStart,
        Event,
    },
};

use super::{
    Fill,
    Style,
};
use crate::{
    reader::driver::xml_read_loop,
    writer::driver::{
        write_end_tag,
        write_start_tag,
    },
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Debug)]
pub(crate) struct Fills {
    fill: Vec<Fill>,
}

impl Fills {
    #[inline]
    pub(crate) fn fill(&self) -> &[Fill] {
        &self.fill
    }

    #[inline]
    #[deprecated(since = "3.0.0", note = "Use fill()")]
    pub(crate) fn get_fill(&self) -> &[Fill] {
        self.fill()
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn fill_mut(&mut self) -> &mut Vec<Fill> {
        &mut self.fill
    }

    #[inline]
    #[allow(dead_code)]
    #[deprecated(since = "3.0.0", note = "Use fill_mut()")]
    pub(crate) fn get_fill_mut(&mut self) -> &mut Vec<Fill> {
        self.fill_mut()
    }

    #[inline]
    pub(crate) fn set_fill(&mut self, value: Fill) -> &mut Self {
        self.fill.push(value);
        self
    }

    pub(crate) fn set_style(&mut self, style: &Style) -> u32 {
        match style.fill() {
            Some(v) => {
                let hash_code = v.hash_code();
                let mut id = 0;
                for fill in &self.fill {
                    if fill.hash_code() == hash_code {
                        return id;
                    }
                    id += 1;
                }
                self.set_fill(v.clone());
                id
            }
            None => 0,
        }
    }

    pub(crate) fn set_attributes<R: std::io::BufRead>(
        &mut self,
        reader: &mut Reader<R>,
        _e: &BytesStart,
    ) {
        xml_read_loop!(
            reader,
            Event::Start(ref e) => {
                if e.name().local_name().into_inner() == b"fill" {
                    let mut obj = Fill::default();
                    obj.set_attributes(reader, e);
                    self.set_fill(obj);
                }
            },
            // `<fill/>` occupies a fillId slot; dropping it shifts every later
            // fillId. See `differential_formats.rs` for why `set_attributes`
            // must not be called on an Empty event.
            Event::Empty(ref e) => {
                if e.name().local_name().into_inner() == b"fill" {
                    self.set_fill(Fill::default());
                }
            },
            Event::End(ref e) => {
                if e.name().local_name().into_inner() == b"fills" {
                    return
                }
            },
            Event::Eof => panic!("Error: Could not find {} end element", "fills")
        );
    }

    pub(crate) fn write_to(&self, writer: &mut Writer<Cursor<Vec<u8>>>) {
        if !self.fill.is_empty() {
            // fills
            write_start_tag(
                writer,
                "fills",
                vec![("count", &self.fill.len().to_string()).into()],
                false,
            );

            // fill
            for fill in &self.fill {
                fill.write_to(writer);
            }

            write_end_tag(writer, "fills");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(xml: &str) -> Fills {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(false);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().local_name().into_inner() == b"fills" => {
                    let start = e.clone();
                    let mut obj = Fills::default();
                    obj.set_attributes(&mut reader, &start);
                    return obj;
                }
                Ok(Event::Eof) => panic!("no <fills> in fixture"),
                Ok(_) => {}
                Err(e) => panic!("xml error: {e:?}"),
            }
        }
    }

    /// A self-closing `<fill/>` still occupies a fillId slot.
    ///
    /// Cells reference fills by index, so dropping one silently repaints every
    /// cell below it with the wrong fill.
    #[test]
    fn self_closing_fill_keeps_its_index() {
        let obj = parse(concat!(
            r#"<fills count="3">"#,
            "<fill/>",
            r#"<fill><patternFill patternType="solid"/></fill>"#,
            "<fill/>",
            "</fills>",
        ));
        assert_eq!(obj.fill().len(), 3, "all three fills must be present");
        assert!(
            obj.fill()[1].pattern_fill().is_some(),
            "the solid fill must still be at fillId 1, not shifted to 0"
        );
    }
}
