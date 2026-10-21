// Project should be about logic and building skills soo im not writing the ascii and some of those
// messages and i used AI(gemini) on that

use crossterm::event::{self, Event, KeyCode};
use notify_rust::Notification;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    DefaultTerminal, Frame,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    time::{Duration, Instant},
};

// config
#[derive(Deserialize, Serialize)]
struct Config {
    pet_name: String,
}

// screens
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

// get exact hour for is_night_time()
fn get_current_hour() -> u8 {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs_in_day = duration.as_secs() % 86400;

    let local_secs = (secs_in_day + 21600) % 86400;
    (local_secs / 3600) as u8
}

fn is_night_time() -> bool {
    let hour = get_current_hour();
    hour >= 23 || hour < 5
}

// Pet data
#[derive(Clone, Copy)]
pub struct Pet {
    pub name: &'static str,
    pub species: &'static str,
    pub ascii: &'static str,
    pub ascii_blink: &'static str,
    pub personality: &'static str,
    pub catchphrase: &'static str,
    pub break_time: &'static str,
    pub pause_msg: &'static str,
    pub resume_msg: &'static str,
    pub interact_msg: &'static str,
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
            ascii_blink: r#"
  /\_/\  
 ( -.- ) 
  > ^ <  
"#,
            personality: "Playful, energetic, easily distracted",
            catchphrase: "Look, if we knock out this focus block, I promise we can tear up some toilet paper roll later. Deal?",
            break_time: "Okay, break time! Do you want coffee, or are we staring blankly at the wall pretending to meditate?",
            pause_msg: "Hey! Who hit pause? The mouse was right in front of us! Don't blue-ball my hunting instincts.",
            resume_msg: "Fine, fine, let's get back to it. Tap the keyboard or I'm stepping on the power button.",
            interact_msg: "*Knocks your coffee mug slightly closer to the edge* Oh, sorry, was that important? Back to work.",
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
            ascii_blink: r#"
    _____
   /     \
  ( -   - )  🐢
   \_____/
"#,
            personality: "Calm, patient, wise, steady pace",
            catchphrase: "Rome wasn't built in a day, and neither is clean code. Breathe in, breathe out, let's move an inch.",
            break_time: "Shell time. Retract into your sweater, drop your shoulders, and forget deadlines exist for 5 minutes.",
            pause_msg: "Taking a breather? Smart. Just don't get *too* comfy, or we'll wake up next winter.",
            resume_msg: "Alright, poke the head back out of the shell. One quiet step at a time.",
            interact_msg: "*Slowly blinks with absolute maternal disappointment disguised as profound patience* You're doing fine, kid.",
        },
        Pet {
            name: "Barkley",
            species: "Dog",
            ascii: r#"
/^\_/\^\
(  o.o  )
 )  _    (
 /     \
"#,
            ascii_blink: r#"
/^\_/\^\
(  -.-  )
 )  _    (
 /     \
"#,
            personality: "Ultra-loyal, enthusiastic, cheering team player",
            catchphrase: "AWOOO! We got this! Look at how many lines of code you wrote! You're basically a tech wizard!",
            break_time: "BREAK TIME! Quick, look out the window, is that a squirrel?! No? Okay, let's do jumping jacks!",
            pause_msg: "Wait, why stopped?! Are we getting treats? No treats?! Then why are we stopped?! Let's go!",
            resume_msg: "YES! Zoomies activated! Let's shred this ticket and conquer the backlog! 🎾",
            interact_msg: "*Puts muddy paws right on your lap and stares into your soul* You're amazing! Can we eat cheese later?",
        },
        Pet {
            name: "Hoot",
            species: "Owl",
            ascii: r#"
  /\_/\
 (o.o )
 (>   <)
"#,
            ascii_blink: r#"
  /\_/\
 (-.- )
 (>   <)
"#,
            personality: "Intellectual, night owl, detail-oriented",
            catchphrase: "Fascinating. Another ticket closed. The architecture is slowly forming like a delicate web of tech debt.",
            break_time: "Refuel time. Rest your eyes from the blue light before you start hallucinating syntax errors.",
            pause_msg: "Interrupting a deep train of thought? Bold move. Hope you remember your context stack.",
            resume_msg: "Ah, returning to the ethereal glow of the terminal. Excellent. Let's synthesize some logic.",
            interact_msg: "*Rotates head 180 degrees* I see you opening tab #47 on StackOverflow. Let's refocus, shall we?",
        },
        Pet {
            name: "Hop",
            species: "Bunny",
            ascii: r#"
 (\_/) 
 ( •_•)
/>🐰
"#,
            ascii_blink: r#"
 (\_/) 
 ( -.-)
/>🐰
"#,
            personality: "Gentle, swift, encouraging",
            catchphrase: "Boing! Let's hop over these annoying little bugs like they're tiny puddles!",
            break_time: "Time to stretch those legs! Grab a snack or bounce around the room before our next leap!",
            pause_msg: "Did a sudden noise scare you? Don't freeze like a rabbit in headlights, we've got this!",
            resume_msg: "Ears up, eyes forward! Let's hop right back into the flow state!",
            interact_msg: "*Twitches nose rapidly* You're typing so fast I think your keyboard is smoking. Go you!",
        },
        Pet {
            name: "Linus Torvalds",
            species: "Kernel Engineer",
            ascii: r#"
        .---.
       /     \
       \.@-@./
       /`\_/`\
     //  _   \\
    | \     )(_
   /`\_`>  </ \
jgs\__/'---'\__/
"#,
            ascii_blink: r#"
        .---.
       /     \
       \.-.-./
       /`\_/`\
     //  _   \\
    | \     )(_
   /`\_`>  </ \
jgs\__/'---'\__/
"#,
            personality: "UnGentle, Abusing, Cracked",
            catchphrase: "Ay! why am i here? who put me with this vibecoder? aaaaaaaaaaaaa...",
            break_time: "Oh look, a break. Of course you need a break, you've barely written three lines.",
            pause_msg: "Dakh hauar polay abar pause kore..",
            resume_msg: "Shamne paile reee...",
            interact_msg: "oi oi ... Gaye hat dibi na akdom akdom mamla dia dimu...",
        },
    ]
}

