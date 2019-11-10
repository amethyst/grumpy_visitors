use amethyst::{
    assets::AssetStorage,
    core::{
        ecs::{DispatcherBuilder, Join, Read, ReadExpect, ReadStorage, SystemData, World},
        transform::Transform,
        Parent,
    },
    error::Error,
    renderer::{
        batch::{GroupIterator, OneLevelBatch, OrderedOneLevelBatch},
        bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
        pipeline::{PipelineDescBuilder, PipelinesBuilder},
        pod::SpriteArgs,
        rendy::{
            command::{QueueId, RenderPassEncoder},
            factory::Factory,
            graph::{
                render::{PrepareResult, RenderGroup, RenderGroupDesc},
                GraphContext, NodeBuffer, NodeImage,
            },
            hal::{self, device::Device, pso},
            mesh::AsVertex,
            shader::{PathBufShaderInfo, Shader, ShaderKind, SourceLanguage, SpirvShader},
        },
        resources::Tint,
        sprite::{SpriteRender, SpriteSheet},
        sprite_visibility::SpriteVisibility,
        submodules::{DynamicVertexBuffer, FlatEnvironmentSub, TextureId, TextureSub},
        types::{Backend, Texture},
        util,
    },
};
use derivative::Derivative;

use std::path::PathBuf;

use gv_core::ecs::components::Player;

use crate::ecs::systems::{CustomSpriteSortingSystem, SpriteOrdering};

/// A [RenderPlugin] for drawing 2d objects with flat shading.
/// Required to display sprites defined with [SpriteRender] component.
#[derive(Default, Debug)]
pub struct PaintMagePlugin {
    target: Target,
}

impl<B: Backend> RenderPlugin<B> for PaintMagePlugin {
    fn on_build<'a, 'b>(
        &mut self,
        _world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(
            CustomSpriteSortingSystem::new(),
            "custom_sprite_sorting_system",
            &[],
        );
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Opaque, DrawFlat2DDesc::new().builder())?;
            ctx.add(
                RenderOrder::Transparent,
                DrawFlat2DTransparentDesc::new().builder(),
            )?;
            Ok(())
        });
        Ok(())
    }
}

lazy_static::lazy_static! {
    static ref VERTEX_SRC: SpirvShader = PathBufShaderInfo::new(
        PathBuf::from("resources/shaders/paint_mage.vert"),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref VERTEX: SpirvShader = SpirvShader::new(
        (*VERTEX_SRC).spirv().unwrap().to_vec(),
        (*VERTEX_SRC).stage(),
        "main",
    );

    static ref FRAGMENT_SRC: SpirvShader = PathBufShaderInfo::new(
        PathBuf::from("resources/shaders/paint_mage.frag"),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref FRAGMENT: SpirvShader = SpirvShader::new(
        (*FRAGMENT_SRC).spirv().unwrap().to_vec(),
        (*FRAGMENT_SRC).stage(),
        "main",
    );
}

/// Draw opaque sprites without lighting.
#[derive(Clone, Debug, PartialEq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawFlat2DDesc;

impl DrawFlat2DDesc {
    /// Create instance of `DrawFlat2D` render group
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for DrawFlat2DDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {
        let env = FlatEnvironmentSub::new(factory)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_sprite_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            false,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        Ok(Box::new(DrawFlat2D::<B> {
            pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            sprites: Default::default(),
        }))
    }
}

/// Draws opaque 2D sprites to the screen without lighting.
#[derive(Debug)]
pub struct DrawFlat2D<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, SpriteArgs>,
    sprites: OneLevelBatch<TextureId, SpriteArgs>,
}

impl<B: Backend> RenderGroup<B, World> for DrawFlat2D<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        let (sprite_sheet_storage, tex_storage, visibility, sprite_renders, transforms, tints) =
            <(
                Read<'_, AssetStorage<SpriteSheet>>,
                Read<'_, AssetStorage<Texture>>,
                ReadExpect<'_, SpriteVisibility>,
                ReadStorage<'_, SpriteRender>,
                ReadStorage<'_, Transform>,
                ReadStorage<'_, Tint>,
            )>::fetch(world);

        self.env.process(factory, index, world);

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;

        sprites_ref.clear_inner();

        {
            (
                &sprite_renders,
                &transforms,
                tints.maybe(),
                &visibility.visible_unordered,
            )
                .join()
                .filter_map(|(sprite_render, global, tint, _)| {
                    let (batch_data, texture) = SpriteArgs::from_data(
                        &tex_storage,
                        &sprite_sheet_storage,
                        &sprite_render,
                        &global,
                        tint,
                    )?;
                    let (tex_id, _) = textures_ref.insert(
                        factory,
                        world,
                        texture,
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )?;
                    Some((tex_id, batch_data))
                })
                .for_each_group(|tex_id, batch_data| {
                    sprites_ref.insert(tex_id, batch_data.drain(..))
                });
        }

        self.textures.maintain(factory, world);

        {
            sprites_ref.prune();
            self.vertex.write(
                factory,
                index,
                self.sprites.count() as u64,
                self.sprites.data(),
            );
        }

        PrepareResult::DrawRecord
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _world: &World,
    ) {
        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);
        self.vertex.bind(index, 0, 0, &mut encoder);
        for (&tex, range) in self.sprites.iter() {
            if self.textures.loaded(tex) {
                self.textures.bind(layout, 1, tex, &mut encoder);
                unsafe {
                    encoder.draw(0..4, range);
                }
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _world: &World) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}
/// Describes drawing transparent sprites without lighting.
#[derive(Clone, Debug, PartialEq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawFlat2DTransparentDesc;

