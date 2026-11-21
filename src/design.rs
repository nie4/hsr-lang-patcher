use std::io::{Cursor, Read};

use anyhow::bail;
use byteorder::{BE, LE, ReadBytesExt};

use crate::Result;

pub struct Target {
    pub file_md5: String,
    pub size: i32,
    pub offset: i32,
}

pub fn find_target(design_bytes: &[u8]) -> Result<Target> {
    let mut cursor = Cursor::new(design_bytes);
    let format_version = cursor.read_u64::<BE>()? as u32;
    match format_version {
        1 => v1(&mut cursor, -515329346),
        3 => v3(&mut cursor, -515329346),
        4 => v4(&mut cursor, -5186779221241758859),
        _ => bail!("Unknown format version '{format_version}'"),
    }
}

fn v4(cursor: &mut Cursor<&[u8]>, target_hash: i64) -> Result<Target> {
    let bundle_count = cursor.read_u32::<BE>()?;
    let _ = cursor.read_u32::<LE>()?;

    for _ in 0..bundle_count {
        let _ = cursor.read_i64::<BE>(); // v4 i32 -> i64
        let mut file_md5 = [0u8; 0x10];
        cursor.read_exact(&mut file_md5)?;
        let _ = cursor.read_u64::<BE>()?;

        let entry_count = cursor.read_u32::<BE>()?;
        for _ in 0..entry_count {
            let name_hash = cursor.read_i64::<BE>()?; // v4 i32 -> i64
            let size = cursor.read_i32::<BE>()?;
            let offset = cursor.read_i32::<BE>()?;
            if name_hash == target_hash {
                return Ok(Target {
                    file_md5: hex::encode(file_md5),
                    size,
                    offset,
                });
            }
        }

        let length = cursor.read_i16::<BE>()?;
        if length > 0 {
            let mut buffer = vec![0u8; length as usize];
            let _ = cursor.read_exact(&mut buffer)?;
        }
        let _ = cursor.read_u8()?;
    }

    bail!("Failed to find targeted bundle entry")
}

fn v3(cursor: &mut Cursor<&[u8]>, target_hash: i32) -> Result<Target> {
    let bundle_count = cursor.read_u32::<BE>()?;
    let _ = cursor.read_u32::<LE>()?;

    for _ in 0..bundle_count {
        let _ = cursor.read_i32::<BE>()?;
        let mut file_md5 = [0u8; 0x10];
        cursor.read_exact(&mut file_md5)?;
        let _ = cursor.read_u64::<BE>()?;

        let entry_count = cursor.read_u32::<BE>()?;
        for _ in 0..entry_count {
            let name_hash = cursor.read_i32::<BE>()?;
            let size = cursor.read_i32::<BE>()?;
            let offset = cursor.read_i32::<BE>()?;
            if name_hash == target_hash {
                return Ok(Target {
                    file_md5: hex::encode(file_md5),
                    size,
                    offset,
                });
            }
        }

        // v3 new
        let length = cursor.read_i16::<BE>()?;
        if length > 0 {
            let mut buffer = vec![0u8; length as usize];
            let _ = cursor.read_exact(&mut buffer)?;
        }
        let _ = cursor.read_u8()?;
    }

    bail!("Failed to find targeted bundle entry")
}

fn v1(cursor: &mut Cursor<&[u8]>, target_hash: i32) -> Result<Target> {
    let bundle_count = cursor.read_u32::<BE>()?;
    let _ = cursor.read_u32::<LE>()?;

    for _ in 0..bundle_count {
        let _ = cursor.read_i32::<BE>()?;
        let mut file_md5 = [0u8; 0x10];
        cursor.read_exact(&mut file_md5)?;
        let _ = cursor.read_u64::<BE>()?;

        let entry_count = cursor.read_u32::<BE>()?;
        for _ in 0..entry_count {
            let name_hash = cursor.read_i32::<BE>()?;
            let size = cursor.read_i32::<BE>()?;
            let offset = cursor.read_i32::<BE>()?;
            if name_hash == target_hash {
                return Ok(Target {
                    file_md5: hex::encode(file_md5),
                    size,
                    offset,
                });
            }
        }

        let _ = cursor.read_u8()?;
    }

    bail!("Failed to find targeted bundle entry")
}
