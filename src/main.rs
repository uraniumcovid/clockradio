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
}

impl App {
    fn new() -> App {
        App {
            should_quit: false,
            alarm_time: None,
            show_alarm_dialog: false,
            alarm_input: String::new(),
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

        sleep(Duration::from_millis(50)).await;
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let size = f.size();
    
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(5),
        ])
        .split(size);

    let header_block = Block::default()
        .title("Clock Radio")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Rgb(255, 107, 138)));

    let header = Paragraph::new("Press 'a' for alarm, 'q' to quit")
        .block(header_block)
        .alignment(Alignment::Center)
        .style(Style::default().bg(Color::Black).fg(Color::White));

    f.render_widget(header, main_layout[0]);

    let now = Local::now();
    let time_str = now.format("%H:%M").to_string();
    let date_str = now.format("%A, %B %d, %Y").to_string();

    let clock_lines = vec![
        Line::from(vec![Span::styled(
            time_str,
            Style::default()
                .fg(Color::Rgb(255, 107, 138))
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![Span::styled(
            date_str,
            Style::default().fg(Color::White),
        )]),
    ];

    let clock_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Rgb(255, 107, 138)));

    let clock = Paragraph::new(clock_lines)
        .block(clock_block)
        .alignment(Alignment::Center)
        .style(Style::default().bg(Color::Black));

    f.render_widget(clock, main_layout[1]);

    let bottom_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(main_layout[2]);

    let info_text = vec![
        Line::from(vec![Span::styled(
            "Clock Radio",
            Style::default().fg(Color::White),
        )]),
        Line::from(vec![Span::styled(
            "Press 'a' for alarm",
            Style::default().fg(Color::White),
        )]),
    ];

    let info_block = Block::default()
        .title("Info")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Rgb(255, 107, 138)));

    let info_widget = Paragraph::new(info_text)
        .block(info_block)
        .style(Style::default().bg(Color::Black));

    f.render_widget(info_widget, bottom_layout[0]);

    let mut alarm_text = vec![Line::from("No alarm set")];
    if let Some(alarm_time) = app.alarm_time {
        alarm_text = vec![Line::from(vec![Span::styled(
            format!("Alarm: {}", alarm_time.format("%H:%M")),
            Style::default().fg(Color::White),
        )])];
    }

    let alarm_block = Block::default()
        .title("Alarm")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black).fg(Color::Rgb(255, 107, 138)));

    let alarm_widget = Paragraph::new(alarm_text)
        .block(alarm_block)
        .style(Style::default().bg(Color::Black));

    f.render_widget(alarm_widget, bottom_layout[1]);

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
