#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::pedantic)]
#![allow(clippy::expect_used)]

use std::fmt::Display;
use std::marker::PhantomData;
use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;

use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use num_traits::{Float, NumCast, PrimInt};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Length, Min, Percentage};
use ratatui::layout::{Layout, Rect};
use ratatui::macros::text;
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui_form::{Field, Form, FormStyle, ValidationError, Validator};
use unicode_width::UnicodeWidthStr;

pub struct InputForm {
    form: Form,
}

pub struct InputResult {
    name: String,
    age: u8,
}

impl InputForm {
    pub fn new() -> Self {
        let form = ratatui_form::FormBuilder::new()
            .title("Introduce yourself")
            .text("name", "Name")
            .placeholder("Haruka Yui")
            .required()
            .validator(Box::new(StringValidator {
                min_length: 3,
                max_length: 40,
            }))
            .done()
            .field(Box::new(
                IntegerInput::<u8>::new("age", "Age")
                    .placeholder("105")
                    .required()
                    .validator(Box::new(IntegerValidator::<u8> { min: 0, max: 150 }))
                    .initial_value(0.to_string()),
            ))
            .build();

        Self { form }
    }

    pub fn update(&mut self) -> Option<InputResult> {
        match self.form.result() {
            ratatui_form::FormResult::Active => {
                let value = self.form.to_json();
                let name = value
                    .get("name")
                    .expect("This field was required")
                    .to_string();

                let age = value
                    .get("age")
                    .expect("This field was required")
                    .as_u64()
                    .unwrap_or(0) as u8;

                Some(InputResult { name, age })
            }
            ratatui_form::FormResult::Submitted => None,
            ratatui_form::FormResult::Cancelled => None,
        }
    }

    pub fn handle_event(&mut self, event: crossterm::event::Event) {
        match event {
            event::Event::Key(key_event) => {
                self.form.handle_input(key_event);
            }
            event::Event::Paste(text) => {
                for c in text.chars() {
                    self.form
                        .handle_input(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()));
                }
            }
            _ => {}
        }
    }

    pub fn draw(&self, frame: &mut Frame<'_>) {
        let area = frame.area();

        let layout = Layout::vertical([Length(3), Min(0)]);
        let [header_area, main_area] = area.layout(&layout);

        let layout = Layout::vertical([Percentage(100), Min(1)]);
        let [main_area, bottom_area] = main_area.layout(&layout);

        Paragraph::new("Origins: Crab")
            .bold()
            .centered()
            .block(Block::bordered())
            .slow_blink()
            .render(header_area, frame.buffer_mut());

        let help = Line::from(vec![
            Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": next  "),
            Span::styled("Shift-Tab", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": reset  "),
            Span::styled("Up/Down", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": navigate  "),
            Span::styled("Backspace", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": delete before cursor  "),
            Span::styled("Delete", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": delete at cursor  "),
            Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": submit  "),
            Span::styled("Left/Right", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": move  "),
            Span::styled("Ctrl-A", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": mv start  "),
            Span::styled("Ctrl-E", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": mv end  "),
            Span::styled("Ctrl-U", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(": clear  "),
        ]);

        frame
            .buffer_mut()
            .set_line(bottom_area.x, bottom_area.y, &help, bottom_area.width);

        self.form.render(main_area, frame.buffer_mut());
    }
}

struct StringValidator {
    min_length: usize,
    max_length: usize,
}

impl Validator for StringValidator {
    fn validate(&self, value: &str) -> Result<(), String> {
        let len = value.len();

        if len > self.max_length {
            return Err(format!(
                "too long at {value}, expected a range of {} to {}",
                self.min_length, self.max_length
            ));
        }
        if len < self.min_length {
            return Err(format!(
                "too short at {value}, expected a range of {} to {}",
                self.min_length, self.max_length
            ));
        }

        Ok(())
    }
}

struct IntegerValidator<T: PrimInt + Send + Sync + FromStr + Display> {
    min: T,
    max: T,
}

