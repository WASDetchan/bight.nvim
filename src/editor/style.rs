use bight::{evaluator::TableValue, table::CellPos};
use hashbrown::HashMap;

type CellSize = usize;

#[derive(Clone, Copy, Debug)]
pub struct CellStyle<'a> {
    pub width: CellSize,
    pub height: CellSize,
    pub horizontal_separator: Option<&'a str>,
    pub vertical_separator: &'a str,
}

impl Default for CellStyle<'static> {
    fn default() -> Self {
        Self {
            width: 8,
            height: 1,
            horizontal_separator: None,
            vertical_separator: " ",
        }
    }
}

impl CellStyle<'_> {
    pub fn render(&self, value: &TableValue) -> Vec<String> {
        value
            .to_string()
            .lines()
            .chain([""].into_iter().cycle())
            .take(self.height)
            .map(|line| {
                line.chars()
                    .chain([' '].into_iter().cycle())
                    .take(self.width)
                    .chain(self.vertical_separator.chars())
                    .collect()
            })
            .chain(self.horizontal_separator.into_iter().map(|s| {
                s.chars()
                    .cycle()
                    .take(self.width + self.vertical_separator.chars().count())
                    .collect()
            }))
            .collect()
    }
}

pub struct TableStyle<'a> {
    expand: Option<CellPos>,
    default: CellStyle<'a>,
    override_width: HashMap<isize, CellSize>,
    override_height: HashMap<isize, CellSize>,
}

impl<'a> TableStyle<'a> {
    pub fn new(default: CellStyle<'a>) -> Self {
        Self {
            default,
            expand: None,
            override_width: HashMap::default(),
            override_height: HashMap::default(),
        }
    }
    pub fn expand(&mut self, pos: CellPos) {
        self.expand = Some(pos);
    }
    pub fn unexpand(&mut self) {
        self.expand = None;
    }
    pub fn expanded(&self) -> Option<CellPos> {
        self.expand
    }
    pub fn switch_expand(&mut self, pos: CellPos) {
        if self.expanded() == Some(pos) {
            self.unexpand();
        } else {
            self.expand(pos);
        }
    }
    pub fn get_style(&self, pos: CellPos) -> CellStyle<'a> {
        CellStyle {
            width: if self.expand.is_some_and(|p| p.x == pos.x) {
                300
            } else {
                *self
                    .override_width
                    .get(&pos.x)
                    .unwrap_or(&self.default.width)
            },
            height: if self.expand.is_some_and(|p| p.y == pos.y) {
                30
            } else {
                *self
                    .override_height
                    .get(&pos.y)
                    .unwrap_or(&self.default.height)
            },
            ..self.default
        }
    }
}
