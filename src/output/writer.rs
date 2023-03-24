use core::fmt;
use std::io::Write;

use anyhow::Result;
use itertools::{Itertools, Position, WithPosition};

struct Symbols {
    pub(super) down: &'static str,
    pub(super) tee: &'static str,
    pub(super) ell: &'static str,
    pub(super) right: &'static str,
}

const SYMBOLS: Symbols = Symbols {
    down: "│",
    tee: "├",
    ell: "└",
    right: "─",
};

pub(super) struct Writer<'a> {
    indent: usize,
    level: usize,
    last: bool,
    super_last: bool,
    out: &'a mut dyn Write,
}

impl<'a> Writer<'a> {
    pub(super) fn new(indent: usize, out: &'a mut dyn Write) -> Self {
        Self {
            indent,
            level: 0,
            last: false,
            super_last: false,
            out,
        }
    }

    pub(super) fn iter<I>(&mut self, iter: I) -> Iter<'_, 'a, I::IntoIter>
    where
        I: IntoIterator,
    {
        let iter = iter.into_iter().with_position();

        Iter { writer: self, iter }
    }

    pub(super) fn write_line(&mut self, fmt: fmt::Arguments<'_>) -> Result<()> {
        for n in 0..self.current_indent() {
            if !self.super_last && n % self.indent == 0 {
                write!(self.out, "{}", SYMBOLS.down)?;
            } else {
                write!(self.out, " ")?;
            }
        }
        write!(
            self.out,
            "{}{r}{r} ",
            if self.last { SYMBOLS.ell } else { SYMBOLS.tee },
            r = SYMBOLS.right
        )?;

        self.out.write_fmt(fmt)?;
        self.out.write_all(b"\n")?;

        Ok(())
    }

    fn current_indent(&self) -> usize {
        (self.level - 1) * self.indent
    }
}

pub(super) trait LendingIterator {
    type Item<'a>
    where
        Self: 'a;

    fn next(&mut self) -> Option<Self::Item<'_>>;
}

pub(super) struct Iter<'a, 'b, I>
where
    I: Iterator,
{
    writer: &'a mut Writer<'b>,
    iter: WithPosition<I>,
}

impl<'a, 'b, I> LendingIterator for Iter<'a, 'b, I>
where
    I: Iterator,
{
    type Item<'this> = (Writer<'this>, I::Item)
    where
        Self: 'this;

    fn next(&mut self) -> Option<Self::Item<'_>> {
        let (item, last) = match self.iter.next()? {
            Position::First(inner) | Position::Middle(inner) => (inner, false),
            Position::Last(inner) | Position::Only(inner) => (inner, true),
        };
        let writer = Writer {
            indent: self.writer.indent,
            level: self.writer.level + 1,
            last,
            super_last: self.writer.last,
            out: &mut self.writer.out,
        };
        Some((writer, item))
    }
}
