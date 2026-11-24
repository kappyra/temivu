use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use eframe::{
    egui_glow,
    glow::{self, HasContext},
};
use egui::{PaintCallback, PaintCallbackInfo};
use temivu::{GlassMode, GlassProfile, GlassRegion, GlassRenderer};

#[derive(Clone, Default)]
pub(super) struct BackdropGl {
    state: Arc<Mutex<Backend>>,
    failed: Arc<AtomicBool>,
}

#[derive(Default)]
struct Backend {
    context: usize,
    gpu: Option<Result<Gpu, String>>,
}

impl GlassRenderer for BackdropGl {
    fn available(&self) -> bool {
        !self.failed.load(Ordering::Relaxed)
    }

    fn callback(&self, region: GlassRegion) -> PaintCallback {
        let backend = self.clone();
        PaintCallback {
            rect: region.rect,
            callback: Arc::new(egui_glow::CallbackFn::new(move |info, painter| {
                backend.paint(info, painter, region);
            })),
        }
    }
}

impl BackdropGl {
    fn paint(&self, info: PaintCallbackInfo, painter: &egui_glow::Painter, region: GlassRegion) {
        let region = region.with_paint_bounds(info.viewport);
        let optics = Optics::new(region);
        let Some(capture) = Capture::new(&info, &optics) else {
            return;
        };
        let Ok(mut backend) = self.state.lock() else {
            return;
        };
        let gl = painter.gl().as_ref();
        let identity = gl as *const glow::Context as usize;
        if backend.context != identity {
            *backend = Backend {
                context: identity,
                gpu: None,
            };
        }
        // SAFETY: this callback is executed by the painter with its context current.
        // Every GL object below is owned by this adapter in that same context.
        unsafe {
            let gpu = backend.gpu.get_or_insert_with(|| {
                let result = Gpu::new(gl);
                if let Err(error) = &result {
                    self.failed.store(true, Ordering::Relaxed);
                    eprintln!("Backdrop renderer unavailable: {error}");
                }
                result
            });
            let Ok(gpu) = gpu else { return };
            if !gpu.paint(
                gl,
                painter.intermediate_fbo(),
                &info,
                capture,
                region,
                &optics,
            ) {
                gpu.destroy(gl);
                backend.gpu = Some(Err("framebuffer capture failed".to_owned()));
                self.failed.store(true, Ordering::Relaxed);
                eprintln!("Backdrop renderer disabled: framebuffer capture failed");
            }
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        if let Ok(mut backend) = self.state.lock()
            && backend.context == gl as *const glow::Context as usize
            && let Some(Ok(gpu)) = backend.gpu.take()
        {
            // SAFETY: eframe supplies the current rendering context during on_exit.
            unsafe { gpu.destroy(gl) };
        }
    }
}

#[derive(Clone, Copy)]
struct Capture {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    step: i32,
}

impl Capture {
    fn new(info: &PaintCallbackInfo, optics: &Optics) -> Option<Self> {
        let visible = info.viewport.intersect(info.clip_rect);
        if !visible.is_positive() {
            return None;
        }
        let sigma = optics.sigma.into_iter().fold(0.0_f32, f32::max);
        let mut step = 1;
        while sigma * info.pixels_per_point > 3.65 * step as f32 && step < 32 {
            step *= 2;
        }
        let padding = sigma * 3.25 + optics.lens + 4.0;
        let capture = egui::epaint::ViewportInPixels::from_points(
            &visible.expand(padding),
            info.pixels_per_point,
            info.screen_size_px,
        );
        // Align every stage of the blur to the framebuffer. Moving and clipping
        // a surface should not move the sampling grid across the scene.
        let align_start = |origin: i32| origin / step * step;
        let align_end = |end: i32, limit: u32| ((end + step - 1) / step * step).min(limit as i32);
        let x = align_start(capture.left_px);
        let y = align_start(capture.from_bottom_px);
        let width = align_end(capture.left_px + capture.width_px, info.screen_size_px[0]) - x;
        let height = align_end(
            capture.from_bottom_px + capture.height_px,
            info.screen_size_px[1],
        ) - y;
        if width < 2 || height < 2 {
            return None;
        }
        Some(Self {
            x,
            y,
            width,
            height,
            step,
        })
    }

