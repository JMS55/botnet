use crate::{
    Antenna, ArchivedAntenna, ArchivedBay, ArchivedBot, ArchivedEntity, ArchivedResource, Bay, Bot,
    Entity, EntityID, Resource, BAY_SIZE,
};

impl Entity {
    pub fn is_bot(&self) -> bool {
        match self {
            Self::Bot(_) => true,
            _ => false,
        }
    }

    pub fn is_antenna_controlled_by(&self, player_id: EntityID) -> bool {
        match self {
            Self::Antenna(Antenna { controller_id, .. }) => *controller_id == player_id,
            _ => false,
        }
    }

    pub fn unwrap_as_antenna(&self) -> &Antenna {
        match self {
            Self::Antenna(antenna) => antenna,
            _ => unreachable!(),
        }
    }

    pub fn unwrap_mut_as_antenna(&mut self) -> &mut Antenna {
        match self {
            Self::Antenna(antenna) => antenna,
            _ => unreachable!(),
        }
    }

    pub fn is_resource(&self) -> bool {
        match self {
            Self::Resource(_) => true,
            _ => false,
        }
    }

    pub fn unwrap_as_resource(&self) -> &Resource {
        match self {
            Self::Resource(resource) => resource,
            _ => unreachable!(),
        }
    }
}

impl ArchivedEntity {
    pub fn is_bot(&self) -> bool {
        match self {
            Self::Bot(_) => true,
            _ => false,
        }
    }

    pub fn is_antenna_controlled_by(&self, player_id: EntityID) -> bool {
        match self {
            Self::Antenna(ArchivedAntenna { controller_id, .. }) => *controller_id == player_id,
            _ => false,
        }
    }

    pub fn unwrap_as_antenna(&self) -> &ArchivedAntenna {
        match self {
            Self::Antenna(antenna) => antenna,
            _ => unreachable!(),
        }
    }

    pub fn is_resource(&self) -> bool {
        match self {
            Self::Resource(_) => true,
            _ => false,
        }
    }

    pub fn unwrap_as_resource(&self) -> &ArchivedResource {
        match self {
            Self::Resource(resource) => resource,
            _ => unreachable!(),
        }
    }
}

impl Bay {
    pub fn get_entity_at_position(&self, x: u32, y: u32) -> Option<&Entity> {
        if x as usize >= BAY_SIZE || y as usize >= BAY_SIZE {
            return None;
        }

        self.cells[x as usize][y as usize]
            .map(|entity_id| self.entities.get(&entity_id))
            .flatten()
            .map(|(entity, _, _)| entity)
    }

    pub fn get_mut_entity_at_position(&mut self, x: u32, y: u32) -> Option<&mut Entity> {
        if x as usize >= BAY_SIZE || y as usize >= BAY_SIZE {
            return None;
        }

        self.cells[x as usize][y as usize]
            .map(|entity_id| self.entities.get_mut(&entity_id))
            .flatten()
            .map(|(entity, _, _)| entity)
    }

    pub fn get_bot_ids(&self) -> Box<[EntityID]> {
        self.entities
            .iter()
            .filter(|(_, (entity, _, _))| entity.is_bot())
            .map(|(entity_id, _)| *entity_id)
            .collect()
    }

    pub fn get_bot(&self, entity_id: EntityID) -> Option<&Bot> {
        match self.entities.get(&entity_id) {
            Some((Entity::Bot(bot), _, _)) => Some(bot),
            _ => None,
        }
    }

    pub fn get_bot_mut(&mut self, entity_id: EntityID) -> Option<&mut Bot> {
        match self.entities.get_mut(&entity_id) {
            Some((Entity::Bot(bot), _, _)) => Some(bot),
            _ => None,
        }
    }
}

impl ArchivedBay {
    pub fn get_entity_at_position(&self, x: u32, y: u32) -> Option<&ArchivedEntity> {
        if x as usize >= BAY_SIZE || y as usize >= BAY_SIZE {
            return None;
        }

        self.cells[x as usize][y as usize]
            .as_ref() // TODO: Remove as_ref() once rkyv updates
            .map(|entity_id| self.entities.get(&entity_id))
            .flatten()
            .map(|(entity, _, _)| entity)
    }

    pub fn get_bot(&self, entity_id: EntityID) -> Option<&ArchivedBot> {
        match self.entities.get(&entity_id) {
            Some((ArchivedEntity::Bot(bot), _, _)) => Some(bot),
            _ => None,
        }
    }
}
