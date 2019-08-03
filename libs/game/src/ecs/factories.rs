use amethyst::{
    assets::{Handle, Prefab},
    core::Transform,
    ecs::Entity,
    prelude::{Builder, World},
    renderer::{
        palette::LinSrgba,
        rendy::mesh::{Position, TexCoord},
        SpriteRender, SpriteSheet,
    },
    utils::tag::Tag,
    window::ScreenDimensions,
};

use ha_animation_prefabs::GameSpriteAnimationPrefab;
use ha_core::math::{Vector2, ZeroVector};

use crate::{
    ecs::{
        components::{damage_history::DamageHistory, *},
        resources::{GameLevelState, HEALTH_UI_SCREEN_PADDING},
        tags::*,
    },
    utils::graphic_helpers::{create_color_material, create_mesh},
};

pub fn create_player(
    world: &mut World,
    prefab_handle: Handle<Prefab<GameSpriteAnimationPrefab>>,
) -> Entity {
    let mut transform = Transform::default();
    transform.set_translation_z(10.0);

    let (half_screen_width, half_screen_height) = {
        let screen_dimensions = world.read_resource::<ScreenDimensions>();
        (
            screen_dimensions.width() / 2.0,
            screen_dimensions.height() / 2.0,
        )
    };

    world
        .create_entity()
        .with(transform)
        .with(prefab_handle)
        .with(PlayerActions::default())
        .with(WorldPosition::new(Vector2::zero()))
        .with(Player::new())
        .with(DamageHistory::default())
        .with(HealthUiGraphics {
            screen_position: Vector2::new(
                -half_screen_width + HEALTH_UI_SCREEN_PADDING,
                -half_screen_height + HEALTH_UI_SCREEN_PADDING,
            ),
            scale_ratio: 1.0,
            health: 1.0,
        })
        .build()
}

pub fn create_landscape(world: &mut World, landscape_texture_handle: Handle<SpriteSheet>) {
    let mut transform = Transform::default();
    transform.set_translation_z(-1.0);

    world
        .create_entity()
        .with(Tag::<Landscape>::default())
        .with(transform)
        .with(SpriteRender {
            sprite_sheet: landscape_texture_handle.clone(),
            sprite_number: 0,
        })
        .build();
}

pub fn create_debug_scene_border(world: &mut World) {
    let border_width = 3.0;

    let screen_dimensions = world.read_resource::<GameLevelState>().dimensions;
    let half_screen_width = screen_dimensions.x / 2.0;
    let half_screen_height = screen_dimensions.y / 2.0;

    let generate_rectangle = |positions: &mut Vec<Position>,
                              tex_coords: &mut Vec<TexCoord>,
                              left_bottom: Vector2,
                              right_top: Vector2| {
        positions.push(Position([left_bottom.x, right_top.y, 0.0]));
        tex_coords.push(TexCoord([0.0, 1.0]));
        positions.push(Position([left_bottom.x, left_bottom.y, 0.0]));
        tex_coords.push(TexCoord([0.0, 0.0]));
        positions.push(Position([right_top.x, left_bottom.y, 0.0]));
        tex_coords.push(TexCoord([1.0, 0.0]));
        positions.push(Position([right_top.x, left_bottom.y, 0.0]));
        tex_coords.push(TexCoord([1.0, 0.0]));
        positions.push(Position([right_top.x, right_top.y, 0.0]));
        tex_coords.push(TexCoord([1.0, 1.0]));
        positions.push(Position([left_bottom.x, right_top.y, 0.0]));
        tex_coords.push(TexCoord([0.0, 1.0]));
    };

    let mut positions = Vec::with_capacity(24);
    let mut tex_coords = Vec::with_capacity(24);
    // Top.
    generate_rectangle(
        &mut positions,
        &mut tex_coords,
        Vector2::new(-half_screen_width, half_screen_height - border_width),
        Vector2::new(half_screen_width, half_screen_height),
    );
    // Right.
    generate_rectangle(
        &mut positions,
        &mut tex_coords,
        Vector2::new(half_screen_width - border_width, -half_screen_height),
        Vector2::new(half_screen_width, half_screen_height),
    );
    // Bottom.
    generate_rectangle(
        &mut positions,
        &mut tex_coords,
        Vector2::new(-half_screen_width, -half_screen_height),
        Vector2::new(half_screen_width, -half_screen_height + border_width),
    );
    // Left.
    generate_rectangle(
        &mut positions,
        &mut tex_coords,
        Vector2::new(-half_screen_width, -half_screen_height),
        Vector2::new(-half_screen_width + border_width, half_screen_height),
    );

    let mesh = create_mesh(world, positions, tex_coords);
    let color = LinSrgba::new(0.0, 0.0, 1.0, 1.0);
    let material = create_color_material(world, color);
    let transform = Transform::default();

    world
        .create_entity()
        .with(mesh)
        .with(material)
        .with(transform)
        .build();
}
