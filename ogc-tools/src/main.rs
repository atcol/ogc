extern crate clap;
extern crate reqwest;
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
  widgets::{Block, Borders, List, Row, Table, Text},
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
  View(TuiCmd),
}

#[derive(Clap)]
struct TuiCmd {
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
    SubCommand::View(w) => match get(w.url).await {
      Ok(xml) => {
        if let Ok(wms) = ogc::wms::get_capabilities_string(xml) {
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
                let horiz_chunks = Layout::default()
                  .direction(Direction::Horizontal)
                  .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                  .split(f.size());

                if let Some(layer_list) = &wms.capability.layer {
                  let items = layer_list.layers.iter().map(|l| Text::raw(&l.title));

                  let list = List::new(items)
                    .block(
                      Block::default()
                        .title(&wms.service.title)
                        .borders(Borders::ALL),
                    )
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().modifier(Modifier::ITALIC))
                    .highlight_symbol(">>");

                  f.render_widget(list, horiz_chunks[0]);

                  let row_style = Style::default().fg(Color::White);
                  let table = Table::new(
                    ["Name", "SRS", "BBox", "BBox (Lat Lon)"].into_iter(),
                    vec![Row::Data(["A", "B", "C", "D"].into_iter())].into_iter(),
                  )
                  .header_style(
                    Style::default()
                      .fg(Color::Yellow)
                      .modifier(Modifier::UNDERLINED),
                  )
                  .widths(&[
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                  ])
                  .style(Style::default().fg(Color::White))
                  .column_spacing(1);
                  f.render_widget(table, horiz_chunks[1]);
                } else {
                  f.render_widget(Block::default().title("No layers"), horiz_chunks[0]);
                }
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
                  )
                  .unwrap();
                  terminal.show_cursor().unwrap();
                  break;
                }
                _ => {}
              },
              Event::Tick => {}
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
