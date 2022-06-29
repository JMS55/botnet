use crate::{ArchivedBay, ArchivedBot, ArchivedCell, Cell, BAY_SIZE};
use std::collections::{HashMap, VecDeque};

impl ArchivedBot {
    /// Find a path to the nearest cell that satisfies a goal function.
    pub fn find_path_to<F>(&self, goal_function: F, bay: &ArchivedBay) -> Option<Vec<(u32, u32)>>
    where
        F: Fn(&ArchivedCell, u32, u32) -> bool,
    {
        let mut frontier = VecDeque::new();
        let mut came_from: HashMap<(u32, u32), (u32, u32)> = HashMap::new();
        frontier.push_back((self.x, self.y));
        came_from.insert((self.x, self.y), (self.x, self.y));

        let mut goal = None;
        let mut real_goal = (0, 0);
        let mut neighbors_goal = |(x, y)| {
            for neighbor in neighbors(x, y) {
                if let Some((neighbor_x, neighbor_y)) = neighbor {
                    if (goal_function)(
                        &bay.cells[neighbor_x as usize][neighbor_y as usize],
                        neighbor_x,
                        neighbor_y,
                    ) {
                        real_goal = (neighbor_x, neighbor_y);
                        return true;
                    }
                }
            }
            false
        };

        while let Some(visiting) = frontier.pop_front() {
            if (neighbors_goal)(visiting) {
                goal = Some(visiting);
                break;
            }

            for neighbor in empty_neighbors(visiting, bay) {
                if let Some(neighbor) = neighbor {
                    if !came_from.contains_key(&neighbor) {
                        frontier.push_back(neighbor);
                        came_from.insert(neighbor, visiting);
                    }
                }
            }
        }

        goal.map(|goal| {
            let mut path = vec![real_goal];
            let mut previous = goal;
            while previous != (self.x, self.y) {
                path.push(previous);
                previous = came_from[&previous];
            }
            path
        })
    }
}

/// Goal function for finding any resource.
pub const RESOURCE: fn(&ArchivedCell, u32, u32) -> bool = is_resource;
fn is_resource(cell: &ArchivedCell, _x: u32, _y: u32) -> bool {
    match cell {
        ArchivedCell::Resource(_) => true,
        _ => false,
    }
}

fn empty_neighbors((x, y): (u32, u32), bay: &ArchivedBay) -> [Option<(u32, u32)>; 4] {
    [
        (y != 0)
            .then(|| (x, y - 1))
            .filter(|(x, y)| bay.cells[*x as usize][*y as usize] == Cell::Empty),
        (y as usize != BAY_SIZE - 1)
            .then(|| (x, y + 1))
            .filter(|(x, y)| bay.cells[*x as usize][*y as usize] == Cell::Empty),
        (x != 0)
            .then(|| (x - 1, y))
            .filter(|(x, y)| bay.cells[*x as usize][*y as usize] == Cell::Empty),
        (x as usize != BAY_SIZE - 1)
            .then(|| (x + 1, y))
            .filter(|(x, y)| bay.cells[*x as usize][*y as usize] == Cell::Empty),
    ]
}

fn neighbors(x: u32, y: u32) -> [Option<(u32, u32)>; 4] {
    [
        (y != 0).then(|| (x, y - 1)),
        (y as usize != BAY_SIZE - 1).then(|| (x, y + 1)),
        (x != 0).then(|| (x - 1, y)),
        (x as usize != BAY_SIZE - 1).then(|| (x + 1, y)),
    ]
}
