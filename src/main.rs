use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use serde::{Deserialize, Serialize};
use std::{fs, time::Duration};

// config
#[derive(Deserialize, Serialize)]
struct Config {
    pet_name: String,
}

// main page and the actual timer or mechanism page
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum CurrentScreen {
    #[default]
    Main,
    Mech,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pet_name: "Pomo".to_string(),
        }
    }
}

// load the config & read-write the config
fn load_config() -> Config {
    let path = "config.toml";
    if let Ok(contents) = fs::read_to_string(path) {
        toml::from_str(&contents).unwrap_or_default()
    } else {
        let default_config = Config::default();
        if let Ok(toml_string) = toml::to_string(&default_config) {
            let _ = fs::write(path, toml_string);
        }
        default_config
    }
}

// set app config to file
fn save_config(pet_name: &str) {
    let config = Config {
        pet_name: pet_name.to_string(),
    };
    if let Ok(toml_string) = toml::to_string(&config) {
        let _ = fs::write("config.toml", toml_string);
    }
}

// data

#[derive(Clone, Copy)]
pub struct Pet {
    pub name: &'static str,
    pub species: &'static str,
    pub ascii: &'static str,
    pub personality: &'static str,
    pub catchphrase: &'static str,
    pub power_up: &'static str,
}

pub fn get_pets() -> Vec<Pet> {
    vec![
        Pet {
            name: "Pomo",
            species: "Cat",
            ascii: r#"
  /\_/\  
 ( o.o ) 
  > ^ <  
"#,
            personality: "Playful, energetic, easily distracted",
            catchphrase: "Finish the work. Then we can play 🐾",
            power_up: "Catnap: Recommends short 5-minute quick refresh breaks",
        },
        Pet {
            name: "Shelly",
            species: "Turtle",
            ascii: r#"
    _____
   /     \
  ( o   o )  🐢
   \_____/
"#,
            personality: "Calm, patient, wise, steady pace",
            catchphrase: "Slow and steady wins the race. Take a breath...",
            power_up: "Deep Work: 60-minute low-stress endurance sessions",
        },
        Pet {
            name: "Barkley",
            species: "Dog",
            ascii: r#"
 /^\_/\^\
(  o.o  )
 )  _  (
 /     \
"#,
            personality: "Ultra-loyal, enthusiastic, cheering team player",
            catchphrase: "Woof woof! You're doing amazing, let's focus again! 🎾",
            power_up: "Fetch Mode: Gives rewarding virtual treats after every timer",
        },
        Pet {
            name: "Hoot",
            species: "Owl",
            ascii: r#"
  /\_/\
 (o.o )
 (>   <)
"#,
            personality: "Intellectual, night owl, detail-oriented",
            catchphrase: "Hoot! Knowledge is built one pomodoro at a time. 🦉",
            power_up: "Night Owl Study: Custom dimmed palette for late-night focus",
        },
        Pet {
            name: "Hop",
            species: "Bunny",
            ascii: r#"
 (\_/) 
 ( •_•)
/>🐰
"#,
            personality: "Gentle, swift, encouraging",
            catchphrase: "Hop hop! Let's leap over these tasks together! 🐇",
            power_up: "Sprint Mode: Fast 15-minute ultra-focused intervals",
        },
        Pet {
            name: "Linus Torvalds",
            species: "Kernel Engineer",
            ascii: r#"
        .---.
       /     \
       \.@-@./
       /`\_/`\
     //  _  \\
    | \     )(_
   /`\_`>  </ \
jgs\__/'---'\__/
"#,
            personality: "UnGentle, Abusing, Cracked",
            catchphrase: "Ay! why am i here? who put me with this vibecoder? aaaaaaaaaaaaa...",
            power_up: "Dont even think that im gonna help you to do anything....",
        },
    ]
}

// App init

struct App {
    selecting: bool,
    screen: CurrentScreen,
    pets: Vec<Pet>,
    selected_index: usize,
    pet: Pet,
    full_text: String,
}

impl App {
    fn new(config: &Config) -> Self {
        let pets = get_pets();
        let selected_index = pets
            .iter()
            .position(|p| p.name.eq_ignore_ascii_case(&config.pet_name))
            .unwrap_or(0);
        let pet = pets[selected_index];
        let full_text = pet.catchphrase.to_string();
        Self {
            selecting: false,
            screen: CurrentScreen::Main,
            pets,
            selected_index,
            pet,
            full_text,
        }
    }

