use either::Either;
use sauron_vdom::{Attribute, Callback, Event};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::Style,
    widgets::{
        reflow::{LineComposer, LineTruncator, Styled, WordWrapper},
        Block, Text, Widget,
    },
};

fn get_line_offset(line_width: u16, text_area_width: u16, alignment: Alignment) -> u16 {
    match alignment {
        Alignment::Center => (text_area_width / 2).saturating_sub(line_width / 2),
        Alignment::Right => text_area_width.saturating_sub(line_width),
        Alignment::Left => 0,
    }
}

/// A widget to display some text.
///
/// # Examples
///
/// ```
/// # use itui::widgets::{Block, Borders, Paragraph, Text};
/// # use itui::style::{Style, Color};
/// # use itui::layout::{Alignment};
/// # fn main() {
/// let text = [
///     Text::raw("First line\n"),
///     Text::styled("Second line\n", Style::default().fg(Color::Red))
/// ];
/// Paragraph::new(text.iter())
///     .block(Block::default().title("Paragraph").borders(Borders::ALL))
///     .style(Style::default().fg(Color::White).bg(Color::Black))
///     .alignment(Alignment::Center)
///     .wrap(true);
/// # }
/// ```
pub struct Paragraph<'a, 't, T, MSG>
where
    T: Iterator<Item = &'t Text<'t>>,
{
    /// A block to wrap the widget in
    block: Option<Block<'a, MSG>>,
    /// area occupied by this paragraph
    area: Rect,
    /// Widget style
    style: Style,
    /// Wrap the text or not
    wrapping: bool,
    /// The text to display
    text: T,
    /// Should we parse the text for embedded commands
    raw: bool,
    /// Scroll
    scroll: u16,
    /// Aligenment of the text
    alignment: Alignment,
    /// events attached to this block
    pub events: Vec<Attribute<&'static str, Event, MSG>>,
}

impl<'a, 't, T, MSG> Paragraph<'a, 't, T, MSG>
where
    T: Iterator<Item = &'t Text<'t>>,
    MSG: 'static,
{
    pub fn new(text: T) -> Self {
        Paragraph {
            block: None,
            style: Default::default(),
            wrapping: false,
            raw: false,
            text,
            scroll: 0,
            alignment: Alignment::Left,
            area: Default::default(),
            events: vec![],
        }
    }

    pub fn block(mut self, block: Block<'a, MSG>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn wrap(mut self, flag: bool) -> Self {
        self.wrapping = flag;
        self
    }

    pub fn raw(mut self, flag: bool) -> Self {
        self.raw = flag;
        self
    }

    pub fn scroll(mut self, offset: u16) -> Self {
        self.scroll = offset;
        self
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn area(mut self, area: Rect) -> Self {
        self.area = area;
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
}

impl<'a, 't, 'b, T, MSG> Widget for Paragraph<'a, 't, T, MSG>
where
    T: Iterator<Item = &'t Text<'t>>,
    MSG: 'static,
{
    fn get_area(&self) -> Rect {
        match &self.block {
            Some(b) => b.inner(),
            None => self.area,
        }
    }
    fn draw(&mut self, buf: &mut Buffer) {
        let text_area = self.get_area();
        if let Some(ref mut b) = self.block {
            b.draw(buf);
        }

        if text_area.height < 1 {
            return;
        }

        self.background(buf, self.style.bg);

        let style = self.style;
        let mut styled = self.text.by_ref().flat_map(|t| match *t {
            Text::Raw(ref d) => {
                let data: &'t str = d; // coerce to &str
                Either::Left(UnicodeSegmentation::graphemes(data, true).map(|g| Styled(g, style)))
            }
            Text::Styled(ref d, s) => {
                let data: &'t str = d; // coerce to &str
                Either::Right(UnicodeSegmentation::graphemes(data, true).map(move |g| Styled(g, s)))
            }
        });

        let mut line_composer: Box<dyn LineComposer> = if self.wrapping {
            Box::new(WordWrapper::new(&mut styled, text_area.width))
        } else {
            Box::new(LineTruncator::new(&mut styled, text_area.width))
        };
        let mut y = 0;
        while let Some((current_line, current_line_width)) = line_composer.next_line() {
            if y >= self.scroll {
                let mut x = get_line_offset(current_line_width, text_area.width, self.alignment);
                for Styled(symbol, style) in current_line {
                    buf.get_mut(text_area.left() + x, text_area.top() + y - self.scroll)
                        .set_symbol(symbol)
                        .set_style(*style);
                    x += symbol.width() as u16;
                }
            }
            y += 1;
            if y >= text_area.height + self.scroll {
                break;
            }
        }
    }
}
