use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Seek, SeekFrom, Write},
    path::Path,
};

use crate::Result;
use byteorder::{ReadBytesExt, WriteBytesExt};
use varint_rs::{VarintReader, VarintWriter};

use crate::design_index::DataEntry;

pub struct AllowedLanguage<'a> {
    data_entry: &'a DataEntry,
    bytes_path: &'a Path,
}

impl<'a> AllowedLanguage<'a> {
    pub const VALID_LANGUAGES: [&'static str; 4] = ["cn", "en", "kr", "jp"];

    pub fn new(data_entry: &'a DataEntry, bytes_path: &'a Path) -> Self {
        Self {
            data_entry,
            bytes_path,
        }
    }

    pub fn serialize_rows(&self, rows: Vec<AllowedLanguageRow>) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        cursor.write_u8(0)?;
        cursor.write_i8_varint(rows.len() as i8)?;

        for row in rows {
            let row_data = row.serialize()?;
            cursor.write_all(&row_data)?;
        }

        Ok(buffer)
    }

    pub fn parse(&self) -> Result<Vec<AllowedLanguageRow>> {
        let mut excel_file = BufReader::new(File::open(self.bytes_path)?);
        excel_file.seek(SeekFrom::Start(self.data_entry.offset as u64))?;

        let mut buffer = vec![0u8; self.data_entry.size as usize];
        excel_file.read_exact(&mut buffer)?;

        let mut cursor = Cursor::new(buffer);

        cursor.read_u8()?;

        let count = cursor.read_i8_varint()? as usize;
        let mut rows = Vec::with_capacity(count);

        for _ in 0..count {
            let bitmask = cursor.read_u8()?;
            let mut row = AllowedLanguageRow::default();

            if bitmask & 1 << 0 != 0 {
                row.area = Some(Self::read_string(&mut cursor)?);
            }
            if bitmask & 1 << 1 != 0 {
                row.row_type = Some(cursor.read_u8()?);
            }
            if bitmask & 1 << 2 != 0 {
                row.language_list = Some(Self::read_string_array(&mut cursor)?);
            }
            if bitmask & 1 << 3 != 0 {
                row.default_language = Some(Self::read_string(&mut cursor)?);
            }

            rows.push(row);
        }

        Ok(rows)
    }

    #[inline]
    fn read_string(cursor: &mut Cursor<Vec<u8>>) -> Result<String> {
        let length = cursor.read_u8()? as usize;
        let mut buffer = vec![0u8; length];
        Read::read_exact(cursor, &mut buffer)?;
        String::from_utf8(buffer).map_err(|e| anyhow::anyhow!(e))
    }

    #[inline]
    fn read_string_array(cursor: &mut Cursor<Vec<u8>>) -> Result<Vec<String>> {
        let length = cursor.read_i8_varint()? as usize;
        let mut strings = Vec::with_capacity(length);
        for _ in 0..length {
            strings.push(Self::read_string(cursor)?);
        }
        Ok(strings)
    }
} // HI

#[derive(Default, Debug)]
pub struct AllowedLanguageRow {
    area: Option<String>,
    row_type: Option<u8>,
    language_list: Option<Vec<String>>,
    default_language: Option<String>,
}

impl AllowedLanguageRow {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        let bitmask = [
            self.area.is_some(),
            self.row_type.is_some(),
            self.language_list.is_some(),
            self.default_language.is_some(),
        ]
        .iter()
        .enumerate()
        .fold(0u8, |acc, (i, &set)| acc | ((set as u8) << i));

        cursor.write_u8(bitmask)?;

        if let Some(ref area) = self.area {
            Self::write_string(&mut cursor, area)?;
        }
        if let Some(row_type) = self.row_type {
            cursor.write_u8(row_type)?;
        }
        if let Some(ref language_list) = self.language_list {
            Self::write_string_array(&mut cursor, language_list)?;
        }
        if let Some(ref default_language) = self.default_language {
            Self::write_string(&mut cursor, default_language)?;
        }

        Ok(buffer)
    }

    #[inline]
    fn write_string(cursor: &mut Cursor<&mut Vec<u8>>, s: &str) -> Result<()> {
        let bytes = s.as_bytes();
        cursor.write_u8(bytes.len() as u8)?;
        cursor.write_all(bytes)?;
        Ok(())
    }

    #[inline]
    fn write_string_array(cursor: &mut Cursor<&mut Vec<u8>>, strings: &[String]) -> Result<()> {
        cursor.write_i8_varint(strings.len() as i8)?;
        for s in strings {
            Self::write_string(cursor, s)?;
        }
        Ok(())
    }

    pub fn update_language(&mut self, lang: &str) {
        self.default_language = Some(lang.to_string());
        self.language_list = Some(vec![lang.to_string()]);
    }

    pub fn area(&self) -> Option<&str> {
        self.area.as_deref()
    }

    pub fn is_text(&self) -> bool {
        self.row_type.is_none()
    }

    pub fn is_voice(&self) -> bool {
        self.row_type == Some(1)
    }
}
