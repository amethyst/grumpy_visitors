use amethyst::{
    ecs::{ReadExpect, Resources, SystemData},
    renderer::{
        pass::{DrawFlat2DDesc, DrawFlat2DTransparentDesc, DrawFlatDesc},
        rendy::{
            graph::present::PresentNode,
            hal::command::{ClearDepthStencil, ClearValue},
        },
        types::DefaultBackend,
        Factory, Format, GraphBuilder, GraphCreator, Kind, RenderGroupDesc, SubpassBuilder,
    },
    ui::DrawUiDesc,
    window::{ScreenDimensions, Window},
};

use std::ops::Deref;

use crate::render_groups::DrawHealthUiDesc;

#[derive(Default)]
pub struct RenderGraph {
    dimensions: Option<ScreenDimensions>,
    dirty: bool,
}

impl GraphCreator<DefaultBackend> for RenderGraph {
    fn rebuild(&mut self, res: &Resources) -> bool {
        let new_dimensions = res.try_fetch::<ScreenDimensions>();
        if self.dimensions.as_ref() != new_dimensions.as_ref().map(std::ops::Deref::deref) {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.deref().clone());
            false
        } else {
            self.dirty
        }
    }

    fn builder(
        &mut self,
        factory: &mut Factory<DefaultBackend>,
        res: &Resources,
    ) -> GraphBuilder<DefaultBackend, Resources> {
        self.dirty = false;

        let window = <ReadExpect<'_, Window>>::fetch(res);
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind = Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);

        let surface = factory.create_surface(&window);
        let surface_format = factory.get_surface_format(&surface);

        let mut graph_builder = GraphBuilder::new();
        let color = graph_builder.create_image(
            window_kind,
            1,
            surface_format,
            Some(ClearValue::Color([0.05, 0.05, 0.05, 1.0].into())),
        );
        let depth = graph_builder.create_image(
            window_kind,
            1,
            Format::D32Sfloat,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        let pass = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawFlat2DDesc::new().builder())
                .with_group(DrawFlatDesc::new().builder())
                .with_group(DrawHealthUiDesc::new().builder())
                .with_group(DrawFlat2DTransparentDesc::new().builder())
                .with_group(DrawUiDesc::new().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );
        graph_builder.add_node(PresentNode::builder(factory, surface, color).with_dependency(pass));

        graph_builder
    }
}
