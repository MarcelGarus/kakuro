use std::fmt::{self, Display, Formatter};

pub type Value = u8;
pub enum Box {
    Wall {
        vertical_sum: Option<Value>,
        horizontal_sum: Option<Value>,
    },
    Empty,
}
pub struct Board {
    pub boxes: Vec<Vec<Box>>, // Outer is vertical, inner horizontal.
}

pub trait ParseBoard {
    fn parse_board(&self) -> Board;
}
impl ParseBoard for str {
    fn parse_board(&self) -> Board {
        let boxes = self
            .lines()
            .map(|line| {
                line.split(' ')
                    .filter(|word| !word.is_empty())
                    .map(|word| {
                        if word.chars().all(|c| c == '_') {
                            Box::Empty
                        } else if word.chars().all(|c| c == 'W') {
                            Box::Wall {
                                horizontal_sum: None,
                                vertical_sum: None,
                            }
                        } else {
                            fn parse_sum(sum_str: &str) -> Option<Value> {
                                let sum: Result<Value, _> = sum_str.parse();
                                match sum {
                                    Ok(sum) => Some(sum),
                                    Err(_) => {
                                        if sum_str == "W" {
                                            None
                                        } else {
                                            panic!("Invalid sum {:?}.", sum_str);
                                        }
                                    }
                                }
                            }
                            let parts = word.split('\\').collect::<Vec<_>>();
                            if parts.len() != 2 {
                                panic!("Unknown box {:?}!", word);
                            }
                            Box::Wall {
                                vertical_sum: parse_sum(parts[0]),
                                horizontal_sum: parse_sum(parts[1]),
                            }
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        Board { boxes }
    }
}

impl Display for Box {
    // Always 5 chars.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Box::Wall {
                vertical_sum,
                horizontal_sum,
            } => match (vertical_sum, horizontal_sum) {
                (None, None) => "WWWWW".fmt(f),
                (vertical, horizontal) => {
                    fn fmt_sum(s: &Option<Value>) -> String {
                        match s {
                            None => "W".to_string(),
                            Some(s) => format!("{}", s),
                        }
                    }
                    write!(f, "{:>2}/{:2}", fmt_sum(vertical), fmt_sum(horizontal))
                }
            },
            Box::Empty => "_____".fmt(f),
        }
    }
}
impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let num_lines = self.boxes.len();
        for (i, line) in self.boxes.iter().enumerate() {
            for b in line {
                b.fmt(f)?;
                ' '.fmt(f)?;
            }
            if i < num_lines - 1 {
                '\n'.fmt(f)?;
            }
        }
        Ok(())
    }
}
