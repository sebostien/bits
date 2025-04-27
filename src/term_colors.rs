use std::io::{self, Write};

pub struct TermColors {}

impl TermColors {
    pub fn print_colors() -> anyhow::Result<()> {
        let mut stdout = io::stdout();

        for i in 0..=255 {
            write!(stdout, "\x1b[48;5;{i}m{i:3}\x1b[0m ")?;
            if (i == 15) || (i > 15) && ((i - 15) % 6 == 0) {
                stdout.write_all("\n".as_bytes())?;
            }
        }

        Ok(stdout.flush()?)
    }
}
