# hsr-lang-patcher
An easy to use tool for changing the in-game language

## How to use:
1. Download the latest version from [releases](https://github.com/nie4/hsr-lang-patcher/releases)
2. Move `hsr-lang-patcher.exe` to the same folder where the game is located
3. Run the .exe and follow the shown instructions
4. If you want to use voices make sure to run the client first before copying the audio folder to prevent it from being removed

## CLI usage:
```
hsr-lang-patcher [GAME_PATH | DESIGNDATA_PATH] -lang:0XX,1YY
```

### Notes
- If `hsr-lang-patcher` is placed in the **correct game directory**, you can run it **without any arguments**, and it will automatically detect the required paths.
- If you provide arguments manually, use the format described below.

### Arguments
- **GAME_PATH / DESIGNDATA_PATH**  
  The first argument can be either:
  - The path to the game's installation directory, **or**
  - The path directly to the `DesignData` folder.

- **-lang:0XX,1YY** *(optional)*  
  Sets both text and voice languages.  
  The `-lang:` argument **must always include both modes**, separated by a comma.  
  - `0` = text language  
  - `1` = voice language  
  - `XX` / `YY` are two-letter language codes (e.g., `cn`, `en`, `kr`, `jp`)

  **Example:**
  - `-lang:0en,1en` -> English text + English voice  

## Compiling:
```bash
cargo build -r
```

## Requirements:
- [rust](https://www.rust-lang.org/tools/install) for compiling

## References
- [HSR_Downloader](https://github.com/Hiro420/HSR_Downloader)
