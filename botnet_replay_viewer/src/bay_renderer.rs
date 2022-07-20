use crate::animation::Animation;
use botnet_api::{Bay, Entity, EntityID, PartialEntity, PartialEntityType, Resource, BAY_SIZE};
use macroquad::prelude::{
    clear_background, draw_circle, draw_texture_ex, get_time, rand, vec2, Color, Conf,
    DrawTextureParams, Texture2D, Vec2,
};
use std::collections::HashMap;
use std::f32::consts::PI;

pub const TILE_SIZE: f32 = 32.0;

pub fn window_conf() -> Conf {
    Conf {
        window_title: "BotnetReplayViewer".to_owned(),
        window_width: (BAY_SIZE + 1) as i32 * TILE_SIZE as i32,
        window_height: (BAY_SIZE + 1) as i32 * TILE_SIZE as i32,
        window_resizable: false,
        ..Default::default()
    }
}

pub struct BayRenderer {
    pub animation: Option<Animation>,
    pub entity_render_parameters: HashMap<EntityID, EntityRenderParameters>,
    textures: [Texture2D; 3],
}

pub struct EntityRenderParameters {
    pub position: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
    pub color: Color,
}

impl BayRenderer {
    pub fn new() -> Self {
        let bot_texture =
            Texture2D::from_file_with_format(include_bytes!("../assets/ship_E32.png"), None);
        let resource_texture = Texture2D::from_file_with_format(
            include_bytes!("../assets/meteor_detailedLarge.png"),
            None,
        );
        let antenna_texture =
            Texture2D::from_file_with_format(include_bytes!("../assets/station_C.png"), None);
        let textures = [bot_texture, resource_texture, antenna_texture];

        Self {
            animation: None,
            entity_render_parameters: HashMap::new(),
            textures,
        }
    }

    pub fn prepare(&mut self, bay: &Bay) {
        self.prepare_existing_entities(bay);
        self.prepare_new_entities(bay);
    }

    /// Update or remove existing EntityRenderParameters.
    fn prepare_existing_entities(&mut self, bay: &Bay) {
        self.entity_render_parameters
            .retain(|entity_id, entity_render_parameters| {
                if let Some((entity, x, y)) = bay.entities.get(entity_id) {
                    entity_render_parameters.position = vec2(*x as f32, *y as f32);
                    match entity {
                        Entity::Bot(_) => {}
                        _ => entity_render_parameters.rotation = 0.0,
                    }
                    entity_render_parameters.scale = Vec2::ONE;
                    entity_render_parameters.color = entity_color(entity);
                    true
                } else {
                    false
                }
            });
    }

    /// Add EntityRenderParameters for new entities.
    fn prepare_new_entities(&mut self, bay: &Bay) {
        let new_entities = bay
            .entities
            .iter()
            .filter(|(entity_id, _)| !self.entity_render_parameters.contains_key(entity_id))
            .collect::<Box<[_]>>();

        for (entity_id, (entity, x, y)) in new_entities.iter() {
            self.entity_render_parameters.insert(
                **entity_id,
                EntityRenderParameters {
                    position: vec2(*x as f32, *y as f32),
                    rotation: 0.0,
                    scale: Vec2::ONE,
                    color: entity_color(entity),
                },
            );
        }
    }

    pub fn draw_bay(&mut self, bay: &Bay) {
        // Tick animation
        if let Some(animation) = &mut self.animation {
            let animation_finished = animation.tick(&mut self.entity_render_parameters);
            if animation_finished {
                self.animation = None;
            }
        }

        // Draw bay
        clear_background(Color::from_rgba(50, 49, 59, 244));
        self.draw_ground();
        self.draw_entities(bay);
    }

    fn draw_ground(&self) {
        for x in 0..BAY_SIZE {
            for y in 0..BAY_SIZE {
                draw_circle(
                    (x + 1) as f32 * TILE_SIZE,
                    (y + 1) as f32 * TILE_SIZE,
                    if x * y % 2 == 0 { 2.0 } else { 3.0 },
                    if x * y % 2 == 0 {
                        Color::from_rgba(93, 71, 118, 255)
                    } else {
                        Color::from_rgba(70, 60, 94, 255)
                    },
                );
            }
        }
    }

    fn draw_entities(&mut self, bay: &Bay) {
        for (entity_id, entity) in &bay.entities {
            match &entity.0 {
                Entity::Bot(bot) => {
                    self.draw_entity(*entity_id, 0, TILE_SIZE - 8.0, 0.0, None);

                    if let Some(resource) = bot.held_resource {
                        self.draw_entity(
                            *entity_id,
                            0,
                            TILE_SIZE - 18.0,
                            0.0,
                            Some(entity_color(&Entity::Resource(resource))),
                        );
                    }
                }
                Entity::Antenna(_) => self.draw_entity(
                    *entity_id,
                    2,
                    TILE_SIZE,
                    (get_time() / 3.0) as f32 % PI,
                    None,
                ),
                Entity::Interconnect { .. } => todo!(),
                Entity::Resource(_) => {
                    rand::srand(*entity_id);
                    self.draw_entity(
                        *entity_id,
                        1,
                        TILE_SIZE - rand::gen_range(0.0, 6.0),
                        rand::gen_range(0.0, PI),
                        None,
                    );
                }
                Entity::PartialEntity(PartialEntity { entity_type, .. }) => match entity_type {
                    PartialEntityType::Antenna => {
                        self.draw_entity(*entity_id, 2, TILE_SIZE, 0.0, None);
                    }
                },
            }
        }
    }

    fn draw_entity(
        &self,
        entity_id: EntityID,
        texture_index: usize,
        base_size: f32,
        base_rotation: f32,
        color_override: Option<Color>,
    ) {
        let entity_render_parameters = self.entity_render_parameters.get(&entity_id).unwrap();
        let size = base_size * entity_render_parameters.scale;
        let position = entity_render_parameters.position * Vec2::splat(TILE_SIZE)
            + Vec2::splat(TILE_SIZE)
            - (size / 2.0);

        draw_texture_ex(
            self.textures[texture_index],
            position.x,
            position.y,
            color_override.unwrap_or(entity_render_parameters.color),
            DrawTextureParams {
                dest_size: Some(size),
                rotation: base_rotation + entity_render_parameters.rotation,
                ..Default::default()
            },
        );
    }
}

fn entity_color(entity: &Entity) -> Color {
    match entity {
        Entity::Bot(_) => Color::from_rgba(255, 255, 255, 255),
        Entity::Antenna(_) => Color::from_rgba(141, 216, 148, 255),
        Entity::Interconnect { .. } => todo!(),
        Entity::Resource(resource) => match resource {
            Resource::Copper => Color::from_rgba(243, 167, 135, 255),
            Resource::Gold => Color::from_rgba(253, 254, 137, 255),
            Resource::Silicon => Color::from_rgba(133, 83, 149, 255),
            Resource::Plastic => Color::from_rgba(69, 147, 165, 255),
        },
        Entity::PartialEntity(_) => Color::from_rgba(133, 218, 235, 100),
    }
}
