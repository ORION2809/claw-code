use std::io::{self, IsTerminal, Write};

use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};

const FOOTER_LINES: usize = 2;
const MIN_PAGE_BODY_LINES: usize = 3;

pub fn print_report(report: &str) -> io::Result<()> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() || !should_page(report)? {
        println!("{report}");
        return Ok(());
    }

    Pager::new(report).run()
}

fn should_page(report: &str) -> io::Result<bool> {
    let (_, rows) = terminal::size()?;
    let height = usize::from(rows);
    Ok(height > FOOTER_LINES && report.lines().count() > page_body_height(height))
}

fn page_body_height(terminal_rows: usize) -> usize {
    terminal_rows
        .saturating_sub(FOOTER_LINES)
        .max(MIN_PAGE_BODY_LINES)
}

struct Pager {
    lines: Vec<String>,
    offset: usize,
}

impl Pager {
    fn new(report: &str) -> Self {
        Self {
            lines: report.lines().map(ToOwned::to_owned).collect(),
            offset: 0,
        }
    }

    fn run(&mut self) -> io::Result<()> {
        let _terminal = TerminalPagerGuard::enter()?;
        self.render()?;

        loop {
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('j') | KeyCode::Down => self.step_down()?,
                KeyCode::Char('k') | KeyCode::Up => self.step_up(),
                KeyCode::Char('g') | KeyCode::Home => self.jump_to_start(),
                KeyCode::Char('G') | KeyCode::End => self.jump_to_end()?,
                KeyCode::Char(' ') | KeyCode::PageDown => self.page_down()?,
                KeyCode::PageUp => self.page_up(),
                _ => {}
            }
            self.render()?;
        }

        Ok(())
    }

    fn render(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        let (_, rows) = terminal::size()?;
        let height = usize::from(rows);
        let body_height = page_body_height(height);
        let end = (self.offset + body_height).min(self.lines.len());

        execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
        for line in &self.lines[self.offset..end] {
            writeln!(stdout, "{line}")?;
        }

        for _ in end - self.offset..body_height {
            writeln!(stdout)?;
        }

        let visible_end = end.min(self.lines.len());
        let footer = format!(
            "[j/k scroll, space page, g/G jump, q quit] lines {}-{} of {}",
            self.offset + 1,
            visible_end.max(self.offset + 1),
            self.lines.len()
        );
        writeln!(stdout, "{}", "─".repeat(footer.len().max(24)))?;
        write!(stdout, "{footer}")?;
        stdout.flush()
    }

    fn step_down(&mut self) -> io::Result<()> {
        let max_offset = self.max_offset()?;
        self.offset = (self.offset + 1).min(max_offset);
        Ok(())
    }

    fn step_up(&mut self) {
        self.offset = self.offset.saturating_sub(1);
    }

    fn page_down(&mut self) -> io::Result<()> {
        let (_, rows) = terminal::size()?;
        let body_height = page_body_height(usize::from(rows));
        let max_offset = self.max_offset()?;
        self.offset = (self.offset + body_height).min(max_offset);
        Ok(())
    }

    fn page_up(&mut self) {
        if let Ok((_, rows)) = terminal::size() {
            let body_height = page_body_height(usize::from(rows));
            self.offset = self.offset.saturating_sub(body_height);
        }
    }

    fn jump_to_start(&mut self) {
        self.offset = 0;
    }

    fn jump_to_end(&mut self) -> io::Result<()> {
        self.offset = self.max_offset()?;
        Ok(())
    }

    fn max_offset(&self) -> io::Result<usize> {
        let (_, rows) = terminal::size()?;
        let body_height = page_body_height(usize::from(rows));
        Ok(self.lines.len().saturating_sub(body_height))
    }
}

struct TerminalPagerGuard;

impl TerminalPagerGuard {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalPagerGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::page_body_height;

    #[test]
    fn page_body_height_reserves_footer_rows() {
        assert_eq!(page_body_height(10), 8);
        assert_eq!(page_body_height(2), 3);
    }
}
