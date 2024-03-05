#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    separator: Option<char>,
    tokens: Vec<PatternToken>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PatternToken {
    Exact(String),
    OneWildcard,
    ManyWildcard,
    AnyChar,
}

impl Pattern {
    pub fn new(pattern: &str, separator: Option<char>) -> Pattern {
        let mut tokens = vec![];
        for ch in pattern.chars() {
            match ch {
                '*' => {
                    if tokens.last() == Some(&PatternToken::OneWildcard) {
                        *tokens.last_mut().unwrap() = PatternToken::ManyWildcard;
                    } else {
                        tokens.push(PatternToken::OneWildcard);
                    }
                }
                '?' => tokens.push(PatternToken::AnyChar),
                _ => match tokens.last_mut() {
                    Some(PatternToken::Exact(ref mut s)) => s.push(ch),
                    _ => tokens.push(PatternToken::Exact(ch.into())),
                },
            }
        }
        Pattern { separator, tokens }
    }

    pub fn is_match(&self, string: &str) -> bool {
        #[derive(Debug)]
        enum MatchState {
            Precise,
            ScanAhead { stop_at_separator: bool },
        }
        let mut state = MatchState::Precise;
        let mut tokens = &self.tokens[..];
        let mut search_str = string;
        while !tokens.is_empty() {
            match tokens.first().unwrap() {
                PatternToken::Exact(s) => {
                    if search_str.starts_with(s) {
                        // Exact match passed. Consume the token and string and continue
                        tokens = &tokens[1..];
                        search_str = &search_str[s.len()..];
                        state = MatchState::Precise;
                    } else {
                        match state {
                            MatchState::Precise => {
                                // Can't possibly match
                                return false;
                            }
                            MatchState::ScanAhead { stop_at_separator } => {
                                if search_str.is_empty() {
                                    // End of input, can't match
                                    return false;
                                }
                                if stop_at_separator
                                    && self
                                        .separator
                                        .is_some_and(|sep| search_str.starts_with(sep))
                                {
                                    // Found the separator. Consume a char and revert to precise
                                    // mode
                                    search_str = &search_str[1..];
                                    state = MatchState::Precise;
                                } else {
                                    // Skip the non-matching char and continue
                                    search_str = &search_str[1..];
                                }
                            }
                        }
                    }
                }
                PatternToken::OneWildcard => {
                    // Set the mode to ScanAhead, stopping at separator
                    state = MatchState::ScanAhead {
                        stop_at_separator: true,
                    };
                    tokens = &tokens[1..];
                }
                PatternToken::ManyWildcard => {
                    // Set the mode to ScanAhead, ignoring separator
                    state = MatchState::ScanAhead {
                        stop_at_separator: false,
                    };
                    tokens = &tokens[1..];
                }
                PatternToken::AnyChar => {
                    if !search_str.is_empty() {
                        // Take a char from the search str and continue
                        search_str = &search_str[1..];
                        tokens = &tokens[1..];
                    } else {
                        // End of input
                        return false;
                    }
                }
            }
        }
        #[cfg(test)]
        {
            println!(
                "end, state={:?}, search_str={:?}, tokens={:?}",
                state, search_str, tokens
            );
        }
        if !search_str.is_empty() {
            // If the search str is not empty at the end
            match state {
                // We didn't end with a wildcard, so this is a fail
                MatchState::Precise => false,
                // This could be a match as long as the separator isn't contained in the remainder
                MatchState::ScanAhead {
                    stop_at_separator: true,
                } => {
                    if let Some(separator) = self.separator {
                        !search_str.contains(separator)
                    } else {
                        // No separator specified, so this is a success
                        true
                    }
                }
                // Always a success, no matter what remains
                MatchState::ScanAhead {
                    stop_at_separator: false,
                } => true,
            }
        } else {
            // The match has succeeded - there is nothing more to match
            true
        }
    }
}

