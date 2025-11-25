use rustyline::completion::{Completer as CompleterTrait, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::hint::{Hinter as HinterTrait, HistoryHinter};
use rustyline::validate::MatchingBracketValidator;
use rustyline::Context;
use rustyline::{Helper, Highlighter, Validator};

/// REPL Helper for autocomplete and highlighting
#[derive(Helper, Validator, Highlighter)]
pub struct ReplHelper {
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    #[rustyline(Highlighter)]
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
    #[allow(dead_code)]
    colored_prompt: String,
}

impl ReplHelper {
    pub fn new() -> Self {
        Self {
            validator: MatchingBracketValidator::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            colored_prompt: "".to_owned(),
        }
    }
}

impl CompleterTrait for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let commands = vec![
            "/help",
            "/settings",
            "/clear",
            "/status",
            "/model",
            "/permissions",
            "/quit",
            "/exit",
            "/version",
        ];

        // Only complete if starting with /
        if line.starts_with('/') {
            let matches: Vec<Pair> = commands
                .iter()
                .filter(|cmd| cmd.starts_with(line))
                .map(|cmd| Pair {
                    display: cmd.to_string(),
                    replacement: cmd.to_string(),
                })
                .collect();

            return Ok((0, matches));
        }

        Ok((0, Vec::with_capacity(0)))
    }
}

impl HinterTrait for ReplHelper {
    type Hint = String;
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        if line == "/" {
            return Some(" (Tab for: help, settings, clear...)".to_string());
        }
        self.hinter.hint(line, pos, ctx)
    }
}
