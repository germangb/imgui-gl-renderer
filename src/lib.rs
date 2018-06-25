extern crate gl;
extern crate imgui;

use imgui::{ImDrawIdx, ImDrawVert, ImGui, Ui};

use gl::types::*;

use std::mem;

macro_rules! check {
    ($gl:expr) => {{
        let result = $gl;

        let error = gl::GetError();
        if error != gl::NO_ERROR {
            panic!("[GL-ERROR] GL ERROR : {}", error);
        }

        result
    }};
}

pub struct Renderer {
    objects: Objects,
}

impl Renderer {
    pub unsafe fn init(ctx: &mut ImGui) -> Self {
        Self {
            objects: Objects::init(ctx),
        }
    }

    pub unsafe fn render(&self, ui: Ui) -> Result<(), ()> {
        ui.render(|ui, draw| {
            self.objects.update_mesh(draw.idx_buffer, draw.vtx_buffer);

            check!(gl::BindVertexArray(self.objects.vao));
            check!(gl::UseProgram(self.objects.shader));

            //check!(gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE));

            let (w, h) = ui.imgui().display_size();

            let matrix = [
                [2.0 / w as f32, 0.0, 0.0, 0.0],
                [0.0, 2.0 / -(h as f32), 0.0, 0.0],
                [0.0, 0.0, -1.0, 0.0],
                [-1.0, 1.0, 0.0, 1.0],
            ];

            check!(gl::UniformMatrix4fv(
                self.objects.u_matrix,
                1,
                gl::FALSE,
                matrix.as_ptr() as _
            ));
            check!(gl::Uniform1i(self.objects.u_texture, 0));

            check!(gl::ActiveTexture(gl::TEXTURE0));
            check!(gl::BindTexture(gl::TEXTURE_2D, self.objects.texture));

            //TODO fix
            check!(gl::Enable(gl::SCISSOR_TEST));
            check!(gl::Enable(gl::BLEND));
            check!(gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA));

            let mut id_offset = 0;
            for cmd in draw.cmd_buffer.iter() {
                check!(gl::Scissor(
                    cmd.clip_rect.x as _,
                    (h - cmd.clip_rect.w) as _,
                    (cmd.clip_rect.z - cmd.clip_rect.x) as _,
                    (cmd.clip_rect.w - cmd.clip_rect.y) as _,
                ));

                check!(gl::DrawElements(
                    gl::TRIANGLES,
                    cmd.elem_count as _,
                    gl::UNSIGNED_SHORT,
                    id_offset as _,
                ));
                id_offset += cmd.elem_count * mem::size_of::<ImDrawIdx>() as u32;
            }

            check!(gl::Disable(gl::SCISSOR_TEST));
            check!(gl::Disable(gl::BLEND));
            check!(gl::UseProgram(0));
            check!(gl::BindVertexArray(0));

            Ok(())
        })
    }
}

struct Objects {
    vbo: GLuint,
    ebo: GLuint,
    vao: GLuint,
    shader: GLuint,
    texture: GLuint,
    u_matrix: GLint,
    u_texture: GLint,
}

impl Objects {
    unsafe fn init(ctx: &mut ImGui) -> Self {
        let (vbo, ebo, vao) = init_mesh();
        let (shader, u_matrix, u_texture) = init_program();
        let mut obj = Self {
            vbo,
            ebo,
            vao,
            shader,
            u_matrix,
            u_texture,
            texture: init_texture(ctx),
        };
        obj
    }

    unsafe fn update_mesh(&self, idx: &[ImDrawIdx], vert: &[ImDrawVert]) {
        let idx_size = mem::size_of::<ImDrawIdx>();
        let vert_size = mem::size_of::<ImDrawVert>();

        check!(gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo));
        check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo));

        check!(gl::BufferSubData(
            gl::ARRAY_BUFFER,
            0,
            (vert.len() * vert_size) as _,
            vert.as_ptr() as _,
        ));

        check!(gl::BufferSubData(
            gl::ELEMENT_ARRAY_BUFFER,
            0,
            (idx.len() * idx_size) as _,
            idx.as_ptr() as _,
        ));

        check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
        check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));
    }
}

impl Drop for Objects {
    fn drop(&mut self) {
        unsafe {
            check!(gl::DeleteTextures(1, &self.texture));
            check!(gl::DeleteBuffers(2, [self.vbo, self.ebo].as_ptr()));
            check!(gl::DeleteVertexArrays(1, &self.vao));
            check!(gl::DeleteProgram(self.shader));
        }
    }
}

