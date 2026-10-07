use crate::renderer::{buffer_collection::BufferCollection, shader_type::ShaderType};

pub fn render_scene(
    encoder: &mut wgpu::CommandEncoder,
    color_view: &wgpu::TextureView,
    depth_view: &wgpu::TextureView,
    buffer_collection: &BufferCollection,
    pipeline_displacement: &wgpu::RenderPipeline,
    pipeline_displacement_bones: &wgpu::RenderPipeline,
) {
    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Render Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: color_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.2,
                    g: 0.247,
                    b: 0.314,
                    a: 1.0,
                }),
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Discard,
            }),
            stencil_ops: None,
        }),
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });

    let mut current_shader = ShaderType::Displacement;
    render_pass.set_pipeline(&pipeline_displacement);

    for mesh in 0..buffer_collection.vertex_buffers.len() {
        let shader_type = &buffer_collection.shader_type[mesh];

        if shader_type != &current_shader {
            match shader_type {
                ShaderType::Displacement => {
                    render_pass.set_pipeline(&pipeline_displacement);
                    current_shader = ShaderType::Displacement;
                }

                ShaderType::DisplacementBones => {
                    render_pass.set_pipeline(&pipeline_displacement_bones);
                    current_shader = ShaderType::DisplacementBones;
                }
            }
        }

        for i in 0..buffer_collection.vertex_buffers[mesh].len() {
            render_pass
                .set_vertex_buffer(0, buffer_collection.vertex_buffers[mesh][i].slice(..));

            render_pass.set_bind_group(
                0,
                &buffer_collection.uniform_bind_groups[mesh][i],
                &[],
            );

            render_pass.draw(0..buffer_collection.num_vertices[mesh][i], 0..1);
        }
    }
}
