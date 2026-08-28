// sst
use std::{
    collections::HashMap,
    io::Cursor,
};

use quick_xml::{
    Reader,
    Writer,
    events::{
        BytesStart,
        Event,
    },
};

use super::{
    CellValue,
    SharedStringItem,
};
use crate::{
    helper::const_str::SHEET_MAIN_NS,
    reader::driver::xml_read_loop,
    writer::driver::{
        write_end_tag,
        write_start_tag,
    },
};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Default, Debug)]
pub(crate) struct SharedStringTable {
    shared_string_item: Vec<SharedStringItem>,
    map:                HashMap<u64, usize>,
    regist_count:       usize,
}

impl SharedStringTable {
    #[inline]
    pub(crate) fn shared_string_item(&self) -> &[SharedStringItem] {
        &self.shared_string_item
    }

    #[inline]
    #[deprecated(since = "3.0.0", note = "Use shared_string_item()")]
    pub(crate) fn get_shared_string_item(&self) -> &[SharedStringItem] {
        self.shared_string_item()
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn shared_string_item_mut(&mut self) -> &mut Vec<SharedStringItem> {
        &mut self.shared_string_item
    }

    #[inline]
    #[allow(dead_code)]
    #[deprecated(since = "3.0.0", note = "Use shared_string_item_mut()")]
    pub(crate) fn get_shared_string_item_mut(&mut self) -> &mut Vec<SharedStringItem> {
        self.shared_string_item_mut()
    }

    #[inline]
    pub(crate) fn set_shared_string_item(&mut self, value: SharedStringItem) -> &mut Self {
        self.shared_string_item.push(value);
        self
    }

    #[inline]
    pub(crate) fn has_value(&self) -> bool {
        !self.shared_string_item.is_empty()
    }

    pub(crate) fn set_cell(&mut self, value: &CellValue) -> usize {
        self.regist_count += 1;

        let mut shared_string_item = SharedStringItem::default();

        if let Some(v) = value.text() {
            shared_string_item.set_text(v);
        }
        if let Some(v) = value.rich_text() {
            shared_string_item.set_rich_text(v);
        }

        let hash_code = shared_string_item.hash_u64();
        let n = if let Some(v) = self.map.get(&hash_code) {
            *v
        } else {
            let n = self.shared_string_item.len();
            self.map.insert(hash_code, n);
            self.set_shared_string_item(shared_string_item);
            n
        };
        n
    }

    pub(crate) fn set_attributes<R: std::io::BufRead>(
        &mut self,
        reader: &mut Reader<R>,
        _e: &BytesStart,
    ) {
        let mut n: usize = 0;
        xml_read_loop!(
            reader,
            Event::Start(ref e) => {
                if e.name().local_name().into_inner() == b"si" {
                    let mut shared_string_item = SharedStringItem::default();
                    shared_string_item.set_attributes(reader, e);

                    let hash_code = shared_string_item.hash_u64();
                    self.map.insert(hash_code, n);
                    self.set_shared_string_item(shared_string_item);

                    n += 1;
                }
            },
            // `<si/>` is an EMPTY shared string. Its POSITION is the index
            // cells refer to, so dropping it shifts every later string and
            // silently puts the WRONG TEXT in cells — a quiet corruption
            // rather than a crash. Push a default and advance in step.
            Event::Empty(ref e) => {
                if e.name().local_name().into_inner() == b"si" {
                    let shared_string_item = SharedStringItem::default();
                    let hash_code = shared_string_item.hash_u64();
                    self.map.insert(hash_code, n);
                    self.set_shared_string_item(shared_string_item);
                    n += 1;
                }
            },
            Event::End(ref e) => {
                if e.name().local_name().into_inner() == b"sst" {
                    return
                }
            },
            Event::Eof => panic!("Error: Could not find {} end element", "sst")
        );
    }

    pub(crate) fn write_to(&self, writer: &mut Writer<Cursor<Vec<u8>>>) {
        // sst
        write_start_tag(
            writer,
            "sst",
            vec![
                ("xmlns", SHEET_MAIN_NS).into(),
                ("count", self.regist_count.to_string()).into(),
                (
                    "uniqueCount",
                    &self.shared_string_item.len().to_string(),
                )
                    .into(),
            ],
            false,
        );

        // si
        for obj in &self.shared_string_item {
            obj.write_to(writer);
        }

        write_end_tag(writer, "sst");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(xml: &str) -> SharedStringTable {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(false);
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().local_name().into_inner() == b"sst" => {
                    let start = e.clone();
                    let mut obj = SharedStringTable::default();
                    obj.set_attributes(&mut reader, &start);
                    return obj;
                }
                Ok(Event::Eof) => panic!("no <sst> in fixture"),
                Ok(_) => {}
                Err(e) => panic!("xml error: {e:?}"),
            }
        }
    }

    /// A self-closing `<si/>` is an EMPTY shared string and still owns its index.
    ///
    /// The reader handled only `Event::Start`, so `<si/>` was dropped and every
    /// later string shifted down one. Cells index into this table by position,
    /// so the failure mode is silently WRONG CELL TEXT, not a crash — which is
    /// why it needs a test rather than a bug report.
    #[test]
    fn self_closing_si_keeps_later_strings_aligned() {
        let obj = parse(concat!(
            r#"<sst count="3" uniqueCount="3">"#,
            "<si/>",
            "<si><t>SECOND</t></si>",
            "<si><t>THIRD</t></si>",
            "</sst>",
        ));
        let items = obj.shared_string_item();
        assert_eq!(items.len(), 3, "the empty <si/> must occupy index 0");
        let text_at = |i: usize| {
            items[i]
                .text()
                .map(|t| t.value().to_string())
                .unwrap_or_default()
        };
        assert_eq!(text_at(0), "", "index 0 is the empty <si/>");
        assert_eq!(text_at(1), "SECOND", "index 1 must NOT have shifted to 0");
        assert_eq!(text_at(2), "THIRD");
    }
}
