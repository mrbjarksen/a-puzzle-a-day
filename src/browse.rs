use crate::board::Board;
use crate::board::square::{Date, DateMap};

pub fn browse(_boards: DateMap<Vec<Board>>, _date: Date) -> std::io::Result<()> {
    println!("Browsing solutions is not implemented yet");
    Ok(())
}
