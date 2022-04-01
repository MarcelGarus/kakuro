use crate::game::{Input, Output};

pub fn solve(input: &Input) -> Output {
    let mut solution_attempt = Vec::new();
    for _ in 0..input.num_boxes {
        solution_attempt.push(1);
    }

    let mut solutions = vec![];
    'search: loop {
        // Check if this is a solution.
        if input.is_solution(&solution_attempt) {
            solutions.push(solution_attempt.clone());
        }

        // Change the solution attempt by incrementing it if interpreted as a
        // binary number.
        let mut i = solution_attempt.len() - 1;
        loop {
            solution_attempt[i] += 1;
            if solution_attempt[i] == 10 {
                solution_attempt[i] = 0;
                if i == 0 {
                    break 'search;
                }
                i -= 1;
            } else {
                break;
            }
        }
    }
    solutions
}