// app init
struct App {
    selecting: bool,
    screen: CurrentScreen,
    pets: Vec<Pet>,
    selected_index: usize,
    pet: Pet,
    pet_message: String,
    frame_counter: usize,
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
    last_tick: Instant,
}

fn send_notification(pet_name: &str, title: &str, body: &str) {
    let _ = Notification::new()
        .appname(pet_name)
        .summary(title)
        .body(body)
        .timeout(3000)
        .show();
}

impl App {
    fn new(config: &Config) -> Self {
        let pets = get_pets();
        let mut selected_index = pets
            .iter()
            .position(|p| p.name.eq_ignore_ascii_case(&config.pet_name))
            .unwrap_or(0);

        // the hoot is only in night
        if is_night_time() {
            if let Some(hoot_idx) = pets.iter().position(|p| p.name == "Hoot") {
                selected_index = hoot_idx;
            }
        }

        let pet = pets[selected_index];
        let mut pet_message = pet.catchphrase.to_string();

        if is_night_time() {
            pet_message = "Hoot! ki vai eto raite ki korbar ligga aiso ?".to_string();
        }

        Self {
            selecting: false,
            screen: CurrentScreen::Main,
            pets,
            selected_index,
            pet,
            pet_message,
            frame_counter: 0,
            session_repeatation_buff: String::new(),
            session_repeatation: String::new(),
            break_time_buff: String::new(),
            break_time: String::new(),
            input_step: MechInputStep::SessionCount,
            is_paused: false,
            is_work_phase: true,
            current_session: 1,
            target_sessions: 1,
            remaining_secs: 25 * 60,
            last_tick: Instant::now(),
        }
    }

