use crate::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::line,
    widgets::{Borders, Widget},
};
use sauron_vdom::{Attribute, Callback, Event};

/// Base widget to be used with all upper level ones. It may be used to display a box border around
/// the widget and/or add a title.
///
/// # Examples
///
/// ```
/// # use itui::widgets::{Block, Borders};
/// # use itui::style::{Style, Color};
/// # fn main() {
/// Block::default()
///     .title("Block")
///     .title_style(Style::default().fg(Color::Red))
///     .borders(Borders::LEFT | Borders::RIGHT)
///     .border_style(Style::default().fg(Color::White))
///     .style(Style::default().bg(Color::Black));
/// # }
/// ```
#[derive(Clone)]
pub struct Block<'a, MSG> {
    /// Optional title place on the upper left of the block
    title: Option<&'a str>,
    /// Title style
    title_style: Style,
    /// Visible borders
    borders: Borders,
    /// Border style
    border_style: Style,
    /// Widget style
    style: Style,
    /// area of the block,
    area: Rect,
    /// events attached to this block
    pub events: Vec<Attribute<&'static str, Event, MSG>>,
}

impl<'a, MSG> Default for Block<'a, MSG> {
    fn default() -> Self {
        Block {
            title: None,
            title_style: Default::default(),
            borders: Borders::NONE,
            border_style: Default::default(),
            style: Default::default(),
            area: Default::default(),
            events: vec![],
        }
    }
}

impl<'a, MSG> Block<'a, MSG>
where
    MSG: 'static,
{
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn area(mut self, area: Rect) -> Self {
        self.area = area;
        self
    }

    pub fn title_style(mut self, style: Style) -> Self {
        self.title_style = style;
        self
    }

    pub fn border_style(mut self, style: Style) -> Self {
        self.border_style = style;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn borders(mut self, flag: Borders) -> Self {
        self.borders = flag;
        self
    }
    pub fn triggers_event(&self, event: &Event) -> Option<&Callback<Event, MSG>> {
        match event {
            Event::MouseEvent(me) => {
                let x = me.coordinate.x();
                let y = me.coordinate.y();
                if self.area.hit(x, y) {
                    for listener in &self.events {
                        if me.r#type == listener.name {
                            return listener.get_callback();
                        }
                    }
                    None
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Compute the inner area of a block based on its border visibility rules.
    pub fn inner(&self) -> Rect {
        if self.area.width < 2 || self.area.height < 2 {
            return Rect::default();
        }
        let mut inner = self.area;
        if self.borders.intersects(Borders::LEFT) {
            inner.x += 1;
            inner.width -= 1;
        }
        if self.borders.intersects(Borders::TOP) || self.title.is_some() {
            inner.y += 1;
            inner.height -= 1;
        }
        if self.borders.intersects(Borders::RIGHT) {
            inner.width -= 1;
        }
        if self.borders.intersects(Borders::BOTTOM) {
            inner.height -= 1;
        }
        inner
    }
}

impl<'a, MSG> Widget for Block<'a, MSG> {
    fn get_area(&self) -> Rect {
        self.area
    }

    fn draw(&mut self, buf: &mut Buffer) {
        if self.area.width < 2 || self.area.height < 2 {
            return;
        }

        self.background(buf, self.style.bg);

        // Sides
        if self.borders.intersects(Borders::LEFT) {
            for y in self.area.top()..self.area.bottom() {
                buf.get_mut(self.area.left(), y)
                    .set_symbol(line::VERTICAL)
                    .set_style(self.border_style);
            }
        }
        if self.borders.intersects(Borders::TOP) {
            for x in self.area.left()..self.area.right() {
                buf.get_mut(x, self.area.top())
                    .set_symbol(line::HORIZONTAL)
                    .set_style(self.border_style);
            }
        }
        if self.borders.intersects(Borders::RIGHT) {
            let x = self.area.right() - 1;
            for y in self.area.top()..self.area.bottom() {
                buf.get_mut(x, y)
                    .set_symbol(line::VERTICAL)
                    .set_style(self.border_style);
            }
        }
        if self.borders.intersects(Borders::BOTTOM) {
            let y = self.area.bottom() - 1;
            for x in self.area.left()..self.area.right() {
                buf.get_mut(x, y)
                    .set_symbol(line::HORIZONTAL)
                    .set_style(self.border_style);
            }
        }

        // Corners
        if self.borders.contains(Borders::LEFT | Borders::TOP) {
            buf.get_mut(self.area.left(), self.area.top())
                .set_symbol(line::TOP_LEFT)
                .set_style(self.border_style);
        }
        if self.borders.contains(Borders::RIGHT | Borders::TOP) {
            buf.get_mut(self.area.right() - 1, self.area.top())
                .set_symbol(line::TOP_RIGHT)
                .set_style(self.border_style);
        }
        if self.borders.contains(Borders::LEFT | Borders::BOTTOM) {
            buf.get_mut(self.area.left(), self.area.bottom() - 1)
                .set_symbol(line::BOTTOM_LEFT)
                .set_style(self.border_style);
        }
        if self.borders.contains(Borders::RIGHT | Borders::BOTTOM) {
            buf.get_mut(self.area.right() - 1, self.area.bottom() - 1)
                .set_symbol(line::BOTTOM_RIGHT)
                .set_style(self.border_style);
        }

        if self.area.width > 2 {
            if let Some(title) = self.title {
                let lx = if self.borders.intersects(Borders::LEFT) {
                    1
                } else {
                    0
                };
                let rx = if self.borders.intersects(Borders::RIGHT) {
                    1
                } else {
                    0
                };
                let width = self.area.width - lx - rx;
                buf.set_stringn(
                    self.area.left() + lx,
                    self.area.top(),
                    title,
                    width as usize,
                    self.title_style,
                );
            }
        }
    }
}
