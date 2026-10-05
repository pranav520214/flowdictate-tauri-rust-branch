//! # Deterministic Cleanup (Stage A)
//!
//! Rule-based text cleanup that handles common transcription artifacts
//! without requiring a language model.

/// Configurable options for deterministic text cleanup.
#[derive(Debug, Clone)]
pub struct DeterministicOptions {
    pub remove_fillers: bool,
    pub expand_spoken_punctuation: bool,
    pub auto_capitalize: bool,
    pub normalize_whitespace: bool,
}

impl Default for DeterministicOptions {
    fn default() -> Self {
        Self {
            remove_fillers: true,
            expand_spoken_punctuation: true,
            auto_capitalize: true,
            normalize_whitespace: true,
        }
    }
}

/// Apply Stage A deterministic cleanup to raw transcript text.
pub fn clean_text(input: &str, options: &DeterministicOptions) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut text = input.to_string();

    // 1. Remove filler words if enabled
    if options.remove_fillers {
        text = remove_filler_words(&text);
    }

    // 2. Expand spoken punctuation if enabled
    if options.expand_spoken_punctuation {
        text = expand_spoken_punctuation(&text);
    }

    // 3. Normalize whitespace (collapse multiple spaces, trim)
    if options.normalize_whitespace {
        text = normalize_whitespace(&text);
    }

    // 4. Normalize spacing around punctuation
    text = fix_punctuation_spacing(&text);

    // 5. Auto-capitalize first word and after sentence boundaries (. ! ?)
    if options.auto_capitalize {
        text = capitalize_sentences(&text);
    }

    text
}

/// Remove common filler words.
fn remove_filler_words(text: &str) -> String {
    let fillers = [
        " um ", " uh ", " er ", " ah ", " Um ", " Uh ", " Er ", " Ah ",
    ];
    let mut result = format!(" {} ", text);
    for filler in fillers {
        while result.contains(filler) {
            result = result.replace(filler, " ");
        }
    }
    result.trim().to_string()
}

/// Convert spoken punctuation keywords to actual punctuation marks.
fn expand_spoken_punctuation(text: &str) -> String {
    let replacements = [
        (" period", "."),
        (" full stop", "."),
        (" comma", ","),
        (" question mark", "?"),
        (" exclamation point", "!"),
        (" exclamation mark", "!"),
        (" colon", ":"),
        (" semicolon", ";"),
        (" open parenthesis", " ("),
        (" close parenthesis", ")"),
        (" new line", "\n"),
        (" newline", "\n"),
        (" new paragraph", "\n\n"),
    ];

    let mut result = format!(" {}", text);
    for (spoken, symbol) in replacements {
        result = result.replace(spoken, symbol);
    }
    result.trim().to_string()
}

/// Collapse multiple spaces into single spaces.
fn normalize_whitespace(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut last_was_space = false;

    for c in text.chars() {
        if c == ' ' || c == '\t' {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(c);
            last_was_space = false;
        }
    }

    result.trim().to_string()
}

/// Ensure no space precedes punctuation like .,!?:; and at least one space follows.
fn fix_punctuation_spacing(text: &str) -> String {
    let result = text
        .replace(" .", ".")
        .replace(" ,", ",")
        .replace(" !", "!")
        .replace(" ?", "?")
        .replace(" :", ":")
        .replace(" ;", ";");

    // Fix missing space after punctuation if followed by an alphanumeric character
    let mut spaced = String::with_capacity(result.len() + 8);
    let chars: Vec<char> = result.chars().collect();
    for i in 0..chars.len() {
        spaced.push(chars[i]);
        if matches!(chars[i], '.' | ',' | '!' | '?' | ':' | ';') && i + 1 < chars.len() {
            let next = chars[i + 1];
            if next.is_alphabetic() {
                spaced.push(' ');
            }
        }
    }

    spaced
}

/// Capitalize the first character of each sentence.
fn capitalize_sentences(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut capitalize_next = true;

    for c in text.chars() {
        if capitalize_next && c.is_alphabetic() {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
            if matches!(c, '.' | '!' | '?' | '\n') {
                capitalize_next = true;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_fillers_and_normalizes() {
        let opts = DeterministicOptions::default();
        let raw = "hello um world uh this is a test period";
        let cleaned = clean_text(raw, &opts);
        assert_eq!(cleaned, "Hello world this is a test.");
    }

    #[test]
    fn capitalizes_sentences() {
        let opts = DeterministicOptions::default();
        let raw = "first sentence. second sentence? third sentence!";
        let cleaned = clean_text(raw, &opts);
        assert_eq!(cleaned, "First sentence. Second sentence? Third sentence!");
    }

    #[test]
    fn handles_spoken_punctuation() {
        let opts = DeterministicOptions::default();
        let raw = "item one comma item two comma item three period";
        let cleaned = clean_text(raw, &opts);
        assert_eq!(cleaned, "Item one, item two, item three.");
    }
}
