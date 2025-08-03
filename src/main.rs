#![feature(try_blocks)]

use std::{
    env,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write, stdin, stdout},
    path::PathBuf,
};

use crossterm::{execute, style::Stylize, terminal::SetTitle};
use eyre::{OptionExt, Result, eyre};
use inquire::Select;

use crate::{allowed_language::AllowedLanguage, design_data::DesignData};

pub mod allowed_language;
pub mod design_data;

pub const STREAMING_ASSETS_PATH: &str = "StarRail_Data/StreamingAssets/DesignData/Windows";

struct AppContext {
    game_path: PathBuf,
}

impl AppContext {
    pub fn new(game_path: Option<String>) -> Result<Self> {
        Ok(Self {
            game_path: Self::determine_game_path(game_path)?,
        })
    }

    pub fn run(&self) -> Result<()> {
        let index_hash = self.get_index_hash()?;
        let design_data = DesignData::parse(&self.game_path, &index_hash)?;
        let (excel_data, excel_file) = design_data
            .find_excel_data_and_file(-515329346)
            .ok_or_eyre("Failed to find excel lol")?;

        let allowed_language = AllowedLanguage::new(&self.game_path, excel_data, excel_file);
        let mut parsed_excel = allowed_language.parse()?;

        // These are the only available langs in beta so it doesnt hurt to hardcode them
        let langs = vec!["cn", "en", "kr", "jp"];
        let voice_ans =
            Select::new("What language should be used for voice?", langs.clone()).prompt()?;
        let text_ans = Select::new("What language should be used for text?", langs).prompt()?;

        // type None is text
        // type Some(1) is voice
        for (area, r#type, lang) in [
            ("os", None, &text_ans),
            ("cn", Some(1), &voice_ans),
            ("os", Some(1), &voice_ans),
            ("cn", None, &text_ans),
        ] {
            let target_row = parsed_excel
                .iter_mut()
                .find(|row| row.area == Some(area.to_string()) && row.r#type == r#type)
                .ok_or_eyre(format!(
                    "{} AllowedLanguageRow not found",
                    area.to_uppercase()
                ))?;

            target_row.default_language = Some(lang.to_string());
            target_row.language_list = Some(vec![lang.to_string()]);
        }

        let data = allowed_language.serialize_rows(parsed_excel)?;

        let file_path = self.game_path.join(format!(
            "{STREAMING_ASSETS_PATH}/{}.bytes",
            excel_file.file_hash
        ));

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

        print!("Press enter to exit");
        stdout().flush().unwrap();
        stdin().read_line(&mut String::new()).unwrap();

        Ok(())
    }

    fn get_index_hash(&self) -> Result<String> {
        let path = self
            .game_path
            .join(format!("{STREAMING_ASSETS_PATH}/M_DesignV.bytes"));
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

    fn determine_game_path(game_path: Option<String>) -> Result<PathBuf> {
        let path = match game_path {
            Some(path) => PathBuf::from(path),
            None => env::current_dir()?,
        };

        if path.join("StarRail.exe").is_file() {
            Ok(path)
        } else {
            Err(eyre!(
                "Game path not found!\nmake sure this .exe is in the root folder or pass the game path as an argument"
            ))
        }
    }
}

fn main() {
    let result: Result<()> = try {
        execute!(
            stdout(),
            SetTitle(format!(
                "{} v{} | Made by nie",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
        )?;

        let app = AppContext::new(env::args().nth(1))?;
        app.run()?;
    };

    if let Err(e) = result {
        eprintln!("Error: {:?}", e);

        print!("Press enter to exit");
        stdout().flush().unwrap();
        stdin().read_line(&mut String::new()).unwrap();
    }
}