#[test]
fn test_pattern_new() {
    assert_eq!(
        Pattern::new("", Some('/')),
        Pattern {
            separator: Some('/'),
            tokens: vec![]
        }
    );
    assert_eq!(
        Pattern::new("", None),
        Pattern {
            separator: None,
            tokens: vec![]
        }
    );
    assert_eq!(
        Pattern::new("org.freedesktop.DBus", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![PatternToken::Exact("org.freedesktop.DBus".into()),]
        }
    );
    assert_eq!(
        Pattern::new("*", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![PatternToken::OneWildcard,]
        }
    );
    assert_eq!(
        Pattern::new("**", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![PatternToken::ManyWildcard,]
        }
    );
    assert_eq!(
        Pattern::new("?", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![PatternToken::AnyChar,]
        }
    );
    assert_eq!(
        Pattern::new("org.freedesktop.*", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![
                PatternToken::Exact("org.freedesktop.".into()),
                PatternToken::OneWildcard,
            ]
        }
    );
    assert_eq!(
        Pattern::new("org.freedesktop.**", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![
                PatternToken::Exact("org.freedesktop.".into()),
                PatternToken::ManyWildcard,
            ]
        }
    );
    assert_eq!(
        Pattern::new("org.*.DBus", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![
                PatternToken::Exact("org.".into()),
                PatternToken::OneWildcard,
                PatternToken::Exact(".DBus".into()),
            ]
        }
    );
    assert_eq!(
        Pattern::new("org.**.DBus", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![
                PatternToken::Exact("org.".into()),
                PatternToken::ManyWildcard,
                PatternToken::Exact(".DBus".into()),
            ]
        }
    );
    assert_eq!(
        Pattern::new("org.**.?Bus", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![
                PatternToken::Exact("org.".into()),
                PatternToken::ManyWildcard,
                PatternToken::Exact(".".into()),
                PatternToken::AnyChar,
                PatternToken::Exact("Bus".into()),
            ]
        }
    );
    assert_eq!(
        Pattern::new("org.free*top", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![
                PatternToken::Exact("org.free".into()),
                PatternToken::OneWildcard,
                PatternToken::Exact("top".into()),
            ]
        }
    );
    assert_eq!(
        Pattern::new("org.free**top", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![
                PatternToken::Exact("org.free".into()),
                PatternToken::ManyWildcard,
                PatternToken::Exact("top".into()),
            ]
        }
    );
    assert_eq!(
        Pattern::new("org.**top", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![
                PatternToken::Exact("org.".into()),
                PatternToken::ManyWildcard,
                PatternToken::Exact("top".into()),
            ]
        }
    );
    assert_eq!(
        Pattern::new("**top", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![
                PatternToken::ManyWildcard,
                PatternToken::Exact("top".into()),
            ]
        }
    );
    assert_eq!(
        Pattern::new("org.free**", Some('.')),
        Pattern {
            separator: Some('.'),
            tokens: vec![
                PatternToken::Exact("org.free".into()),
                PatternToken::ManyWildcard,
            ]
        }
    );
}

#[test]
fn test_pattern_is_match_empty() {
    let pat = Pattern {
        separator: Some('.'),
        tokens: vec![],
    };
    assert!(pat.is_match(""));
    assert!(!pat.is_match("anystring"));
    assert!(!pat.is_match("anystring.anyotherstring"));
}

#[test]
fn test_pattern_is_match_exact() {
    let pat = Pattern {
        separator: Some('.'),
        tokens: vec![PatternToken::Exact("specific".into())],
    };
    assert!(pat.is_match("specific"));
    assert!(!pat.is_match(""));
    assert!(!pat.is_match("specifi"));
    assert!(!pat.is_match("specifica"));
}

#[test]
fn test_pattern_is_match_one_wildcard() {
    let pat = Pattern {
        separator: Some('.'),
        tokens: vec![
            PatternToken::Exact("foo.".into()),
            PatternToken::OneWildcard,
            PatternToken::Exact(".baz".into()),
        ],
    };
    assert!(pat.is_match("foo.bar.baz"));
    assert!(pat.is_match("foo.grok.baz"));
    assert!(pat.is_match("foo..baz"));
    assert!(!pat.is_match("foo.ono.notmatch.baz"));
    assert!(!pat.is_match(""));
    assert!(!pat.is_match("specifi"));
    assert!(!pat.is_match("specifica.baz"));
    assert!(!pat.is_match("foo.specifica"));
}

#[test]
fn test_pattern_is_match_one_wildcard_at_end() {
    let pat = Pattern {
        separator: Some('.'),
        tokens: vec![
            PatternToken::Exact("foo.".into()),
            PatternToken::OneWildcard,
        ],
    };
    assert!(pat.is_match("foo.bar"));
    assert!(pat.is_match("foo.grok"));
    assert!(pat.is_match("foo."));
    assert!(!pat.is_match("foo.ono.notmatch.baz"));
    assert!(!pat.is_match(""));
    assert!(!pat.is_match("specifi"));
    assert!(!pat.is_match("specifica.baz"));
}

