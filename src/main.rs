use rand::Rng;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

const WIDTH: usize = 9;
const HEIGHT: usize = 9;
const MAX_VALUE: u32 = 9;
const NUM_CELLS: usize = WIDTH * HEIGHT;
const ALL_VALUES: u32 = (1u32 << MAX_VALUE) - 1;
const VALUE_SET: u32 = 1u32 << 31;

const fn is_value_set(value: u32) -> bool {
    value & VALUE_SET != 0
}
const fn get_value(mask: u32) -> u32 {
    32 - (mask & !VALUE_SET).leading_zeros()
}
const fn value_mask(value: u32) -> u32 {
    1u32 << (value - 1)
}
const fn value_count(mask: u32) -> u32 {
    (mask & !VALUE_SET).count_ones()
}

const fn get_row(row: usize) -> [usize; WIDTH] {
    let row_base: usize = row * WIDTH;
    [
        row_base + 0,
        row_base + 1,
        row_base + 2,
        row_base + 3,
        row_base + 4,
        row_base + 5,
        row_base + 6,
        row_base + 7,
        row_base + 8,
    ]
}

const fn get_col(col: usize) -> [usize; WIDTH] {
    [
        0 * WIDTH + col,
        1 * WIDTH + col,
        2 * WIDTH + col,
        3 * WIDTH + col,
        4 * WIDTH + col,
        5 * WIDTH + col,
        6 * WIDTH + col,
        7 * WIDTH + col,
        8 * WIDTH + col,
    ]
}

const fn get_box(box_index: usize) -> [usize; WIDTH] {
    let boxi = box_index / 3;
    let boxj = box_index % 3;
    let box_base = boxi * 3 * WIDTH + boxj * 3;
    [
        box_base + 0 * WIDTH + 0,
        box_base + 0 * WIDTH + 1,
        box_base + 0 * WIDTH + 2,
        box_base + 1 * WIDTH + 0,
        box_base + 1 * WIDTH + 1,
        box_base + 1 * WIDTH + 2,
        box_base + 2 * WIDTH + 0,
        box_base + 2 * WIDTH + 1,
        box_base + 2 * WIDTH + 2,
    ]
}

const HOUSES: [[usize; WIDTH]; 27] = [
    get_row(0),
    get_row(1),
    get_row(2),
    get_row(3),
    get_row(4),
    get_row(5),
    get_row(6),
    get_row(7),
    get_row(8),
    get_col(0),
    get_col(1),
    get_col(2),
    get_col(3),
    get_col(4),
    get_col(5),
    get_col(6),
    get_col(7),
    get_col(8),
    get_box(0),
    get_box(1),
    get_box(2),
    get_box(3),
    get_box(4),
    get_box(5),
    get_box(6),
    get_box(7),
    get_box(8),
];

fn new_board() -> [u32; NUM_CELLS] {
    [ALL_VALUES; NUM_CELLS]
}

fn set_value(board: &mut [u32; NUM_CELLS], cell: usize, value: u32) -> bool {
    let existing_mask: u32 = board[cell];
    let value_mask = value_mask(value);
    if existing_mask & value_mask == 0 {
        return false;
    }
    if existing_mask & VALUE_SET != 0 {
        return true;
    }

    board[cell] = value_mask | VALUE_SET;

    let inv_value_mask = !value_mask;

    let i = cell / WIDTH;
    let j = cell % WIDTH;

    // Eliminate from all the cells in the same column
    for col in 0..WIDTH {
        let cur_cell = i * WIDTH + col;
        if cur_cell != cell {
            board[cur_cell] &= inv_value_mask;
            if board[cur_cell] & !VALUE_SET == 0 {
                return false;
            }
        }
    }

    // Eliminiate from all the cells in the same row
    for row in 0..HEIGHT {
        let cur_cell = row * WIDTH + j;
        if cur_cell != cell {
            board[cur_cell] &= inv_value_mask;
            if board[cur_cell] & !VALUE_SET == 0 {
                return false;
            }
        }
    }

    // Eliminate from all the cells in the same box
    let box_base = (i / 3) * 3 * WIDTH + (j / 3) * 3;
    for boxi in 0..3 {
        for boxj in 0..3 {
            let cur_cell = box_base + (boxi * WIDTH + boxj);
            if cur_cell != cell {
                board[cur_cell] &= inv_value_mask;
                if board[cur_cell] & !VALUE_SET == 0 {
                    return false;
                }
            }
        }
    }
    return true;
}

