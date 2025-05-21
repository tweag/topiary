use crate::error::Error;

trait RunOf {
    fn is_run_of(&self, what: char) -> bool;
}

impl RunOf for String {
    fn is_run_of(&self, what: char) -> bool {
        !self.is_empty() && self.trim_start_matches(what).is_empty()
    }
}

/// Post-process stdin linewise (a poor-man's sed)
pub fn postprocess() -> Result<(), Error> {
    for line in std::io::stdin().lines() {
        match line? {
            // Update the title line with the correct section number (1) and a custom section name
            line if line.starts_with(".TH") => {
                println!(".TH TOPIARY 1 \"\" \"\" \"Topiary Manual\"");
            }

            // Reduce subheading underlines by two characters
            line if line.is_run_of('=') => {
                let width = line.len() - 2;
                println!("{:=<width$}", "");
            }

            // Everything else
            line => {
                println!("{line}");
            }
        }
    }

    Ok(())
}
