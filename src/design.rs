use std::io::{Cursor, Read, Seek, SeekFrom};

use byteorder::{BE, LE, ReadBytesExt};

use crate::Result;

#[derive(Default)]
pub struct Target {
    pub parent_file_hash: String,
    pub name_hash: i64,
    pub size: i32,
    pub offset: i32,
}

pub fn find_target(design_bytes: &[u8]) -> Result<Target> {
    let mut cursor = Cursor::new(design_bytes);
    v3(&mut cursor, -5186779221241758859).or_else(|_| {
        cursor.seek(SeekFrom::Start(0x0))?;
        v2(&mut cursor, -515329346).or_else(|_| {
            cursor.seek(SeekFrom::Start(0x0))?;
            v1(&mut cursor, -515329346)
        })
    })
}

fn v3(cursor: &mut Cursor<&[u8]>, target_hash: i64) -> Result<Target> {
    let _ = cursor.read_u64::<LE>()?;
    let file_count = cursor.read_u32::<BE>()?;
    let _ = cursor.read_u32::<LE>()?;

    let mut target = Target::default();

    for _ in 0..file_count {
        let _ = cursor.read_i32::<BE>()?;
        let _ = cursor.read_u32::<BE>(); // v3 new 4 bytes
        let mut file_hash_bytes = [0u8; 0x10];
        cursor.read_exact(&mut file_hash_bytes)?;
        target.parent_file_hash = hex::encode(file_hash_bytes);
        let _ = cursor.read_u64::<BE>()?;

        let entry_count = cursor.read_u32::<BE>()?;
        for _ in 0..entry_count {
            let name_hash = cursor.read_i64::<BE>()?; // v3 i32 -> i64
            let size = cursor.read_i32::<BE>()?;
            let offset = cursor.read_i32::<BE>()?;
            if name_hash == target_hash {
                target.name_hash = name_hash;
                target.size = size;
                target.offset = offset;
                break;
            } else {
                continue;
            }
        }

        if target.name_hash != 0 {
            break;
        }

        let _ = cursor.read_u24::<BE>()?;
    }

    Ok(target)
}

fn v2(cursor: &mut Cursor<&[u8]>, target_hash: i32) -> Result<Target> {
    let _ = cursor.read_u64::<LE>()?;
    let file_count = cursor.read_u32::<BE>()?;
    let _ = cursor.read_u32::<LE>()?;

    let mut target = Target::default();

    for _ in 0..file_count {
        let _ = cursor.read_i32::<BE>()?;
        let mut file_hash_bytes = [0u8; 0x10];
        cursor.read_exact(&mut file_hash_bytes)?;
        target.parent_file_hash = hex::encode(file_hash_bytes);
        let _ = cursor.read_u64::<BE>()?;

        let entry_count = cursor.read_u32::<BE>()?;
        for _ in 0..entry_count {
            let name_hash = cursor.read_i32::<BE>()?;
            let size = cursor.read_i32::<BE>()?;
            let offset = cursor.read_i32::<BE>()?;
            if name_hash == target_hash {
                target.name_hash = name_hash as i64;
                target.size = size;
                target.offset = offset;
                break;
            } else {
                continue;
            }
        }

        if target.name_hash != 0 {
            break;
        }

        let _ = cursor.read_u24::<BE>()?; // v2 u8 -> u24
    }

    Ok(target)
}

fn v1(cursor: &mut Cursor<&[u8]>, target_hash: i32) -> Result<Target> {
    let _ = cursor.read_u64::<LE>()?;
    let file_count = cursor.read_u32::<BE>()?;
    let _ = cursor.read_u32::<LE>()?;

    let mut target = Target::default();

    for _ in 0..file_count {
        let _ = cursor.read_i32::<BE>()?;
        let mut file_hash_bytes = [0u8; 0x10];
        cursor.read_exact(&mut file_hash_bytes)?;
        target.parent_file_hash = hex::encode(file_hash_bytes);
        let _ = cursor.read_u64::<BE>()?;

        let entry_count = cursor.read_u32::<BE>()?;
        for _ in 0..entry_count {
            let name_hash = cursor.read_i32::<BE>()?;
            let size = cursor.read_i32::<BE>()?;
            let offset = cursor.read_i32::<BE>()?;
            if name_hash == target_hash {
                target.name_hash = name_hash as i64;
                target.size = size;
                target.offset = offset;
                break;
            } else {
                continue;
            }
        }

        if target.name_hash != 0 {
            break;
        }

        let _ = cursor.read_u8()?;
    }

    Ok(target)
}
