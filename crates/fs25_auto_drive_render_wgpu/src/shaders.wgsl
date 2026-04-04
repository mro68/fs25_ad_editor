// Shader fuer Node-Rendering (2D)

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
    @location(2) rim_color: vec4<f32>,     // aussen / Markierung
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
    
    // Skaliere das Quad mit der Instanz-Groesse
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
    // Kreisfoermige Form via Distance Field
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(in.uv, center);

    let radius = 0.48;
    let norm_dist = dist / radius;

    // Selektion-Darstellung: aa_params.z → 0=Gradient, 1=Ring
    // rim_color.a kodiert Innendurchmesser/Aussendurchmesser (ID/AD).
    let style = uniforms.aa_params.z;
    let inner_ratio = clamp(in.rim_color.a, 0.0, 1.0);
    let gradient_hold_ratio = clamp(0.5 * inner_ratio * inner_ratio, 0.0, 1.0);
    var rgb: vec3<f32>;

    if (style > 0.5) {
        // Ring:
        // AD = Groessenfaktor/100 * Nodedurchmesser
        // ID = Nodedurchmesser
        // -> bis ID bleibt die Nodefarbe, zwischen ID und AD erscheint die Ringfarbe.
        let ring_t = smoothstep(inner_ratio - 0.02, inner_ratio + 0.02, norm_dist);
        rgb = mix(in.color.rgb, in.rim_color.rgb, ring_t);
    } else {
        // Farbverlauf:
        // - Zentrum = Nodefarbe
        // - bei (50/Groessenfaktor) * Nodedurchmesser = weiterhin Nodefarbe
        // - bei AD = Selektionsfarbe
        let mix_t = smoothstep(gradient_hold_ratio, 1.0, norm_dist);
        rgb = mix(in.color.rgb, in.rim_color.rgb, mix_t);
    }

    // Screen-space adaptives Anti-Aliasing am Rand (unveraendert)
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
    
    // in.position ist in Weltkoordinaten (z.B. -1024..1024 fuer 2048x2048 Map)
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
    
    // Verwende Texture-Alpha * Opacity fuer finales Alpha
    let final_alpha = texture_color.a * background_uniforms.opacity;
    
    return vec4<f32>(texture_color.rgb, final_alpha);
}

// === Map-Marker-Rendering (Pin-Symbol als Textur) ===

// Textur-Bindings fuer das Pin-Icon (group(0), bindings 1+2)
@group(0) @binding(1)
var marker_texture: texture_2d<f32>;
@group(0) @binding(2)
var marker_sampler: sampler;

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
    
    // Skaliere das Quad mit der Instanz-Groesse, Breite um 25% reduzieren
    let scaled_pos = vec2<f32>(vertex.position.x, vertex.position.y) * instance.instance_size;
    
    // Verschiebe Pin nach unten (negatives Y), damit die Spitze auf dem Node liegt
    // Pin-Spitze ist bei uv.y = 0 (unten nach Y-Flip-Entfernung) → verschieben mit -size
    let pin_offset = vec2<f32>(0.0, -instance.instance_size);
    
    // Verschiebe zur Instanz-Position mit Offset
    let world_pos = scaled_pos + instance.instance_position + pin_offset;
    
    // Transformiere mit View-Projektion
    out.clip_position = uniforms.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    out.color = instance.instance_color;
    out.outline_color = instance.instance_outline_color;
    
    // UV: [-1..1] → [0..1], keine Y-Flip (Textur-Y=0 ist unten = Pin-Spitze)
    out.uv = vec2<f32>(vertex.position.x * 0.5 + 0.5, vertex.position.y * 0.5 + 0.5);
    
    return out;
}

@fragment
fn fs_marker(in: MarkerVertexOutput) -> @location(0) vec4<f32> {
    // Textur-Sampling: Pin-Icon als Textur mit Alpha-Maske
    let tex_color = textureSample(marker_texture, marker_sampler, in.uv);

    // Transparente Bereiche verwerfen
    if (tex_color.a < 0.01) {
        discard;
    }

    // Tinting: Textur-Alpha definiert die Pin-Form, instance_color faerbt den Pin.
    // Strichdicke wird durch stroke-width im SVG gesteuert (zur Laufzeit neu rasterisiert).
    return vec4<f32>(in.color.rgb, tex_color.a * in.color.a);
}
