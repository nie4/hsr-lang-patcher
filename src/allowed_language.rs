use std::{
    fs::File,
    io::{BufReader, Cursor, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use crate::Result;
use byteorder::{ReadBytesExt, WriteBytesExt};
use varint_rs::{VarintReader, VarintWriter};

use crate::{
    design_data::{DataEntry, FileEntry},
};

#[derive(Default, Debug)]
pub struct AllowedLanguageRow {
    pub area: Option<String>,
    pub r#type: Option<u8>,
    pub language_list: Option<Vec<String>>,
    pub default_language: Option<String>,
}

impl AllowedLanguageRow {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        let mut bitmask = 0u8;
        if self.area.is_some() {
            bitmask |= 1 << 0;
        }
        if self.r#type.is_some() {
            bitmask |= 1 << 1;
        }
        if self.language_list.is_some() {
            bitmask |= 1 << 2;
        }
        if self.default_language.is_some() {
            bitmask |= 1 << 3;
        }

        cursor.write_u8(bitmask)?;

        if let Some(ref area) = self.area {
            Self::write_string(&mut cursor, area)?;
        }
        if let Some(type_val) = self.r#type {
            cursor.write_u8(type_val)?;
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
}

pub struct AllowedLanguage<'a> {
    design_data_dir: PathBuf,
    excel_data_entry: &'a DataEntry,
    excel_file_entry: &'a FileEntry,
}

impl<'a> AllowedLanguage<'a> {
    pub fn new<T: AsRef<Path>>(
        design_data_dir: T,
        excel_data_entry: &'a DataEntry,
        excel_file_entry: &'a FileEntry,
    ) -> Self {
        Self {
            design_data_dir: design_data_dir.as_ref().to_path_buf(),
            excel_data_entry,
            excel_file_entry,
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
        let excel_path = self
            .design_data_dir
            .join(format!("{}.bytes", self.excel_file_entry.file_hash));

        let mut excel_file = BufReader::new(File::open(excel_path)?);
        excel_file.seek(SeekFrom::Start(self.excel_data_entry.offset as u64))?;

        let mut buffer = vec![0u8; self.excel_data_entry.size as usize];
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
                row.r#type = Some(cursor.read_u8()?);
            }
            if bitmask & 1 << 2 != 0 {
                row.language_list = Some(Self::read_string_array(&mut cursor)?);
            }
            if bitmask & 1 << 3 != 0 {
                row.default_language = Some(Self::read_string(&mut cursor)?);
            }

            rows.push(row);
        }

        drop(excel_file);

        Ok(rows)
    }

    #[inline]
    fn read_string(cursor: &mut Cursor<Vec<u8>>) -> Result<String> {
        let length = cursor.read_u8()? as usize;
        let mut buffer = vec![0u8; length];
        Read::read_exact(cursor, &mut buffer)?;
        unsafe { Ok(String::from_utf8_unchecked(buffer)) }
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
