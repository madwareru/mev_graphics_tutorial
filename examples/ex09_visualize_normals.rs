use mev_graphics_tutorial::{
    software_buffer::{
        SoftwareBuffer,
        Color24,
        ex09_visualize_normals::DrawNormalsShader
    },
    obj_loader::ObjModel,
};

const DRAGON_TEXTURE_BYTES : &[u8] = include_bytes!("../assets/dragon.png");
const DRAGON_MODEL_TEXT: &str = include_str!("../assets/dragon.obj");

pub fn main() {
    let mut buffer = SoftwareBuffer::new(640, 480);
    buffer.clear(Color24 { r: 0x18, g: 0x18, b: 0x18 });
    let mut depth_texture = vec![0.0; buffer.get_width() as usize * buffer.get_height() as usize];

    let image = image::load_from_memory(DRAGON_TEXTURE_BYTES).expect("Failed to load image");
    let image = image.to_rgb8();

    let mut texture = vec![Color24{r: 0, g: 0, b: 0}; image.width() as usize * image.height() as usize ];
    for (i, pixel) in image.pixels().enumerate() {
        texture[i] = Color24 {
            r: pixel[0],
            g: pixel[1],
            b: pixel[2]
        }
    }

    let dragon_model = ObjModel::load_from_string(DRAGON_MODEL_TEXT).unwrap();

    let model_matrix = glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, 0.0)) *
        glam::Mat4::from_scale(glam::Vec3::new(2.0, 2.0, 2.0));

    let view_matrix = glam::camera::rh::view::look_at_mat4(
        glam::Vec3::new(450.0, -500.0, -500.0),
        glam::Vec3::new(0.0, 0.0, 0.0),
        glam::Vec3::new(0.0, 1.0, 0.0)
    );

    let proj_matrix = glam::camera::rh::proj::opengl::perspective(
        std::f32::consts::FRAC_PI_2,
        buffer.get_width() as f32 / buffer.get_height() as f32,
        0.1,
        1000.0
    );

    buffer.draw_obj_model(
        &dragon_model,
        &mut depth_texture,
        &DrawNormalsShader{
            model_matrix,
            view_matrix,
            proj_matrix
        }
    );
    buffer.print_as_ppm();
}