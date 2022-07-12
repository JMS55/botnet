use crate::animation::Animation;
use botnet_api::{Bay, Entity, EntityID, BAY_SIZE};
use macroquad::prelude::{
    clear_background, draw_circle, draw_circle_lines, draw_texture_ex, rand, vec2, Color, Conf,
    DrawTextureParams, Texture2D, Vec2,
};
use std::collections::HashMap;
use std::f32::consts::TAU;

pub const TILE_SIZE: u32 = 32;

pub fn window_conf() -> Conf {
    Conf {
        window_title: "BotnetReplayViewer".to_owned(),
        window_width: BAY_SIZE as i32 * TILE_SIZE as i32,
        window_height: BAY_SIZE as i32 * TILE_SIZE as i32,
        window_resizable: false,
        ..Default::default()
    }
}

pub struct BayRenderer {
    pub animation: Option<Animation>,
    pub entity_render_parameters: HashMap<EntityID, EntityRenderParameters>,
    textures: [Texture2D; 2],
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
            Texture2D::from_file_with_format(include_bytes!("../assets/ship_E.png"), None);
        let resource_texture = Texture2D::from_file_with_format(
            include_bytes!("../assets/meteor_detailedLarge.png"),
            None,
        );
        let textures = [bot_texture, resource_texture];

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
                match bay.entities.get(entity_id) {
                    None => false,

                    Some(Entity::Bot(_)) => {
                        entity_render_parameters.position = todo!();
                        // Keep existing bot rotation
                        entity_render_parameters.scale = Vec2::ONE;
                        entity_render_parameters.color = todo!();
                        true
                    }

                    _ => {
                        entity_render_parameters.position = todo!();
                        entity_render_parameters.rotation = 0.0;
                        entity_render_parameters.scale = Vec2::ONE;
                        entity_render_parameters.color = todo!();
                        true
                    }
                }
            });
    }

    /// Add EntityRenderParameters for new entities.
    fn prepare_new_entities(&mut self, bay: &Bay) {
        let is_not_wall = |entity_id: &&EntityID| match bay.entities.get(entity_id) {
            Some(Entity::Wall) => false,
            _ => true,
        };

        for entity_id in bay
            .entities
            .keys()
            .filter(|entity_id| !self.entity_render_parameters.contains_key(entity_id))
            .filter(is_not_wall)
        {
            self.entity_render_parameters.insert(
                *entity_id,
                EntityRenderParameters {
                    position: todo!(),
                    rotation: 0.0,
                    scale: Vec2::ONE,
                    color: todo!(),
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
        clear_background(Color::from_rgba(24, 25, 22, 255));
        self.draw_ground();
        self.draw_walls();
        self.draw_entities(bay);
    }

    fn draw_ground(&self) {
        for x in 1..BAY_SIZE - 1 {
            for y in 1..BAY_SIZE - 1 {
                draw_circle(
                    x as f32 * TILE_SIZE as f32 + (TILE_SIZE / 2) as f32,
                    y as f32 * TILE_SIZE as f32 + (TILE_SIZE / 2) as f32,
                    2.0,
                    Color::from_rgba(44, 45, 42, 255),
                );
            }
        }
    }

    fn draw_walls(&self) {
        for i in 0..BAY_SIZE {
            for (x, y) in [(i, 0), (i, BAY_SIZE - 1), (0, i), (BAY_SIZE - 1, i)] {
                draw_circle_lines(
                    x as f32 * TILE_SIZE as f32 + (TILE_SIZE / 2) as f32,
                    y as f32 * TILE_SIZE as f32 + (TILE_SIZE / 2) as f32,
                    6.0,
                    2.0,
                    Color::from_rgba(44, 45, 42, 255),
                );
            }
        }
    }

    fn draw_entities(&mut self, bay: &Bay) {
        for (entity_id, entity) in &bay.entities {
            match entity {
                Entity::Wall => {}
                Entity::Bot { .. } => self.draw_entity(*entity_id, 0, (TILE_SIZE - 8) as f32, 0.0),
                Entity::Resource { .. } => {
                    rand::srand(*entity_id);
                    self.draw_entity(
                        *entity_id,
                        1,
                        TILE_SIZE as f32 - rand::gen_range(0.0, 6.0),
                        rand::gen_range(0.0, TAU),
                    );
                }
                Entity::Interconnect { .. } => todo!(),
                Entity::Antenna { .. } => todo!(),
            }
        }
    }

    fn draw_entity(
        &self,
        entity_id: EntityID,
        texture_index: usize,
        base_size: f32,
        base_rotation: f32,
    ) {
        let entity_render_parameters = self.entity_render_parameters.get(&entity_id).unwrap();
        let size = base_size * entity_render_parameters.scale;
        let position = entity_render_parameters.position * Vec2::splat(TILE_SIZE as f32)
            + Vec2::splat(TILE_SIZE as f32 / 2.0)
            - (size / 2.0);

        draw_texture_ex(
            self.textures[texture_index],
            position.x,
            position.y,
            entity_render_parameters.color,
            DrawTextureParams {
                dest_size: Some(size),
                rotation: base_rotation + entity_render_parameters.rotation,
                ..Default::default()
            },
        );
    }
}
