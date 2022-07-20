use botnet_api::{Antenna, Entity, EntityID, PartialEntity, PartialEntityType};
use extension_traits::extension;

#[extension(pub trait PartialEntityTypeExt)]
impl PartialEntityType {
    /// Create a new PartialEntity for this PartialEntityType.
    fn new_partial_entity(&self) -> Entity {
        Entity::PartialEntity(match self {
            Self::Antenna => PartialEntity {
                entity_type: *self,
                contributed_copper: 0,
                required_copper: 2,
                contributed_gold: 0,
                required_gold: 2,
                contributed_silicon: 0,
                required_silicon: 2,
                contributed_plastic: 0,
                required_plastic: 2,
            },
        })
    }
}

#[extension(pub trait PartialEntityExt)]
impl Entity {
    /// Convert a partial entity into a full entity.
    fn partial_entity_into_entity(&mut self, controller_id: EntityID) {
        match self {
            Self::PartialEntity(partial_entity) => match partial_entity.entity_type {
                PartialEntityType::Antenna => {
                    *self = Entity::Antenna(Antenna {
                        controller_id,
                        stored_copper: 0,
                        stored_gold: 0,
                        stored_silicon: 0,
                        stored_plastic: 0,
                    });
                }
            },
            _ => unreachable!(),
        }
    }
}
