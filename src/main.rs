use clap::Parser;
use nirmana_solitaire::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Initial state of the game, as twelve numbers without spaces
    #[arg(short, long)]
    board: String,

    /// If present, indicates play mode. Numbers indicate the indices of moves to make.
    /// The final result will be the accumulated score, and the state of the board.
    #[arg(short, long, num_args=..)]
    moves: Vec<u8>,
}

fn main() {
    let args = Args::parse();

    let board: Vec<u8> = args
        .board
        .chars()
        .map(|x| x.to_digit(10).expect("board chars must be digits") as u8)
        .collect();
    let sum: u8 = board.iter().sum();
    assert!(
        sum <= 50,
        "Initial state can't have more than 50 stones, was {}",
        sum
    );
    let mut state = State::try_from(&*board).unwrap();

    println!("{}", state);
    if args.moves.len() > 0 {
        match state.play_moves(&*args.moves) {
            Ok(score) => println!("Final score: {}", score),
            Err(err) => println!("{} idx:{} move:{}", err.msg, err.idx, err.mv),
        }
        println!("Final state: {}", state);
    } else {
        let mut search = Searcher::new();
        let reporter = |sol: &Solution| println!("New best score: {}", sol);
        search.report_best_fn = &reporter;
        println!("Final solution: {}", search.search(state));
    }
}