    fn blit_step(self) -> i32 {
        // An odd edge needs a clamped partial texel. Use the exact shader
        // reduction there instead of stretching the whole copy to fit.
        if self.step > 1 && self.width % 2 == 0 && self.height % 2 == 0 {
            2
        } else {
            1
        }
    }
}

struct Optics {
    sigma: [f32; 2],
    lens: f32,
    bevel: f32,
    polish: f32,
    saturation: f32,
}

impl Optics {
    fn new(region: GlassRegion) -> Self {
        let (sigma, lens, bevel, polish, saturation) = match region.mode {
            GlassMode::Frost => (14.0, 0.0, 0.0, 0.0, 1.11),
            GlassMode::SoftPrism => (7.4, 8.5, 19.0, 0.68, 1.05),
            GlassMode::Prism => (3.9, 13.0, 21.0, 1.0, 1.03),
        };
        let mut sigma = [sigma; 2];
        if region.profile == GlassProfile::Dock {
            sigma[1] *= 1.25;
        }
        let tint = region.tint.clamp(0.0, 1.0);
        Self {
            sigma: sigma.map(|value| value * region.tuning.blur),
            lens: lens * (1.0 - 0.35 * tint) * region.tuning.refraction,
            bevel,
            polish: polish * region.tuning.rim,
            saturation,
        }
    }
}

#[derive(Clone, Copy)]
struct Surface {
    texture: glow::Texture,
    framebuffer: glow::Framebuffer,
    size: [i32; 2],
}

impl Surface {
    unsafe fn new(gl: &glow::Context) -> Result<Self, String> {
        unsafe {
            let texture = gl.create_texture()?;
            let framebuffer = gl
                .create_framebuffer()
                .inspect_err(|_| gl.delete_texture(texture))?;
            Ok(Self {
                texture,
                framebuffer,
                size: [0; 2],
            })
        }
    }

    unsafe fn reserve(&mut self, gl: &glow::Context, size: [i32; 2], limit: i32) -> bool {
        if size[0] <= self.size[0] && size[1] <= self.size[1] {
            return true;
        }
        if size.into_iter().any(|edge| edge > limit) {
            return false;
        }
        // Grow in blocks so a resize or a moving crop cannot reallocate GPU
        // storage one pixel at a time. Shaders still clamp to the live extent.
        let size = std::array::from_fn(|axis| {
            let edge = size[axis].max(self.size[axis]);
            ((edge + 63) / 64 * 64).min(limit)
        });
        // SAFETY: the callback owns the current context and texture.
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                size[0],
                size[1],
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(None),
            );
            for parameter in [glow::TEXTURE_MIN_FILTER, glow::TEXTURE_MAG_FILTER] {
                gl.tex_parameter_i32(glow::TEXTURE_2D, parameter, glow::LINEAR as i32);
            }
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(self.framebuffer));
            gl.framebuffer_texture_2d(
                glow::DRAW_FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(self.texture),
                0,
            );
            if gl.check_framebuffer_status(glow::DRAW_FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                return false;
            }
            self.size = size;
        }
        true
    }

    unsafe fn destroy(self, gl: &glow::Context) {
        unsafe {
            gl.delete_framebuffer(self.framebuffer);
            gl.delete_texture(self.texture);
        }
    }
}

#[derive(Clone, Copy)]
struct SceneSample {
    surface: Surface,
    extent: [i32; 2],
    step: i32,
}

impl SceneSample {
    fn texels(self) -> [f32; 4] {
        [
            self.extent[0] as f32,
            self.extent[1] as f32,
            self.surface.size[0] as f32,
            self.surface.size[1] as f32,
        ]
    }
}

struct Gpu {
    program: glow::Program,
    vao: glow::VertexArray,
    surfaces: [Surface; 3],
    texture_limit: i32,
    scaled_blit: bool,
    filter: Filter,
    uniforms: [Option<glow::UniformLocation>; 9],
}

