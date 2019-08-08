#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(i8)]
enum Token {
    MS,
    PS,
    D0,
    D19,
    Pt,
    Exp,
    EOF,
}

impl Token {
    #[inline(always)]
    fn from_char(ch: char) -> Result<Self, ()> {
        Ok(match ch {
            '-' => MS,
            '+' => PS,
            '0' => D0,
            '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => D19,
            '.' => Pt,
            'e' | 'E' => Exp,
            _ => return Err(()),
        })
    }
}

use Token::*;

type State = i8;
type StateTable = [[State; 7]; 10];

const START: State = 0;
const END: State = 11;

macro_rules! state_table {
    {$($state:expr => [$($token:expr => $target:expr $(,)?)+]$(,)?)+} => {{
        let mut __table : StateTable = [[-1;7];10];
        $(
            let __row = &mut __table[$state as usize];
            $(
                __row[$token as usize] = $target;
            )+
        )+

        __table
    }};
}

fn init_table() -> StateTable {
    state_table! {
        START => [MS => 1, D0 => 2, D19 => 3],
        1 => [D0 => 2, D19 => 3],
        2 => [Pt => 5, Exp => 7, EOF => END],
        3 => [D0 => 4, D19 => 4, Pt => 5, Exp => 7, EOF => END],
        4 => [D0 => 4, D19 => 4, Pt => 5, Exp => 7, EOF => END],
        5 => [D0 => 6, D19 => 6],
        6 => [D0 => 6, D19 => 6, Exp => 7, EOF => END],
        7 => [D0 => 9, D19 => 9, MS => 8, PS => 8],
        8 => [D0 => 9, D19 => 9],
        9 => [D0 => 9, D19 => 9, EOF => END],
    }
}

lazy_static! {
    static ref TABLE: StateTable = init_table();
}

pub fn validate_number(s: &str) -> bool {
    let mut chars = s.chars().peekable();
    let mut state: State = START;

    loop {
        let row = unsafe { TABLE.get_unchecked(state as usize) };

        let tk = match chars.peek() {
            None => Token::EOF,
            Some(&ch) => match Token::from_char(ch) {
                Err(_) => return false,
                Ok(tk) => tk,
            },
        };

        let &target = unsafe { row.get_unchecked(tk as usize) };
        if target != -1 {
            chars.next();
            state = target;
        } else {
            return false;
        }

        if state == END {
            return chars.peek().is_none();
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
