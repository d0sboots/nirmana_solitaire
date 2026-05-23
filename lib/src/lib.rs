#![no_std]

#[cfg(feature = "std")]
extern crate std;

use core::fmt;

/// The state of a game, compressed into 8 bytes for memory efficiency.
/// Out of the 12 possible rotations, the lexiographically smallest is always the one used, by
/// calling #unique().
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct State {
    /// The state is stored as 12 unsigned 5-bit fields, which takes 60 bits.
    /// The high 4 bits are unused and should always be 0.
    /// On game start, the highest number a bowl can contain is 8, and the sum of all bowls is 50.
    /// You can make at most two moves before emptying a bowl, and the smallest empty amount is 2.
    /// Thus, in theory you could increment a bowl twice for every two stones removed. (In really
    /// extreme cases, you can increment more than once in a single move, but this is not
    /// sustainable.)
    /// `(50 - 8) / 2 = 21`, `21 + 8 = 29`, so even in the most extreme conceivable scenario 5 bits is enough.
    v: u64,
}

#[derive(Debug)]
pub struct PlayMovesErr {
    pub msg: &'static str,
    /// The failing move number
    pub mv: usize,
    /// The position of the failing move (0-based)
    pub idx: usize,
}

impl State {
    /// Return the score for playing this move, or -1 if it is not legal (no stones)
    #[inline]
    pub fn play(&mut self, mv: usize) -> i32 {
        let mut idx = mv * 5;
        let mut stones = (self.v >> idx) & 31;
        if stones == 0 {
            return -1;
        }
        let mut v = self.v ^ (stones << idx);
        // We use SIMD techniques to increment all the bowls at once.
        if stones >= 12 {
            stones -= 12;
            v += 0x08421_08421_08421;
            if stones >= 12 {
                // Stones being 24 or more doesn't happen, but just in case...
                stones -= 12;
                v += 0x08421_08421_08421;
            }
        }
        let s5 = stones * 5;
        let inc = 0x08421_08421_08421 >> (60 - s5);
        // The low bit of inc starts 5 bits to the left of where we got stones, since
        // we start adding in the next bowl over.
        let rotated = (inc << (idx + 5)) & !(0xf << 60) | (inc >> (55 - idx));
        v += rotated;
        idx += s5 as usize;
        if idx >= 60 {
            idx -= 60;
        }
        let mut bucket = ((v >> idx) & 31) as i32;
        if bucket < 2 || bucket > 3 {
            self.v = v;
            return 0;
        }
        let mut score = bucket;
        loop {
            v ^= (bucket as u64) << idx;
            if idx == 0 {
                idx = 60;
            }
            idx -= 5;
            bucket = ((v >> idx) & 31) as i32;
            if bucket < 2 || bucket > 3 {
                self.v = v;
                return score;
            }
            score += bucket * bucket;
        }
    }

    /// Play a sequence of moves, returning a descriptive error and halting on illegal moves
    pub fn play_moves(&mut self, moves: &[u8]) -> Result<i32, PlayMovesErr> {
        let mut danger = false;
        let mut score = 0;
        for (i, mv) in moves.into_iter().enumerate() {
            let act = self.play((*mv).into());
            if act < 0 {
                return Err(PlayMovesErr {
                    msg: "Invalid move/no stones in bowl",
                    mv: (*mv).into(),
                    idx: i,
                });
            }
            if act == 0 {
                if danger {
                    return Err(PlayMovesErr {
                        msg: "Move #{}: 2nd move in a row with no capture (starting from {})",
                        mv: (*mv).into(),
                        idx: i,
                    });
                }
                danger = true;
                continue;
            }
            danger = false;
            score += act;
        }
        Ok(score)
    }

    pub fn into_slice(self, slice: &mut [u8]) {
        let mut x = self.v;
        for i in 0..12 {
            slice[i] = (x & 31) as u8;
            x >>= 5;
        }
    }

    pub fn stones(self) -> u8 {
        let mut r = (self.v & 0x07c1f_07c1f_07c1f) + ((self.v & 0xf83e0_f83e0_f83e0) >> 5);
        r += r >> 10;
        // If state were arbitrary, the sum could overflow a u8. However, we don't allow this,
        // so we limit the result to 8 bits to improve codegen down the line.
        ((r + (r >> 20) + (r >> 40)) & 0xff) as u8
    }
}

impl fmt::Debug for State {
    /// Custom format that return much-more useful hex
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "State {{ {:#x} }}", self.v)
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tmp = [0u8; 12];
        self.into_slice(&mut tmp);
        write!(f, "State {:?}", tmp)
    }
}

impl TryFrom<&[u8]> for State {
    type Error = (&'static str, usize);

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 12 {
            return Err(("Input must be exactly size 12!", value.len()));
        }
        let mut v = 0u64;
        for i in (0..12).rev() {
            let x = value[i];
            if x > 31 {
                return Err(("Input value > 31!", x.into()));
            }
            v <<= 5;
            v |= u64::from(x);
        }
        Ok(State { v })
    }
}

#[cfg(feature = "std")]
impl From<State> for std::vec::Vec<u8> {
    fn from(value: State) -> Self {
        let mut v = std::vec::Vec::with_capacity(12);
        v.resize(12, 0);
        value.into_slice(v.as_mut_slice());
        v
    }
}

#[derive(Debug, Clone)]
pub struct Solution {
    pub score: i32,
    pub num_moves: u8,
    moves_storage: [u8; 50],
}

impl Solution {
    pub fn new() -> Self {
        Self {
            score: 0,
            num_moves: 0,
            moves_storage: [0; 50],
        }
    }

    pub fn moves(&self) -> &[u8] {
        &self.moves_storage[..self.num_moves.into()]
    }

