use amethyst::{
    core::ecs::{Join, ReadStorage, SystemData, World},
    error::Error,
    renderer::{
        bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
        pipeline::{PipelineDescBuilder, PipelinesBuilder},
        pod::{IntoPod, ViewArgs},
        rendy::{
            command::{QueueId, RenderPassEncoder},
            factory::Factory,
            graph::{
                render::{PrepareResult, RenderGroup, RenderGroupDesc},
                GraphContext, NodeBuffer, NodeImage,
            },
            hal::{self, device::Device, format::Format, pso},
            mesh::{AsVertex, VertexFormat},
            shader::{PathBufShaderInfo, Shader, ShaderKind, SourceLanguage, SpirvShader},
        },
        submodules::{
            gather::CameraGatherer, DynamicIndexBuffer, DynamicUniform, DynamicVertexBuffer,
        },
        types::Backend,
        util,
    },
};
use glsl_layout::{float, vec2, vec3, AsStd140};

use std::path::PathBuf;

use gv_client_shared::{
    ecs::components::HealthUiGraphics, utils::graphic_helpers::generate_rectangle_vertices,
};
use gv_core::math::Vector3;

#[derive(Default, Debug)]
pub struct HealthUiPlugin {
    target: Target,
}

impl<B: Backend> RenderPlugin<B> for HealthUiPlugin {
    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Overlay, DrawHealthUiDesc::new().builder())?;
            Ok(())
        });
        Ok(())
    }
}

lazy_static::lazy_static! {
    static ref VERTEX_SRC: SpirvShader = PathBufShaderInfo::new(
        PathBuf::from("resources/shaders/health_ui.vert"),
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
        PathBuf::from("resources/shaders/health_ui.frag"),
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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
#[repr(C, align(4))]
pub struct HealthUiVertexData {
    pub uv: vec2,
    pub position: vec3,
    pub translation: vec2,
    pub scale: float,
    pub health: float,
}

impl AsVertex for HealthUiVertexData {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rg32Sfloat, "uv"),
            (Format::Rgb32Sfloat, "position"),
            (Format::Rg32Sfloat, "translation"),
            (Format::R32Sfloat, "scale"),
            (Format::R32Sfloat, "health"),
        ))
    }
}

#[derive(Debug)]
pub struct DrawHealthUiDesc;

impl DrawHealthUiDesc {
    pub fn new() -> Self {
        Self
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for DrawHealthUiDesc {
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
        let env = DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX)?;

        let (pipeline, pipeline_layout) = build_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            vec![env.raw_layout()],
        )?;

        Ok(Box::new(DrawHealthUi::<B> {
            pipeline,
            pipeline_layout,
            env,
            vertex: DynamicVertexBuffer::new(),
            index: DynamicIndexBuffer::new(),
            indices_count: 0,
            players_count: 0,
        }))
    }
}

#[derive(Debug)]
pub struct DrawHealthUi<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: DynamicUniform<B, ViewArgs>,
    vertex: DynamicVertexBuffer<B, HealthUiVertexData>,
    index: DynamicIndexBuffer<B, u16>,
    indices_count: u32,
    players_count: u32,
}

impl<B: Backend> RenderGroup<B, World> for DrawHealthUi<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        world: &World,
    ) -> PrepareResult {
        let camera = CameraGatherer::gather(world);
        self.env.write(factory, index, camera.projview);

        let health_ui_graphics = <ReadStorage<'_, HealthUiGraphics>>::fetch(world);

        let mut vertices = Vec::new();
        let (positions, uv, indices) = generate_rectangle_vertices(
            Vector3::new(0.0, 0.0, 100.0),
            Vector3::new(180.0, 180.0, 100.0),
        );

        let mut vertices_data = positions.iter().zip(uv.iter()).cycle();
        for health_ui_graphics in (&health_ui_graphics).join() {
            for (position, uv) in vertices_data.by_ref().take(positions.len()) {
                vertices.push(HealthUiVertexData {
                    uv: uv.0.into(),
                    position: position.0.into(),
                    translation: health_ui_graphics.screen_position.into_pod(),
                    scale: health_ui_graphics.scale_ratio,
                    health: health_ui_graphics.health,
                });
            }
        }

        self.indices_count = indices.len() as u32;
        self.players_count = health_ui_graphics.count() as u32;
        self.vertex
            .write(factory, index, vertices.len() as u64, Some(vertices));
        self.index
            .write(factory, index, indices.len() as u64, Some(indices));

        PrepareResult::DrawRecord
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _: hal::pass::Subpass<'_, B>,
        _: &World,
    ) {
        if self.players_count > 0 {
            let layout = &self.pipeline_layout;
            encoder.bind_graphics_pipeline(&self.pipeline);
            self.env.bind(index, layout, 0, &mut encoder);
            self.index.bind(index, 0, &mut encoder);
            self.vertex.bind(index, 0, 0, &mut encoder);

            unsafe {
                encoder.draw_indexed(0..self.indices_count, 0, 0..self.players_count);
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

fn build_pipeline<B: Backend>(
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
                    HealthUiVertexData::vertex(),
                    pso::VertexInputRate::Instance(1),
                )])
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleList))
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
