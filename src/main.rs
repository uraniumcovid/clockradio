use anyhow::Result;
use chrono::{Local, DateTime};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};
use std::{
    io,
    time::Duration,
};
use tokio::time::sleep;

#[derive(Parser)]
#[command(name = "clockradio")]
#[command(about = "A simple TUI clock radio with weather and alarm")]
struct Cli {}

struct App {
    should_quit: bool,
    alarm_time: Option<DateTime<Local>>,
    show_alarm_dialog: bool,
    alarm_input: String,
    animation_frame: u32,
}

impl App {
    fn new() -> App {
        App {
            should_quit: false,
            alarm_time: None,
            show_alarm_dialog: false,
            alarm_input: String::new(),
            animation_frame: 0,
        }
    }


    fn handle_key_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('a') => self.show_alarm_dialog = true,
            KeyCode::Esc => {
                self.show_alarm_dialog = false;
                self.alarm_input.clear();
            }
            KeyCode::Enter => {
                if self.show_alarm_dialog {
                    if let Ok(time) = chrono::NaiveTime::parse_from_str(&self.alarm_input, "%H:%M") {
                        let now = Local::now();
                        let mut alarm_date = now.date_naive();
                        
                        if time <= now.time() {
                            alarm_date = alarm_date.succ_opt().unwrap_or(alarm_date);
                        }
                        
                        self.alarm_time = Some(alarm_date.and_time(time).and_local_timezone(Local).unwrap());
                    }
                    self.show_alarm_dialog = false;
                    self.alarm_input.clear();
                }
            }
            KeyCode::Backspace => {
                if self.show_alarm_dialog {
                    self.alarm_input.pop();
                }
            }
            KeyCode::Char(c) => {
                if self.show_alarm_dialog {
                    self.alarm_input.push(c);
                }
            }
            _ => {}
        }
    }
}

fn get_ascii_digit(digit: char) -> Vec<&'static str> {
    match digit {
        '0' => vec![
            "███████",
            "██   ██",
            "██   ██",
            "██   ██",
            "██   ██",
            "██   ██",
            "███████",
        ],
        '1' => vec![
            "   ██  ",
            "  ███  ",
            "   ██  ",
            "   ██  ",
            "   ██  ",
            "   ██  ",
            "███████",
        ],
        '2' => vec![
            "███████",
            "      ██",
            "      ██",
            "███████",
            "██     ",
            "██     ",
            "███████",
        ],
        '3' => vec![
            "███████",
            "      ██",
            "      ██",
            "███████",
            "      ██",
            "      ██",
            "███████",
        ],
        '4' => vec![
            "██   ██",
            "██   ██",
            "██   ██",
            "███████",
            "      ██",
            "      ██",
            "      ██",
        ],
        '5' => vec![
            "███████",
            "██     ",
            "██     ",
            "███████",
            "      ██",
            "      ██",
            "███████",
        ],
        '6' => vec![
            "███████",
            "██     ",
            "██     ",
            "███████",
            "██   ██",
            "██   ██",
            "███████",
        ],
        '7' => vec![
            "███████",
            "      ██",
            "      ██",
            "      ██",
            "      ██",
            "      ██",
            "      ██",
        ],
        '8' => vec![
            "███████",
            "██   ██",
            "██   ██",
            "███████",
            "██   ██",
            "██   ██",
            "███████",
        ],
        '9' => vec![
            "███████",
            "██   ██",
            "██   ██",
            "███████",
            "      ██",
            "      ██",
            "███████",
        ],
        ':' => vec![
            "       ",
            "   ██  ",
            "   ██  ",
            "       ",
            "   ██  ",
            "   ██  ",
            "       ",
        ],
        _ => vec![
            "       ",
            "       ",
            "       ",
            "       ",
            "       ",
            "       ",
            "       ",
        ],
    }
}

fn format_time_ascii(time_str: &str) -> Vec<String> {
    let mut lines = vec![String::new(); 7];
    
    for ch in time_str.chars() {
        let digit_lines = get_ascii_digit(ch);
        for (i, line) in digit_lines.iter().enumerate() {
            if i < lines.len() {
                lines[i].push_str(line);
                lines[i].push(' ');
            }
        }
    }
    
    lines
}

