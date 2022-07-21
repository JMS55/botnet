use crate::bay_renderer::TILE_SIZE;
use botnet_api::{Bay, Entity, EntityID, PartialEntityType, Resource, BAY_SIZE};
use macroquad::prelude::{mouse_position, vec2, Color, Vec2, WHITE};
use macroquad::ui::{root_ui, Skin, Ui};

pub fn draw_tooltip(bay: &Bay) {
    // Check if hovering over the game
    let mouse_position = Vec2::from(mouse_position());
    let hovered_tile = (mouse_position / TILE_SIZE).round() - Vec2::ONE;
    if hovered_tile.x < 0.0
        || hovered_tile.y < 0.0
        || hovered_tile.x >= BAY_SIZE as f32
        || hovered_tile.y >= BAY_SIZE as f32
    {
        return;
    }

    // If hovering over an entity
    if let Some(entity_id) = bay.cells[hovered_tile.x as usize][hovered_tile.y as usize] {
        // Setup ui styles
        let window_style = root_ui()
            .style_builder()
            .color(Color::from_rgba(68, 71, 116, 230))
            .build();
        let label_style = root_ui()
            .style_builder()
            .font(include_bytes!("../assets/Inter-3.19/Inter-Medium.ttf"))
            .unwrap()
            .font_size(12)
            .text_color(WHITE)
            .build();
        let default_skin = root_ui().default_skin();
        root_ui().push_skin(&Skin {
            window_style,
            label_style,
            ..default_skin
        });

        // Draw tooltip
        root_ui().window(
            (hovered_tile.y as u64 * BAY_SIZE as u64) + hovered_tile.x as u64,
            Vec2::splat(8.0),
            vec2(240.0, 120.0),
            |ui| {
                draw_entity_tooltip(
                    ui,
                    entity_id,
                    hovered_tile.x as u32,
                    hovered_tile.y as u32,
                    bay,
                );
            },
        );

        root_ui().pop_skin();
    }
}

pub fn draw_entity_tooltip(
    ui: &mut Ui,
    entity_id: EntityID,
    entity_x: u32,
    entity_y: u32,
    bay: &Bay,
) {
    let entity = bay.get_entity_at_position(entity_x, entity_y).unwrap();

    // Draw entity id and coordinates
    ui.label(
        None,
        &format!("EntityID: {entity_id}, X: {entity_x}, Y: {entity_y}",),
    );

    // Draw entity type
    let entity_type = match entity {
        Entity::Bot(_) => "Bot",
        Entity::Antenna(_) => "Antenna",
        Entity::Interconnect { .. } => todo!(),
        Entity::Resource(resource) => match resource {
            Resource::Copper => "Resource (Copper)",
            Resource::Gold => "Resource (Gold)",
            Resource::Silicon => "Resource (Silicon)",
            Resource::Plastic => "Resource (Plastic)",
        },
        Entity::PartialEntity(partial_entity) => match partial_entity.entity_type {
            PartialEntityType::Antenna => "PartialEntity (Antenna)",
        },
    };
    ui.label(None, &format!("Entity Type: {entity_type}"));

    // Draw entity information
    draw_entity_tooltip_details(ui, entity);
}

fn draw_entity_tooltip_details(ui: &mut Ui, entity: &Entity) {
    match entity {
        Entity::Bot(bot) => {
            ui.label(None, &format!("    ControllerID: {}", bot.controller_id));
            ui.label(
                None,
                &format!(
                    "    Held Resource: {}",
                    match bot.held_resource {
                        Some(Resource::Copper) => "Copper",
                        Some(Resource::Gold) => "Gold",
                        Some(Resource::Silicon) => "Silicon",
                        Some(Resource::Plastic) => "Plastic",
                        None => "None",
                    }
                ),
            );
            ui.label(None, &format!("    Energy: {}", bot.energy));
        }

        Entity::Antenna(antenna) => {
            ui.label(
                None,
                &format!("    ControllerID: {}", antenna.controller_id),
            );
            ui.label(
                None,
                &format!("    Stored Copper: {}", antenna.stored_copper),
            );
            ui.label(None, &format!("    Stored Gold: {}", antenna.stored_gold));
            ui.label(
                None,
                &format!("    Stored Silicon: {}", antenna.stored_silicon),
            );
            ui.label(
                None,
                &format!("    Stored Plastic: {}", antenna.stored_plastic),
            );
        }

        Entity::Interconnect { .. } => todo!(),

        Entity::Resource(_) => {}

        Entity::PartialEntity(partial_entity) => {
            ui.label(
                None,
                &format!(
                    "    Copper: {}/{}",
                    partial_entity.contributed_copper, partial_entity.required_copper,
                ),
            );
            ui.label(
                None,
                &format!(
                    "    Gold: {}/{}",
                    partial_entity.contributed_gold, partial_entity.required_gold,
                ),
            );
            ui.label(
                None,
                &format!(
                    "    Silicon: {}/{}",
                    partial_entity.contributed_silicon, partial_entity.required_silicon,
                ),
            );
            ui.label(
                None,
                &format!(
                    "    Plastic: {}/{}",
                    partial_entity.contributed_plastic, partial_entity.required_plastic,
                ),
            );
        }
    }
}
