use amethyst::{
    animation::{
        AnimationPrefab, AnimationSetPrefab, InterpolationFunction, Sampler, SpriteRenderChannel,
        SpriteRenderPrimitive,
    },
    assets::Prefab,
    core::{transform::Transform, Named},
    renderer::{
        formats::texture::ImageFormat,
        sprite::{
            prefab::{
                SpriteRenderPrefab, SpriteScenePrefab, SpriteSheetPrefab, SpriteSheetReference,
            },
            SpriteList, SpritePosition, Sprites,
        },
        TexturePrefab, Transparent,
    },
};
use failure;
use image;
use ron::ser::{to_string_pretty, PrettyConfig};
use serde_derive::{Deserialize, Serialize};
use texture_packer::{
    exporter::ImageExporter, importer::ImageImporter, texture::Texture, Frame, TexturePacker,
    TexturePackerConfig,
};

use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

use gv_animation_prefabs::{AnimationId, GameSpriteAnimationPrefab};

struct SpriteSceneData {
    sprite_sheet: SpriteSheetPrefab,
    indices: HashMap<String, Vec<SpriteRenderPrimitive>>,
}

#[derive(Serialize, Deserialize)]
struct SpritePrefab {
    entities: Vec<SpritePrefabEntity>,
}

#[derive(Serialize, Deserialize)]
struct SpritePrefabEntity {
    name_tag: String,
    z_translation: Option<f32>,
    animations: Vec<AnimationDefinition>,
}

#[derive(Serialize, Deserialize, Clone)]
struct AnimationDefinition {
    animation_id: AnimationId,
    name_prefix: String,
    directory: String,
    center_x: f32,
    center_y: f32,
    #[serde(default)]
    frames_count: usize,
    #[serde(default = "default_interpolation_step")]
    interpolation_step: f32,
}

fn default_interpolation_step() -> f32 {
    1.0 / 60.0
}

struct FramesMap(BTreeMap<String, Frame>);

fn main() -> Result<(), failure::Error> {
    let config = TexturePackerConfig {
        allow_rotation: false,
        ..Default::default()
    };

    let mut packer = TexturePacker::new_skyline(config);

    let import_path = match env::var("GRUMPY_IMPORT_PATH") {
        Ok(path) => path,
        Err(env::VarError::NotPresent) => "assets_packer/input".to_owned(),
        Err(env::VarError::NotUnicode(_)) => panic!("GRUMPY_IMPORT_PATH value is invalid"),
    };
    let import_path = Path::new(&import_path);

    let prefabs_file = File::open(import_path.join("prefabs.ron"))
        .expect("Expected prefabs.ron file at import path");
    let prefabs: HashMap<String, SpritePrefab> =
        ron::de::from_reader(prefabs_file).expect("Failed to parse prefab definition files");

    for sprite_prefab in prefabs.values() {
        for prefab_entity in &sprite_prefab.entities {
            for animation_definition in &prefab_entity.animations {
                for i in 1..=animation_definition.frames_count {
                    let name = format!("{}_{:04}.png", animation_definition.name_prefix, i);
                    let path = import_path
                        .join(&animation_definition.directory)
                        .join(&name);
                    let texture = ImageImporter::import_from_file(&path).unwrap();

                    packer.pack_own(name, texture);
                }
            }
        }
    }

    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("output");
    let output_file = output_dir.join("atlas.png");

    let frames_map = FramesMap(
        packer
            .get_frames()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    );

    let SpriteSceneData {
        sprite_sheet,
        indices,
    } = construct_sprite_scene(
        &prefabs,
        &frames_map,
        packer.width(),
        packer.height(),
        &output_file,
    );

    fs::create_dir_all(&output_dir).unwrap();

    // Creating atlas file.
    let exporter = ImageExporter::export(&packer).unwrap();
    let mut file = File::create(output_file)?;
    exporter.write_to(&mut file, image::PNG)?;

    // Creating prefab files.
    for (prefab_name, output_prefab) in create_prefabs(prefabs, sprite_sheet, indices) {
        let ron_metadata = to_string_pretty(
            &output_prefab,
            PrettyConfig {
                new_line: "\n".to_owned(),
                ..PrettyConfig::default()
            },
        )?;
        let mut file = File::create(output_dir.join(format!("{}.ron", prefab_name)))?;
        file.write_all(ron_metadata.as_bytes())?;
    }
    Ok(())
}

