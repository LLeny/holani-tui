use std::{collections::HashMap, io::Stdout, time::Duration};
use holani::{mikey::video::{LYNX_SCREEN_HEIGHT, LYNX_SCREEN_WIDTH}, suzy::registers::{Joystick, Switches}};
use ratatui::{crossterm::{self, event::{Event, KeyCode, KeyEventKind}}, layout::{Constraint, Layout}, prelude::CrosstermBackend, style::Color, symbols::Marker, widgets::canvas::{Canvas, Painter, Shape}, Terminal};

use crate::runner::{runner_config::{Input, RunnerConfig}, Runner};

const BUTTON_DECAY: u8 = 15;
const INPUT_POLL: Duration = Duration::from_millis(2);

macro_rules! set_button {
    ($slf: expr, $btn: expr, $value: expr) => {
        match $btn {
            Input::Pause => $slf.switches.set(Switches::pause, $value),
            Input::Up => $slf.joystick.set(Joystick::up, $value),
            Input::Down => $slf.joystick.set(Joystick::down, $value),
            Input::Left => $slf.joystick.set(Joystick::left, $value),
            Input::Right => $slf.joystick.set(Joystick::right, $value),
            Input::Outside => $slf.joystick.set(Joystick::outside, $value),
            Input::Inside => $slf.joystick.set(Joystick::inside, $value),
            Input::Option1 => $slf.joystick.set(Joystick::option_1, $value),
            Input::Option2 => $slf.joystick.set(Joystick::option_2, $value),
        }
    }
}

struct ScreenView<'a> {
    rgb_buffer: &'a Vec<u8>,
}

impl Shape for ScreenView<'_> {
    fn draw(&self, painter: &mut Painter) {
        self.rgb_buffer.chunks_exact(3).enumerate().for_each(|(i, rgb)|{
            let x = i % LYNX_SCREEN_WIDTH as usize;
            let y = i / LYNX_SCREEN_WIDTH as usize;
            painter.paint(x, y, Color::Rgb(rgb[0], rgb[1], rgb[2]));
        }); 
    }
}

pub(crate) struct App {
    keyboard_frames: HashMap<Input, u8>,
    joystick: Joystick,
    switches: Switches,
    config: RunnerConfig,
    input_tx: kanal::Sender<(u8, u8)>,
    _runner: Runner,
    update_display_rx: kanal::Receiver<Vec<u8>>,
}

impl App {
    pub fn new(config: RunnerConfig) -> Self {

        let mut runner = Runner::new(config.clone());
        let (input_tx, update_display_rx, _rotation) = runner.initialize_thread();
    
        Self {
            keyboard_frames: HashMap::new(),
            joystick: Joystick::empty(),
            switches: Switches::empty(),
            config,
            input_tx,
            _runner: runner,
            update_display_rx,
        }
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
        let display_rx = self.update_display_rx.clone();
        let mut exit = false;
        while !exit {
            exit = self.handle_keyboard();
            if let Ok(Some(rgb_buffer)) = display_rx.try_recv() {           
                terminal.draw(move |f| {
                    let [_, main] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(f.area());
                        let canvas = Canvas::default()
                        .x_bounds([0., LYNX_SCREEN_WIDTH as f64])
                        .y_bounds([0., LYNX_SCREEN_HEIGHT as f64])
                        .marker(Marker::Block)
                        .paint(|ctx| {
                            ctx.draw(&ScreenView { rgb_buffer: &rgb_buffer });
                        });
                
                    f.render_widget(canvas, main);
                }).unwrap();

                self.input_decay();
            }
        }
    }

    fn read_inputs(&mut self) -> bool {
        while let Ok(has_event) = crossterm::event::poll(INPUT_POLL) {
            if !has_event {
                return false;
            }
            let Ok(event) = crossterm::event::read() else {
                return false;
            };
            if let Event::Key(input) = event {
                let code = input.code;
                let pressed = input.kind == KeyEventKind::Press || input.kind == KeyEventKind::Repeat;
                if !pressed {
                    return false;
                }
                if code == KeyCode::Esc {
                    return true;
                }
                if let Some(btn) = self.config.button_mapping().get_key_value(&code) {
                    let input = *btn.1;
                    set_button!(self, input, true);
                    self.keyboard_frames.insert(input, BUTTON_DECAY);
                } 
            }
        }   
        false
    }

    fn input_decay(&mut self) {
        self.keyboard_frames.iter_mut().for_each(|(_, counter)| {
            *counter = counter.saturating_sub(1);
        });

        self.keyboard_frames.retain(|btn, counter| {
            if *counter == 0 {
                set_button!(self, *btn, false);
                false
            } else {
                true
            }
        });
    }

    pub fn handle_keyboard(&mut self) -> bool {
        let sw = self.switches;
        let joy = self.joystick;

        let exit_requested = self.read_inputs();

        if joy != self.joystick || sw != self.switches {
            self.input_tx.send((self.joystick.bits(), self.switches.bits())).unwrap();
        }

        exit_requested
    }
}