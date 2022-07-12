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
    entity_render_overrides: HashMap<EntityID, EntityRenderOverrides>,
    textures: [Texture2D; 2],
}

#[derive(Default)]
pub struct EntityRenderOverrides {
    pub position_offset: Option<Vec2>,
    pub rotation: Option<f32>,
    pub scale: Option<Vec2>,
    pub color: Option<Color>,
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
            entity_render_overrides: HashMap::new(),
            textures,
        }
    }

    pub fn draw_bay(&mut self, bay: &Bay) {
        // Tick animation
        let mut animation_finished = false;
        if let Some(animation) = &mut self.animation {
            animation_finished = animation.tick(&mut self.entity_render_overrides);
        }

        // Draw bay
        clear_background(Color::from_rgba(24, 25, 22, 255));
        self.draw_ground();
        self.draw_walls();
        self.draw_entities(bay);

        // Cleanup animation if finished
        if animation_finished {
            self.animation = None;
            self.cleanup_entity_render_overrides(bay);
        }
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

    // TODO: Cleanup/splitup
    fn draw_entities(&mut self, bay: &Bay) {
        for x in 0..BAY_SIZE {
            for y in 0..BAY_SIZE {
                if let Some(entity_id) = bay.cells[x][y] {
                    let entity = bay.entities.get(&entity_id).unwrap();
                    let entity_render_overrides =
                        &*self.entity_render_overrides.entry(entity_id).or_default();

                    match entity {
                        Entity::Wall => {}

                        // TODO: Need to account for scale override
                        Entity::Bot { .. } => {
                            let position = (vec2(x as f32, y as f32) * TILE_SIZE as f32)
                                + vec2(4.0, 4.0)
                                + entity_render_overrides
                                    .position_offset
                                    .unwrap_or(Vec2::ZERO);
                            let rotation = entity_render_overrides.rotation.unwrap_or(0.0);
                            let scale = entity_render_overrides.scale.unwrap_or(Vec2::ONE);
                            let color = entity_render_overrides
                                .color
                                .unwrap_or(Color::from_rgba(180, 180, 180, 255));
                            draw_texture_ex(
                                self.textures[0],
                                position.x,
                                position.y,
                                color,
                                DrawTextureParams {
                                    dest_size: Some(
                                        vec2((TILE_SIZE - 8) as f32, (TILE_SIZE - 8) as f32)
                                            * scale,
                                    ),
                                    rotation,
                                    ..Default::default()
                                },
                            );
                        }

                        // TODO: Need to account for scale override
                        Entity::Resource { .. } => {
                            rand::srand((x * y) as u64);
                            let size_modifier = rand::gen_range(0.0, 6.0);

                            let position = (vec2(x as f32, y as f32) * TILE_SIZE as f32)
                                + vec2(size_modifier / 2.0, size_modifier / 2.0)
                                + entity_render_overrides
                                    .position_offset
                                    .unwrap_or(Vec2::ZERO);
                            let rotation = entity_render_overrides
                                .rotation
                                .unwrap_or(rand::gen_range(0.0, TAU));
                            let scale = entity_render_overrides.scale.unwrap_or(Vec2::ONE);
                            let color = entity_render_overrides
                                .color
                                .unwrap_or(Color::from_rgba(96, 96, 96, 255));

                            draw_texture_ex(
                                self.textures[1],
                                position.x,
                                position.y,
                                color,
                                DrawTextureParams {
                                    dest_size: Some(
                                        vec2(
                                            TILE_SIZE as f32 - size_modifier,
                                            TILE_SIZE as f32 - size_modifier,
                                        ) * scale,
                                    ),
                                    rotation,
                                    ..Default::default()
                                },
                            );
                        }

                        Entity::Interconnect { .. } => todo!(),

                        Entity::Antenna { .. } => todo!(),
                    }
                }
            }
        }
    }

    fn cleanup_entity_render_overrides(&mut self, bay: &Bay) {
        let entity_ids = self
            .entity_render_overrides
            .keys()
            .copied()
            .collect::<Box<[_]>>();

        for entity_id in entity_ids.into_iter() {
            match bay.entities.get(&entity_id).unwrap() {
                Entity::Bot { .. } => {
                    let previous_rotation = self
                        .entity_render_overrides
                        .get(entity_id)
                        .unwrap()
                        .rotation;

                    self.entity_render_overrides.insert(
                        *entity_id,
                        EntityRenderOverrides {
                            rotation: previous_rotation,
                            ..Default::default()
                        },
                    )
                }

                _ => self.entity_render_overrides.remove(entity_id),
            };
        }
    }
}
