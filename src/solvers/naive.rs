use crate::game::{Input, Output};

pub fn solve(input: &Input) -> Output {
    let mut attempt = vec![1; input.num_cells];
    let mut solutions = vec![];
    'search: loop {
        // println!(
        //     "Evaluating attempt {}",
        //     attempt
        //         .iter()
        //         .map(|number| format!("{}", number))
        //         .collect::<String>()
        // );

        // Check if this is a solution.
        if input.is_solution(&attempt) {
            solutions.push(attempt.clone());
        }

        // Change the solution attempt by incrementing it if interpreted as a
        // binary number.
        let mut i = attempt.len() - 1;
        loop {
            attempt[i] += 1;
            if attempt[i] == 10 {
                attempt[i] = 0;
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
