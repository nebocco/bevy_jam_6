use anyhow::Error;
use good_lp::{
    Expression, IntoAffineExpression, ProblemVariables, Solution, SolutionStatus, SolverModel,
    Variable, default_solver, variable::variable, variables,
};
use indicatif::{MultiProgress, ProgressIterator};
use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn main() {
    // run_solver();
    compare_two_solver();
}

fn run_solver() {
    let seed = 42; // Example seed, can be any u64 value
    let mut generator = Generator::new(seed);

    let width = 8; // Example width
    let height = 8; // Example height

    for i in 0..10 {
        let Ok(level) = generator.generate_random_level(width, height) else {
            println!("Failed to generate level");
            continue;
        };

        println!("# Level {i}");
        display_level(&level);

        let lp_solver = LpSolver;
        let Ok(solution) = lp_solver.solve_minimal_bombs(&level) else {
            println!("Failed to solve level");
            continue;
        };

        println!("Solution found:");
        println!("-  Bombs placed: {:?}", solution.bombs);
        display_solution(&solution);
    }
}

fn compare_two_solver() {
    let seed = 42; // Example seed, can be any u64 value
    let mut generator = Generator::new(seed);

    const ITER_COUNT: usize = 50;

    let m = MultiProgress::new();
    let pb_outer = m.add(indicatif::ProgressBar::new(4));

    for aside in 5..8 {
        let pb_inner = m.add(indicatif::ProgressBar::new(ITER_COUNT as u64));
        pb_outer.inc(1);
        for i in 0..ITER_COUNT {
            pb_inner.inc(1);
            let Ok(level) = generator.generate_random_level(aside, aside) else {
                println!("Failed to generate level");
                continue;
            };

            let lp_solver = LpSolver;
            let Ok(solution1) = lp_solver.solve_minimal_bombs(&level) else {
                println!("Failed to solve level");
                continue;
            };

            let Ok(solution2) = lp_solver.solve_minimal_affected_areas(&level) else {
                println!("Failed to solve level");
                continue;
            };

            if solution1.bombs.len() >= solution2.bombs.len() {
                println!(
                    "Both solutions have the same number of bombs: {}",
                    solution1.bombs.len()
                );
                continue;
            }

            if solution1.count_affected_cells() <= solution2.count_affected_cells() {
                println!(
                    "Both solutions have the same number of affected areas: {}",
                    solution1.count_affected_cells()
                );
                // continue;
            } else {
                println!("*** DIFFERENT SOLUTIONS FOUND ***");
            }

            println!("# Level {i}");
            display_level(&level);

            println!("Min Bombs Solution:");
            println!(
                "Bombs: {}, Affected Areas: {}",
                solution1.bombs.len(),
                solution1.count_affected_cells()
            );
            display_solution(&solution1);

            println!("Min Affected Areas Solution:");
            println!(
                "Bombs: {}, Affected Areas: {}",
                solution2.bombs.len(),
                solution2.count_affected_cells()
            );
            display_solution(&solution2);

            let level_str = ron::ser::to_string(&level).unwrap();
            println!("Level RON: {}", level_str);
        }
        pb_inner.finish();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum Item {
    BombSmall,
    BombMedium,
    BombLarge,
    BombHorizontal,
    BombVertical,
    Null,
    Rock,
    Jewel,
    Eraser,
    Enemy,
}

impl Item {
    pub fn is_bomb(&self) -> bool {
        matches!(
            self,
            Item::BombSmall
                | Item::BombMedium
                | Item::BombLarge
                | Item::BombHorizontal
                | Item::BombVertical
        )
    }

    pub const fn to_sprite_index(self) -> usize {
        match self {
            Item::BombSmall => 0,
            Item::BombMedium => 1,
            Item::BombLarge => 2,
            Item::BombHorizontal => 3,
            Item::BombVertical => 4,
            Item::Null => 7,
            Item::Rock => 8,
            Item::Jewel => 10,
            Item::Enemy => 11,
            Item::Eraser => 12,
        }
    }
}

impl Item {
    pub fn impact_zone(&self) -> &'static [(i8, i8)] {
        match self {
            // . . . . .
            // . x x x .
            // . x # x .
            // . x x x.
            // . . . . .
            Item::BombSmall => &[
                (-1, 1),
                (0, 1),
                (1, 1),
                (-1, 0),
                (0, 0),
                (1, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
            ],

            // . . x . .
            // . x x x .
            // x x # x x
            // . x x x .
            // . . x . .
            Item::BombMedium => &[
                (0, 2),
                (-1, 1),
                (0, 1),
                (1, 1),
                (-2, 0),
                (-1, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (-1, -1),
                (0, -1),
                (1, -1),
                (0, -2),
            ],

            // . . . x . . .
            // . . x x x . .
            // . x x x x x .
            // x x x # x x x
            // . x x x x x .
            // . . x x x . .
            // . . . x . . .
            Item::BombLarge => &[
                (0, 3),
                (-1, 2),
                (0, 2),
                (1, 2),
                (-2, 1),
                (-1, 1),
                (0, 1),
                (1, 1),
                (2, 1),
                (-3, 0),
                (-2, 0),
                (-1, 0),
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (-2, -1),
                (-1, -1),
                (0, -1),
                (1, -1),
                (2, -1),
                (-1, -2),
                (0, -2),
                (1, -2),
                (0, -3),
            ],

            // . . x . .
            // . . x . .
            // . . # . .
            // . . x . .
            // . . x . .
            Item::BombVertical => &[
                (0, 10),
                (0, 9),
                (0, 8),
                (0, 7),
                (0, 6),
                (0, 5),
                (0, 4),
                (0, 3),
                (0, 2),
                (0, 1),
                (0, 0),
                (0, -1),
                (0, -2),
                (0, -3),
                (0, -4),
                (0, -5),
                (0, -6),
                (0, -7),
                (0, -8),
                (0, -9),
                (0, -10),
            ],

            // . . . . .
            // . . . . .
            // x x # x x
            // . . . . .
            // . . . . .
            Item::BombHorizontal => &[
                (10, 0),
                (9, 0),
                (8, 0),
                (7, 0),
                (6, 0),
                (5, 0),
                (4, 0),
                (3, 0),
                (2, 0),
                (1, 0),
                (0, 0),
                (-1, 0),
                (-2, 0),
                (-3, 0),
                (-4, 0),
                (-5, 0),
                (-6, 0),
                (-7, 0),
                (-8, 0),
                (-9, 0),
                (-10, 0),
            ],

            Item::Eraser => &[(0, 0)],

            Item::Rock | Item::Jewel | Item::Enemy | Item::Null => &[],
        }
    }
}

impl From<u8> for Item {
    fn from(value: u8) -> Self {
        match value {
            0 => Item::BombSmall,
            1 => Item::BombMedium,
            2 => Item::BombLarge,
            3 => Item::BombHorizontal,
            4 => Item::BombVertical,
            255 => Item::Eraser,
            _ => panic!("Invalid item index"),
        }
    }
}

struct Generator {
    seed: u64,
    rng: ChaCha20Rng,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Level {
    width: usize,
    height: usize,
    objects: HashMap<(usize, usize), Item>,
    fire: (usize, usize),
}

impl Generator {
    fn new(seed: u64) -> Self {
        Self {
            seed,
            rng: ChaCha20Rng::seed_from_u64(seed),
        }
    }

    fn generate_random_level(&mut self, width: usize, height: usize) -> Result<Level, ()> {
        let mut objects = HashMap::new();

        let object_num = ((width * height) / 8 + self.rng.random_range(0..5))
            .saturating_sub(self.rng.random_range(0..5))
            .clamp(4, 12);
        self.genrate_objects(&mut objects, object_num, (width, height));

        let bomb_num = self.rng.random_range(1..4);
        self.generate_bombs(&mut objects, bomb_num, (width, height));

        let bomb_positions: Vec<_> = objects
            .iter()
            .filter(|(_, item)| item.is_bomb())
            .map(|(&pos, _)| pos)
            .collect();

        if bomb_positions.is_empty() {
            return Err(());
        }

        let &fire = bomb_positions.choose(&mut self.rng).unwrap();

        Ok(Level {
            width,
            height,
            objects,
            fire,
        })
    }

    fn genrate_objects(
        &mut self,
        objects: &mut HashMap<(usize, usize), Item>,
        object_num: usize,
        (width, height): (usize, usize),
    ) {
        for _ in 0..object_num {
            let positions_candidate: Vec<_> = (0..width)
                .flat_map(|x| (0..height).map(move |y| (x, y)))
                .filter(|pos| !objects.contains_key(pos))
                .collect();

            let &(x, y) = positions_candidate.choose(&mut self.rng).unwrap();

            let item = if self.rng.random_bool(0.6) {
                Item::Rock
            } else {
                Item::Jewel
            };

            objects.insert((x, y), item);
        }
    }

    fn generate_bombs(
        &mut self,
        objects: &mut HashMap<(usize, usize), Item>,
        bomb_num: usize,
        (width, height): (usize, usize),
    ) {
        for _ in 0..bomb_num {
            for _retry_count in 0..100 {
                let item = match self.rng.random_range(0..7) {
                    0..3 => Item::BombSmall,
                    3..5 => Item::BombMedium,
                    5 => Item::BombHorizontal,
                    _ => Item::BombVertical,
                };

                let possible_positions = (0..width)
                    .flat_map(|x| (0..height).map(move |y| (x, y)))
                    .filter(|pos| check_possibly_placed(pos, item, objects))
                    .collect::<Vec<_>>();

                if possible_positions.is_empty() {
                    continue;
                } else {
                    let &(x, y) = possible_positions.choose(&mut self.rng).unwrap();
                    objects.insert((x, y), item);
                    break;
                }
            }
        }
    }
}

struct LpSolver;

struct Variables {
    is_placed: HashMap<(usize, usize, usize, Item), Variable>,
    is_affected: HashMap<(usize, usize), Variable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LevelSolution {
    level: Level,
    bombs: HashMap<(usize, usize), Item>,
    is_affected: Vec<Vec<bool>>,
}

impl LevelSolution {
    fn count_affected_cells(&self) -> usize {
        self.is_affected.iter().flatten().filter(|&&x| x).count()
    }
}

impl LpSolver {
    const MAX_DEPTH: usize = 10; // Maximum depth for the search

    const BOMBS: [Item; 4] = [
        Item::BombSmall,
        Item::BombMedium,
        Item::BombHorizontal,
        Item::BombVertical,
    ];

    fn solve_minimal_bombs(&self, level: &Level) -> Result<LevelSolution, Error> {
        let (vars, variables) = self.define_variables(level);

        let objective = self.define_objective_function_minimum_bombs(level, &variables);

        let mut problem = vars.minimise(objective).using(default_solver);
        problem.set_parameter("log", "0");

        self.define_constraints(&mut problem, level, &variables);

        let solution = problem.solve()?;

        match solution.status() {
            SolutionStatus::Optimal => self.build_solution(&solution, level, &variables),
            _ => Err(Error::msg("No feasible solution found")),
        }
    }

    fn solve_minimal_affected_areas(&self, level: &Level) -> Result<LevelSolution, Error> {
        let (vars, variables) = self.define_variables(level);

        let objective = self.define_objective_function_affected_areas(level, &variables);

        let mut problem = vars.minimise(objective).using(default_solver);
        problem.set_parameter("log", "0");

        self.define_constraints(&mut problem, level, &variables);

        let solution = problem.solve()?;

        match solution.status() {
            SolutionStatus::Optimal => self.build_solution(&solution, level, &variables),
            _ => Err(Error::msg("No feasible solution found")),
        }
    }

    fn define_variables(&self, level: &Level) -> (ProblemVariables, Variables) {
        // Define variables for the LP solver
        let mut vars = variables!();

        let mut is_placed = HashMap::new();
        for x in 0..level.width {
            for y in 0..level.height {
                for depth in 0..Self::MAX_DEPTH {
                    // Create a variable for each bomb type at each position
                    // The depth is not used in this example, but can be used for more complex scenarios
                    for item in Self::BOMBS {
                        is_placed.insert(
                            (x, y, depth, item),
                            vars.add(
                                variable()
                                    .binary()
                                    .name(format!("is_placed_{}_{}_{}_{:?}", x, y, depth, item)),
                            ),
                        );
                    }
                }
            }
        }

        let mut is_affected = HashMap::new();
        for x in 0..level.width {
            for y in 0..level.height {
                is_affected.insert(
                    (x, y),
                    vars.add(variable().binary().name(format!("is_affected_{}_{}", x, y))),
                );
            }
        }

        (
            vars,
            Variables {
                is_placed,
                is_affected,
            },
        )
    }

    fn define_constraints<S: SolverModel>(
        &self,
        problem: &mut S,
        level: &Level,
        variables: &Variables,
    ) {
        // in each cell, at most one bomb can be placed
        for (x, y) in (0..level.width).flat_map(|x| (0..level.height).map(move |y| (x, y))) {
            let lhs = Self::BOMBS
                .iter()
                .flat_map(|&item| {
                    (0..Self::MAX_DEPTH).map(move |depth| variables.is_placed[&(x, y, depth, item)])
                })
                .fold(Expression::default(), |acc, var| acc + var);
            problem.add_constraint(
                lhs.leq(1.0.into_expression())
                    .set_name(format!("cell_{}_{}", x, y)),
            );
        }

        // already placed bombs
        for (&pos, &item) in &level.objects {
            if item.is_bomb() {
                if pos == level.fire {
                    problem.add_constraint(
                        variables.is_placed[&(pos.0, pos.1, 0, item)]
                            .into_expression()
                            .eq(1.0.into_expression())
                            .set_name(format!("placed_bomb_{}_{}", pos.0, pos.1)),
                    );
                } else {
                    problem.add_constraint(
                        (1..Self::MAX_DEPTH)
                            .fold(Expression::default(), |acc, depth| {
                                acc + variables.is_placed[&(pos.0, pos.1, depth, item)]
                            })
                            .eq(1.0.into_expression())
                            .set_name(format!("placed_bomb_at_{}_{}", pos.0, pos.1)),
                    );
                };
            }
        }

        // already placed objects
        for (pos, &item) in &level.objects {
            if !item.is_bomb() {
                problem.add_constraint(
                    Self::BOMBS
                        .iter()
                        .flat_map(|&bomb| {
                            (0..Self::MAX_DEPTH)
                                .map(move |depth| variables.is_placed[&(pos.0, pos.1, depth, bomb)])
                        })
                        .fold(Expression::default(), |acc, var| acc + var)
                        .eq(0.0.into_expression())
                        .set_name(format!("no_bomb_at_{}_{}", pos.0, pos.1)),
                );
            }
        }

        // bombs affect their impact zones
        // lower bound
        for (x, y) in (0..level.width).flat_map(|x| (0..level.height).map(move |y| (x, y))) {
            for &item in &Self::BOMBS {
                let bomb_var = (0..Self::MAX_DEPTH).fold(Expression::default(), |acc, depth| {
                    acc + variables.is_placed[&(x, y, depth, item)]
                });

                for &(dx, dy) in item.impact_zone() {
                    let affected_x = (x as i8 + dx) as usize;
                    let affected_y = (y as i8 + dy) as usize;
                    if let Some(&affected_var) =
                        variables.is_affected.get(&(affected_x, affected_y))
                    {
                        // If the bomb is placed, the affected cell must be affected
                        problem.add_constraint(
                            affected_var
                                .into_expression()
                                .geq(bomb_var.clone())
                                .set_name(format!(
                                    "{:?}_{}_{}_affects_{}_{}",
                                    item, x, y, affected_x, affected_y
                                )),
                        );
                    }
                }
            }
        }

        // upper bound
        for (x, y) in (0..level.width).flat_map(|x| (0..level.height).map(move |y| (x, y))) {
            let parent_positions: Vec<_> = (0..level.width)
                .flat_map(|px| (0..level.height).map(move |py| (px, py)))
                .flat_map(|(px, py)| {
                    Self::BOMBS
                        .iter()
                        .filter(move |&&item| check_is_position_affected((px, py), item, (x, y)))
                        .map(move |&item| (px, py, item))
                })
                .flat_map(|(px, py, item)| {
                    (0..Self::MAX_DEPTH).map(move |depth| (px, py, depth, item))
                })
                .collect();

            problem.add_constraint(
                variables.is_affected[&(x, y)]
                    .into_expression()
                    .leq(parent_positions.iter().fold(
                        Expression::default(),
                        |acc, &(px, py, depth, item)| {
                            acc + variables.is_placed[&(px, py, depth, item)]
                        },
                    ))
                    .set_name(format!("bombs_affecting_{}_{}", x, y)),
            );
        }

        // rocks must be affected
        for (pos, &item) in &level.objects {
            if item == Item::Rock {
                let affected_var = variables.is_affected[pos];
                problem.add_constraint(
                    affected_var
                        .into_expression()
                        .geq(1.0.into_expression())
                        .set_name(format!("rock_{}_{}_affected", pos.0, pos.1)),
                );
            }
        }

        // jewels must not be affected
        for (pos, &item) in &level.objects {
            if item == Item::Jewel {
                let affected_var = variables.is_affected[pos];
                problem.add_constraint(
                    affected_var
                        .into_expression()
                        .eq(0.0.into_expression())
                        .set_name(format!("jewel_{}_{}_not_affected", pos.0, pos.1)),
                );
            }
        }

        // if a bomb is not on fire, there must be at least one bomb affecting it
        for (x, y, depth) in (0..level.width)
            .flat_map(|x| (0..level.height).map(move |y| (x, y)))
            .flat_map(|(x, y)| (0..Self::MAX_DEPTH).map(move |depth| (x, y, depth)))
        {
            if (x, y) == level.fire {
                continue; // Skip the fire position
            }

            // position and items which can affect the position
            let possible_parents: Vec<(usize, usize, usize, Item)> = (0..level.width)
                .flat_map(|x| (0..level.height).map(move |y| (x, y)))
                .filter(|&(px, py)| (px, py) != (x, y))
                .flat_map(|(px, py)| Self::BOMBS.iter().map(move |&item| (px, py, item)))
                .filter(|&(px, py, item)| check_is_position_affected((px, py), item, (x, y)))
                .flat_map(|(px, py, item)| (0..depth).map(move |depth| (px, py, depth, item)))
                .collect();

            problem.add_constraint(
                possible_parents
                    .iter()
                    .fold(Expression::default(), |acc, &(px, py, pdepth, item)| {
                        acc + variables.is_placed[&(px, py, pdepth, item)]
                    })
                    .geq(
                        Self::BOMBS
                            .iter()
                            .fold(Expression::default(), |acc, &item| {
                                acc + variables.is_placed[&(x, y, depth, item)]
                            })
                            .into_expression(),
                    )
                    .set_name(format!("bombs_affecting_{}_{}", x, y)),
            );
        }
    }

    fn define_objective_function_minimum_bombs(
        &self,
        _level: &Level,
        variables: &Variables,
    ) -> impl IntoAffineExpression {
        // Set the objective function for the LP solver
        // For example, minimize the number of bombs used

        variables
            .is_placed
            .values()
            .fold(Expression::default(), |acc, &var| acc + var)
    }

    fn define_objective_function_affected_areas(
        &self,
        level: &Level,
        variables: &Variables,
    ) -> impl IntoAffineExpression {
        // Set the objective function to maximize the number of affected areas
        (0..level.width)
            .flat_map(|x| (0..level.height).map(move |y| (x, y)))
            .filter_map(|pos| variables.is_affected.get(&pos))
            .fold(Expression::default(), |acc, &var| acc + var)
            * 100.0
            + self.define_objective_function_minimum_bombs(level, variables)
    }

    fn build_solution<S: Solution>(
        &self,
        solution: &S,
        level: &Level,
        variables: &Variables,
    ) -> Result<LevelSolution, Error> {
        // Run the LP solver and return the solution
        // This is a placeholder; actual implementation would involve calling an LP solver library
        let mut bombs = HashMap::new();
        for (x, y) in (0..level.width).flat_map(|x| (0..level.height).map(move |y| (x, y))) {
            for depth in 0..Self::MAX_DEPTH {
                for &item in &Self::BOMBS {
                    let var = variables.is_placed[&(x, y, depth, item)];
                    if solution.value(var) > 0.5 && !level.objects.contains_key(&(x, y)) {
                        bombs.insert((x, y), item);
                    }
                }
            }
        }

        let is_affected: Vec<Vec<bool>> = (0..level.height)
            .map(|y| {
                (0..level.width)
                    .map(|x| {
                        variables
                            .is_affected
                            .get(&(x, y))
                            .map_or(false, |&var| solution.value(var) > 0.5)
                    })
                    .collect()
            })
            .collect();

        Ok(LevelSolution {
            level: level.clone(),
            bombs,
            is_affected,
        })
    }
}

fn check_is_position_affected(
    (px, py): (usize, usize),
    item: Item,
    (x, y): (usize, usize),
) -> bool {
    item.impact_zone().iter().any(|&(dx, dy)| {
        let affected_x = (px as i8 + dx) as usize;
        let affected_y = (py as i8 + dy) as usize;
        affected_x == x && affected_y == y
    })
}

fn check_possibly_placed(
    pos: &(usize, usize),
    item: Item,
    objects: &HashMap<(usize, usize), Item>,
) -> bool {
    !objects.contains_key(pos)
        && !item.impact_zone().iter().any(|(dx, dy)| {
            let new_x = (pos.0 as i8 + dx) as u8;
            let new_y = (pos.1 as i8 + dy) as u8;
            objects
                .get(&(new_x as usize, new_y as usize))
                .and_then(|&item| Some(item == Item::Jewel))
                .unwrap_or(false)
        })
}

fn display_level(level: &Level) {
    println!("Area: {} x {}:", level.width, level.height);
    display_objects(level);
    println!("Fire at: {:?}", level.fire);
}

fn display_solution(solution: &LevelSolution) {
    let LevelSolution {
        mut level,
        bombs,
        is_affected,
    } = solution.clone();

    for ((x, y), item) in &bombs {
        level.objects.insert((*x, *y), *item);
    }

    display_objects(&level);
}

fn display_objects(level: &Level) {
    for y in 0..level.height {
        for x in 0..level.width {
            let letter = match level.objects.get(&(x, y)) {
                Some(item) => match item {
                    Item::BombSmall => "S",
                    Item::BombMedium => "M",
                    Item::BombLarge => "L",
                    Item::BombHorizontal => "H",
                    Item::BombVertical => "V",
                    Item::Null => "x",
                    Item::Rock => "R",
                    Item::Jewel => "J",
                    Item::Eraser => "E",
                    Item::Enemy => "X",
                },
                None => ".",
            };

            let fire = if (x, y) == level.fire { "*" } else { "" };
            print!(" {}{}", letter, fire);
        }
        println!();
    }
}