impl Gpu {
    unsafe fn new(gl: &glow::Context) -> Result<Self, String> {
        unsafe {
            // Scaling a multisampled framebuffer is unsupported. The normal app
            // framebuffer is single-sampled, so an unusual host falls back cleanly.
            if gl.get_parameter_i32(glow::SAMPLES) > 0 {
                return Err("a single-sampled target is required".into());
            }
            let program = program(gl, FRAGMENT)?;
            let vao = gl
                .create_vertex_array()
                .inspect_err(|_| gl.delete_program(program))?;
            let mut surfaces = Vec::with_capacity(3);
            let setup = (|| {
                for _ in 0..3 {
                    surfaces.push(Surface::new(gl)?);
                }
                Filter::new(gl)
            })();
            let filter = setup.inspect_err(|_| {
                for &surface in &surfaces {
                    surface.destroy(gl);
                }
                gl.delete_vertex_array(vao);
                gl.delete_program(program);
            })?;
            let uniforms = [
                "u_texture",
                "u_detail",
                "u_region",
                "u_corners",
                "u_capture",
                "u_texels",
                "u_detail_texels",
                "u_effect",
                "u_layer",
            ]
            .map(|name| gl.get_uniform_location(program, name));
            Ok(Self {
                program,
                vao,
                surfaces: [surfaces[0], surfaces[1], surfaces[2]],
                texture_limit: gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE),
                scaled_blit: true,
                filter,
                uniforms,
            })
        }
    }

    unsafe fn paint(
        &mut self,
        gl: &glow::Context,
        destination: Option<glow::Framebuffer>,
        info: &PaintCallbackInfo,
        capture: Capture,
        region: GlassRegion,
        optics: &Optics,
    ) -> bool {
        unsafe {
            let scale = info.pixels_per_point;
            gl.active_texture(glow::TEXTURE0);
            gl.disable(glow::SCISSOR_TEST);
            gl.disable(glow::BLEND);
            let filtered = self.filter_backdrop(gl, destination, capture, optics, scale);
            gl.bind_framebuffer(glow::FRAMEBUFFER, destination);
            gl.enable(glow::SCISSOR_TEST);
            gl.enable(glow::BLEND);
            let Some((blurred, detail)) = filtered else {
                return false;
            };
            let viewport = info.viewport_in_pixels();
            gl.viewport(
                viewport.left_px,
                viewport.from_bottom_px,
                viewport.width_px,
                viewport.height_px,
            );
            gl.active_texture(glow::TEXTURE1);
            gl.bind_texture(glow::TEXTURE_2D, Some(detail.surface.texture));
            gl.active_texture(glow::TEXTURE0);
            gl.bind_texture(glow::TEXTURE_2D, Some(blurred.surface.texture));

            let [
                texture,
                detail_texture,
                bounds,
                corners,
                captured,
                texels,
                detail_texels,
                effect,
                layer,
            ] = &self.uniforms;
            let center = region.rect.center();
            gl.use_program(Some(self.program));
            gl.uniform_1_i32(texture.as_ref(), 0);
            gl.uniform_1_i32(detail_texture.as_ref(), 1);
            gl.uniform_4_f32(
                bounds.as_ref(),
                center.x * scale,
                info.screen_size_px[1] as f32 - center.y * scale,
                region.rect.width() * scale * 0.5,
                region.rect.height() * scale * 0.5,
            );
            gl.uniform_4_f32(
                corners.as_ref(),
                region.corners.nw * scale,
                region.corners.ne * scale,
                region.corners.sw * scale,
                region.corners.se * scale,
            );
            gl.uniform_4_f32(
                captured.as_ref(),
                capture.x as f32,
                capture.y as f32,
                (blurred.step as f32).recip(),
                (detail.step as f32).recip(),
            );
            gl.uniform_4_f32_slice(texels.as_ref(), &blurred.texels());
            gl.uniform_4_f32_slice(detail_texels.as_ref(), &detail.texels());
            gl.uniform_4_f32(
                effect.as_ref(),
                optics.lens * scale,
                optics.bevel * scale,
                optics.polish,
                optics.saturation,
            );
            gl.uniform_2_f32(layer.as_ref(), region.strength, scale.recip());
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            // The painter restores its program, VAO and viewport after the callback.
            true
        }
    }

    unsafe fn filter_backdrop(
        &mut self,
        gl: &glow::Context,
        destination: Option<glow::Framebuffer>,
        capture: Capture,
        optics: &Optics,
        scale: f32,
    ) -> Option<(SceneSample, SceneSample)> {
        unsafe {
            let step = if self.scaled_blit {
                capture.blit_step()
            } else {
                1
            };
            let detail = match self.copy_scene(gl, destination, capture, step) {
                Some(sample) => sample,
                None if step == 2 => {
                    // Keep live glass on drivers that reject a scaled blit.
                    self.scaled_blit = false;
                    self.copy_scene(gl, destination, capture, 1)?
                }
                None => return None,
            };

            let mut sigma = optics.sigma.map(|value| value * scale);
            if sigma[0].max(sigma[1]) <= 0.05 {
                return Some((detail, detail));
            }

            let reduced_sigma = |value: f32| (value.powi(2) - 0.25).max(0.0).sqrt() * 0.5;
            gl.bind_vertex_array(Some(self.vao));
            let (detail, work) = if detail.step == 1 && capture.step > 1 {
                // The bevel uses the same detail resolution with the shader
                // fallback as it does when a scaled blit is available.
                (self.reduce_scene(gl, detail, 1)?, [0, 2])
            } else {
                (detail, [1, 2])
            };
            if detail.step == 2 {
                sigma = sigma.map(reduced_sigma);
            }
            let mut source = detail;
            let mut next = 0;
            while source.step < capture.step {
                source = self.reduce_scene(gl, source, work[next])?;
                next = 1 - next;
                sigma = sigma.map(reduced_sigma);
            }
            let extent = source.extent;
            let [scratch_index, result_index] = [work[next], work[1 - next]];
            if !self.surfaces[scratch_index].reserve(gl, extent, self.texture_limit)
                || !self.surfaces[result_index].reserve(gl, extent, self.texture_limit)
            {
                return None;
            }
            let scratch = self.surfaces[scratch_index];
            let result = self.surfaces[result_index];
            gl.viewport(0, 0, extent[0], extent[1]);
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(scratch.framebuffer));
            self.filter.prepare(gl, extent, source.surface.size);
            self.filter.pass(
                gl,
                source.surface.texture,
                [1.0, 0.0],
                Kernel::new(sigma[0]),
            );
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(result.framebuffer));
            self.filter.prepare(gl, extent, scratch.size);
            self.filter
                .pass(gl, scratch.texture, [0.0, 1.0], Kernel::new(sigma[1]));
            Some((
                SceneSample {
                    surface: result,
                    extent,
                    step: source.step,
                },
                detail,
            ))
        }
    }

    unsafe fn reduce_scene(
        &mut self,
        gl: &glow::Context,
        source: SceneSample,
        target: usize,
    ) -> Option<SceneSample> {
        unsafe {
            let half = source.extent.map(|edge| (edge + 1) / 2);
            if !self.surfaces[target].reserve(gl, half, self.texture_limit) {
                return None;
            }
            let surface = self.surfaces[target];
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(surface.framebuffer));
            gl.viewport(0, 0, half[0], half[1]);
            self.filter.prepare(gl, source.extent, source.surface.size);
            self.filter.reduce(gl, source.surface.texture);
            Some(SceneSample {
                surface,
                extent: half,
                step: source.step * 2,
            })
        }
    }

    unsafe fn copy_scene(
        &mut self,
        gl: &glow::Context,
        destination: Option<glow::Framebuffer>,
        capture: Capture,
        step: i32,
    ) -> Option<SceneSample> {
        unsafe {
            let extent = [capture.width / step, capture.height / step];
            if !self.surfaces[0].reserve(gl, extent, self.texture_limit) {
                return None;
            }
            let surface = self.surfaces[0];
            gl.bind_framebuffer(glow::DRAW_FRAMEBUFFER, Some(surface.framebuffer));
            gl.bind_framebuffer(glow::READ_FRAMEBUFFER, destination);
            gl.blit_framebuffer(
                capture.x,
                capture.y,
                capture.x + capture.width,
                capture.y + capture.height,
                0,
                0,
                extent[0],
                extent[1],
                glow::COLOR_BUFFER_BIT,
                if step == 1 {
                    glow::NEAREST
                } else {
                    glow::LINEAR
                },
            );
            (gl.get_error() == glow::NO_ERROR).then_some(SceneSample {
                surface,
                extent,
                step,
            })
        }
    }

    unsafe fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
            for surface in self.surfaces {
                surface.destroy(gl);
            }
            gl.delete_program(self.filter.program);
            gl.delete_program(self.program);
        }
    }
}