impl DrawFlat2DTransparentDesc {
    /// Create instance of `DrawFlat2D` render group
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for DrawFlat2DTransparentDesc {
    fn build(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _world: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {
        let env = FlatEnvironmentSub::new(factory)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_sprite_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            true,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        Ok(Box::new(DrawFlat2DTransparent::<B> {
            pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            sprites: Default::default(),
            change: Default::default(),
        }))
    }
}

/// Draws transparent sprites without lighting.
#[derive(Debug)]
pub struct DrawFlat2DTransparent<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, SpriteArgs>,
    sprites: OrderedOneLevelBatch<TextureId, SpriteArgs>,
    change: util::ChangeDetection,
}

impl<B: Backend> RenderGroup<B, World> for DrawFlat2DTransparent<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        let (
            sprite_sheet_storage,
            tex_storage,
            sprite_ordering,
            sprite_renders,
            transforms,
            players,
            parents,
        ) = <(
            Read<'_, AssetStorage<SpriteSheet>>,
            Read<'_, AssetStorage<Texture>>,
            ReadExpect<'_, SpriteOrdering>,
            ReadStorage<'_, SpriteRender>,
            ReadStorage<'_, Transform>,
            ReadStorage<'_, Player>,
            ReadStorage<'_, Parent>,
        )>::fetch(world);

        self.env.process(factory, index, world);
        self.sprites.swap_clear();
        let mut changed = false;

        let sprites_ref = &mut self.sprites;
        let textures_ref = &mut self.textures;

        {
            let mut joined = (&sprite_renders, &transforms, &parents).join();

            sprite_ordering
                .0
                .iter()
                .filter_map(|e| joined.get_unchecked(e.id()))
                .filter_map(|(sprite_render, global, parent)| {
                    if !players.contains(parent.entity) {
                        return None;
                    }

                    let (batch_data, texture) = SpriteArgs::from_data(
                        &tex_storage,
                        &sprite_sheet_storage,
                        &sprite_render,
                        &global,
                        None,
                    )?;
                    let (tex_id, this_changed) = textures_ref.insert(
                        factory,
                        world,
                        texture,
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )?;
                    changed = changed || this_changed;
                    Some((tex_id, batch_data))
                })
                .for_each_group(|tex_id, batch_data| {
                    sprites_ref.insert(tex_id, batch_data.drain(..));
                });
        }

        self.textures.maintain(factory, world);
        changed = changed || self.sprites.changed();

        {
            self.vertex.write(
                factory,
                index,
                self.sprites.count() as u64,
                Some(self.sprites.data()),
            );
        }

        self.change.prepare_result(index, changed)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _world: &World,
    ) {
        let layout = &self.pipeline_layout;
        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);
        self.vertex.bind(index, 0, 0, &mut encoder);
        for (&tex, range) in self.sprites.iter() {
            if self.textures.loaded(tex) {
                self.textures.bind(layout, 1, tex, &mut encoder);
                unsafe {
                    encoder.draw(0..4, range);
                }
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &World) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}

fn build_sprite_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    transparent: bool,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    let shader_vertex = unsafe { VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { FRAGMENT.module(factory).unwrap() };

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(SpriteArgs::vertex(), pso::VertexInputRate::Instance(1))])
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleStrip))
                .with_shaders(util::simple_shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: if transparent {
                        Some(pso::BlendState::PREMULTIPLIED_ALPHA)
                    } else {
                        None
                    },
                }])
                .with_depth_test(pso::DepthTest {
                    fun: pso::Comparison::Less,
                    write: !transparent,
                }),
        )
        .build(factory, None);

    unsafe {
        factory.destroy_shader_module(shader_vertex);
        factory.destroy_shader_module(shader_fragment);
    }

    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(pipeline_layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), pipeline_layout)),
    }
}
