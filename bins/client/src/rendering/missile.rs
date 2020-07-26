use amethyst::{
    core::{
        ecs::{DispatcherBuilder, Join, ReadStorage, SystemData, World},
        math::{convert, Matrix4, Vector4},
        transform::Transform,
    },
    error::Error,
    renderer::{
        bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
        pipeline::{PipelineDescBuilder, PipelinesBuilder},
        pod::IntoPod,
        rendy::{
            command::{QueueId, RenderPassEncoder},
            factory::Factory,
            graph::{
                render::{PrepareResult, RenderGroup, RenderGroupDesc},
                GraphContext, NodeBuffer, NodeImage,
            },
            hal::{self, device::Device, format::Format, pso},
            mesh::AsVertex,
            shader::{PathBufShaderInfo, Shader, ShaderKind, SourceLanguage, SpirvShader},
            util::types::vertex::VertexFormat,
        },
        submodules::{DynamicVertexBuffer, FlatEnvironmentSub},
        types::Backend,
        util,
    },
};
use derivative::Derivative;
use glsl_layout::{float, vec2, AsStd140};

use std::path::PathBuf;

use gv_core::ecs::{
    components::{missile::Missile, Dead},
    system_data::time::GameTimeService,
};
use gv_game::{ecs::systems::missile::MISSILE_TTL_SECS, utils::entities::missile_energy};

/// A [RenderPlugin] for drawing 2d objects with flat shading.
/// Required to display sprites defined with [SpriteRender] component.
#[derive(Default, Debug)]
pub struct MissilePlugin {
    target: Target,
}

impl<B: Backend> RenderPlugin<B> for MissilePlugin {
    fn on_build<'a, 'b>(
        &mut self,
        _world: &mut World,
        _builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(
                RenderOrder::AfterTransparent,
                DrawMissileDesc::new().builder(),
            )?;
            Ok(())
        });
        Ok(())
    }
}

lazy_static::lazy_static! {
    static ref VERTEX_SRC: SpirvShader = PathBufShaderInfo::new(
        PathBuf::from("resources/shaders/missile.vert"),
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
        PathBuf::from("resources/shaders/missile.frag"),
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

#[derive(Clone, Debug, PartialEq, Derivative)]
#[derivative(Default(bound = ""))]
pub struct DrawMissileDesc;

impl DrawMissileDesc {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for DrawMissileDesc {
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
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_sprite_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            vec![env.raw_layout()],
        )?;

        Ok(Box::new(DrawMissile::<B> {
            pipeline,
            pipeline_layout,
            env,
            vertex,
            missiles_count: 0,
        }))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(4))]
pub struct MissileVertexData {
    pub pos: vec2,
    pub seconds_since_spawn: float,
    pub opacity: float,
    /// Time to live (from 1.0 to 0.0).
    pub ttl: float,
}

impl AsVertex for MissileVertexData {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rg32Sfloat, "pos"),
            (Format::R32Sfloat, "seconds_since_spawn"),
            (Format::R32Sfloat, "opacity"),
            (Format::R32Sfloat, "ttl"),
        ))
    }
}

#[derive(Debug)]
pub struct DrawMissile<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: FlatEnvironmentSub<B>,
    vertex: DynamicVertexBuffer<B, MissileVertexData>,
    missiles_count: u32,
}

impl<B: Backend> RenderGroup<B, World> for DrawMissile<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        let (game_time_service, transforms, missiles, dead) = <(
            GameTimeService<'_>,
            ReadStorage<'_, Transform>,
            ReadStorage<'_, Missile>,
            ReadStorage<'_, Dead>,
        )>::fetch(world);

        self.env.process(factory, index, world);

        let vertices = (&transforms, &missiles, dead.maybe())
            .join()
            .map(|(transform, missile, dead)| {
                let transform = convert::<_, Matrix4<f32>>(*transform.global_matrix());
                let pos = (transform * Vector4::new(0.0, 0.0, 0.0, 1.0))
                    .xy()
                    .into_pod();
                let seconds_since_spawn = game_time_service.seconds_to_frame(missile.frame_spawned);
                let opacity = missile_energy(
                    &missile,
                    dead.map_or(false, |dead| {
                        dead.is_dead(game_time_service.game_frame_number())
                    }),
                    &game_time_service,
                    game_time_service.game_frame_number(),
                );
                let ttl = dead.map_or(1.0, |dead| {
                    1.0 - game_time_service
                        .seconds_to_frame(dead.dead_since_frame)
                        .clamp(0.0, MISSILE_TTL_SECS)
                        / MISSILE_TTL_SECS
                });
                MissileVertexData {
                    pos,
                    seconds_since_spawn,
                    opacity,
                    ttl,
                }
            })
            .collect::<Vec<_>>();

        self.missiles_count = vertices.len() as u32;
        self.vertex
            .write(factory, index, vertices.len() as u64, Some(vertices));

        PrepareResult::DrawRecord
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _world: &World,
    ) {
        if self.missiles_count > 0 {
            let layout = &self.pipeline_layout;
            encoder.bind_graphics_pipeline(&self.pipeline);
            self.env.bind(index, layout, 0, &mut encoder);
            self.vertex.bind(index, 0, 0, &mut encoder);
            unsafe {
                encoder.draw(0..4, 0..self.missiles_count);
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
                .with_vertex_desc(&[(
                    MissileVertexData::vertex(),
                    pso::VertexInputRate::Instance(1),
                )])
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
                    blend: Some(pso::BlendState::ALPHA),
                }])
                .with_depth_test(pso::DepthTest {
                    fun: pso::Comparison::Less,
                    write: false,
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