#[derive(PartialEq, Eq)]
enum LogicResult {
    NONE,
    CHANGED,
    INVALID,
}

fn check_valid(board: &[u32; NUM_CELLS]) -> bool {
    for cell in 0..NUM_CELLS {
        let mask = board[cell];
        if mask == 0 {
            return false;
        }
    }
    for house in HOUSES {
        let mut at_least_once = 0;
        for cell in house {
            at_least_once |= board[cell];
        }
        if value_count(at_least_once & !VALUE_SET) != MAX_VALUE {
            return false;
        }
    }

    return true;
}

fn set_naked_single(board: &mut [u32; NUM_CELLS]) -> LogicResult {
    for cell in 0..NUM_CELLS {
        let mask = board[cell];
        if mask == 0 {
            return LogicResult::INVALID;
        }

        if !is_value_set(mask) && value_count(mask) == 1 {
            let value = get_value(mask);
            if !set_value(board, cell, value) {
                return LogicResult::INVALID;
            }
            return LogicResult::CHANGED;
        }
    }
    return LogicResult::NONE;
}

fn set_hidden_single(board: &mut [u32; NUM_CELLS]) -> LogicResult {
    for house in HOUSES {
        let mut value_set_mask = 0;
        let mut at_least_once = 0;
        let mut more_than_once = 0;
        for cell in house {
            let mask = board[cell];
            if is_value_set(mask) {
                value_set_mask |= mask;
            } else {
                more_than_once |= mask & at_least_once;
                at_least_once |= mask;
            }
        }
        value_set_mask &= !VALUE_SET;

        let values_present = at_least_once | value_set_mask;
        if value_count(values_present) != MAX_VALUE {
            return LogicResult::INVALID;
        }

        let exactly_once = at_least_once & !more_than_once;
        if exactly_once != 0 {
            for cell in house {
                let mask = board[cell];
                let once_mask = mask & exactly_once;
                if once_mask != 0 {
                    if value_count(once_mask) > 1 {
                        return LogicResult::INVALID;
                    }

                    let once_value = get_value(once_mask);
                    if !set_value(board, cell, once_value) {
                        return LogicResult::INVALID;
                    }
                    return LogicResult::CHANGED;
                }
            }
        }
    }

    return LogicResult::NONE;
}

fn set_single(board: &mut [u32; NUM_CELLS]) -> LogicResult {
    let naked_result = set_naked_single(board);
    if naked_result != LogicResult::NONE {
        return naked_result;
    }
    return set_hidden_single(board);
}

fn print_board(output: &mut File, board: &[u32; NUM_CELLS], num_solutions: usize) -> () {
    let mut board_chars: Vec<u8> = Vec::with_capacity(NUM_CELLS);
    for cell in 0..NUM_CELLS {
        let mask = board[cell];
        if mask & VALUE_SET == 0 || mask == 0 {
            board_chars.push('.' as u8);
        } else {
            let value = get_value(mask);
            board_chars.push(value as u8 + '0' as u8);
        }
    }
    let line = String::from_utf8(board_chars).unwrap();
    writeln!(output, "{},{}", line, num_solutions).unwrap();
}

fn unset_cells(board: [u32; NUM_CELLS]) -> Vec<usize> {
    let mut cells: Vec<usize> = Vec::with_capacity(NUM_CELLS);
    for cell in 0..NUM_CELLS {
        let value = board[cell];
        if value & VALUE_SET == 0 && value != 0 {
            cells.push(cell);
        }
    }
    return cells;
}

struct BoardInfo {
    board: [u32; NUM_CELLS],
    given_count: usize,
}

impl BoardInfo {
    fn new_blank() -> BoardInfo {
        let board = new_board();
        let given_count = 0;
        return BoardInfo { board, given_count };
    }

    fn new(board: [u32; NUM_CELLS], given_count: usize) -> BoardInfo {
        BoardInfo { board, given_count }
    }
}

fn best_cell(board: &[u32; NUM_CELLS]) -> usize {
    let mut best_cell = NUM_CELLS;
    let mut best_count = MAX_VALUE + 1;
    for cell in 0..NUM_CELLS {
        let mask = board[cell];
        if is_value_set(mask) {
            continue;
        }

        let count = value_count(mask);
        if count == 2 {
            return cell;
        }
        if count < best_count {
            best_cell = cell;
            best_count = count;
        }
    }
    return best_cell;
}

