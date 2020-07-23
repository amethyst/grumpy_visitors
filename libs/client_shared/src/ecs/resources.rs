use amethyst::{
    assets::{Handle, Loader, Prefab},
    prelude::World,
    renderer::{rendy::mesh::MeshBuilder, Material, Mesh, SpriteSheet},
    ui::FontHandle,
};

use std::{io, time::Instant};

use gv_animation_prefabs::GameSpriteAnimationPrefab;
use gv_core::{
    math::Vector3,
    net::{server_message::DisconnectReason, NetIdentifier},
};

use crate::utils::graphic_helpers::generate_rectangle_vertices;

pub const HEALTH_UI_SCREEN_PADDING: f32 = 40.0;

pub struct DummyAssetHandles {
    pub dummy_prefab: Handle<Prefab<GameSpriteAnimationPrefab>>,
}

#[derive(Clone)]
pub struct AssetHandles {
    pub mage_prefab: Handle<Prefab<GameSpriteAnimationPrefab>>,
    pub beetle_prefab: Handle<Prefab<GameSpriteAnimationPrefab>>,
    pub landscape: Handle<SpriteSheet>,
    pub ui_font: FontHandle,
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
    pub is_active: bool,
    pub is_host: bool,
    pub connection_status: ConnectionStatus,
    pub player_net_id: NetIdentifier,
}

impl MultiplayerRoomState {
    pub fn new() -> Self {
        Self {
            is_active: false,
            is_host: false,
            connection_status: ConnectionStatus::NotConnected,
            player_net_id: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = MultiplayerRoomState::new();
    }
}

impl Default for MultiplayerRoomState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum ConnectionStatus {
    NotConnected,
    Connecting(Instant),
    Connected(NetIdentifier),
    Disconnecting,
    Disconnected(DisconnectReason),
    ServerStartFailed,
    ConnectionFailed(Option<io::Error>),
}

impl ConnectionStatus {
    pub fn is_not_connected(&self) -> bool {
        matches!(self, ConnectionStatus::NotConnected
            | ConnectionStatus::Disconnected(_)
            | ConnectionStatus::ConnectionFailed(_)
            | ConnectionStatus::ServerStartFailed)
    }

    pub fn is_connecting(&self) -> bool {
        matches!(self, ConnectionStatus::Connecting(_))
    }

    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionStatus::Connected(_))
    }

    pub fn connection_id(&self) -> Option<NetIdentifier> {
        if let ConnectionStatus::Connected(connection_id) = self {
            Some(*connection_id)
        } else {
            None
        }
    }
}
