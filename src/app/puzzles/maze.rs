use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode};
use knossos::maze::{Algorithm, GameMap, OrthogonalMaze, OrthogonalMazeBuilder, algorithms};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Min, Percentage};
use ratatui::layout::{Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Widget};
use tui_popup::Popup;

use crate::app::puzzles::IPuzzle;
use crate::app::puzzles::timer::Timer;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct Position {
    x: u16,
    y: u16,
}

pub struct Maze {
    grid: Vec<Vec<char>>,
    completed: bool,
    failed: bool,
    timer: Timer,
    player: Position,
    goal: Position,
    start: Position,
    chaser: Position,
    steps: VecDeque<Position>,
    last_tick: Option<Instant>,
    caught: bool,
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

        let mut start = Position::default();
        let mut goal = Position::default();

        for (y, row) in grid.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                if ch == START {
                    start.x = x as u16;
                    start.y = y as u16;
                } else if ch == GOAL {
                    goal.x = x as u16;
                    goal.y = y as u16;
                }
            }
        }

        Self {
            grid,
            player: start,
            goal,
            start,
            timer: Timer::new(timeout),
            completed: false,
            failed: false,
            steps: Default::default(),
            chaser: Position { x: 0, y: 0 },
            last_tick: None,
            caught: false,
        }
    }

    pub fn try_move(&mut self, dx: i16, dy: i16) {
        let Position { x, y } = self.player;
        let nx = x as i16 + dx;
        let ny = y as i16 + dy;
        if nx < 0 || ny < 0 {
            return;
        }
        let (nx, ny) = (nx as usize, ny as usize);
        if self.grid[ny][nx] != '#' {
            self.player.x = nx as u16;
            self.player.y = ny as u16;
        }

        self.steps.push_front(Position { x, y });
    }

    pub fn draw_grid(&self, area: Rect, buf: &mut Buffer) {
        for (y, row) in self.grid.iter().enumerate() {
            for (x, &ch) in row.iter().enumerate() {
                let (px, py) = (area.x + x as u16, area.y + y as u16);
                if px >= area.right() || py >= area.bottom() {
                    continue;
                }
                let (symbol, style) = if (x as u16, y as u16) == (self.player.x, self.player.y) {
                    ("🦀", Style::default().fg(Color::LightRed))
                } else if (x as u16, y as u16) == (self.chaser.x, self.chaser.y) {
                    ("🦝", Style::default().fg(Color::Black))
                } else if ch == '#' {
                    ("█", Style::default().fg(Color::DarkGray))
                } else if ch == GOAL {
                    ("G", Style::default().fg(Color::DarkGray))
                } else if ch == START {
                    ("S", Style::default().fg(Color::DarkGray))
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
    }
}

impl IPuzzle for Maze {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical([Percentage(100), Min(1)]);
        let [main_area, bottom_area] = area.layout(&layout);

        self.timer.render(bottom_area, frame.buffer_mut());

        self.draw_grid(main_area, frame.buffer_mut());

        Block::new()
            .bg(match self.timer.percentage_done() {
                0_f64..40.0 => Color::White,
                40_f64..50.0 => Color::LightYellow,
                50_f64..75.0 => Color::Yellow,
                75_f64..90.0 => Color::LightRed,
                90_f64..100.0 => Color::Red,
                100_f64 => Color::Black,
                _ => unreachable!(),
            })
            .render(main_area, frame.buffer_mut());

        if self.timedout() {
            let mut text = Text::default();
            text.push_line("You've run out of time. Shame.");
            text.push_line("I thought you could do it, it seems I had my hopes up too high.");
            text.push_line("Press Enter to continue...");
            Popup::new(text)
                .title("🦀 King")
                .render(area, frame.buffer_mut());
        }
    }
    fn update(&mut self) {
        self.timer.update();
        if !self.timer.is_paused() && !self.timedout() {
            if let Some(tick) = self.last_tick {
                let now = Instant::now();
                let duration = now.duration_since(tick);
                let tick_wait = match self.timer.percentage_done() {
                    0_f64..40.0 => Duration::from_secs(5),
                    40_f64..60.0 => Duration::from_secs(2),
                    60_f64..80.0 => Duration::from_secs_f64(0.5),
                    80.0..100.0 => Duration::from_millis(100),
                    // When the time is zero, the racoon sprints towards King
                    100.0 => Duration::ZERO,
                    _ => unreachable!(),
                };

                // Advance the racoon's steps
                if duration >= tick_wait
                    && let Some(step) = self.steps.pop_back()
                {
                    self.chaser = step;
                }
            }

            if self.chaser == self.player {
                self.caught = true;
            }
        }
    }

    fn handle_events(&mut self, event: Event) -> color_eyre::Result<()> {
        if let Event::Key(key) = event
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
            String::from("This is a place infested with disgusting rodents."),
            String::from(
                "Our goal is to reach the end of the maze before one of them catches up to us.",
            ),
            String::from(
                "If you let one of those racoons so as touch me, I won't let you off lightly.",
            ),
            String::from(
                "Furthermore, with each passing moment, the temperature in this room will increase.",
            ),
            String::from(
                "If we are unable to escape before it reaches it's peak, I will be boiled alive.",
            ),
            String::from("You will provide me with instructions to follow as I navigate the maze."),
            String::from(
                "Let's get started as soon as possible; the rodents will likely follow my scent to track me down.",
            ),
        ]
    }

    fn completed(&self) -> bool {
        self.player == self.goal
    }

    fn failed(&self) -> bool {
        !self.completed && (self.timedout() || self.caught)
    }

    fn can_pause(&self) -> bool {
        true
    }

    fn toggle_pause(&mut self, pause: bool) {
        if pause {
            self.timer.pause();
        } else {
            self.timer.unpause();
        }
    }

    fn failure_message(&self) -> Vec<String> {
        if self.caught {
            vec![
                String::from("Gah! You let me get caught!"),
                String::from("What in the ocean are you doing?!"),
            ]
        } else if self.timedout() {
            vec![String::from("Well, that's it. Our time is up.")]
        } else {
            unreachable!()
        }
    }

    fn is_paused(&self) -> bool {
        self.timer.is_paused()
    }
}