fn count_solutions(board: &[u32; NUM_CELLS], max_solutions: usize) -> usize {
    let mut board_stack: Vec<[u32; NUM_CELLS]> = Vec::new();
    board_stack.push(*board);
    let mut solutions = 0;
    while solutions < max_solutions && !board_stack.is_empty() {
        let mut board = board_stack.pop().unwrap();
        let mut result : LogicResult;
        loop {
            result = set_single(&mut board);
            if result != LogicResult::CHANGED {
                break;
            }
        }
        if result == LogicResult::INVALID {
            continue;
        }

        let cell_index = best_cell(&board);
        if cell_index == NUM_CELLS {
            solutions += 1;
        } else {
            let candidate_value = get_value(board[cell_index]);
            let candidate_mask = value_mask(candidate_value);

            let mut backtrack_board = board.clone();
            backtrack_board[cell_index] &= !candidate_mask;
            if backtrack_board[cell_index] != 0 {
                board_stack.push(backtrack_board);
            }
            if set_value(&mut board, cell_index, candidate_value) && check_valid(&board)
            {
                board_stack.push(board);
            }
        }
    }

    return solutions;
}

fn main() {
    let mut rng = rand::thread_rng();
    let start_time = Instant::now();
    let path = "randomboards.txt";
    let mut output = File::create(path).unwrap();
    let mut board_stack: Vec<BoardInfo> = Vec::new();

    for num_givens in 1..NUM_CELLS + 1 {
        println!("Generating boards with {} givens...", num_givens);
        let mut num_invalid = 0;
        let mut num_unique = 0;
        let mut num_multi = 0;
        for _board_num in 0..100000 {
            board_stack.clear();
            board_stack.push(BoardInfo::new_blank());

            let mut num_backtracks = 0;
            let mut given_count = 0;
            while given_count < num_givens {
                if board_stack.len() == 0 {
                    println!("Impossible!");
                    return;
                }

                let board_info = board_stack.pop().unwrap();
                let mut board = board_info.board;
                given_count = board_info.given_count;

                match set_single(&mut board) {
                    LogicResult::NONE => {
                        let unset_cells = unset_cells(board);
                        if unset_cells.len() == 0 {
                            continue;
                        }
                        let cell_index = unset_cells[rng.gen_range(0..unset_cells.len())];
                        let mask = board[cell_index];
                        let num_candidates = value_count(mask);
                        let mut candidate_index = rng.gen_range(0..num_candidates);
                        let mut candidate_value = 0;
                        for candidate in 1..MAX_VALUE + 1 {
                            if value_mask(candidate) & mask != 0 {
                                if candidate_index == 0 {
                                    candidate_value = candidate;
                                    break;
                                }
                                candidate_index -= 1;
                            }
                        }
                        let candidate_mask = value_mask(candidate_value);

                        let mut backtrack_board = board.clone();
                        backtrack_board[cell_index] &= !candidate_mask;
                        if backtrack_board[cell_index] != 0 && check_valid(&backtrack_board) {
                            board_stack.push(BoardInfo::new(backtrack_board, given_count));
                        }

                        if set_value(&mut board, cell_index, candidate_value) && check_valid(&board)
                        {
                            given_count += 1;
                            board_stack.push(BoardInfo::new(board, given_count));
                        } else {
                            num_backtracks += 1;
                            if num_backtracks > 100 {
                                board_stack.clear();
                                board_stack.push(BoardInfo::new_blank());
                                num_backtracks = 0;
                            }
                        }
                    }
                    LogicResult::CHANGED => {
                        given_count += 1;
                        board_stack.push(BoardInfo::new(board, given_count));

                        let mut real_given_count = 0;
                        for cell in 0..NUM_CELLS {
                            if board[cell] & VALUE_SET != 0 {
                                real_given_count += 1;
                            }
                        }
                        if real_given_count != given_count {
                            println!(
                                "Given count does not match {} vs {}",
                                real_given_count, given_count
                            );
                        }
                    }
                    LogicResult::INVALID => {
                        continue;
                    }
                }
            }

            let board = board_stack.pop().unwrap().board;
            let num_solutions = count_solutions(&board, 2);
            print_board(&mut output, &board, num_solutions);

            if num_solutions == 0{
                num_invalid += 1;
            } else if num_solutions == 1 {
                num_unique += 1;
            } else {
                num_multi += 1;
            }
        }

        println!("{}, {}, {}", num_invalid, num_unique, num_multi);
    }

    println!("Took {:?}", Instant::now() - start_time);
}