    // set selected pet
    fn select_pet(&mut self, index: usize) {
        self.selected_index = index;
        self.pet = self.pets[index];
        self.full_text = self.pet.catchphrase.to_string();
        save_config(self.pet.name);
    }
}

// Main frame

fn main() -> color_eyre::Result<()> {
    let config = load_config();
    let mut app = App::new(&config);
    let mut terminal = ratatui::init();

    let res = run(&mut terminal, &mut app);
    ratatui::restore();
    Ok(res?)
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    let tick_rate = Duration::from_millis(30);

    loop {
        terminal.draw(|frame| main_frame(frame, app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if app.selecting {
                            app.selecting = false;
                        } else if app.screen == CurrentScreen::Mech {
                            app.screen = CurrentScreen::Main;
                        } else {
                            break;
                        }
                    }
                    KeyCode::Char('t') => {
                        if !app.selecting {
                            app.screen = CurrentScreen::Mech; // get the the actual mechanism
                        }
                    }
                    KeyCode::Char('s') => {
                        app.selecting = !app.selecting;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.selecting {
                            if app.selected_index > 0 {
                                app.selected_index -= 1;
                            } else {
                                app.selected_index = app.pets.len() - 1;
                            }
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.selecting {
                            if app.selected_index < app.pets.len() - 1 {
                                app.selected_index += 1;
                            } else {
                                app.selected_index = 0;
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if app.selecting {
                            app.select_pet(app.selected_index);
                            app.selecting = false;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// Main Ui
fn main_frame(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .title_top(Line::from("Pomo").right_aligned())
        .title_bottom(
            Line::from(match app.screen {
                CurrentScreen::Main => format!(
                    "Current Pet: {} | Press 's' to Switch Pet | 't' to Start Focus | 'q' to Quit",
                    app.pet.name
                ),
                CurrentScreen::Mech => {
                    format!("{} is watching you... | Press 'q' to Return", app.pet.name).to_string()
                }
            })
            .centered(),
        );

    let inner_area = main_block.inner(area);
    frame.render_widget(main_block, area);

    match app.screen {
        CurrentScreen::Main => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8),
                    Constraint::Length(1),
                    Constraint::Min(0),
                ])
                .split(inner_area);

            let pet_widget = Paragraph::new(app.pet.ascii)
                .style(Style::default().fg(Color::Cyan))
                .centered();

            let greeting_widget = Paragraph::new(app.full_text.clone()).centered();
            let inst_widget = Paragraph::new("Press [t] to start focusing").centered();

            frame.render_widget(pet_widget, chunks[0]);
            frame.render_widget(greeting_widget, chunks[1]);
            frame.render_widget(inst_widget, chunks[2]);

            if app.selecting {
                let select_widget = Paragraph::new(select_lines(app)).left_aligned();
                frame.render_widget(select_widget, chunks[2]);
            }
        }
        CurrentScreen::Mech => {
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(48), Constraint::Min(0)])
                .split(inner_area);

            let left_sidebar_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(8), Constraint::Min(6)])
                .split(horizontal_chunks[0]);

            let pet_widget = Paragraph::new(app.pet.ascii)
                .style(Style::default().fg(Color::Cyan))
                .centered();

            let tell_it_to_have_the_pref =
                format!("Customize your work Session from the right pane ->");

            let pet_chat_box = Paragraph::new(tell_it_to_have_the_pref)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Green))
                        .title(format!(" {} ", app.pet.name)),
                )
                .wrap(ratatui::widgets::Wrap { trim: true });

            let right_content = format!("What kind of session are you planning today?").green();

            let right_widget = Paragraph::new(right_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title(" Workspace "),
                )
                .centered()
                .wrap(ratatui::widgets::Wrap { trim: true });

            frame.render_widget(pet_widget, left_sidebar_chunks[0]);
            frame.render_widget(pet_chat_box, left_sidebar_chunks[1]);
            frame.render_widget(right_widget, horizontal_chunks[1]);
        }
    }
}

fn select_lines(app: &App) -> Vec<Line<'_>> {
    let mut lines = vec![Line::from("- Select your pet:")];
    for (i, pet) in app.pets.iter().enumerate() {
        let marker = if i == app.selected_index {
            "[*]"
        } else {
            "[ ]"
        };
        let personality = pet.personality.to_lowercase();
        let text = format!("{} {} ({})", marker, pet.name, personality);
        if i == app.selected_index {
            lines.push(Line::from(text).style(Style::default().fg(Color::Yellow)));
        } else {
            lines.push(Line::from(text).style(Style::default().fg(Color::Gray)));
        }
    }
    lines
}
