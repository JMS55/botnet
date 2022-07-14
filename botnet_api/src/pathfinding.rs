use crate::{ArchivedBay, ArchivedBot, ArchivedEntity, BAY_SIZE};
use std::collections::{HashMap, VecDeque};

impl ArchivedBot {
    /// Find a path to the nearest cell that satisfies a goal function.
    pub fn find_path_to<'a, F>(
        &'a self,
        goal_function: F,
        bay: &'a ArchivedBay,
    ) -> Option<Vec<(u32, u32)>>
    where
        F: Fn(u32, u32, &'a ArchivedBot, &'a ArchivedBay) -> bool,
    {
        let mut frontier = VecDeque::new();
        let mut came_from: HashMap<(u32, u32), (u32, u32)> = HashMap::new();
        frontier.push_back((self.x, self.y));
        came_from.insert((self.x, self.y), (self.x, self.y));

        let mut goal = None;
        let mut real_goal = (0, 0);
        let mut is_neighboring_goal = |(x, y)| {
            for neighbor in neighbors(x, y) {
                if let Some((neighbor_x, neighbor_y)) = neighbor {
                    if (goal_function)(neighbor_x, neighbor_y, self, bay) {
                        real_goal = (neighbor_x, neighbor_y);
                        return true;
                    }
                }
            }
            false
        };

        while let Some(visiting) = frontier.pop_front() {
            if (is_neighboring_goal)(visiting) {
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

/// Goal function for finding an antenna with the same controller as the bot that is pathfinding.
pub const ANTENNA: fn(u32, u32, &ArchivedBot, &ArchivedBay) -> bool = antenna_exists_at_position;
fn antenna_exists_at_position(x: u32, y: u32, bot: &ArchivedBot, bay: &ArchivedBay) -> bool {
    bay.get_entity_at_position(x, y)
        .map(|entity| entity.is_antenna_controlled_by(bot.controller_id))
        .unwrap_or(false)
}

/// Goal function for finding any resource.
pub const RESOURCE: fn(u32, u32, &ArchivedBot, &ArchivedBay) -> bool = resource_exists_at_position;
fn resource_exists_at_position(x: u32, y: u32, _bot: &ArchivedBot, bay: &ArchivedBay) -> bool {
    bay.get_entity_at_position(x, y)
        .map(ArchivedEntity::is_resource)
        .unwrap_or(false)
}

fn empty_neighbors((x, y): (u32, u32), bay: &ArchivedBay) -> [Option<(u32, u32)>; 4] {
    [
        (y != 0)
            .then(|| (x, y - 1))
            .filter(|(x, y)| bay.cells[*x as usize][*y as usize].is_none()),
        (y as usize != BAY_SIZE - 1)
            .then(|| (x, y + 1))
            .filter(|(x, y)| bay.cells[*x as usize][*y as usize].is_none()),
        (x != 0)
            .then(|| (x - 1, y))
            .filter(|(x, y)| bay.cells[*x as usize][*y as usize].is_none()),
        (x as usize != BAY_SIZE - 1)
            .then(|| (x + 1, y))
            .filter(|(x, y)| bay.cells[*x as usize][*y as usize].is_none()),
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