fn construct_sprite_scene(
    prefabs: &HashMap<String, SpritePrefab>,
    frames: &FramesMap,
    atlas_width: u32,
    atlas_height: u32,
    output_file_path: impl AsRef<Path>,
) -> SpriteSceneData {
    let frames = &frames.0;
    let mut indices = HashMap::new();
    let mut sprites = Vec::new();

    let mut indexed_animation_definitions = HashMap::new();
    for (prefab_name, sprite_prefab) in prefabs {
        for prefab_entity in &sprite_prefab.entities {
            for animation_definition in &prefab_entity.animations {
                indexed_animation_definitions.insert(
                    animation_definition.name_prefix.clone(),
                    (prefab_name.clone(), animation_definition.clone()),
                );
            }
        }
    }

    for (sprite_index, (filename, frame)) in frames.iter().enumerate() {
        let name_prefix = &filename[0..filename.len() - 9];
        let (_prefab_name, animation_definition) = indexed_animation_definitions
            .get_mut(name_prefix)
            .expect("Expected an indexed AnimationDefinition");

        let sprite_center_x = frame.source.w as f32 - animation_definition.center_x;
        let sprite_center_y = animation_definition.center_y;
        let cropped_source_x = (frame.source.w - frame.source.x - frame.frame.w) as f32;
        let cropped_source_y = frame.source.y as f32;
        // Revert amethyst center aligning, apply sprite center taking into account frame cropping.
        let offsets = Some([
            frame.frame.w as f32 / 2.0 - sprite_center_x + cropped_source_x,
            frame.frame.h as f32 / 2.0 - sprite_center_y + cropped_source_y,
        ]);

        sprites.push(SpritePosition {
            x: frame.frame.x,
            y: frame.frame.y,
            width: frame.frame.w,
            height: frame.frame.h,
            offsets,
            flip_horizontal: false,
            flip_vertical: false,
        });

        let number = &filename[filename.len() - 8..filename.len() - 4];
        let i = number
            .parse::<usize>()
            .expect("Expected a PNG file with 4 digit index in the filename")
            - 1;
        indices.entry(name_prefix.to_owned()).or_insert_with(|| {
            vec![SpriteRenderPrimitive::SpriteIndex(0); animation_definition.frames_count]
        })[i] = SpriteRenderPrimitive::SpriteIndex(sprite_index);
    }

    let sprite_sheet = SpriteSheetPrefab::Sheet {
        texture: TexturePrefab::File(
            output_file_path.as_ref().to_str().unwrap().to_owned(),
            Box::new(ImageFormat::default()),
        ),
        sprites: vec![Sprites::List(SpriteList {
            texture_width: atlas_width,
            texture_height: atlas_height,
            sprites,
        })],
        name: Some("atlas".to_owned()),
    };

    SpriteSceneData {
        sprite_sheet,
        indices,
    }
}

fn create_prefabs(
    prefabs: HashMap<String, SpritePrefab>,
    sprite_sheet: SpriteSheetPrefab,
    indices: HashMap<String, Vec<SpriteRenderPrimitive>>,
) -> HashMap<String, Prefab<GameSpriteAnimationPrefab>> {
    let mut prefabs: HashMap<String, Prefab<GameSpriteAnimationPrefab>> = prefabs
        .into_iter()
        .map(|(prefab_name, sprite_prefab)| {
            let mut prefab = Prefab::new();
            for prefab_entity in sprite_prefab.entities {
                prefab.add(
                    Some(0),
                    Some(GameSpriteAnimationPrefab {
                        name_tag: Named::new(prefab_entity.name_tag),
                        sprite_scene: SpriteScenePrefab {
                            sheet: None,
                            render: Some(SpriteRenderPrefab::new(
                                Some(SpriteSheetReference::Name("atlas".to_owned())),
                                0,
                            )),
                            transform: Some(prefab_entity.z_translation.map_or_else(
                                Transform::default,
                                |z_translation| {
                                    let mut transform = Transform::default();
                                    transform.set_translation_z(z_translation);
                                    transform
                                },
                            )),
                        },
                        animation_set: AnimationSetPrefab {
                            animations: prefab_entity
                                .animations
                                .into_iter()
                                .map(|animation| {
                                    (animation.animation_id, {
                                        let mut animation_prefab = AnimationPrefab::default();
                                        let output = indices[&animation.name_prefix].clone();
                                        animation_prefab.samplers = vec![(
                                            0,
                                            SpriteRenderChannel::SpriteIndex,
                                            Sampler {
                                                input: (0..animation.frames_count)
                                                    .map(|i| {
                                                        i as f32 * animation.interpolation_step
                                                    })
                                                    .collect(),
                                                output,
                                                function: InterpolationFunction::Step,
                                            },
                                        )];
                                        animation_prefab
                                    })
                                })
                                .collect(),
                        },
                        transparent: Transparent,
                    }),
                );
            }
            (prefab_name, prefab)
        })
        .collect();

    prefabs.insert(
        "dummy".to_owned(),
        Prefab::new_main(GameSpriteAnimationPrefab {
            name_tag: Named::new(String::new()),
            sprite_scene: SpriteScenePrefab {
                sheet: Some(sprite_sheet),
                render: None,
                transform: None,
            },
            animation_set: AnimationSetPrefab {
                animations: Vec::new(),
            },
            transparent: Transparent,
        }),
    );
    prefabs
}
