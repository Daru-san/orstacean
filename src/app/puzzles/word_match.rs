use std::collections::BTreeMap;
use std::time::Duration;

use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use rand::seq::SliceRandom;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::Min;
use ratatui::layout::Constraint::Percentage;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, Borders};
use ratatui_hypertile::HypertileWidget;
use ratatui_hypertile::{Hypertile, HypertileAction, MoveScope, PaneId, PaneSnapshot, Towards};

use crate::app::puzzles::IPuzzle;
use crate::app::puzzles::timer::Timer;

type PaneMap = BTreeMap<PaneId, (char, Color)>;

pub struct WordMatch {
    word: &'static str,
    panes: PaneMap,
    layout: Hypertile,
    completed: bool,
    timer: Option<Timer>,
}

impl WordMatch {
    pub fn new(word: &'static str, timeout: Option<Duration>) -> color_eyre::Result<Self> {
        let mut layout = Hypertile::new();

        let mut chars: Vec<char> = word.chars().collect();

        let mut rng = rand::rng();
        // shuffle chars
        chars.shuffle(&mut rng);

        layout.set_split_policy(ratatui_hypertile::SplitPolicy::Half);

        let mut panes = PaneMap::new();

        let mut leaf_ids: Vec<PaneId> = Vec::new();

        for (i, c) in chars.into_iter().enumerate() {
            if i == 0 {
                let id = PaneId::ROOT;
                leaf_ids.push(PaneId::ROOT);
                panes.insert(id, (c, Color::White));
                continue;
            }
            let target = leaf_ids[(i - 1) % leaf_ids.len()];
            loop {
                layout.apply_action(HypertileAction::FocusPrev);
                let focused = layout.focused_pane().unwrap();
                if focused.eq(&target) {
                    break;
                }
            }
            let new_id = layout.split_focused(Direction::Horizontal)?;
            leaf_ids.push(new_id);
            panes.insert(new_id, (c, Color::White));
        }
        Ok(Self {
            word,
            layout,
            panes,
            completed: false,
            timer: timeout.map(Timer::new),
        })
    }

    fn check_sorted(&self) -> bool {
        let word = self
            .layout
            .panes()
            .into_iter()
            .filter_map(|pane| self.panes.get(&pane.id).copied().map(|pane| pane.0))
            .collect::<String>();

        word.eq(&self.word)
    }
}

impl IPuzzle for WordMatch {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(ref mut timer) = self.timer {
            let layout = Layout::vertical([Percentage(100), Min(1)]);
            let [main_area, bottom_area] = area.layout(&layout);
            timer.render(bottom_area, frame.buffer_mut());

            frame.render_stateful_widget(
                HypertileWidget::new(|pane, buf| render_pane(pane, buf, &self.panes)),
                main_area,
                &mut self.layout,
            );
        } else {
            frame.render_stateful_widget(
                HypertileWidget::new(|pane, buf| render_pane(pane, buf, &self.panes)),
                area,
                &mut self.layout,
            );
        }
    }

    fn update(&mut self) {
        if let Some(ref mut timer) = self.timer {
            timer.update();
        }
    }

    fn completed(&self) -> bool {
        self.check_sorted() && self.completed
    }
    fn failed(&self) -> bool {
        self.completed && !self.check_sorted()
    }
    fn handle_events(&mut self, event: Event) -> color_eyre::Result<()> {
        let Event::Key(key) = event else {
            return Ok(());
        };

        let none = key.modifiers == KeyModifiers::NONE;
        let shift = key.modifiers == KeyModifiers::SHIFT;
        let ctrl = key.modifiers == KeyModifiers::CONTROL;

        match key.code {
            KeyCode::Enter => {}
            KeyCode::Left | KeyCode::Char('h') if none => {
                focus(&mut self.layout, Direction::Horizontal, Towards::Start)
            }
            KeyCode::Right | KeyCode::Char('l') if none => {
                focus(&mut self.layout, Direction::Horizontal, Towards::End)
            }
            KeyCode::Up | KeyCode::Char('k') if none => {
                focus(&mut self.layout, Direction::Vertical, Towards::Start)
            }
            KeyCode::Down | KeyCode::Char('j') if none => {
                focus(&mut self.layout, Direction::Vertical, Towards::End)
            }

            KeyCode::Left | KeyCode::Char('H') if shift => {
                move_pane(&mut self.layout, Direction::Horizontal, Towards::Start)
            }
            KeyCode::Right | KeyCode::Char('L') if shift => {
                move_pane(&mut self.layout, Direction::Horizontal, Towards::End)
            }
            KeyCode::Up | KeyCode::Char('K') if shift => {
                move_pane(&mut self.layout, Direction::Vertical, Towards::Start)
            }
            KeyCode::Down | KeyCode::Char('J') if shift => {
                move_pane(&mut self.layout, Direction::Vertical, Towards::End)
            }
            KeyCode::Char('s') if ctrl => {
                self.completed = true;
            }
            _ => {}
        }
        Ok(())
    }

    fn keys_hints<'a>(&self) -> Line<'a> {
        Line::from(vec![
            Span::styled("C-Q", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": quit  "),
            Span::styled("C-R", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": reset  "),
            Span::styled("C-S", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": submit  "),
            Span::styled("H or ◄", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": left  "),
            Span::styled("J or ▲", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": up  "),
            Span::styled("K or ▼", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": down  "),
            Span::styled("L or ►", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": right  "),
            Span::styled("S-<D>", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": move  "),
        ])
    }

    fn instructions(&self) -> Vec<String> {
        vec![
            String::from("Welcome to the Word Match Puzzle."),
            String::from("This one is rather simple."),
            String::from(
                "Arrange the letters in the grid until they make up the word in question.",
            ),
            String::from("No time limitation shall be put on you during this puzzle."),
        ]
    }

    fn toggle_pause(&mut self, pause: bool) {
        if let Some(timer) = self.timer.as_mut() {
            if pause {
                timer.pause();
            } else {
                timer.unpause();
            }
        }
    }

    fn is_paused(&self) -> bool {
        self.timer.as_ref().is_some_and(|timer| timer.is_paused())
    }

    fn can_pause(&self) -> bool {
        self.timer.is_some()
    }
}

fn move_pane(layout: &mut Hypertile, direction: Direction, towards: Towards) {
    layout.apply_action(HypertileAction::MoveFocused {
        direction,
        towards,
        scope: MoveScope::Window,
    });
}

fn focus(layout: &mut Hypertile, direction: Direction, towards: Towards) {
    layout.apply_action(HypertileAction::FocusDirection { direction, towards });
}

fn render_pane(pane: PaneSnapshot, buf: &mut Buffer, panes: &PaneMap) {
    let (title, color) = panes
        .get(&pane.id)
        .map(|(t, c)| (*t, *c))
        .unwrap_or(('%', Color::White));

    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .title(title.to_string());
    if pane.is_focused {
        block = block
            .border_set(border::THICK)
            .border_style(Style::default().fg(color).bold());
    }
    block.render(pane.rect, buf);
}