impl<T: PrimInt + Send + Sync + FromStr<Err = ParseIntError> + Display> Validator
    for IntegerValidator<T>
{
    fn validate(&self, value: &str) -> Result<(), String> {
        let value: T = value.parse().map_err(|e| format!("Invalid int: {e}"))?;
        if value > self.max {
            return Err(format!(
                "{value} too large, expected a range of {} to {}",
                self.min, self.max
            ));
        }
        if value < self.min {
            return Err(format!(
                "{value} too small, expected a range of {} to {}",
                self.min, self.max
            ));
        }

        Ok(())
    }
}

struct FloatValidator<T: Float + Send + Sync + FromStr + Display> {
    min: T,
    max: T,
}

impl<T: Float + Send + Sync + FromStr<Err = ParseFloatError> + Display> Validator
    for FloatValidator<T>
{
    fn validate(&self, value: &str) -> Result<(), String> {
        let value: T = value.parse().map_err(|e| format!("Invalid float `{e}`"))?;
        if value > self.max {
            return Err(format!(
                "{value} too large, expected a range of {} to {}",
                self.min, self.max
            ));
        }
        if value < self.min {
            return Err(format!(
                "{value} too small, expected a range of {} to {}",
                self.min, self.max
            ));
        }

        Ok(())
    }
}

struct IntegerInput<T: PrimInt + FromStr<Err = ParseIntError>> {
    id: String,
    label: String,
    value: String,
    cursor_position: usize,
    placeholder: Option<String>,
    required: bool,
    validators: Vec<Box<dyn Validator>>,
    validation_errors: Vec<ValidationError>,
    phantom: PhantomData<T>,
}

impl<T: PrimInt + FromStr<Err = ParseIntError>> IntegerInput<T> {
    /// Creates a new text input field.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: String::new(),
            cursor_position: 0,
            placeholder: None,
            required: false,
            validators: Vec::new(),
            validation_errors: Vec::new(),
            phantom: Default::default(),
        }
    }

    /// Sets a placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Marks this field as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Adds a validator to this field.
    pub fn validator(mut self, validator: Box<dyn Validator>) -> Self {
        self.validators.push(validator);
        self
    }

    /// Sets the initial value.
    pub fn initial_value(mut self, value: impl Into<String>) -> Self {
        let value: String = value.into();
        if value.parse::<T>().is_err() {
            return self;
        }
        self.value = value;
        self.cursor_position = self.value.len();
        self
    }

    fn insert_char(&mut self, c: char) {
        if c.is_numeric() || (self.cursor_position == 0 && c == '-') {
            let mut str = self.value.clone();
            str.insert(self.cursor_position, c);
            if str.parse::<T>().is_err() {
                return;
            }
            self.value.insert(self.cursor_position, c);
            self.cursor_position += c.len_utf8();
        }
    }

    fn delete_char_before_cursor(&mut self) {
        if self.cursor_position > 0 {
            let prev_char_boundary = self.value[..self.cursor_position]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.value.remove(prev_char_boundary);
            self.cursor_position = prev_char_boundary;
        }
    }

    fn delete_char_at_cursor(&mut self) {
        if self.cursor_position < self.value.len() {
            self.value.remove(self.cursor_position);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position = self.value[..self.cursor_position]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.value.len() {
            self.cursor_position = self.value[self.cursor_position..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_position + i)
                .unwrap_or(self.value.len());
        }
    }

    fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    fn move_cursor_end(&mut self) {
        self.cursor_position = self.value.len();
    }
}

impl<T: PrimInt + Send + Sync + FromStr<Err = ParseIntError>> Field for IntegerInput<T> {
    fn id(&self) -> &str {
        &self.id
    }

    fn label(&self) -> &str {
        &self.label
    }

