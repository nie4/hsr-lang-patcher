use std::{
    fs::File,
    io::{BufReader, Cursor, Read},
    path::Path,
};

use byteorder::{BE, LE, ReadBytesExt};
use eyre::Result;

use crate::STREAMING_ASSETS_PATH;

#[derive(Default, Debug)]
pub struct DataEntry {
    pub name_hash: i32,
    pub size: i32,
    pub offset: i32,
}

#[derive(Default, Debug)]
pub struct FileEntry {
    pub name_hash: i32,
    pub file_hash: String,
    pub read_size: u64,
    pub entry_count: u32,
    pub entries: Vec<DataEntry>,
    pub unk_1: u8,
}

#[derive(Default, Debug)]
pub struct DesignDataHeader {
    pub unk_1: u64,
    pub file_count: u32,
    pub unk_2: u32,
    pub files: Vec<FileEntry>,
}

pub struct DesignData {
    header: DesignDataHeader,
}

impl DesignData {
    pub fn parse<T: AsRef<Path>>(game_path: T, index_hash: &str) -> Result<Self> {
        let design_v_path = format!("{STREAMING_ASSETS_PATH}/DesignV_{index_hash}.bytes",);
        let path = game_path.as_ref().join(design_v_path);

        let file = File::open(path)?;
        let mut reader = BufReader::new(&file);

        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let mut cursor = Cursor::new(buffer);

        let mut header = DesignDataHeader {
            unk_1: cursor.read_u64::<LE>()?,
            file_count: cursor.read_u32::<BE>()?,
            unk_2: cursor.read_u32::<LE>()?,
            files: Default::default(),
        };

        for _ in 0..header.file_count {
            let name_hash = cursor.read_i32::<BE>()?;

            let mut file_hash_bytes = [0u8; 0x10];
            cursor.read_exact(&mut file_hash_bytes)?;
            let file_hash = hex::encode(file_hash_bytes);

            let read_size = cursor.read_u64::<BE>()?;
            let entry_count = cursor.read_u32::<BE>()?;

            let mut entries = Vec::with_capacity(entry_count as usize);
            for _ in 0..entry_count {
                entries.push(DataEntry {
                    name_hash: cursor.read_i32::<BE>()?,
                    size: cursor.read_i32::<BE>()?,
                    offset: cursor.read_i32::<BE>()?,
                });
            }

            header.files.push(FileEntry {
                name_hash,
                file_hash,
                read_size,
                entry_count,
                entries,
                unk_1: cursor.read_u8()?,
            });
        }

        Ok(Self { header })
    }

    pub fn find_excel_data_and_file(&self, target_hash: i32) -> Option<(&DataEntry, &FileEntry)> {
        self.header.files.iter().find_map(|file| {
            file.entries
                .iter()
                .find(|entry| entry.name_hash == target_hash)
                .map(|entry| (entry, file))
        })
    }
}