unsafe fn init_program() -> (GLuint, GLint, GLint) {
    let vert = create_shader(gl::VERTEX_SHADER, include_str!("shaders/shader.vert"));
    let frag = create_shader(gl::FRAGMENT_SHADER, include_str!("shaders/shader.frag"));

    let program = check!(gl::CreateProgram());
    check!(gl::AttachShader(program, vert));
    check!(gl::AttachShader(program, frag));
    check!(gl::LinkProgram(program));

    check!(gl::DeleteShader(vert));
    check!(gl::DeleteShader(frag));

    //println!("program {}", program);
    (
        program,
        check!(gl::GetUniformLocation(program, "u_matrix\0".as_ptr() as _)),
        check!(gl::GetUniformLocation(program, "u_texture\0".as_ptr() as _)),
    )
}

unsafe fn create_shader(type_: GLenum, source: &str) -> GLuint {
    let shader = check!(gl::CreateShader(type_));
    check!(gl::ShaderSource(
        shader,
        1,
        [source.as_ptr() as _].as_ptr(),
        [source.len() as _].as_ptr()
    ));
    check!(gl::CompileShader(shader));

    let mut log = vec![0u8; 1024];
    let mut len = 0;
    check!(gl::GetShaderInfoLog(
        shader,
        1024,
        &mut len,
        log.as_ptr() as _
    ));
    assert_eq!(len, 0);
    shader
}

unsafe fn init_mesh() -> (GLuint, GLuint, GLuint) {
    let mut vbo = 0;
    let mut ebo = 0;
    let mut vao = 0;

    let buffer_init = vec![0u8; 1024 * 512];

    check!(gl::GenBuffers(1, &mut vbo));
    check!(gl::GenBuffers(1, &mut ebo));
    check!(gl::GenVertexArrays(1, &mut vao));

    check!(gl::BindVertexArray(vao));
    check!(gl::BindBuffer(gl::ARRAY_BUFFER, vbo));
    check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo));

    check!(gl::BufferData(
        gl::ARRAY_BUFFER,
        buffer_init.len() as _,
        buffer_init.as_ptr() as _,
        gl::STREAM_DRAW,
    ));

    check!(gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        buffer_init.len() as _,
        buffer_init.as_ptr() as _,
        gl::STREAM_DRAW,
    ));

    check!(gl::EnableVertexAttribArray(0));
    check!(gl::EnableVertexAttribArray(1));
    check!(gl::EnableVertexAttribArray(2));

    let stride = 8 + 8 + 4; // vec2 + vec2 + ivec4
    check!(gl::VertexAttribPointer(
        0,
        2,
        gl::FLOAT,
        gl::FALSE,
        stride,
        0 as _
    ));
    check!(gl::VertexAttribPointer(
        1,
        2,
        gl::FLOAT,
        gl::FALSE,
        stride,
        8 as _
    ));
    check!(gl::VertexAttribPointer(
        2,
        4,
        gl::UNSIGNED_BYTE,
        gl::FALSE,
        stride,
        16 as _
    ));

    check!(gl::BindVertexArray(0));
    check!(gl::DisableVertexAttribArray(0));
    check!(gl::DisableVertexAttribArray(1));
    check!(gl::DisableVertexAttribArray(2));
    check!(gl::BindBuffer(gl::ARRAY_BUFFER, 0));
    check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0));

    (vbo, ebo, vao)
}

unsafe fn init_texture(ctx: &mut ImGui) -> GLuint {
    let handle = ctx.prepare_texture(|t| {
        let mut handle = 0;
        check!(gl::GenTextures(1, &mut handle));
        check!(gl::ActiveTexture(gl::TEXTURE0));
        check!(gl::BindTexture(gl::TEXTURE_2D, handle));
        check!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MIN_FILTER,
            gl::NEAREST as _
        ));
        check!(gl::TexParameteri(
            gl::TEXTURE_2D,
            gl::TEXTURE_MAG_FILTER,
            gl::NEAREST as _
        ));
        check!(gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::RGBA8 as _,
            t.width as _,
            t.height as _,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            t.pixels.as_ptr() as _,
        ));
        check!(gl::BindTexture(gl::TEXTURE_2D, 0));
        handle
    });
    ctx.set_texture_id(handle as _);
    handle
}
