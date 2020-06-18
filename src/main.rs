extern crate clap;
use clap::Clap;
use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::{
  error::Error,
  io::{stdout, Write},
  sync::mpsc,
  thread,
  time::{Duration, Instant},
};
use tui::{
  backend::CrosstermBackend,
  layout::{Constraint, Direction, Layout},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, Row, Table},
  Terminal,
};

#[derive(Clap)]
#[clap(version = "1.0", author = "Alex Collins <grampz@pm.me>")]
struct Args {
  #[clap(subcommand)]
  subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
  Wms(WmsCmd),
}

#[derive(Clap)]
struct WmsCmd {
  /// The URL to read
  #[clap(short, long)]
  url: String,
}

enum Event<I> {
  Input(I),
  Tick,
}

async fn get(url: String) -> Result<String, reqwest::Error> {
  reqwest::get(&url).await?.text().await
}

#[tokio::main]
async fn main() -> Result<(), String> {
  execute!(stdout(), EnterAlternateScreen).unwrap();
  let args: Args = Args::parse();
  match args.subcmd {
    SubCommand::Wms(w) => match get(w.url).await {
      Ok(xml) => {
        if let Ok(wms) = ogc::wms::from_string(xml) {
          let stdout = io::stdout();
          let backend = CrosstermBackend::new(stdout);
          let mut terminal = Terminal::new(backend).unwrap();
          terminal.hide_cursor().unwrap();
          let (tx, rx) = mpsc::channel();
          let tick_rate = Duration::from_millis(1000);
          thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
              // poll for tick rate duration, if no events, sent tick event.
              if event::poll(tick_rate - last_tick.elapsed()).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                  tx.send(Event::Input(key)).unwrap();
                }
              }
              if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
              }
            }
          });

          loop {
            terminal
              .draw(|mut f| {
                let chunks = Layout::default()
                  .direction(Direction::Vertical)
                  .constraints([Constraint::Percentage(5), Constraint::Percentage(95)].as_ref())
                  .split(f.size());

                let block = Block::default()
                  .title(&wms.service.title)
                  .borders(Borders::TOP);
                f.render_widget(block, chunks[0]);
                let row_style = Style::default().fg(Color::White);
                let table = Table::new(
                  ["Title", "SRS"].into_iter(),
                  wms
                    .capability
                    .layer
                    .layers
                    .iter()
                    .map(|l| Row::StyledData(vec![&l.title, &l.srs].into_iter(), row_style)),
                )
                .header_style(
                  Style::default()
                    .fg(Color::Yellow)
                    .modifier(Modifier::UNDERLINED),
                )
                .widths(&[
                  Constraint::Percentage(40),
                  Constraint::Length(40),
                  Constraint::Length(20),
                ])
                .style(Style::default().fg(Color::White))
                .column_spacing(1);
                f.render_widget(table, chunks[1]);
              })
              .unwrap();
            match rx.recv().unwrap() {
              Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                  disable_raw_mode().unwrap();
                  execute!(
                    terminal.backend_mut(),
                    LeaveAlternateScreen,
                    DisableMouseCapture
                  ).unwrap();
                  terminal.show_cursor().unwrap();
                  break;
                }
                _ => {}
              },
              Event::Tick => {

              }
            }
          }
        }
        execute!(stdout(), LeaveAlternateScreen);
        Ok(())
      }
      Err(e) => {
        println!("Bad URL? {:?}", e);
        Err("Failed to read URL".to_string())
      }
    },
  }
}