    // set selected pet with night-mode restriction check
    fn select_pet(&mut self, index: usize) {
        if is_night_time() && self.pets[index].name != "Hoot" {
            self.pet_message =
                "Hoot! nh shob ghum vai, ami asi dine ghumani raite jagi...".to_string();
            return;
        }

        self.selected_index = index;
        self.pet = self.pets[index];
        self.pet_message = self.pet.catchphrase.to_string();
        save_config(self.pet.name);
    }
}

// Main
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
        app.frame_counter = app.frame_counter.wrapping_add(1);
        terminal.draw(|frame| main_frame(frame, app))?;

        if app.screen == CurrentScreen::TimerScreen && !app.is_paused {
            if app.last_tick.elapsed() >= Duration::from_secs(1) {
                if app.remaining_secs > 0 {
                    app.remaining_secs -= 1;
                } else {
                    if app.is_work_phase {
                        // --- FOCUS SESSION ENDED, BREAK STARTED ---
                        let break_mins: u64 = app.break_time.parse().unwrap_or(5);
                        app.remaining_secs = break_mins * 60;
                        app.is_work_phase = false;
                        app.pet_message = app.pet.break_time.to_string();

                        send_notification(&app.pet.name, "Break Time!", &app.pet.break_time);
                    } else {
                        app.current_session += 1;
                        if app.current_session > app.target_sessions {
                            app.screen = CurrentScreen::Completed;
                            app.pet_message =
                                format!("Woohoo! We crushed all {} sessions!", app.target_sessions);

                            send_notification(
                                &app.pet.name,
                                "Mission Completed!",
                                &format!(
                                    "All {} sessions finished successfully!",
                                    app.target_sessions
                                ),
                            );
                        } else {
                            app.remaining_secs = 25 * 60;
                            app.is_work_phase = true;
                            app.pet_message = app.pet.catchphrase.to_string();

                            send_notification(
                                &app.pet.name,
                                &format!(
                                    "Session {} of {}",
                                    app.current_session, app.target_sessions
                                ),
                                &app.pet.catchphrase,
                            );
                        }
                    }
                }
                app.last_tick = Instant::now();
            }
        }

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                // pet it
                if let KeyCode::Char('f') = key.code {
                    app.pet_message = app.pet.interact_msg.to_string();
                }

                if app.screen == CurrentScreen::TimerScreen {
                    match key.code {
                        KeyCode::Char('p') => {
                            app.is_paused = true;
                            app.pet_message = app.pet.pause_msg.to_string();
                            continue;
                        }
                        KeyCode::Char('c') => {
                            app.is_paused = false;
                            app.last_tick = Instant::now();
                            app.pet_message = app.pet.resume_msg.to_string();
                            continue;
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.screen = CurrentScreen::Main;
                            app.pet_message = app.pet.catchphrase.to_string();
                            continue;
                        }
                        _ => {}
                    }
                }

                if app.screen == CurrentScreen::Completed {
                    match key.code {
                        KeyCode::Enter | KeyCode::Char('q') | KeyCode::Esc => {
                            app.screen = CurrentScreen::Main;
                            app.pet_message = app.pet.catchphrase.to_string();
                            continue;
                        }
                        _ => {}
                    }
                }

                if app.screen == CurrentScreen::Mech {
                    match key.code {
                        KeyCode::Char(c) => match app.input_step {
                            MechInputStep::SessionCount => {
                                if c.is_ascii_digit() {
                                    app.session_repeatation_buff.push(c);
                                }
                            }
                            MechInputStep::BreakTime => {
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
                                let _break_mins: u64 = app.break_time.parse().unwrap_or(5);

                                app.current_session = 1;
                                app.is_work_phase = true;
                                app.remaining_secs = 25 * 60;
                                app.is_paused = false;
                                app.last_tick = Instant::now();
                                app.pet_message = app.pet.catchphrase.to_string();
                            }
                        },
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.screen = CurrentScreen::Main;
                            app.pet_message = app.pet.catchphrase.to_string();
                        }
                        _ => {}
                    }
                    continue;
                }

                // screen ctrls
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if app.selecting {
                            app.selecting = false;
                        } else {
                            break;
                        }
                    }
                    KeyCode::Char('n') => {
                        app.screen = CurrentScreen::Completed;
                    }
                    KeyCode::Char('t') => {
                        if !app.selecting {
                            app.screen = CurrentScreen::Mech;
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

// Main UI Layout and Rendering
fn main_frame(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .title_top(Line::from("Pomo").right_aligned())
        .title_bottom(
            Line::from(match app.screen {
                CurrentScreen::Main => format!(
                    "Current Pet: {} | [s] Switch Pet | [t] Start Focus | [f] Interact | [q] Quit",
                    app.pet.name
                ),
                CurrentScreen::Mech => {
                    format!("{} is watching you... | [q] Return", app.pet.name).to_string()
                }
                CurrentScreen::TimerScreen => format!(
                    "{} is watching | [p] Pause | [c] Continue | [f] Interact | [q] Quit",
                    app.pet.name
                )
                .to_string(),
                CurrentScreen::Completed => {
                    format!("Session Finished! | [Enter] or [q] Return to Main")
                }
            })
            .centered(),
        );

    let inner_area = main_block.inner(area);
    frame.render_widget(main_block, area);

    let is_blinking = (app.frame_counter / 45) % 2 == 1;
    let current_ascii = if is_blinking {
        app.pet.ascii_blink
    } else {
        app.pet.ascii
    };

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

            let pet_widget = Paragraph::new(current_ascii)
                .style(Style::default().fg(Color::Cyan))
                .centered();

            let greeting_widget = Paragraph::new(app.pet_message.clone()).centered();
            let inst_widget =
                Paragraph::new("Press [t] to start focusing | Press [f] to pet/interact")
                    .centered();

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

            let pet_widget = Paragraph::new(current_ascii)
                .style(Style::default().fg(Color::Cyan))
                .centered();

            let pet_chat_box = Paragraph::new(app.pet_message.clone())
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
                     [Press Enter to finalize config]",
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

            let pet_widget = Paragraph::new(current_ascii)
                .style(Style::default().fg(Color::Cyan))
                .centered();

            let pet_chat_box = Paragraph::new(app.pet_message.clone())
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
                  Controls: [p] Pause | [c] Continue | [f] Interact | [q] Quit",
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

            let pet_widget = Paragraph::new(current_ascii)
                .style(Style::default().fg(Color::Cyan))
                .centered();

            let pet_chat_box = Paragraph::new(app.pet_message.clone())
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
                "\n🎉 Yeeee onek kisu sira laiso yeee! 🎉\n\n\
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
    let night_mode = is_night_time();
    let header = if night_mode {
        "- Select your pet (🌙 Night Mode: rate Owl dai ase lagbo ni ?):"
    } else {
        "- Select your pet:"
    };

    let mut lines = vec![Line::from(header)];
    for (i, pet) in app.pets.iter().enumerate() {
        let is_asleep = night_mode && pet.name != "Hoot";
        let marker = if i == app.selected_index {
            "[*]"
        } else {
            "[ ]"
        };
        let status_suffix = if is_asleep { " (Asleep 💤)" } else { "" };
        let personality = pet.personality.to_lowercase();
        let text = format!("{} {} ({}){}", marker, pet.name, personality, status_suffix);

        if i == app.selected_index {
            lines.push(Line::from(text).style(Style::default().fg(Color::Yellow)));
        } else if is_asleep {
            lines.push(Line::from(text).style(Style::default().fg(Color::DarkGray)));
        } else {
            lines.push(Line::from(text).style(Style::default().fg(Color::Gray)));
        }
    }
    lines
}

