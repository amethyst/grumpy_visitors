use amethyst::{
    assets::AssetStorage,
    core::ecs::{Join, Read, ReadStorage, SystemData, World},
    error::Error,
    renderer::{
        bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
        pipeline::{PipelineDescBuilder, PipelinesBuilder},
        pod::ViewArgs,
        rendy::{
            command::{QueueId, RenderPassEncoder},
            factory::Factory,
            graph::{
                render::{PrepareResult, RenderGroup, RenderGroupDesc},
                GraphContext, NodeBuffer, NodeImage,
            },
            hal::{self, device::Device, pso},
            mesh::{AsVertex, Position, TexCoord, VertexFormat},
            shader::{PathBufShaderInfo, Shader, ShaderKind, SourceLanguage, SpirvShader},
        },
        submodules::{gather::CameraGatherer, DynamicUniform},
        types::Backend,
        util, Mesh,
    },
};

use std::path::PathBuf;

use gv_client_shared::ecs::{components::HealthUiGraphics, resources::HealthUiMesh};
use gv_core::math::Vector2;

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

// TODO: hey mate, you don't use it.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct HealthUiArgs;

impl HealthUiArgs {
    fn vertex_formats() -> Vec<VertexFormat> {
        vec![TexCoord::vertex(), Position::vertex()]
    }
}

impl AsVertex for HealthUiArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((Position::vertex(), TexCoord::vertex()))
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
        }))
    }
}

#[derive(Debug)]
pub struct DrawHealthUi<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: DynamicUniform<B, ViewArgs>,
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

        PrepareResult::DrawRecord
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _: hal::pass::Subpass<'_, B>,
        world: &World,
    ) {
        let (mesh_storage, health_ui_mesh_handle, health_ui_graphics) = <(
            Read<'_, AssetStorage<Mesh>>,
            Option<Read<'_, HealthUiMesh>>,
            ReadStorage<'_, HealthUiGraphics>,
        )>::fetch(world);
        if health_ui_mesh_handle.is_none() {
            return;
        }

        let layout = &self.pipeline_layout;

        encoder.bind_graphics_pipeline(&self.pipeline);
        self.env.bind(index, layout, 0, &mut encoder);

        let mesh_id = health_ui_mesh_handle.as_ref().unwrap().0.id();
        if let Some(mesh) = B::unwrap_mesh(unsafe { mesh_storage.get_by_id_unchecked(mesh_id) }) {
            mesh.bind(0, &HealthUiArgs::vertex_formats(), &mut encoder)
                .expect("Expected to bind a Mesh");
            for health_ui_graphics in (health_ui_graphics).join() {
                let constant = HealthUiVertPushConstant {
                    translation: health_ui_graphics.screen_position,
                    scale: health_ui_graphics.scale_ratio,
                };
                let push_constants: [u32; 3] = unsafe { std::mem::transmute(constant) };
                unsafe {
                    encoder.push_constants(
                        layout,
                        pso::ShaderStageFlags::VERTEX,
                        0,
                        &push_constants,
                    );
                }

                let constant = HealthUiFragPushConstant {
                    health: health_ui_graphics.health,
                };
                let push_constants: [u32; 1] = unsafe { std::mem::transmute(constant) };
                unsafe {
                    encoder.push_constants(
                        layout,
                        pso::ShaderStageFlags::FRAGMENT,
                        std::mem::size_of::<HealthUiVertPushConstant>() as u32,
                        &push_constants,
                    );
                }

                unsafe {
                    encoder.draw_indexed(0..mesh.len(), 0, 0..1);
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

fn build_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error> {
    let push_constants = vec![
        (pso::ShaderStageFlags::VERTEX, 0..4),
        (pso::ShaderStageFlags::FRAGMENT, 0..4),
    ];

    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, push_constants)
    }?;

    let shader_vertex = unsafe { VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { FRAGMENT.module(factory).unwrap() };

    let vertex_desc = HealthUiArgs::vertex_formats()
        .iter()
        .map(|f| (f.clone(), pso::VertexInputRate::Vertex))
        .collect::<Vec<_>>();

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&vertex_desc)
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleList))
                .with_rasterizer(hal::pso::Rasterizer {
                    polygon_mode: hal::pso::PolygonMode::Fill,
                    cull_face: hal::pso::Face::NONE,
                    front_face: hal::pso::FrontFace::Clockwise,
                    depth_clamping: false,
                    depth_bias: None,
                    conservative: false,
                })
                .with_shaders(util::simple_shader_set(
                    &shader_vertex,
                    Some(&shader_fragment),
                ))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_baked_states(hal::pso::BakedStates {
                    viewport: Some(hal::pso::Viewport {
                        rect: hal::pso::Rect {
                            x: 0,
                            y: 0,
                            w: framebuffer_width as i16,
                            h: framebuffer_height as i16,
                        },
                        depth: 0.0..1.0,
                    }),
                    scissor: None,
                    ..Default::default()
                })
                .with_blend_targets(vec![pso::ColorBlendDesc(
                    pso::ColorMask::ALL,
                    pso::BlendState::ALPHA,
                )])
                .with_depth_test(pso::DepthTest::On {
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HealthUiVertPushConstant {
    translation: Vector2,
    scale: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HealthUiFragPushConstant {
    health: f32,
}
