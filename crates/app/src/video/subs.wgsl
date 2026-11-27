// Capas de subtítulo: un quad por bitmap de libass, encima del video.
//
// Todo lo que cambia por capa llega como atributo de instancia (rectángulo en
// clip space, rectángulo en el atlas y color), así que hay un solo draw call
// para todas las capas aunque sean muchas.

struct Capa {
    // x0, y0, x1, y1 en clip space, ya con el letterbox aplicado por la CPU.
    @location(0) rect: vec4<f32>,
    // u0, v0, u1, v1 dentro del atlas de máscaras.
    @location(1) uv: vec4<f32>,
    // RGB del estilo y alfa de la capa.
    @location(2) color: vec4<f32>,
};

struct Salida {
    @builtin(position) posicion: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(0) @binding(0) var atlas: texture_2d<f32>;
@group(0) @binding(1) var muestreo: sampler;

// Dos triángulos por capa. El orden importa: (0,0) (1,0) (0,1) y (0,1) (1,0)
// (1,1) recorren el quad sin quedar del revés (el culling está apagado, pero el
// orden deja la uv coherente en toda la cara).
const ESQUINAS = array<vec2<f32>, 6>(
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
);

// sRGB -> lineal. El color del `.ass` es sRGB y el target es sRGB: el hardware
// ya lo trata en lineal, así que hay que convertir aquí o los colores a medio
// tono salen oscurecidos (los puros, como el rojo, no se notan).
fn a_lineal(c: vec3<f32>) -> vec3<f32> {
    let bajo = c / 12.92;
    let alto = pow((c + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4));
    return select(alto, bajo, c <= vec3<f32>(0.04045));
}

@vertex
fn vs_main(@builtin(vertex_index) indice: u32, capa: Capa) -> Salida {
    let esquina = ESQUINAS[indice];

    var salida: Salida;
    salida.posicion = vec4<f32>(
        mix(capa.rect.x, capa.rect.z, esquina.x),
        mix(capa.rect.y, capa.rect.w, esquina.y),
        0.0,
        1.0,
    );
    salida.uv = mix(capa.uv.xy, capa.uv.zw, esquina);
    salida.color = capa.color;
    return salida;
}

@fragment
fn fs_main(entrada: Salida) -> @location(0) vec4<f32> {
    // La máscara es cobertura: 0 no pinta, 1 pinta entero.
    let cobertura = textureSample(atlas, muestreo, entrada.uv).r;
    let alfa = cobertura * entrada.color.a;

    // Blending por alfa normal (SrcAlpha / OneMinusSrcAlpha): la GPU multiplica
    // el color por este alfa, así que aquí se devuelve el color sin premultiplicar.
    return vec4<f32>(a_lineal(entrada.color.rgb), alfa);
}
