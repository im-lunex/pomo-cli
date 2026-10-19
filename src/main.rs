use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
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
#[derive(Default, PartialEq)]
enum CurrentScreen {
    #[default]
    Main,
    Mech,
    TimerScreen,
    Completed,
}

// space for workspace input handling
#[derive(Default)]
enum MechInputStep {
    #[default]
    SessionCount,
    BreakTime,
    Ready,
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
    pub break_time: &'static str,
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
            break_time: "Its break time -- what would you love to have a coffee or a catroll",
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
            break_time: "Finally! we can rest for a bit",
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
            break_time: "Ye! Break time ---",
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
            break_time: "Hoot! Breaks are really important for holding the focus for long time...",
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
            break_time: "Hop! Hop! Less have a quick break then we start again.. Hop!",
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
            break_time: "Look this lazy ass started to have break again.. Ay get back to work...",
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
    session_repeatation_buff: String,
    session_repeatation: String,
    break_time_buff: String,
    break_time: String,
    input_step: MechInputStep,
    // timer things
    is_paused: bool,
    is_work_phase: bool,
    current_session: usize,
    target_sessions: usize,
    remaining_secs: u64,
    last_tick: std::time::Instant,
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
            session_repeatation_buff: String::new(),
            session_repeatation: String::new(),
            break_time_buff: String::new(),
            break_time: String::new(),
            input_step: MechInputStep::SessionCount,
            is_paused: false,
            is_work_phase: true,
            current_session: 1,
            target_sessions: 1,
            remaining_secs: 25 * 60, // default 25 mins - all sugesst this
            last_tick: std::time::Instant::now(),
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

        if app.screen == CurrentScreen::TimerScreen && !app.is_paused {
            if app.last_tick.elapsed() >= Duration::from_secs(1) {
                if app.remaining_secs > 0 {
                    app.remaining_secs -= 1;
                } else {
                    if app.is_work_phase {
                        let break_mins: u64 = app.break_time.parse().unwrap_or(5);
                        app.remaining_secs = break_mins * 60;
                        app.is_work_phase = false;
                    } else {
                        app.current_session += 1;
                        if app.current_session > app.target_sessions {
                            app.screen = CurrentScreen::Completed;
                        } else {
                            app.remaining_secs = 25 * 60;
                            app.is_work_phase = true;
                        }
                    }
                }
                app.last_tick = std::time::Instant::now();
            }
        }

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if app.screen == CurrentScreen::TimerScreen {
                    match key.code {
                        KeyCode::Char('p') => {
                            app.is_paused = true;
                            continue;
                        }
                        KeyCode::Char('c') => {
                            app.is_paused = false;
                            app.last_tick = std::time::Instant::now();
                            continue;
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.screen = CurrentScreen::Main;
                            continue;
                        }
                        _ => {}
                    }
                }
                if app.screen == CurrentScreen::Completed {
                    match key.code {
                        KeyCode::Enter | KeyCode::Char('q') | KeyCode::Esc => {
                            app.screen = CurrentScreen::Main;
                            continue;
                        }
                        _ => {}
                    }
                }
                if app.screen == CurrentScreen::Mech {
                    match key.code {
                        KeyCode::Char(c) => match app.input_step {
                            MechInputStep::SessionCount => {
                                // only accept num - its tuff to have input error in ratatui
                                if c.is_ascii_digit() {
                                    app.session_repeatation_buff.push(c);
                                }
                            }
                            MechInputStep::BreakTime => {
                                // only accept num - its tuff to have input error in ratatui
                                if c.is_ascii_digit() {
                                    app.break_time_buff.push(c);
                                }
                            }
                            MechInputStep::Ready => {}
                        },
                        KeyCode::Backspace => match app.input_step {
                            MechInputStep::SessionCount => {
                                app.session_repeatation_buff.pop();
                            }
                            MechInputStep::BreakTime => {
                                app.break_time_buff.pop();
                            }
                            MechInputStep::Ready => {}
                        },
                        KeyCode::Enter => match app.input_step {
                            MechInputStep::SessionCount => {
                                app.session_repeatation = app.session_repeatation_buff.clone();
                                app.session_repeatation_buff.clear();
                                app.input_step = MechInputStep::BreakTime;
                            }
                            MechInputStep::BreakTime => {
                                app.break_time = app.break_time_buff.clone();
                                app.break_time_buff.clear();
                                app.input_step = MechInputStep::Ready;
                            }
                            MechInputStep::Ready => {
                                app.screen = CurrentScreen::TimerScreen;
                                app.target_sessions = app.session_repeatation.parse().unwrap_or(1);
                                let break_mins: u64 = app.break_time.parse().unwrap_or(5);

                                app.current_session = 1;
                                app.is_work_phase = true;
                                app.remaining_secs = 25 * 60;
                                app.is_paused = false;
                                app.last_tick = std::time::Instant::now();
                            }
                        },
                        _ => {}
                    }
                }
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
                            app.screen = CurrentScreen::Mech; // get the actual mechanism
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
                CurrentScreen::TimerScreen => {
                    format!("{} is watching you... | Press 'p' to pause | Press 'c' to continue | Press 'q' to quite", app.pet.name).to_string()
                }
                CurrentScreen::Completed => {
                    format!("Press 'q' to get back to Main window")
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

            frame.render_widget(pet_widget, left_sidebar_chunks[0]);
            frame.render_widget(pet_chat_box, left_sidebar_chunks[1]);

            let workspace_content = match app.input_step {
                MechInputStep::SessionCount => format!(
                    "How many sessions do you want? (working sessions, excluding breaks)\n\n\
                     Session Count: {}\n\
                     > {}_\n\n\
                     [Press Enter to set Break Time]",
                    if app.session_repeatation.is_empty() {
                        "None"
                    } else {
                        &app.session_repeatation
                    },
                    app.session_repeatation_buff
                ),
                MechInputStep::BreakTime => format!(
                    "How long should each break be? (in minutes)\n\n\
                     ✓ Session Count: {}\n\n\
                     Break Time (mins): {}\n\
                     > {}_\n\n\
                     [Press Enter to final the config]",
                    app.session_repeatation,
                    if app.break_time.is_empty() {
                        "None"
                    } else {
                        &app.break_time
                    },
                    app.break_time_buff
                ),
                MechInputStep::Ready => format!(
                    "Config Done!\n\n\
     ✓ Session Count: {}\n\
     ✓ Break Time: {} mins\n\n\
     > Press [Enter] to start focusing!",
                    app.session_repeatation, app.break_time
                ),
            };

            let workspace_widget = Paragraph::new(workspace_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title(" Workspace "),
                )
                .wrap(ratatui::widgets::Wrap { trim: true });

            frame.render_widget(workspace_widget, horizontal_chunks[1]);
        }

