use std::{
    env,
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    path::PathBuf,
    process,
};

use anyhow::{Context, anyhow};
use crossterm::{ExecutableCommand, style::Stylize, terminal::SetTitle};

use crate::{allowed_language::AllowedLanguage, args::Args, design_data::DesignData};

mod allowed_language;
mod args;
mod design_data;

pub type Result<T> = anyhow::Result<T>;

fn main() {
    let _ = io::stdout().execute(SetTitle(format!(
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

    let design_data_dir = get_design_data_dir(args.game_path.as_deref())?;
    let index_hash = get_index_hash(&design_data_dir).with_context(|| {
        format!(
            "Failed to get index hash. Are you sure this is the correct directory: '{}'?",
            design_data_dir.display()
        )
    })?;

    let design_v_file = design_data_dir.join(format!("DesignV_{index_hash}.bytes"));
    let design_data = DesignData::parse(&design_v_file)
        .with_context(|| format!("Failed to parse {}", design_v_file.display()))?;

    let (excel_data, excel_file) = design_data
        .find_excel_data_and_file(-515329346)
        .context("Failed to find the correct excel lol")?;

    let allowed_language = AllowedLanguage::new(&design_data_dir, excel_data, excel_file);
    let mut parsed_excel = allowed_language.parse()?;

    let (text_lang, voice_lang) = args.get_or_prompt_languages()?;

    // type None is text
    // type Some(1) is voice
    for (area, r#type, lang) in [
        ("os", None, &text_lang),
        ("cn", Some(1), &voice_lang),
        ("os", Some(1), &voice_lang),
        ("cn", None, &text_lang),
    ] {
        parsed_excel
            .iter_mut()
            .find(|row| row.area() == Some(area.to_string()) && row.r#type() == r#type)
            .with_context(|| format!("{} AllowedLanguageRow not found", area.to_uppercase()))?
            .update_language(lang);
    }

    let data = allowed_language.serialize_rows(parsed_excel)?;
    let file_path = design_data_dir.join(format!("{}.bytes", excel_file.file_hash));

    write_excel_data(
        &file_path,
        excel_data.offset as u64,
        &data,
        excel_data.size as usize,
    )?;

    println!("{}", "Done".bold().green());
    if should_pause {
        wait_for_exit();
    }

    Ok(())
}

fn get_index_hash(design_data_dir: &PathBuf) -> Result<String> {
    let path = design_data_dir.join("M_DesignV.bytes");
    let mut file = File::open(path)?;

    file.seek(SeekFrom::Start(0x1C))?;

    let mut hash = [0u8; 0x10];
    let mut index = 0;
    for _ in 0..4 {
        let mut chunk = [0u8; 4];
        file.read_exact(&mut chunk)?;

        for byte_pos in (0..4).rev() {
            hash[index] = chunk[byte_pos];
            index += 1;
        }
    }

    Ok(hex::encode(hash))
}

fn get_design_data_dir(arg: Option<&str>) -> Result<PathBuf> {
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
        - Pass the DesignData folder path as an argument"
    ))
}

fn write_excel_data(
    file_path: &PathBuf,
    offset: u64,
    data: &[u8],
    excel_size: usize,
) -> Result<()> {
    let mut file = File::options().read(true).write(true).open(file_path)?;
    file.seek(io::SeekFrom::Start(offset))?;
    file.write_all(data)?;

    if data.len() < excel_size {
        file.write_all(&vec![0u8; excel_size - data.len()])?;
    }

    Ok(())
}

fn wait_for_exit() {
    print!("Press enter to exit");
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut String::new());
}
