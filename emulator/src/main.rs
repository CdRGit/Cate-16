mod terminal;

use terminal::*;
use std::io::Read;
use std::io::Write;

fn main() {
    println!("Hello, world!");
    let mut term: Terminal = Terminal::make();

    let mut buffer = [0;1];
    writeln!(term, "Hit a key! ").unwrap();
    let mut count;
    for _ in 0..3146875 {
        count = term.read(&mut buffer).unwrap();
        if count != 0 {
            println!("You have hit: {:?} [{}]", buffer, count);
        }
    }
}
