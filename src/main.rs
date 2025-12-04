use std::{
    env,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom, Write},
    path::PathBuf,
    process,
};

use anyhow::{Context, anyhow};
use crossterm::{ExecutableCommand, style::Stylize, terminal::SetTitle};
use inquire::Select;

use crate::{allowed_language::AllowedLanguage, design_data::DesignData};

mod allowed_language;
mod design_data;

pub type Result<T> = anyhow::Result<T>;

fn main() -> Result<()> {
    let _ = io::stdout().execute(SetTitle(format!(
        "{} v{} | Made by nie",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )));

    let mut args = env::args().skip(1);
    let game_path_arg = args.next();
    let lang_arg = args.next();

    let should_pause = env::args().len() == 1;

    let lang_choices = lang_arg.as_deref().map(parse_lang_arg).transpose()?;

    match run(game_path_arg, lang_choices, should_pause) {
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

pub fn run(
    game_path_arg: Option<String>,
    lang_choices: Option<(&str, &str)>,
    should_pause: bool,
) -> Result<()> {
    let design_data_dir = get_design_data_dir(game_path_arg)?;

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

    let (text_lang, voice_lang) = if let Some((text, voice)) = lang_choices {
        (text, voice)
    } else {
        let langs = vec!["cn", "en", "kr", "jp"];
        let voice =
            Select::new("What language should be used for voice?", langs.clone()).prompt()?;
        let text = Select::new("What language should be used for text?", langs).prompt()?;
        (text, voice)
    };

    // type None is text
    // type Some(1) is voice
    for (area, r#type, lang) in [
        ("os", None, &text_lang),
        ("cn", Some(1), &voice_lang),
        ("os", Some(1), &voice_lang),
        ("cn", None, &text_lang),
    ] {
        let target_row = parsed_excel
            .iter_mut()
            .find(|row| row.area == Some(area.to_string()) && row.r#type == r#type)
            .context(format!(
                "{} AllowedLanguageRow not found",
                area.to_uppercase()
            ))?;

        target_row.default_language = Some(lang.to_string());
        target_row.language_list = Some(vec![lang.to_string()]);
    }

    let data = allowed_language.serialize_rows(parsed_excel)?;

    let file_path = design_data_dir.join(format!("{}.bytes", excel_file.file_hash));

    let mut target_file = OpenOptions::new().read(true).write(true).open(file_path)?;
    target_file.seek(SeekFrom::Start(excel_data.offset as u64))?;
    target_file.write_all(&data)?;

    if data.len() < excel_data.size as usize {
        // Our modified excel is smaller so lets fill the other bytes with zeros
        // I dont think its possible to go over excel_data.size without changing the DesignData struct values and we dont want that

        let remaining_bytes = excel_data.size as usize - data.len();
        let zeros = vec![0u8; remaining_bytes];
        target_file.write_all(&zeros)?;
    }

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

fn get_design_data_dir(arg: Option<String>) -> Result<PathBuf> {
    let path = arg.map_or(env::current_dir()?, |p| PathBuf::from(p));

    if path.join("StarRail.exe").is_file() {
        return Ok(path.join("StarRail_Data/StreamingAssets/DesignData/Windows"));
    }

    if path.join("M_DesignV.bytes").is_file() {
        return Ok(path);
    }

    Err(anyhow!(
        "Needed files not found!\n\
        Make sure to either: \n\
        - Run this .exe from the game's root folder\n\
        - Pass the game root path as an argument\n\
        - Pass the DesignData folder path as an argument"
    ))
}

fn parse_lang_arg(arg: &str) -> Result<(&str, &str)> {
    if !arg.starts_with("-lang:") {
        return Err(anyhow!("Argument must start with '-lang:'"));
    }

    let lang_part = &arg[6..];

    let parts: Vec<&str> = lang_part.split(',').collect();

    if parts.len() != 2 {
        return Err(anyhow!("Expected format: -lang:0en,1en"));
    }

    let mut text_lang = None;
    let mut voice_lang = None;

    for part in parts {
        if part.len() < 3 {
            return Err(anyhow!("Invalid language format"));
        }

        let type_char = &part[0..1];
        let lang = &part[1..];

        if !["cn", "en", "kr", "jp"].contains(&lang) {
            return Err(anyhow!(
                "Invalid language '{lang}'. Must be cn, en, kr, or jp"
            ));
        }

        match type_char {
            "0" => text_lang = Some(lang),
            "1" => voice_lang = Some(lang),
            _ => return Err(anyhow!("Language type must be 0 (text) or 1 (voice)")),
        }
    }

    let text = text_lang.context("Missing text language (0)")?;
    let voice = voice_lang.context("Missing voice language (1)")?;

    Ok((text, voice))
}

fn wait_for_exit() {
    print!("Press enter to exit");
    let _ = io::stdout().flush();
    let _ = io::stdin().read_line(&mut String::new());
}
