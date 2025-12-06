use std::env;

use anyhow::anyhow;
use inquire::Select;

use crate::Result;

const VALID_LANGUAGES: [&str; 4] = ["cn", "en", "kr", "jp"];

pub struct Args {
    pub game_path: Option<String>,
    pub languages: Option<Languages>,
}

pub struct Languages {
    pub text: &'static str,
    pub voice: &'static str,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let args: Vec<String> = env::args().skip(1).collect();

        let mut game_path = None;
        let mut languages = None;

        for arg in &args {
            if let Some(stripped) = arg.strip_prefix('-') {
                if stripped.starts_with("lang:") {
                    languages = Some(Languages::from_arg(arg)?)
                } else {
                    return Err(anyhow!("Unknown argument: '{arg}'"));
                }
            } else if game_path.is_none() {
                game_path = Some(arg.clone());
            }
        }

        Ok(Self {
            game_path,
            languages,
        })
    }

    pub fn get_or_prompt_languages(&self) -> Result<(&'static str, &'static str)> {
        if let Some(langs) = &self.languages {
            return Ok((langs.text, langs.voice));
        }

        let voice = Select::new(
            "What language should be used for voice?",
            VALID_LANGUAGES.to_vec(),
        )
        .prompt()?;
        let text = Select::new(
            "What language should be used for text?",
            VALID_LANGUAGES.to_vec(),
        )
        .prompt()?;

        Ok((
            Self::validate_language(text)?,
            Self::validate_language(voice)?,
        ))
    }

    fn validate_language(lang: &str) -> Result<&'static str> {
        VALID_LANGUAGES
            .iter()
            .find(|&&l| l == lang)
            .ok_or_else(|| anyhow!("Invalid language '{lang}'. Must be cn, en, kr, or jp"))
            .copied()
    }
}

impl Languages {
    fn from_arg(arg: &str) -> Result<Self> {
        let lang_part = arg
            .strip_prefix("-lang:")
            .ok_or_else(|| anyhow!("Argument must start with '-lang:'"))?;

        let parts: Vec<&str> = lang_part.split(',').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Expected format: -lang:0en,1en"));
        }

        let mut text = None;
        let mut voice = None;

        for part in parts {
            if part.len() < 3 {
                return Err(anyhow!("Invalid language format"));
            }

            let (type_char, lang) = part.split_at(1);
            Args::validate_language(lang)?;

            match type_char {
                "0" => text = Some(lang),
                "1" => voice = Some(lang),
                _ => return Err(anyhow!("Language type must be 0 (text) or 1 (voice)")),
            }
        }

        Ok(Self {
            text: Args::validate_language(
                text.ok_or_else(|| anyhow!("Missing text language (0)"))?,
            )?,
            voice: Args::validate_language(
                voice.ok_or_else(|| anyhow!("Missing voice language (1)"))?,
            )?,
        })
    }
}
