use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

/// read_lines_offset_n reads contents from file and splits them by new line.
/// The offset tells at which line number to start.
/// The count determines the number of lines to read (starting from offset):
/// n >= 0: at most n lines
/// n < 0: whole file
pub fn read_lines_offset_n(filename: &str, offset: usize, n: isize) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut ret = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        if i < offset {
            continue;
        }

        if n >= 0 && i > (n as usize + offset) {
            break;
        }

        let line = line?;
        ret.push(line);
    };
    
    Ok(ret)
}