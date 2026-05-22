use clap::Parser;
use nirmana_solitaire::*;
use std::time::Instant;

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

    /// If present, indicates a score hint. A proper hint makes searching faster.
    /// An incorrectly large hint will cause no results to be found.
    #[arg(short, long)]
    hint: Option<i32>,
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
        if let Some(x) = args.hint {
            search.set_hint(x);
        }
        let mut start_time = Instant::now();
        let sol1 = search.search(state);
        let mut elapsed = Instant::now().duration_since(start_time).as_secs_f64();
        println!("Final solution: {} took {:.3}s", sol1, elapsed);
        start_time = Instant::now();
        let sol2 = search.search(state);
        elapsed = Instant::now().duration_since(start_time).as_secs_f64();
        println!("Re-solve: {} took {:.3}s", sol2, elapsed);

        if sol1.moves().len() > 0 {
            let score = state.play_moves(&sol1.moves()[..sol1.moves().len()-1]).unwrap();
            println!("Penultimate state {:?} {} gets score {}", state, state, score);
        }
    }
}
