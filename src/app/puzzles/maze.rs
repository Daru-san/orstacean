use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode};
use knossos::maze::{Algorithm, GameMap, OrthogonalMaze, OrthogonalMazeBuilder, algorithms};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Widget;
use tui_popup::Popup;

use crate::app::puzzles::IPuzzle;
use crate::app::puzzles::timer::Timer;

pub struct Maze {
    grid: Vec<Vec<char>>,
    player: (u16, u16),
    completed: bool,
    failed: bool,
    timer: Timer,
}

const START: char = 'S';
const GOAL: char = 'G';

impl Maze {
    pub fn new(timeout: Duration) -> Maze {
        let maze = OrthogonalMazeBuilder::new().width(840).height(480).build();

        let ascii = maze
            .format(
                GameMap::new()
                    .span(1)
                    .with_start_goal()
                    .start(START)
                    .goal(GOAL),
            )
            .into_inner();

        let grid = ascii
            .lines()
            .map(|line| line.chars().collect())
            .collect::<Vec<Vec<_>>>();

        Self {
            grid,
            player: (0, 0),
            timer: Timer::new(timeout),
            completed: false,
            failed: false,
        }
    }

    pub fn try_move(&mut self, dx: i16, dy: i16) {
        let (x, y) = self.player;
        let nx = x as i16 + dx;
        let ny = y as i16 + dy;
        if nx < 0 || ny < 0 {
            return;
        }
        let (nx, ny) = (nx as usize, ny as usize);
        if self.grid[ny][nx] != '#' {
            self.player = (nx as u16, ny as u16);
        }
    }

    pub fn draw_grid(&self, area: Rect, buf: &mut Buffer) {
        for (y, row) in self.grid.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                let (px, py) = (area.x + x as u16, area.y + y as u16);
                if px >= area.right() || py >= area.bottom() {
                    continue;
                }
                let (symbol, style) = if (x as u16, y as u16) == self.player {
                    ("@", Style::default().fg(Color::Yellow))
                } else if ch == '#' {
                    ("█", Style::default().fg(Color::DarkGray))
                } else {
                    (" ", Style::default())
                };
                buf.cell_mut((px, py))
                    .unwrap()
                    .set_symbol(symbol)
                    .set_style(style);
            }
        }
    }

    fn timedout(&self) -> bool {
        self.timer.done()
        let now = Instant::now();
    }
}

impl IPuzzle for Maze {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([Percentage(100), Min(1)]);
        let [main_area, bottom_area] = area.layout(&layout);

        self.timer.render(bottom_area, frame.buffer_mut());

        self.draw_grid(main_area, frame.buffer_mut());

        if self.timedout() {
            let mut text = Text::default();
            text.push_line("You've run out of time. Shame.");
            text.push_line("I thought you could do it, it seems I had my hopes up too high.");
            text.push_line("Press Enter to continue...");
            Popup::new(text)
                .title("🦀 Ferris")
                .render(area, frame.buffer_mut());
        }
    }
    fn update(&mut self) {
        if self.timedout() {}
    }

    fn handle_events(&mut self) -> color_eyre::Result<()> {
        if let Event::Key(key) = event::read()?
            && key.modifiers.is_empty()
        {
            if self.timedout() {
                if matches!(key.code, KeyCode::Enter) {
                    self.failed = true;
                }
            } else {
                match key.code {
                    KeyCode::Char('h') | KeyCode::Left => self.try_move(-1, 0),
                    KeyCode::Char('l') | KeyCode::Right => self.try_move(1, 0),
                    KeyCode::Char('k') | KeyCode::Up => self.try_move(0, -1),
                    KeyCode::Char('j') | KeyCode::Down => self.try_move(0, 1),
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn keys_hints<'a>(&self) -> ratatui::prelude::Line<'a> {
        Line::from(vec![
            Span::styled("Ctrl-Q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": quit  "),
            Span::styled("H or ◄", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": left  "),
            Span::styled("J or ▲", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": up  "),
            Span::styled("K or ▼", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": down  "),
            Span::styled("L or ►", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": down  "),
        ])
    }

    fn instructions(&self) -> Vec<String> {
        vec![
            String::from("Welcome to the Crabirinth."),
            String::from("Here, we work together."),
            String::from("You will provide me with instructions to follow as I navigate the maze."),
            String::from("We need to reach the end of the maze."),
            String::from("Once we reach the end, we can relax."),
            String::from("We don't have much time, failure will incur serious consequences."),
        ]
    }

    fn completed(&self) -> bool {
        (!self.timedout()) && (self.player == self.goal)
    }

    fn failed(&self) -> bool {
        self.timedout()
    }
}
