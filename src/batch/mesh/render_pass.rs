use batch::mesh::{ ALBEDO_FORMAT, NORMAL_FORMAT, DEPTH_FORMAT, MeshShaders, TargetVertex, mesh::MeshVertexDefinition };
use std::sync::Arc;
use vulkano::{
	format::Format,
	framebuffer::{ RenderPassAbstract, Subpass },
	pipeline::{ GraphicsPipeline, GraphicsPipelineAbstract },
};

pub struct MeshRenderPass {
	pub(super) shaders: Arc<MeshShaders>,
	pub(super) subpass_gbuffers: Subpass<Arc<RenderPassAbstract + Send + Sync>>,
	pub(super) subpass_target: Subpass<Arc<RenderPassAbstract + Send + Sync>>,
	pub(super) pipeline_gbuffers: Arc<GraphicsPipelineAbstract + Send + Sync + 'static>,
	pub(super) pipeline_target: Arc<GraphicsPipelineAbstract + Send + Sync + 'static>,
}
impl MeshRenderPass {
	pub fn new(shaders: Arc<MeshShaders>, format: Format) -> Arc<Self> {
		let render_pass: Arc<RenderPassAbstract + Send + Sync> =
			Arc::new(
				ordered_passes_renderpass!(
					shaders.target_vertices.device().clone(),
					attachments: {
						color: { load: Clear, store: Store, format: ALBEDO_FORMAT, samples: 1, },
						normal: { load: Clear, store: Store, format: NORMAL_FORMAT, samples: 1, },
						depth: { load: Clear, store: Store, format: DEPTH_FORMAT, samples: 1, },
						out: { load: Clear, store: Store, format: format, samples: 1, }
					},
					passes: [
						{ color: [color, normal], depth_stencil: {depth}, input: [] },
						{ color: [out], depth_stencil: {}, input: [color, normal, depth] }
					]
				)
				.unwrap()
			);

		let subpass_gbuffers = Subpass::from(render_pass.clone(), 0).unwrap();
		let subpass_target = Subpass::from(render_pass, 1).unwrap();

		let pipeline_gbuffers =
			Arc::new(
				GraphicsPipeline::start()
					.vertex_input(MeshVertexDefinition::new())
					.vertex_shader(shaders.shader_gbuffers_vertex.main_entry_point(), ())
					.triangle_list()
					.viewports_dynamic_scissors_irrelevant(1)
					.fragment_shader(shaders.shader_gbuffers_fragment.main_entry_point(), ())
					.render_pass(subpass_gbuffers.clone())
					.depth_stencil_simple_depth()
					.build(shaders.target_vertices.device().clone())
					.expect("failed to create pipeline")
			);

		let pipeline_target =
			Arc::new(
				GraphicsPipeline::start()
					.vertex_input_single_buffer::<TargetVertex>()
					.vertex_shader(shaders.shader_target_vertex.main_entry_point(), ())
					.triangle_list()
					.viewports_dynamic_scissors_irrelevant(1)
					.fragment_shader(shaders.shader_target_fragment.main_entry_point(), ())
					.render_pass(subpass_target.clone())
					.build(shaders.target_vertices.device().clone())
					.expect("failed to create pipeline")
			);

		Arc::new(Self {
			shaders: shaders,
			subpass_gbuffers: subpass_gbuffers,
			subpass_target: subpass_target,
			pipeline_gbuffers: pipeline_gbuffers,
			pipeline_target: pipeline_target,
		})
	}

	pub(crate) fn render_pass(&self) -> &Arc<RenderPassAbstract + Send + Sync> {
		self.subpass_gbuffers.render_pass()
	}
}
