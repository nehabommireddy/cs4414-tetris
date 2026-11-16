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