#[derive(Clone, Copy)]
struct Kernel {
    center: f32,
    offsets: [f32; 4],
    weights: [f32; 4],
}

impl Kernel {
    fn new(sigma: f32) -> Self {
        let factor = -0.5 / sigma.max(0.05).powi(2);
        let mut normalization = 1.0;
        let mut offsets = [0.0; 4];
        let mut weights = [0.0; 4];
        for (pair, (offset, weight)) in offsets.iter_mut().zip(&mut weights).enumerate() {
            let first = (2 * pair + 1) as f32;
            let a = (factor * first.powi(2)).exp();
            let b = (factor * (first + 1.0).powi(2)).exp();
            *weight = a + b;
            // Linear sampling evaluates both adjacent Gaussian terms in one read.
            *offset = if *weight > 0.0 {
                first + b / *weight
            } else {
                first
            };
            normalization += 2.0 * *weight;
        }
        Self {
            center: normalization.recip(),
            offsets,
            weights: weights.map(|weight| weight / normalization),
        }
    }
}

struct Filter {
    program: glow::Program,
    uniforms: [Option<glow::UniformLocation>; 7],
}

impl Filter {
    unsafe fn new(gl: &glow::Context) -> Result<Self, String> {
        unsafe {
            let program = program(gl, FILTER)?;
            let uniforms = [
                "u_texture",
                "u_texels",
                "u_axis",
                "u_center",
                "u_offsets",
                "u_weights",
                "u_reduce",
            ]
            .map(|name| gl.get_uniform_location(program, name));
            Ok(Self { program, uniforms })
        }
    }

