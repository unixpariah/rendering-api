mod node;

use crate::{buffers, rectangle};
use std::ops::{Deref, DerefMut};

pub struct Tree {
    render_pipeline: wgpu::RenderPipeline,
    projection_uniform: buffers::ProjectionUniform,
    instance_buffer: buffers::InstanceBuffer,
    index_buffer: buffers::IndexBuffer,
    generic_rect: buffers::VertexBuffer,
    node: Node,
}

impl Tree {
    pub fn new<F>(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, f: F) -> Self
    where
        F: Fn(Node) -> Node,
    {
        let node = f(Node::new());
        let mut instances = Vec::new();
        node.collect_instances(&mut instances);

        let projection_uniform = buffers::ProjectionUniform::new(
            device,
            0.0,
            config.width as f32,
            0.0,
            config.height as f32,
        );

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&projection_uniform.bind_group_layout],
                push_constant_ranges: &[],
            });

        let vertex_buffers = [buffers::Vertex::desc(), buffers::Instance::desc()];

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &vertex_buffers,
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            depth_stencil: None,
            multiview: None,
            cache: None,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        let generic_rect_vertices = [
            buffers::Vertex {
                position: [0.0, 1.0],
            },
            buffers::Vertex {
                position: [1.0, 1.0],
            },
            buffers::Vertex {
                position: [1.0, 0.0],
            },
            buffers::Vertex {
                position: [0.0, 0.0],
            },
        ];

        Self {
            render_pipeline,
            index_buffer: buffers::IndexBuffer::new(device, &[0, 1, 3, 1, 2, 3]),
            generic_rect: buffers::VertexBuffer::new(device, &generic_rect_vertices),
            projection_uniform,
            node,
            instance_buffer: buffers::InstanceBuffer::new(device, &instances),
        }
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.projection_uniform.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.generic_rect.slice(..));

        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(
            0..self.index_buffer.size(),
            0,
            0..self.instance_buffer.size(),
        );
    }
}

impl Deref for Tree {
    type Target = Node;
    fn deref(&self) -> &Self::Target {
        &self.node
    }
}

impl DerefMut for Tree {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

pub struct Node {
    pub children: Vec<Node>,
    pub data: rectangle::Rectangle,
}

impl Deref for Node {
    type Target = rectangle::Rectangle;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for Node {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl Node {
    pub fn new() -> Self {
        return Self {
            data: rectangle::Rectangle::default(),
            children: Vec::new(),
        };
    }

    pub fn position_children(&mut self) {
        let extents = self.get_extents();
        let mut pos = (extents.x, extents.y);

        self.children.iter_mut().for_each(|child| {
            child.x = pos.0;
            child.y = pos.1;

            let extents = child.get_extents();

            pos.1 += extents.height;

            child.position_children();
        })
    }

    pub fn add_child<F>(mut self, f: F) -> Self
    where
        F: Fn(Node) -> Node,
    {
        let node = f(Node::new());
        self.children.push(node);

        self.position_children();

        self
    }

    fn collect_instances(&self, instances: &mut Vec<buffers::Instance>) {
        instances.push(self.data.get_instance());

        self.children
            .iter()
            .for_each(|child| child.collect_instances(instances));
    }
}
