use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(i8)]
enum Token {
    MS,
    PS,
    D0,
    D19,
    Pt,
    Exp,
    EOF,
    NoCond,
    Invalid,
}

impl Token {
    fn from_char(ch: char) -> Self {
        match ch {
            '-' => MS,
            '+' => PS,
            '0' => D0,
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => D19,
            '.' => Pt,
            'e' | 'E' => Exp,
            _ => Invalid,
        }
    }
}

use Token::*;

type State = i8;
type StateTableRow = HashMap<Token, State>;
type StateTable = HashMap<State, StateTableRow>;

const S: State = 0;
const T: State = 12;

macro_rules! state_table {
        {$($state:expr => [$($token:expr => $target:expr $(,)?)+]$(,)?)+} => {{
            let mut __map = StateTable::new();
            $(
                __map.insert($state, {
                    let mut __table = StateTableRow::new();
                    $(
                        __table.insert($token, $target as i8);
                    )+
                    __table
                });
            )+

            __map
        }};
    }

lazy_static! {
    static ref TABLE: StateTable = state_table! {
        S => [MS=>1,NoCond=>2],
        1 => [NoCond => 2],
        2 => [D0=>3,D19=>4],
        3 => [NoCond=>5],
        4 => [D0=>4,D19=>4,NoCond=>5],
        5 => [EOF=>T,Pt=>6,NoCond=>8],
        6 => [D0=>7,D19=>7],
        7 => [D0=>7,D19=>7,NoCond=>8],
        8 => [EOF=>T,Exp=>9],
        9 => [MS=>10,PS=>10,D0=>11,D19=>11],
        10 => [NoCond=>11],
        11 => [D0=>11,D19=>11,EOF=>T]
    };
}

pub fn validate_number(s: &str) -> bool {
    let mut chars = s.chars().peekable();

    let mut state: State = S;
    let mut row = TABLE.get(&state).unwrap();

    loop {
        let tk = chars.peek().cloned().map(Token::from_char).unwrap_or(EOF);

        if tk == Invalid {
            return false;
        }

        if let Some(&target) = row.get(&tk) {
            chars.next();
            state = target;
        } else {
            if let Some(&target) = row.get(&NoCond) {
                state = target;
            } else {
                return false;
            }
        }
        
        if state == T {
            return chars.peek().is_none();
        } else {
            row = TABLE.get(&state).unwrap();
        }
    }
}

#[cfg(test)]
#[test]
fn test_validate_number() {
    assert!(validate_number("0"));
    assert!(validate_number("-0"));
    assert!(validate_number("-0.0"));
    assert!(validate_number("1"));
    assert!(validate_number("-1"));
    assert!(validate_number("1.5"));
    assert!(validate_number("-1.5"));
    assert!(validate_number("3.1416"));
    assert!(validate_number("1E10"));
    assert!(validate_number("1e10"));
    assert!(validate_number("1E+10"));
    assert!(validate_number("1E-10"));
    assert!(validate_number("-1E10"));
    assert!(validate_number("-1e10"));
    assert!(validate_number("-1E+10"));
    assert!(validate_number("-1E-10"));
    assert!(validate_number("1.234E+10"));
    assert!(validate_number("1.234E-10"));
    assert!(validate_number("1e-10000"));

    assert!(!validate_number("+1"));
    assert!(!validate_number("+0"));
    assert!(!validate_number(".123"));
    assert!(!validate_number("1."));
    assert!(!validate_number("INF"));
    assert!(!validate_number("inf"));
    assert!(!validate_number("NAN"));
    assert!(!validate_number("nan"));
}

#[cfg(test)]
#[test]
fn test_print_table() {
    let mut table: [[State; 8]; 13] = [[-1; 8]; 13];
    for (&state, row) in TABLE.iter() {
        for (&token, &target) in row {
            table[state as usize][token as usize] = target;
        }
    }
    println!("{:?}", table);
}
