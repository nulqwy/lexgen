use std::ops::RangeInclusive;

use crate::DFA;

const CHAR_GAP: RangeInclusive<u32> = 0xD800..=0xDFFF;

/// Move range ends out of char gaps
pub fn normalise_ranges<T, A>(dfa: &mut DFA<T, A>) {
    for state in dfa.states.iter_mut() {
        state.range_transitions.retain_mut(|range| {
            let start_nonnormal = CHAR_GAP.contains(&range.start);
            let end_nonnormal = CHAR_GAP.contains(&range.end);

            match (start_nonnormal, end_nonnormal) {
                //        S E
                // ... G .^.^. G ...
                (true, true) => false,
                //         S      E
                // ... G ..^.. G .^.
                (true, false) => {
                    // Move the start after the gap
                    range.start = CHAR_GAP.end() + 1;
                    true
                }
                //  S      E
                // .^. G ..^.. G ...
                (false, true) => {
                    // Move the end before the gap
                    range.end = CHAR_GAP.start() - 1;
                    true
                }
                // Any other case (either both are before or after the gap or contain it).
                // It doesn't matter if the range contains the gap - that's fine
                (false, false) => true,
            }
        });
    }
}

#[test]
fn normalisation_correct() {
    // See test normalisation_sound() (tests/tests.rs:1294)
    // for the equivalent lexer being constructed here

    use crate::ast::{CharOrRange, CharSet, Regex};

    let mut nfa = crate::NFA::new();

    nfa.add_regex(&Default::default(), &Regex::Char('\u{E000}'), None, 1);
    nfa.add_regex(&Default::default(), &Regex::Char('\u{D7FF}'), None, 2);
    nfa.add_regex(
        &Default::default(),
        &Regex::CharSet(CharSet(vec![CharOrRange::Range('\u{D7DD}', '\u{E000}')])),
        None,
        3,
    );

    let mut dfa = crate::nfa_to_dfa(&nfa);

    crate::dfa::normalise::normalise_ranges(&mut dfa);

    for state in dfa.states.iter() {
        for range in state.range_transitions.iter() {
            assert!(!CHAR_GAP.contains(&range.start), "start is in the gap");
            assert!(!CHAR_GAP.contains(&range.end), "end is in the gap");
        }
    }
}
