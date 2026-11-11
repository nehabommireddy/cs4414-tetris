pub fn print_board(board: &[Vec<bool>]) {
    for row in board.iter().rev() {
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
