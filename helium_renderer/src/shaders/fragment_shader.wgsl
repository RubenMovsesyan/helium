struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

// Fagment Shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;



struct CameraUniform {
    view_position: vec4<f32>,
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>
}

@group(2) @binding(0)
var<uniform> light: Light;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // Ambient lighting
    let ambient_strength = 0.1;
    let ambient_color = light.color * ambient_strength;


    // Diffuse lighting
    let light_dir = normalize(light.position - in.world_position);

    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = light.color * diffuse_strength;

    // Specular lighting
    let view_dir = normalize(camera.view_position.xyz - in.world_position);
    let reflect_dir = reflect(-light_dir, in.world_normal);
    // let half_dir = normalize(view_dir + light_dir);
    let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 100.0);
    // let specular_strength = pow(max(dot(view_dir, half_dir), 0.0), 100.0);
    let specular_color = specular_strength * light.color;


    let result = (ambient_color + diffuse_color + specular_color) * object_color.rgb;
    return vec4<f32>(result, object_color.a);
}
