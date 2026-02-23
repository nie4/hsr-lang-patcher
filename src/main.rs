use std::{
    env,
    fs::{self, File},
    io::{Seek, SeekFrom, Write, stdin, stdout},
    path::{Path, PathBuf},
    process,
};

use anyhow::{Context, anyhow};
use crossterm::{ExecutableCommand, style::Stylize, terminal::SetTitle};

use crate::{
    allowed_language::{AllowedLanguage, AllowedLanguageRow},
    args::Args,
    design_index::DesignIndex,
};

mod allowed_language;
mod args;
mod design_index;

pub type Result<T> = anyhow::Result<T>;

fn main() {
    let _ = stdout().execute(SetTitle(format!(
        "{} v{} | Made by nie",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )));

    let should_pause = env::args().len() == 1;

    match run(should_pause) {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("{}: {:?}", "error".red(), e);

            if should_pause {
                wait_for_exit();
            }

            process::exit(1)
        }
    }
}

pub fn run(should_pause: bool) -> Result<()> {
    let args = Args::parse()?;
    let design_data_path = get_design_data_path(args.game_path.as_deref())?;

    let m_design_v_path = design_data_path.join("M_DesignV.bytes");
    let index_hash = get_index_hash(&fs::read(&m_design_v_path)?).with_context(|| {
        format!(
            "Failed to get index hash. Is '{}' the correct directory?",
            m_design_v_path.display()
        )
    })?;

    let design_v_data = fs::read(design_data_path.join(format!("DesignV_{index_hash}.bytes")))?;
    let design_index = DesignIndex::parse(&design_v_data).context("Failed to parse DesignV")?;

    let (data_entry, file_entry) = design_index
        .find_by_hash(-515329346)
        .context("Failed to find the correct excel lol")?;

    let bytes_path = design_data_path.join(format!("{}.bytes", file_entry.file_hash));

    let allowed_language = AllowedLanguage::new(data_entry, &bytes_path);
    let mut allowed_language_rows = allowed_language.parse()?;

    let (text_lang, voice_lang) = args.get_or_prompt_languages()?;
    patch_languages(&mut allowed_language_rows, text_lang, voice_lang)?;

    let data = allowed_language.serialize_rows(allowed_language_rows)?;

    write_data(
        &bytes_path,
        data_entry.offset as u64,
        &data,
        data_entry.size as usize,
    )?;

    println!("{}", "Done".bold().green());

    if should_pause {
        wait_for_exit();
    }

    Ok(())
}

fn patch_languages(
    rows: &mut [AllowedLanguageRow],
    text_lang: &str,
    voice_lang: &str,
) -> Result<()> {
    for (area, lang, voice) in [
        ("os", text_lang, false),
        ("cn", voice_lang, true),
        ("os", voice_lang, true),
        ("cn", text_lang, false),
    ] {
        rows.iter_mut()
            .find(|row| {
                row.area() == Some(area) && if voice { row.is_voice() } else { row.is_text() }
            })
            .with_context(|| format!("{} AllowedLanguageRow not found", area.to_uppercase()))?
            .update_language(lang);
    }

    Ok(())
}

fn get_index_hash(data: &[u8]) -> Result<String> {
    let mut hash = [0u8; 16];
    let mut index = 0;
    for i in 0..4 {
        let offset = 0x1C + (i * 4);
        let chunk = data
            .get(offset..offset + 4)
            .context("M_DesignV.bytes is too short")?;
        for &byte in chunk.iter().rev() {
            hash[index] = byte;
            index += 1;
        }
    }
    Ok(hex::encode(hash))
}

fn get_design_data_path(arg: Option<&str>) -> Result<PathBuf> {
    let path = arg.map_or(env::current_dir()?, |p| PathBuf::from(p));

    if path.join("StarRail.exe").is_file() {
        return Ok(path.join("StarRail_Data/StreamingAssets/DesignData/Windows"));
    }

    if path.join("M_DesignV.bytes").is_file() {
        return Ok(path);
    }

    Err(anyhow!(
        "Could not find required files!\n\
        Make sure to either: \n\
        - Run this .exe from the game's root folder\n\
        - Pass the game's root path as an argument\n\
        - Pass the StreamingAssets/DesignData folder path as an argument"
    ))
}

fn write_data(file_path: &Path, offset: u64, data: &[u8], data_size: usize) -> Result<()> {
    let mut file = File::options().read(true).write(true).open(file_path)?;
    file.seek(SeekFrom::Start(offset))?;
    file.write_all(data)?;

    if data.len() < data_size {
        file.write_all(&vec![0; data_size - data.len()])?;
    }

    Ok(())
}

fn wait_for_exit() {
    print!("Press enter to exit");
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut String::new());
}
