use crate::board::path::Direction::{self, *};

use Square::*;

use std::fmt;

#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug, Hash)]
#[repr(u8)]
pub enum Square {
    Jan =  0, Feb, Mar, Apr, May, Jun,
    Jul =  7, Aug, Sep, Oct, Nov, Dec,
    D01 = 14, D02, D03, D04, D05, D06, D07,
    D08 = 21, D09, D10, D11, D12, D13, D14,
    D15 = 28, D16, D17, D18, D19, D20, D21,
    D22 = 35, D23, D24, D25, D26, D27, D28,
    D29 = 42, D30, D31,
}

impl Square {
    pub fn step(&self, dir: Direction) -> Option<Self> {
        let square = *self as u8;
        match (dir, square % 7, square / 7) {
            (Up,    _, 0) => None,
            (Left,  0, _) => None,
            (Right, 6, _) => None,
            _ => Square::try_from(match dir {
                Up    => square - 7,
                Down  => square + 7,
                Left  => square - 1,
                Right => square + 1,
            }).ok()
        }
    }

    pub fn squares() -> Vec<Square> {
        vec![
            Jan, Feb, Mar, Apr, May, Jun,
            Jul, Aug, Sep, Oct, Nov, Dec,
            D01, D02, D03, D04, D05, D06, D07,
            D08, D09, D10, D11, D12, D13, D14,
            D15, D16, D17, D18, D19, D20, D21,
            D22, D23, D24, D25, D26, D27, D28,
            D29, D30, D31,
        ]
    }
}

#[derive(Debug)]
pub struct IndexError;

impl TryFrom<u8> for Square {
    type Error = IndexError;
    
    fn try_from(index: u8) -> Result<Self, Self::Error> {
        if index == 6 || index == 13 || index > 44 {
            Err(IndexError)
        } else {
            Ok(match index {
                 0 => Jan,  1 => Feb,  2 => Mar,  3 => Apr,  4 => May,  5 => Jun,
                 7 => Jul,  8 => Aug,  9 => Sep, 10 => Oct, 11 => Nov, 12 => Dec,
                14 => D01, 15 => D02, 16 => D03, 17 => D04, 18 => D05, 19 => D06, 20 => D07,
                21 => D08, 22 => D09, 23 => D10, 24 => D11, 25 => D12, 26 => D13, 27 => D14,
                28 => D15, 29 => D16, 30 => D17, 31 => D18, 32 => D19, 33 => D20, 34 => D21,
                35 => D22, 36 => D23, 37 => D24, 38 => D25, 39 => D26, 40 => D27, 41 => D28,
                42 => D29, 43 => D30, 44 => D31,
                _ => unreachable!(),
            })
        }
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Jan => "Jan", Feb => "Feb", Mar => "Mar", Apr => "Apr", May => "May", Jun => "Jun",
            Jul => "Jul", Aug => "Aug", Sep => "Sep", Oct => "Oct", Nov => "Nov", Dec => "Dec",
            D01 =>   "1", D02 =>   "2", D03 =>   "3", D04 =>   "4", D05 =>   "5", D06 =>   "6", D07 =>   "7",
            D08 =>   "8", D09 =>   "9", D10 =>  "10", D11 =>  "11", D12 =>  "12", D13 =>  "13", D14 =>  "14",
            D15 =>  "15", D16 =>  "16", D17 =>  "17", D18 =>  "18", D19 =>  "19", D20 =>  "20", D21 =>  "21",
            D22 =>  "22", D23 =>  "23", D24 =>  "24", D25 =>  "25", D26 =>  "26", D27 =>  "27", D28 =>  "28",
            D29 =>  "29", D30 =>  "30", D31 =>  "31",
        })
    }
}

#[derive(Copy, Clone, Eq, Ord, PartialEq, PartialOrd, Debug, Hash)]
pub struct Date {
    pub month: Square,
    pub day: Square,
}

impl Date {
    pub fn is_valid(&self) -> bool {
        match (self.month, self.day) {
            _ if self.month > Dec || self.day < D01 => false,
            (Apr | Jun | Sep | Nov, D31) => false,
            (Feb, D30 | D31) => false,
            _ => true
        }
    }

    pub fn next(&self) -> Self {
        if !self.is_valid() {
            return Date { month: Jan, day: D01 };
        }

        if let Ok(next_day) = Square::try_from(self.day as u8 + 1) {
            let next_date = Date { month: self.month, day: next_day };
            if next_date.is_valid() {
                return next_date;
            }
        }

        let next_month = match self.month {
            Jun => Jul, Dec => Jan,
            _ => Square::try_from(self.month as u8 + 1).unwrap(),
        };

        Date { month: next_month, day: D01 }
    }

    pub fn prev(&self) -> Self {
        if !self.is_valid() {
            return Date { month: Jan, day: D01 };
        }

        if self.day == D01 {
            let prev_month = match self.month {
                Jan => Dec, Jul => Jun,
                _ => Square::try_from(self.month as u8 - 1).unwrap(),
            };
            let prev_day = match self.month {
                Mar => D29,
                May | Jul | Oct | Dec => D30,
                _ => D31,
            };
            return Date { month: prev_month, day: prev_day };
        }

        Date { month: self.month, day: Square::try_from(self.day as u8 - 1).unwrap() }
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.month, self.day)
    }
}

pub type DateMap<T> = std::collections::HashMap<Date, T>;
