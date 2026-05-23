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
        let mut hint = match args.hint {
            Some(x) => x,
            None => 147,
        };
        let overall_start_time = Instant::now();
        loop {
            search.set_hint(hint);
            let start_time = Instant::now();
            let sol = search.search(state);
            let after = Instant::now();
            let elapsed = after.duration_since(start_time).as_secs_f64();
            let overall = after.duration_since(overall_start_time).as_secs_f64();
            println!(
                "Search with window {} took {:.3}s {} states",
                hint,
                elapsed,
                search.searched_states()
            );
            if sol.moves().len() > 0 {
                println!(
                    "│Mv│Scr│{:>48}│ overall time {:.3}s",
                    "Board before move", overall
                );
                println!("├──┼───┼────────────────────────────────────────────────┤");
                let mut pos = state;
                let mut score = 0;
                let mut board = [0u8; 12];
                for mv in sol.moves() {
                    pos.into_slice(&mut board);
                    println!("│{:2}│{:3}│{:2?}│", mv, score, board);
                    score += pos.play((*mv).into())
                }
                pos.into_slice(&mut board);
                println!("│ -│{:3}│{:2?}│", score, board);
                break;
            }
            if hint < sum.into() {
                println!("No solution found! overall time {:.3}s", overall);
                break;
            }
            // Empirically, the size of the space searched increases by ~an order of magnitude every
            // six stones. Because the parity of score equals the parity of stone count, we only
            // need to consider odd hints (for even scores). The three hints in each group here
            // cluster at approximately the same size/speed - it's possible to make theoretical
            // arguments about why, but ultimately it's an empiracal relation. We test only the last
            // one to avoid duplicate work.
            hint -= 6;
        }
    }
}