    fn render(&self, area: Rect, buf: &mut Buffer, focused: bool, style: &FormStyle) {
        if area.height < 1 || area.width < 1 {
            return;
        }

        // Render label
        let label_style = if focused {
            style.label_focused
        } else {
            style.label
        };

        let required_marker = if self.required { "*" } else { "" };
        let label_text = format!("{}{}: ", self.label, required_marker);
        let label_width = label_text.width().min(area.width as usize);

        let label_span = Span::styled(&label_text, label_style);
        let label_line = Line::from(label_span);
        let label_area = Rect {
            x: area.x,
            y: area.y,
            width: label_width as u16,
            height: 1,
        };
        label_line.render(label_area, buf);

        // Calculate input area
        let input_x = area.x + label_width as u16;
        let input_width = area.width.saturating_sub(label_width as u16);

        if input_width == 0 {
            return;
        }

        // Determine what to display
        let (display_text, display_style) = if self.value.is_empty() {
            if let Some(ref placeholder) = self.placeholder {
                (placeholder.as_str(), style.placeholder)
            } else {
                ("", style.input)
            }
        } else {
            (self.value.as_str(), style.input)
        };

        // Render input value with background
        let input_bg_style = if focused {
            style.input_focused
        } else {
            style.input
        };

        // Fill input area with background
        for x in input_x..input_x + input_width {
            buf[(x, area.y)].set_style(input_bg_style);
            buf[(x, area.y)].set_char(' ');
        }

        // Render the text
        let visible_text: String = display_text.chars().take(input_width as usize).collect();
        for (i, c) in visible_text.chars().enumerate() {
            if input_x + i as u16 >= area.x + area.width {
                break;
            }
            buf[(input_x + i as u16, area.y)].set_char(c);
            buf[(input_x + i as u16, area.y)].set_style(display_style);
        }

        // Render cursor if focused
        if focused {
            let cursor_x = input_x + self.value[..self.cursor_position].width() as u16;
            if cursor_x < area.x + area.width {
                buf[(cursor_x, area.y)].set_style(
                    Style::default()
                        .bg(Color::White)
                        .fg(Color::Black)
                        .add_modifier(Modifier::SLOW_BLINK),
                );
            }
        }

        // Render validation errors if any
        if !self.validation_errors.is_empty() && area.height > 1 {
            let error_msg = &self.validation_errors[0].message;
            let error_span = Span::styled(error_msg, style.error);
            let error_line = Line::from(error_span);
            let error_area = Rect {
                x: input_x,
                y: area.y + 1,
                width: input_width,
                height: 1,
            };
            error_line.render(error_area, buf);
        }
    }

    fn handle_input(&mut self, event: &KeyEvent) -> bool {
        match event.code {
            KeyCode::Char(c) => {
                if event.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'a' => self.move_cursor_home(),
                        'e' => self.move_cursor_end(),
                        'u' => {
                            self.value.clear();
                            self.cursor_position = 0;
                        }
                        _ => return false,
                    }
                } else {
                    self.insert_char(c);
                }
                true
            }
            KeyCode::Backspace => {
                self.delete_char_before_cursor();
                true
            }
            KeyCode::Delete => {
                self.delete_char_at_cursor();
                true
            }
            KeyCode::Left => {
                self.move_cursor_left();
                true
            }
            KeyCode::Right => {
                self.move_cursor_right();
                true
            }
            KeyCode::Home => {
                self.move_cursor_home();
                true
            }
            KeyCode::End => {
                self.move_cursor_end();
                true
            }
            _ => false,
        }
    }

    fn value(&self) -> serde_json::Value {
        serde_json::Value::Number(
            serde_json::Number::from_i128(self.value.parse().expect("Cannot be invalid"))
                .expect("Cannot be invalid"),
        )
    }

    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Check required
        if self.required && self.value.trim().is_empty() {
            errors.push(ValidationError {
                field_id: self.id.clone(),
                message: format!("{} is required", self.label),
            });
        }

        // Run validators
        for validator in &self.validators {
            if let Err(msg) = validator.validate(&self.value) {
                errors.push(ValidationError {
                    field_id: self.id.clone(),
                    message: msg,
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn height(&self) -> u16 {
        if self.validation_errors.is_empty() {
            1
        } else {
            2
        }
    }

    fn is_required(&self) -> bool {
        self.required
    }
}
