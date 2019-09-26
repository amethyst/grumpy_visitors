use amethyst::{
    assets::{Handle, Loader, Prefab},
    prelude::World,
    renderer::{palette::LinSrgba, rendy::mesh::MeshBuilder, Material, Mesh, SpriteSheet},
    ui::FontHandle,
};

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use ha_animation_prefabs::GameSpriteAnimationPrefab;
use ha_core::{math::Vector3, net::NetIdentifier};

use crate::utils::graphic_helpers::{
    create_color_material, create_mesh, generate_circle_vertices, generate_rectangle_vertices,
};

pub const HEALTH_UI_SCREEN_PADDING: f32 = 40.0;

#[derive(Clone)]
pub struct AssetHandles {
    pub hero_prefab: Handle<Prefab<GameSpriteAnimationPrefab>>,
    pub landscape: Handle<SpriteSheet>,
    pub ui_font: FontHandle,
}

#[derive(Clone)]
pub struct MissileGraphics(pub EntityGraphics);

impl MissileGraphics {
    pub fn register(world: &mut World) {
        let (positions, tex_coords) = generate_circle_vertices(5.0, 64);
        let mesh = create_mesh(world, positions, tex_coords);
        let material = create_color_material(world, LinSrgba::new(1.0, 0.0, 0.0, 1.0));
        world.insert(MissileGraphics(EntityGraphics { mesh, material }));
    }
}

#[derive(Clone)]
pub struct HealthUiMesh(pub Handle<Mesh>);

impl HealthUiMesh {
    pub fn register(world: &mut World) {
        let (vertices, tex_coords, indices) = generate_rectangle_vertices(
            Vector3::new(0.0, 0.0, 100.0),
            Vector3::new(180.0, 180.0, 100.0),
        );

        let mesh = {
            let loader = world.fetch::<Loader>();
            loader.load_from_data(
                MeshBuilder::new()
                    .with_vertices(vertices)
                    .with_vertices(tex_coords)
                    .with_indices(indices)
                    .into(),
                (),
                &world.fetch(),
            )
        };
        world.insert(HealthUiMesh(mesh));
    }
}

#[derive(Clone)]
pub struct EntityGraphics {
    pub material: Handle<Material>,
    pub mesh: Handle<Mesh>,
}

pub struct MultiplayerRoomState {
    pub nickname: String,
    pub is_active: bool,
    pub has_started: bool,
    pub has_sent_start_package: bool,
    pub server_addr: SocketAddr,
    pub is_host: bool,
    pub connection_id: NetIdentifier,
    pub player_net_id: NetIdentifier,
}

impl MultiplayerRoomState {
    pub fn new() -> Self {
        Self {
            nickname: "Player".to_owned(),
            is_active: false,
            has_started: false,
            has_sent_start_package: false,
            server_addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 0), 3455)),
            is_host: false,
            connection_id: 0,
            player_net_id: 0,
        }
    }
}

impl Default for MultiplayerRoomState {
    fn default() -> Self {
        Self::new()
    }
}
