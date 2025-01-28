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

@group(2) @binding(0)
var<storage, read> lights: array<f32>;

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var result: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
    for (var light_index: u32 = 0; light_index < arrayLength(&lights); light_index = light_index + 8) {
        let position = vec3<f32>( 
            lights[light_index],
            lights[light_index + 1],
            lights[light_index + 2],
        );

        let color = vec3<f32>(
            lights[light_index + 4],
            lights[light_index + 5],
            lights[light_index + 6]
        );
        
        // Ambient lighting
        let ambient_strength = 0.01;
        let ambient_color = color.rgb * ambient_strength;


        // Diffuse lighting
        let light_dir = normalize(position.xyz - in.world_position);

        let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
        let diffuse_color = color.rgb * diffuse_strength;

        // Specular lighting
        let view_dir = normalize(camera.view_position.xyz - in.world_position);
        let reflect_dir = reflect(-light_dir, in.world_normal);
        // let half_dir = normalize(view_dir + light_dir);
        let specular_strength = pow(max(dot(view_dir, reflect_dir), 0.0), 1000.0);
        // let specular_strength = pow(max(dot(view_dir, half_dir), 0.0), 100.0);
        let specular_color = specular_strength * color.rgb;


        result += (ambient_color + diffuse_color + specular_color) * object_color.rgb;
    }

    return vec4<f32>(result, object_color.a);
}