    unsafe fn reduce(&self, gl: &glow::Context, source: glow::Texture) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(source));
            gl.uniform_1_i32(self.uniforms[6].as_ref(), 1);
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }

    unsafe fn prepare(&self, gl: &glow::Context, samples: [i32; 2], texture_size: [i32; 2]) {
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_1_i32(self.uniforms[0].as_ref(), 0);
            gl.uniform_4_f32(
                self.uniforms[1].as_ref(),
                samples[0] as f32,
                samples[1] as f32,
                texture_size[0] as f32,
                texture_size[1] as f32,
            );
        }
    }

    unsafe fn pass(
        &self,
        gl: &glow::Context,
        source: glow::Texture,
        axis: [f32; 2],
        kernel: Kernel,
    ) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(source));
            gl.uniform_2_f32(self.uniforms[2].as_ref(), axis[0], axis[1]);
            gl.uniform_1_f32(self.uniforms[3].as_ref(), kernel.center);
            gl.uniform_4_f32_slice(self.uniforms[4].as_ref(), &kernel.offsets);
            gl.uniform_4_f32_slice(self.uniforms[5].as_ref(), &kernel.weights);
            gl.uniform_1_i32(self.uniforms[6].as_ref(), 0);
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }
}

unsafe fn program(gl: &glow::Context, fragment: &str) -> Result<glow::Program, String> {
    unsafe {
        let header = if gl.version().is_embedded {
            "#version 300 es\nprecision highp float;\n"
        } else {
            "#version 330 core\n"
        };
        let program = gl.create_program()?;
        let result = (|| {
            for (kind, source) in [
                (glow::VERTEX_SHADER, VERTEX),
                (glow::FRAGMENT_SHADER, fragment),
            ] {
                let shader = gl.create_shader(kind)?;
                gl.shader_source(shader, &format!("{header}{source}"));
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    let error = gl.get_shader_info_log(shader);
                    gl.delete_shader(shader);
                    return Err(error);
                }
                gl.attach_shader(program, shader);
                gl.delete_shader(shader);
            }
            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                return Err(gl.get_program_info_log(program));
            }
            Ok(program)
        })();
        if result.is_err() {
            gl.delete_program(program);
        }
        result
    }
}

