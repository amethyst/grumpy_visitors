use amethyst::{
    core::{
        ecs::prelude::{Entities, Entity, Join, Read, ReadStorage, System, Write},
        math::{Point3, Vector3},
        Parent, Transform,
    },
    renderer::camera::{ActiveCamera, Camera},
};
use derivative::Derivative;

use gv_core::ecs::components::Player;
use std::cmp::Ordering;

#[derive(Default, Debug)]
pub struct SpriteOrdering(pub Vec<Entity>);

/// Determines what entities to be drawn. Will also sort transparent entities back to front based on
/// position on the Z axis.
///
/// The sprite render pass should draw all sprites without semi-transparent pixels, then draw the
/// sprites with semi-transparent pixels from far to near.
///
/// Note that this should run after `Transform` has been updated for the current frame, and
/// before rendering occurs.
#[derive(Derivative)]
#[derivative(Default(bound = ""), Debug(bound = ""))]
pub struct CustomSpriteSortingSystem {
    centroids: Vec<Internals>,
}

#[derive(Debug, Clone)]
struct Internals {
    entity: Entity,
    centroid: Point3<f32>,
    camera_distance: f32,
    from_camera: Vector3<f32>,
}

impl CustomSpriteSortingSystem {
    /// Returns a new sprite visibility sorting system
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> System<'a> for CustomSpriteSortingSystem {
    type SystemData = (
        Entities<'a>,
        Write<'a, SpriteOrdering>,
        Read<'a, ActiveCamera>,
        ReadStorage<'a, Camera>,
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Parent>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut sprite_ordering,
            active,
            camera,
            transform,
            players,
            parents,
        ): Self::SystemData,
    ) {
        let origin = Point3::origin();

        // The camera position is used to determine culling, but the sprites are ordered based on
        // the Z coordinate
        let camera: Option<&Transform> = active
            .entity
            .and_then(|a| transform.get(a))
            .or_else(|| (&camera, &transform).join().map(|ct| ct.1).next());
        let camera_backward = camera
            .map(|c| c.global_matrix().column(2).xyz())
            .unwrap_or_else(Vector3::z);
        let camera_centroid = camera
            .map(|t| t.global_matrix().transform_point(&origin))
            .unwrap_or_else(|| origin);

        self.centroids.clear();
        self.centroids.extend(
            (&*entities, &transform, &parents)
                .join()
                .filter(|(_, _, parent)| players.contains(parent.entity))
                .map(|(e, t, _)| (e, t.global_matrix().transform_point(&origin)))
                // filter entities behind the camera
                .filter(|(_, c)| (c - camera_centroid).dot(&camera_backward) < 0.0)
                .map(|(entity, centroid)| Internals {
                    entity,
                    centroid,
                    camera_distance: (centroid.z - camera_centroid.z).abs(),
                    from_camera: centroid - camera_centroid,
                }),
        );

        // Note: Smaller Z values are placed first, so that semi-transparent sprite colors blend
        // correctly.
        self.centroids.sort_by(|a, b| {
            b.camera_distance
                .partial_cmp(&a.camera_distance)
                .unwrap_or(Ordering::Equal)
        });

        sprite_ordering.0.clear();
        sprite_ordering
            .0
            .extend(self.centroids.iter().map(|c| c.entity));
    }
}
