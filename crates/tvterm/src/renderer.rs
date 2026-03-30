use rmx::prelude::*;
use rmx::std::sync::Arc;
use wgpu::CompositeAlphaMode;
use winit::window::Window;

/// Owns the wgpu device, queue, surface, and egui renderer.
pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub egui_renderer: egui_wgpu::Renderer,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

impl Renderer {
    /// Create a new renderer for the given window.
    pub fn new(window: Arc<Window>) -> AnyResult<Self> {
        pollster::block_on(Self::new_async(window))
    }

    async fn new_async(window: Arc<Window>) -> AnyResult<Self> {
        let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
        desc.backends = wgpu::Backends::VULKAN | wgpu::Backends::GL;
        let instance = wgpu::Instance::new(desc);

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        info!("Using adapter: {:?}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("tvterm"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await?;

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);

        let alpha_mode = if caps.alpha_modes.contains(&CompositeAlphaMode::PreMultiplied) {
            info!("Using PreMultiplied alpha compositing");
            CompositeAlphaMode::PreMultiplied
        } else if caps.alpha_modes.contains(&CompositeAlphaMode::PostMultiplied) {
            info!("Using PostMultiplied alpha compositing");
            CompositeAlphaMode::PostMultiplied
        } else {
            warn!("No premultiplied alpha support; transparency may not work");
            caps.alpha_modes[0]
        };

        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        info!(
            "Surface format: {:?}, alpha modes: {:?}",
            format, caps.alpha_modes
        );

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            format,
            egui_wgpu::RendererOptions::default(),
        );

        Ok(Self {
            device,
            queue,
            egui_renderer,
            surface,
            config,
        })
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn width(&self) -> u32 {
        self.config.width
    }

    pub fn height(&self) -> u32 {
        self.config.height
    }

    pub fn max_texture_side(&self) -> usize {
        self.device.limits().max_texture_dimension_2d as usize
    }

    /// Resize the surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    /// Render a frame with egui output.
    pub fn render(
        &mut self,
        textures_delta: &egui::TexturesDelta,
        clipped_primitives: &[egui::ClippedPrimitive],
        pixels_per_point: f32,
        opacity: f32,
    ) -> AnyResult<()> {
        let output = self.surface.get_current_texture();
        let frame = match output {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            other => bail!("Failed to get surface texture: {:?}", other),
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Update egui textures.
        for (id, image_delta) in &textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        let screen_desc = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point,
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("tvterm render"),
            });

        let user_cmd_bufs = self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            clipped_primitives,
            &screen_desc,
        );

        // Premultiplied alpha: multiply RGB by alpha.
        let a = opacity as f64;
        {
            let mut pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("main"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0 * a,
                                g: 0.0 * a,
                                b: 0.0 * a,
                                a,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
                .forget_lifetime();

            self.egui_renderer
                .render(&mut pass, clipped_primitives, &screen_desc);
        }

        self.queue.submit(
            user_cmd_bufs
                .into_iter()
                .chain(std::iter::once(encoder.finish())),
        );
        frame.present();

        // Free old textures.
        for id in &textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        Ok(())
    }
}
