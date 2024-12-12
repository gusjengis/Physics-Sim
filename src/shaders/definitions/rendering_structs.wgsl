struct Input {
    width: f32, time: f32,
    height: f32,
    xOff: f32, yOff: f32,
    ui_xOff: f32, ui_yOff: f32,
    ui_scale: f32, scale: f32,
    dark: f32,
    x: i32, y: i32,
    rW: i32, rH: i32,
    pressed: i32,
    timestamp: i32,
    p_count: i32,
    $ 3D { cam: Camera, }
}

$ 3D {
struct Camera {
        pos: vec4<f32>
        view_proj: mat4x4<f32>,
        eye: mat4x4<f32>,
        focus: mat4x4<f32>,
    };
}

struct VertexIn {
    @location(0) position: vec2<f32>,
};