#[test]
fn test_pattern_is_match_one_wildcard_at_start() {
    let pat = Pattern {
        separator: Some('.'),
        tokens: vec![
            PatternToken::OneWildcard,
            PatternToken::Exact(".bar".into()),
        ],
    };
    assert!(pat.is_match("foo.bar"));
    assert!(pat.is_match("grok.bar"));
    assert!(pat.is_match(".bar"));
    assert!(!pat.is_match("foo.ono.notmatch.bar"));
    assert!(!pat.is_match(""));
    assert!(!pat.is_match("specifi"));
    assert!(!pat.is_match("specifica.baz"));
}

#[test]
fn test_pattern_is_match_one_wildcard_no_separator() {
    let pat = Pattern {
        separator: None,
        tokens: vec![
            PatternToken::Exact("foo.".into()),
            PatternToken::OneWildcard,
            PatternToken::Exact(".baz".into()),
        ],
    };
    assert!(pat.is_match("foo.bar.baz"));
    assert!(pat.is_match("foo.grok.baz"));
    assert!(pat.is_match("foo..baz"));
    assert!(pat.is_match("foo.this.shouldmatch.baz"));
    assert!(pat.is_match("foo.this.should.match.baz"));
    assert!(!pat.is_match(""));
    assert!(!pat.is_match("specifi"));
    assert!(!pat.is_match("specifica.baz"));
    assert!(!pat.is_match("foo.specifica"));
}

#[test]
fn test_pattern_is_match_many_wildcard() {
    let pat = Pattern {
        separator: Some('.'),
        tokens: vec![
            PatternToken::Exact("foo.".into()),
            PatternToken::ManyWildcard,
            PatternToken::Exact(".baz".into()),
        ],
    };
    assert!(pat.is_match("foo.bar.baz"));
    assert!(pat.is_match("foo.grok.baz"));
    assert!(pat.is_match("foo..baz"));
    assert!(pat.is_match("foo.this.shouldmatch.baz"));
    assert!(pat.is_match("foo.this.should.match.baz"));
    assert!(!pat.is_match(""));
    assert!(!pat.is_match("specifi"));
    assert!(!pat.is_match("specifica.baz"));
    assert!(!pat.is_match("foo.specifica"));
}

#[test]
fn test_pattern_is_match_many_wildcard_at_end() {
    let pat = Pattern {
        separator: Some('.'),
        tokens: vec![
            PatternToken::Exact("foo.".into()),
            PatternToken::ManyWildcard,
        ],
    };
    assert!(pat.is_match("foo.bar"));
    assert!(pat.is_match("foo.grok"));
    assert!(pat.is_match("foo."));
    assert!(pat.is_match("foo.this.should.match"));
    assert!(!pat.is_match(""));
    assert!(!pat.is_match("specifi"));
    assert!(!pat.is_match("specifica.baz"));
}

#[test]
fn test_pattern_is_match_many_wildcard_at_start() {
    let pat = Pattern {
        separator: Some('.'),
        tokens: vec![
            PatternToken::ManyWildcard,
            PatternToken::Exact(".bar".into()),
        ],
    };
    assert!(pat.is_match("foo.bar"));
    assert!(pat.is_match("grok.bar"));
    assert!(pat.is_match("should.match.bar"));
    assert!(pat.is_match(".bar"));
    assert!(!pat.is_match(""));
    assert!(!pat.is_match("specifi"));
    assert!(!pat.is_match("specifica.baz"));
}

#[test]
fn test_pattern_is_match_any_char() {
    let pat = Pattern {
        separator: Some('.'),
        tokens: vec![
            PatternToken::Exact("fo".into()),
            PatternToken::AnyChar,
            PatternToken::Exact(".baz".into()),
        ],
    };
    assert!(pat.is_match("foo.baz"));
    assert!(pat.is_match("foe.baz"));
    assert!(pat.is_match("foi.baz"));
    assert!(!pat.is_match(""));
    assert!(!pat.is_match("fooo.baz"));
    assert!(!pat.is_match("fo.baz"));
}
