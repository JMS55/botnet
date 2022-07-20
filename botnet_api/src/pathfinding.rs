use crate::{ArchivedBay, ArchivedBot, ArchivedEntity, PartialEntityType, BAY_SIZE};
use std::collections::{HashMap, VecDeque};

pub trait GoalFunction: Fn(u32, u32, &ArchivedBot, &ArchivedBay) -> bool {}
impl<T: Fn(u32, u32, &ArchivedBot, &ArchivedBay) -> bool> GoalFunction for T {}

impl ArchivedBot {
    /// Find a path to the nearest cell that satisfies a goal function.
    pub fn find_path_to<F: GoalFunction>(
        &self,
        goal_function: F,
        bay: &ArchivedBay,
    ) -> Option<Vec<(u32, u32)>> {
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

/// Goal function for finding a certain position.
#[allow(non_snake_case)]
pub fn POSITION(x: u32, y: u32) -> impl GoalFunction {
    move |x2: u32, y2: u32, _bot: &ArchivedBot, _bay: &ArchivedBay| x2 == x && y2 == y
}

/// Goal function for finding an antenna with the same controller as the bot that is pathfinding.
#[allow(non_snake_case)]
pub fn ANTENNA(x: u32, y: u32, bot: &ArchivedBot, bay: &ArchivedBay) -> bool {
    bay.get_entity_at_position(x, y)
        .map(|entity| entity.is_antenna_controlled_by(bot.controller_id))
        .unwrap_or(false)
}

/// Goal function for finding any resource.
#[allow(non_snake_case)]
pub fn RESOURCE(x: u32, y: u32, _bot: &ArchivedBot, bay: &ArchivedBay) -> bool {
    bay.get_entity_at_position(x, y)
        .map(ArchivedEntity::is_resource)
        .unwrap_or(false)
}

/// Goal function for finding a partial entity of a certain type.
#[allow(non_snake_case)]
pub fn PARTIAL_ENTITY(partial_entity_type: PartialEntityType) -> impl GoalFunction {
    move |x: u32, y: u32, _bot: &ArchivedBot, bay: &ArchivedBay| {
        bay.get_entity_at_position(x, y)
            .map(|entity| entity.is_partial_entity_of_type(partial_entity_type))
            .unwrap_or(false)
    }
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