const VERTEX: &str = "
void main() {
    vec2 p = vec2(float((gl_VertexID << 1) & 2), float(gl_VertexID & 2));
    gl_Position = vec4(p * 2.0 - 1.0, 0.0, 1.0);
}
";

const FRAGMENT: &str = include_str!("backdrop_gl.frag");

const FILTER: &str = include_str!("backdrop_blur.frag");

#[cfg(test)]
mod tests {
    use super::{Capture, Kernel, Optics};

    #[test]
    fn moving_and_clipping_glass_keeps_the_same_downsample_lattice() {
        let optics = Optics {
            sigma: [7.0; 2],
            lens: 8.0,
            bevel: 20.0,
            polish: 0.7,
            saturation: 1.0,
        };
        let screen = [375, 311];
        let scale = 1.5;
        let point = egui::pos2(170.5, 120.5);
        let size = egui::vec2(120.0, 66.0);
        let pixel = egui::vec2(261.0, 120.0);
        let mut sample_phase = None;
        for move_px in 0..16 {
            let viewport =
                egui::Rect::from_center_size(point + egui::vec2(move_px as f32 / scale, 0.0), size);
            let clip_rect = viewport.with_min_x(viewport.left() + move_px as f32 * 0.9);
            let info = egui::PaintCallbackInfo {
                viewport,
                clip_rect,
                pixels_per_point: scale,
                screen_size_px: screen,
            };
            let capture = Capture::new(&info, &optics).unwrap();
            assert!(capture.step > 1);
            assert_eq!(capture.x % capture.step, 0);
            assert_eq!(capture.y % capture.step, 0);
            assert_eq!(capture.x + capture.width, screen[0] as i32);
            assert_eq!(capture.blit_step(), 1);
            let sample =
                (pixel - egui::vec2(capture.x as f32, capture.y as f32)) / capture.step as f32;
            let phase = [sample.x.fract(), sample.y.fract()];
            assert_eq!(*sample_phase.get_or_insert(phase), phase);
        }
    }

    #[test]
    fn an_even_capture_can_kick_off_the_blur_at_half_resolution() {
        let mut capture = Capture {
            x: 80,
            y: 48,
            width: 224,
            height: 128,
            step: 4,
        };
        assert_eq!(capture.blit_step(), 2);
        capture.height -= 1;
        assert_eq!(capture.blit_step(), 1);
        capture.height += 1;
        capture.step = 1;
        assert_eq!(capture.blit_step(), 1);
    }

    #[test]
    fn blur_kernel_keeps_uniform_color_and_has_no_secondary_peaks() {
        for sigma in [0.0, 1.5, 3.0, 4.0, 8.0] {
            let kernel = Kernel::new(sigma);
            let mut previous = kernel.center;
            let mut total = kernel.center;
            for (pair, (&offset, &weight)) in kernel.offsets.iter().zip(&kernel.weights).enumerate()
            {
                let first = (2 * pair + 1) as f32;
                let fraction = offset - first;
                assert!((0.0..=1.0).contains(&fraction));
                let near = weight * (1.0 - fraction);
                let far = weight * fraction;
                assert!(previous >= near && near >= far);
                previous = far;
                total += 2.0 * weight;
            }
            assert!((total - 1.0).abs() < 1e-6);
        }
    }
}
