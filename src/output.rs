pub fn print_board(board: &[Vec<bool>]) {
    for row in board {
        for &occupied in row {
            if occupied {
                print!("o");
            } else {
                print!(".");
            }
        }
        println!();
    }
}

pub fn multi_print_boards(board1: &[Vec<bool>], board2: &[Vec<bool>]) {
    let mut out = String::new();
    for (row1, row2) in board1.iter().zip(board2) {
        for &occupied in row1 {
            out.push(if occupied { 'o' } else { '.' });
        }
        out.push_str("    ");
        for &occupied in row2 {
            out.push(if occupied { 'o' } else { '.' });
        }
        out.push('\n');
    }
    print!("{out}");
}