fn generate_animated_background(frame: u32, width: u16, height: u16) -> Vec<String> {
    let mut background = Vec::new();
    
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            let char_to_add = if y == height - 3 && x >= 2 && x <= 8 {
                // Street lamp pole
                if x == 5 {
                    '│'
                } else {
                    ' '
                }
            } else if y == height - 4 && x >= 3 && x <= 7 {
                // Street lamp light (animated glow)
                let glow_intensity = (frame as f32 * 0.1).sin() * 0.5 + 0.5;
                if glow_intensity > 0.3 {
                    if x == 5 {
                        '●'
                    } else {
                        '·'
                    }
                } else {
                    if x == 5 {
                        '○'
                    } else {
                        ' '
                    }
                }
            } else if y < height - 5 {
                // Rain/wind effect
                let wind_offset = ((frame as f32 * 0.05).sin() * 2.0) as i32;
                let rain_pos = (x as i32 + y as i32 + wind_offset + (frame / 3) as i32) % 7;
                if rain_pos == 0 && (frame + x as u32) % 13 == 0 {
                    '·'
                } else if rain_pos == 1 && (frame + x as u32) % 17 == 0 {
                    '`'
                } else {
                    ' '
                }
            } else {
                ' '
            };
            line.push(char_to_add);
        }
        background.push(line);
    }
    
    background
}

#[tokio::main]
async fn main() -> Result<()> {
    let _cli = Cli::parse();
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key_event(key.code);
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }


        if let Some(alarm_time) = app.alarm_time {
            if Local::now() >= alarm_time {
                app.alarm_time = None;
            }
        }

        app.animation_frame = app.animation_frame.wrapping_add(1);
        sleep(Duration::from_millis(50)).await;
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let size = f.size();
    
    // Render animated background
    let background_lines = generate_animated_background(app.animation_frame, size.width, size.height);
    let mut bg_spans = Vec::new();
    for line in background_lines {
        bg_spans.push(Line::from(vec![Span::styled(
            line,
            Style::default().fg(Color::Rgb(100, 100, 100)),
        )]));
    }
    
    let background = Paragraph::new(bg_spans)
        .style(Style::default().bg(Color::Black));
    f.render_widget(background, size);
    
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(size);

    let header = Paragraph::new("'a' alarm | 'q' quit")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Rgb(255, 107, 138)));

    f.render_widget(header, main_layout[0]);

    let now = Local::now();
    let time_str = now.format("%H:%M").to_string();
    let date_str = now.format("%A, %B %d, %Y").to_string();

    let ascii_lines = format_time_ascii(&time_str);
    let mut clock_lines = Vec::new();
    
    for line in ascii_lines {
        clock_lines.push(Line::from(vec![Span::styled(
            line,
            Style::default()
                .fg(Color::Rgb(255, 107, 138))
                .add_modifier(Modifier::BOLD),
        )]));
    }
    
    clock_lines.push(Line::from(vec![Span::styled(
        "",
        Style::default().fg(Color::White),
    )]));
    
    clock_lines.push(Line::from(vec![Span::styled(
        date_str,
        Style::default().fg(Color::White),
    )]));

    let clock_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Rgb(255, 107, 138)));

    let clock = Paragraph::new(clock_lines)
        .block(clock_block)
        .alignment(Alignment::Center);

    f.render_widget(clock, main_layout[1]);

    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(main_layout[2]);

    let alarm_text = if let Some(alarm_time) = app.alarm_time {
        format!("Alarm: {}", alarm_time.format("%H:%M"))
    } else {
        "No alarm set".to_string()
    };

    let alarm_widget = Paragraph::new(alarm_text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));

    f.render_widget(alarm_widget, bottom_layout[0]);

    if app.show_alarm_dialog {
        let popup_area = centered_rect(40, 20, size);
        f.render_widget(Clear, popup_area);
        
        let popup_block = Block::default()
            .title("Set Alarm (HH:MM)")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black).fg(Color::Rgb(255, 107, 138)));

        let popup_text = Paragraph::new(app.alarm_input.as_str())
            .block(popup_block)
            .style(Style::default().bg(Color::Black).fg(Color::White));

        f.render_widget(popup_text, popup_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
