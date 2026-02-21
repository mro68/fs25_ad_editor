// Shader für Node-Rendering (2D)

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct InstanceInput {
    @location(1) instance_position: vec2<f32>,
    @location(2) instance_base_color: vec4<f32>,
    @location(3) instance_rim_color: vec4<f32>,
    @location(4) instance_size: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,         // base (mittig)
    @location(1) uv: vec2<f32>,
    @location(2) rim_color: vec4<f32>,     // außen / Markierung
}

struct ConnectionVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct ConnectionVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct Uniforms {
    view_proj: mat4x4<f32>,
    aa_params: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Skaliere das Quad mit der Instanz-Größe
    let scaled_pos = vertex.position * instance.instance_size;
    
    // Verschiebe zur Instanz-Position
    let world_pos = scaled_pos + instance.instance_position;
    
    // Transformiere mit View-Projektion
    out.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    out.color = instance.instance_base_color;
    out.rim_color = instance.instance_rim_color;
    out.uv = vertex.position * 0.5 + 0.5; // -1..1 -> 0..1
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Kreisförmige Form via Distance Field
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(in.uv, center);

    // Mix: mittig = base_color, außen = rim_color
    let radius = 0.48;
    let mix_t = clamp(dist / radius, 0.0, 1.0);
    let rgb = mix(in.color.rgb, in.rim_color.rgb, mix_t);

    // Screen-space adaptives Anti-Aliasing am Rand (unverändert)
    let hard_edges = uniforms.aa_params.y > 0.5;
    var alpha: f32;
    if (hard_edges) {
        alpha = select(0.0, 1.0, dist <= radius);
    } else {
        let edge = max(fwidth(dist) * uniforms.aa_params.x, 0.0005);
        alpha = 1.0 - smoothstep(radius - edge, radius + edge, dist);
    }
    
    return vec4<f32>(rgb, in.color.a * alpha);
}

@vertex
fn vs_connection(in: ConnectionVertexInput) -> ConnectionVertexOutput {
    var out: ConnectionVertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_connection(in: ConnectionVertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

// === Background-Map-Rendering ===

struct BackgroundVertexInput {
    @location(0) position: vec2<f32>,
}

struct BackgroundVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct BackgroundUniforms {
    view_proj: mat4x4<f32>,
    opacity: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
    texture_bounds: vec4<f32>, // min_x, max_x, min_z, max_z
}

@group(0) @binding(0)
var<uniform> background_uniforms: BackgroundUniforms;

@group(0) @binding(1)
var background_texture: texture_2d<f32>;

@group(0) @binding(2)
var background_sampler: sampler;

@vertex
fn vs_background(in: BackgroundVertexInput) -> BackgroundVertexOutput {
    var out: BackgroundVertexOutput;
    
    // in.position ist in Weltkoordinaten (z.B. -1024..1024 für 2048x2048 Map)
    out.clip_position = background_uniforms.view_proj * vec4<f32>(in.position, 0.0, 1.0);
    
    // UV-Koordinaten berechnen aus Weltkoordinaten
    let min_x = background_uniforms.texture_bounds.x;
    let max_x = background_uniforms.texture_bounds.y;
    let min_z = background_uniforms.texture_bounds.z;
    let max_z = background_uniforms.texture_bounds.w;
    
    let u = (in.position.x - min_x) / (max_x - min_x);
    let v = (in.position.y - min_z) / (max_z - min_z);
    
    out.uv = vec2<f32>(u, v);
    
    return out;
}

@fragment
fn fs_background(in: BackgroundVertexOutput) -> @location(0) vec4<f32> {
    // Sample Texture
    let texture_color = textureSample(background_texture, background_sampler, in.uv);
    
    // Verwende Texture-Alpha * Opacity für finales Alpha
    let final_alpha = texture_color.a * background_uniforms.opacity;
    
    return vec4<f32>(texture_color.rgb, final_alpha);
}

// === Map-Marker-Rendering (Pin-Symbol wie Google Maps) ===

struct MarkerInstanceInput {
    @location(1) instance_position: vec2<f32>,
    @location(2) instance_color: vec4<f32>,
    @location(3) instance_outline_color: vec4<f32>,
    @location(4) instance_size: f32,
}

struct MarkerVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) outline_color: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

@vertex
fn vs_marker(
    vertex: VertexInput,
    instance: MarkerInstanceInput,
) -> MarkerVertexOutput {
    var out: MarkerVertexOutput;
    
    // Skaliere das Quad mit der Instanz-Größe
    let scaled_pos = vertex.position * instance.instance_size;
    
    // Verschiebe Pin nach oben (negatives Y = Bildschirm oben), damit die Spitze auf dem Node liegt
    // Pin-Spitze ist bei uv.y = -0.8, also vertex.position.y = 0.8 → kompensieren mit -0.8 * size
    let pin_offset = vec2<f32>(0.0, -0.8 * instance.instance_size);
    
    // Verschiebe zur Instanz-Position mit Offset
    let world_pos = scaled_pos + instance.instance_position + pin_offset;
    
    // Transformiere mit View-Projektion
    out.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    out.color = instance.instance_color;
    out.outline_color = instance.instance_outline_color;
    
    // UV invertieren (Y-Flip), damit Pin richtig herum ist
    out.uv = vec2<f32>(vertex.position.x, -vertex.position.y);
    
    return out;
}

@fragment
fn fs_marker(in: MarkerVertexOutput) -> @location(0) vec4<f32> {
    // Pin-Form: Kreis oben (y > 0) + Träne unten (y <= 0)
    let x = in.uv.x;
    let y = in.uv.y;
    
    // Kreis oben (zentriert bei y=0.3, radius=0.3)
    let circle_center = vec2<f32>(0.0, 0.3);
    let circle_radius = 0.3;
    let dist_to_circle = distance(in.uv, circle_center);
    
    let tip_y = -0.8;
    let hard_edges = uniforms.aa_params.y > 0.5;
    
    // Kombiniere Kreis und Tränenform mit AA aus View-Einstellungen
    var alpha: f32;
    if (y > 0.0) {
        // Oberer Bereich: Kreisförmig
        if (hard_edges) {
            alpha = select(0.0, 1.0, dist_to_circle <= circle_radius);
        } else {
            let edge = max(fwidth(dist_to_circle) * uniforms.aa_params.x, 0.0005);
            alpha = 1.0 - smoothstep(circle_radius - edge, circle_radius + edge, dist_to_circle);
        }
    } else {
        // Unterer Bereich: Tränenform — Breite nimmt nach unten ab
        let width_at_y = circle_radius * (1.0 - (abs(y) * 1.2));
        if (hard_edges) {
            let is_inside = abs(x) <= width_at_y && y >= tip_y;
            alpha = select(0.0, 1.0, is_inside);
        } else {
            let dist_to_side = width_at_y - abs(x);
            let dist_to_bottom = y - tip_y;
            let min_dist = min(dist_to_side, dist_to_bottom);
            let edge = max(fwidth(min_dist) * uniforms.aa_params.x, 0.0005);
            alpha = smoothstep(-edge, edge, min_dist);
        }
    }
    
    // Outline: dünner Rand
    let outline_thickness = 0.08;
    let is_outline: bool = 
        (y > 0.0 && abs(dist_to_circle - circle_radius) < outline_thickness) ||
        (y <= 0.0 && y >= tip_y && abs(abs(x) - (circle_radius * (1.0 - (abs(y) * 1.2)))) < outline_thickness);
    
    let final_color = select(in.color.rgb, in.outline_color.rgb, is_outline);
    
    return vec4<f32>(final_color, alpha);
}
