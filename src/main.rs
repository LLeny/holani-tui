use app::App;
use clap::Parser;
use keycodes::translate_keycode;
use ratatui::crossterm::{event::KeyCode, terminal::{disable_raw_mode, enable_raw_mode}};
use runner::runner_config::{Input, RunnerConfig};
use std::path::PathBuf;

pub(crate) mod keycodes;
pub(crate) mod runner;
pub(crate) mod sound_source;
pub(crate) mod app;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Cartright, can be .o or a .lnx file
    #[arg(short, long)]
    cartridge: PathBuf,

    /// ROM override
    #[arg(short, long)]
    rom: Option<PathBuf>,

    /// Buttons mapping <up>,<down>,<left>,<right>,<out>,<in>,<o1>,<o2>,<pause>
    #[arg(
        short,
        long,
        value_delimiter = ',',
        default_value = "up,down,left,right,q,w,1,2,p"
    )]
    buttons: Option<Vec<String>>,

    /// Mute sound
    #[arg(short, long, default_value_t = false)]
    mute: bool,

    /// Enable Comlynx
    #[arg(short('x'), long, default_value_t = false)]
    comlynx: bool,
}

fn main() {

    env_logger::init();
    let config = process_args();

    let mut terminal = ratatui::init(); 
    
    // let mut stdout = io::stdout();
    // execute!(stdout, EnterAlternateScreen, Hide).unwrap();
    enable_raw_mode().unwrap();

    let mut app = App::new(config);
    
    app.run(&mut terminal);

    disable_raw_mode().unwrap();
    ratatui::restore();
}
  
fn process_args() -> RunnerConfig {
    let args = Args::parse();

    let mut config = RunnerConfig::new();
    if let Some(rom) = args.rom {
        config.set_rom(rom);
    }
    config.set_cartridge(args.cartridge);
    config.set_mute(args.mute);
    config.set_comlynx(args.comlynx);

    let btns = args.buttons.unwrap();
    if btns.len() != 9 {
        panic!("Buttons mapping should be 9 keys.");
    }
    for (s, btn) in btns.iter().zip([
        Input::Up,
        Input::Down,
        Input::Left,
        Input::Right,
        Input::Outside,
        Input::Inside,
        Input::Option1,
        Input::Option2,
        Input::Pause,
    ]) {
        let key = translate_keycode(s);
        if key == KeyCode::Null {
            panic!("Buttons mapping: Unknown key '{}'.", s.as_str());
        }
        config.set_button_mapping(key, btn);
    }

    config
}