    pub fn add_move(&mut self, score: i32, mv: u8) {
        self.score += score;
        self.moves_storage[usize::from(self.num_moves)] = mv;
        self.num_moves += 1;
    }

    pub fn pop_move(&mut self, score: i32) {
        self.score -= score;
        self.num_moves -= 1;
    }
}

impl fmt::Display for Solution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Solution")
            .field("score", &self.score)
            .field("moves", &self.moves())
            .finish()
    }
}

pub struct Searcher<'a> {
    /// Reporting function. This is called with a reference to best whenever
    /// best is updated.
    pub report_best_fn: &'a dyn Fn(&Solution) -> (),

    states: u64,
    sol: Solution,
    best: Solution,
}

// Offset applied to scores in max_score_table, and undone in sol.score. See the comment in max_score_table.
const SCORE_OFFSET: u8 = 3;

// We only need through 50, but we pad to 56 for 8-byte alignment.
const fn max_score_table() -> [u8; 56] {
    let mut result = [0u8; 56];
    // We have to look at how many stones are left over in the initial group, and we assume the
    // rest can make groups of 3 for 9 points each.
    // If we have 2 stones left over, the score will be 2.
    // If we have 3 stones left over, the score will be 3.
    // If we have 4 stones left over, the score will be 6. (We have to score 2 stones in the backtrack phase.)
    //
    // We want the table to be one byte each, but there are range issues: The smallest value is
    // negative, and the largest value is 159. To fit this, we add an offset and use a u8. The
    // offset is subtracted in the single place the table is used.
    result[0] = SCORE_OFFSET;
    result[1] = SCORE_OFFSET - 3;
    result[2] = SCORE_OFFSET + 2;
    result[3] = SCORE_OFFSET + 3;
    let mut i = 4;
    while i < 56 {
        result[i] = result[i - 3] + 9;
        i += 1
    }
    result
}

impl Searcher<'_> {
    pub fn new() -> Self {
        static NOOP: fn(&Solution) = |_| ();
        Self {
            report_best_fn: &NOOP,
            states: 0,
            sol: Solution::new(),
            best: Solution::new(),
        }
    }

    pub fn search(&mut self, state: State) -> Solution {
        self.states = 0;
        self.sol.score = -i32::from(SCORE_OFFSET);
        self.search_impl(state);
        // sol should be returned to default by the end of the search
        assert!(self.sol.num_moves == 0);
        assert!(self.sol.score == -i32::from(SCORE_OFFSET));
        self.best.clone()
    }

    /// Set a window hint for the search. The searcher will only find solutions with a score larger
    /// than the value. Setting this too large will cause the searcher to return no solutions,
    /// generally quickly. Setting it to 0 takes a bit longer than a well-tuned guess.
    pub fn set_hint(&mut self, hint: i32) {
        self.best.score = hint;
    }

    /// The number of states the last search searched
    pub fn searched_states(&self) -> u64 {
        self.states
    }

    // This uses branch-and-bound with a depth-first search to efficiently find an optimal solution.
    // We have an absolute (and cheap to calculate) upper-bound on the score at any position based
    // solely on the number of stones, since you can't do better than 9 points per 3 stones. This
    // could be used as a consistent heuristic for A*, which provably explores the least number of
    // nodes in the tree. However, since A* requires a priority queue of to-be-expanded states, the
    // stack-based nature of the DFS here works out better - expanding slightly more nodes is more
    // than repaid by doing the work faster.
    // In the same vein, there are practical speedups that are possible by memoizing values at the
    // bottom of the tree, where many possible lines converge to the same states. But by avoiding
    // the need for a hashmap, it makes it much easier to parallelize the search.
    fn search_impl(&mut self, pos: State) -> i32 {
        self.states += 1;
        let stones = pos.stones();
        if stones == 0 {
            // We've completed the game. Due to max check below, this case doesn't trigger as often
            // as it might seem. We have to check if this makes a new best, in case it needs to be
            // reported.
            if self.sol.score + i32::from(SCORE_OFFSET) > self.best.score {
                self.report_sol();
            }
            return 0;
        }
        // We fold the adjustment of SCORE_OFFSET into sol.score here, since it is executed for
        // every search node.
        const TABLE: [u8; 56] = max_score_table();
        if i32::from(TABLE[usize::from(stones)]) + self.sol.score <= self.best.score {
            return -1;
        }
        let mut local_score = -1;
        for i in 0..12 {
            let mut p1 = pos;
            let res1 = p1.play(i);
            if res1 < 0 {
                continue; // No stones in bowl
            }
            self.sol.add_move(res1, i as u8);
            if res1 > 0 {
                let value = self.search_impl(p1);
                if value >= 0 && value + res1 > local_score {
                    local_score = value + res1;
                }
            } else {
                // In danger, do another move
                for j in 0..12 {
                    let mut p2 = p1;
                    let res2 = p2.play(j);
                    if res2 <= 0 {
                        continue; // No stones in bowl, or no capture
                    }
                    self.sol.add_move(res2, j as u8);
                    let value = self.search_impl(p2);
                    if value >= 0 && value + res2 > local_score {
                        local_score = value + res2;
                    }
                    self.sol.pop_move(res2);
                }
            }
            self.sol.pop_move(res1);
        }
        local_score
    }

    #[cold]
    fn report_sol(&mut self) {
        // Report a new best solution. To do this, we must reconstruct the Solution
        // entries for the parts we got from the cache. This is a bit expensive, but
        // fortunately improving our max score can happen very few times.
        self.best = self.sol.clone();
        self.best.score += i32::from(SCORE_OFFSET);
        (self.report_best_fn)(&self.best);
    }
}