        CurrentScreen::TimerScreen => {
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

            let pet_chat_box = Paragraph::new(if app.is_work_phase {
                app.pet.catchphrase
            } else {
                app.pet.break_time
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .title(format!(" {} ", app.pet.name)),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

            frame.render_widget(pet_widget, left_sidebar_chunks[0]);
            frame.render_widget(pet_chat_box, left_sidebar_chunks[1]);

            let minutes = app.remaining_secs / 60;
            let seconds = app.remaining_secs % 60;
            let time_str = format!("{:02}:{:02}", minutes, seconds);

            let phase_title = if app.is_work_phase {
                "🔥 FOCUS SESSION"
            } else {
                "☕ BREAK TIME"
            };
            let status_str = if app.is_paused { " (PAUSED) " } else { "" };

            let timer_content = format!(
                "\n{}{} \n\n\
         ┌─────────────┐\n\
         │   {}   │\n\
         └─────────────┘\n\n\
         Session: {} / {}\n\
         Break Target: {} mins\n\n\
         Controls: [p] Pause | [c] Continue | [q] Quit",
                phase_title,
                status_str,
                time_str,
                app.current_session,
                app.target_sessions,
                app.break_time
            );

            let timer_widget = Paragraph::new(timer_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(if app.is_work_phase {
                            Color::Yellow
                        } else {
                            Color::Green
                        }))
                        .title(" Live Timer Workspace "),
                )
                .centered()
                .wrap(ratatui::widgets::Wrap { trim: true });

            frame.render_widget(timer_widget, horizontal_chunks[1]);
        }
        CurrentScreen::Completed => {
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

            let pet_chat_box = Paragraph::new("Woohoo! You legend. We crushed all the sessions!")
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow))
                        .title(format!(" {} ", app.pet.name)),
                )
                .wrap(ratatui::widgets::Wrap { trim: true });

            frame.render_widget(pet_widget, left_sidebar_chunks[0]);
            frame.render_widget(pet_chat_box, left_sidebar_chunks[1]);

            let complete_content = format!(
                "\n🎉 YOU HAVE COMPLETED THE FULL WORK SESSION! 🎉\n\n\
         Total Sessions Finished: {}\n\n\
         Amazing discipline and deep focus!\n\
         Time to celebrate or take a well-earned rest.\n\n\
         > Press [Enter] or [q] to return to Main Menu",
                app.target_sessions
            );

            let complete_widget = Paragraph::new(complete_content)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Magenta))
                        .title(" Mission Accomplished! "),
                )
                .centered()
                .wrap(ratatui::widgets::Wrap { trim: true });

            frame.render_widget(complete_widget, horizontal_chunks[1]);
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
