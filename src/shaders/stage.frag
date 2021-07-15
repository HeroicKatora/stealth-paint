#version 450
/** This is a special shader to convert to/from color spaces and texture
 * formats that are not natively supported. This works by introducing a staging
 * texture that is in the correct byte representation of the supposed format
 * but whose texel format is some chosen, supported format with the same byte
 * layout. This is then rendered to and from the final texture that is in
 * linear RGB space (or some other supported reference space).
 *
 * For example, consider Oklab, a recent perceptual color space combining good
 * aspects from HCL but with transfer functions that optimized towards
 * uniformity as measured in CIEDE2000. This is (of course) not yet supported
 * natively and even less so if we don't have any available native GL features.
 * Instead, when we need it for painting we load it from the staging texture,
 * where it is in Lab form, as a linear RGB texture. Using this fragment shader
 * you're viewing we then paint this onto an intermediate texture and in this
 * step we calculate the actual linear RGB values.
 *
 * In other conversions we might load a texture as u32 gray scale and demux
 * individual components. Note that this also allows us to do our own
 * quantization. In this way our calculation can happen in full floating point
 * precision but the result appears as if it performed in u8 or whatever the
 * input precision might be.
 *
 * Since the conversion share some amount of code this is a single source file
 * with multiple entry points.
 */
layout (location = 0) in vec2 uv;
layout (location = 0) out vec4 f_color;

/* Not all those bindings will be bound!
 */
layout (set = 1, binding = 0, r8ui) uniform restrict readonly uimage2D image_r8ui;
layout (set = 1, binding = 1, r16ui) uniform restrict readonly uimage2D image_r16ui;
layout (set = 1, binding = 2, r32ui) uniform restrict readonly uimage2D image_r32ui;
layout (set = 1, binding = 3, rgba16ui) uniform restrict readonly uimage2D image_rgba16ui;
layout (set = 1, binding = 4, rgba32ui) uniform restrict readonly uimage2D image_rgba32ui;

/* Output images. Same as input but writeonly instead.
 */
layout (set = 1, binding = 16, r8ui) uniform restrict writeonly uimage2D oimage_r8ui;
layout (set = 1, binding = 17, r16ui) uniform restrict writeonly uimage2D oimage_r16ui;
layout (set = 1, binding = 18, r32ui) uniform restrict writeonly uimage2D oimage_r32ui;
layout (set = 1, binding = 19, rgba16ui) uniform restrict writeonly uimage2D oimage_rgba16ui;
layout (set = 1, binding = 20, rgba32ui) uniform restrict writeonly uimage2D oimage_rgba32ui;

layout (set = 2, binding = 0, std140) uniform Parameter {
  uvec4 space;
} parameter;

// FIXME: this could and should be an auto-generated header with cbindgen

const uint TRANSFER_Bt709 = 0;
const uint TRANSFER_Bt470M = 1;
const uint TRANSFER_Bt601 = 2;
const uint TRANSFER_Smpte240 = 3;
const uint TRANSFER_Linear = 4;
const uint TRANSFER_Srgb = 5;
const uint TRANSFER_Bt2020_10bit = 6;
const uint TRANSFER_Bt2020_12bit = 7;
const uint TRANSFER_Smpte2084 = 8;
const uint TRANSFER_Bt2100Pq = 9;
const uint TRANSFER_Bt2100Hlg = 10;
const uint TRANSFER_LinearScene = 11;

uint get_transfer() {
  return parameter.space.x;
}

const uint SAMPLE_PARTS_A = 0;
const uint SAMPLE_PARTS_R = 1;
const uint SAMPLE_PARTS_G = 2;
const uint SAMPLE_PARTS_B = 3;
const uint SAMPLE_PARTS_Luma = 4;
const uint SAMPLE_PARTS_LumaA = 5;
const uint SAMPLE_PARTS_Rgb = 6;
const uint SAMPLE_PARTS_Bgr = 7;
const uint SAMPLE_PARTS_Rgba = 8;
const uint SAMPLE_PARTS_Rgbx = 9;
const uint SAMPLE_PARTS_Bgra = 10;
const uint SAMPLE_PARTS_Bgrx = 11;
const uint SAMPLE_PARTS_Argb = 12;
const uint SAMPLE_PARTS_Xrgb = 13;
const uint SAMPLE_PARTS_Abgr = 14;
const uint SAMPLE_PARTS_Xbgr = 15;
const uint SAMPLE_PARTS_Yuv = 16;

uint get_sample_parts() {
  return parameter.space.y;
}

const uint SAMPLE_BITS_Int8 = 0;
const uint SAMPLE_BITS_Int332 = 1;
const uint SAMPLE_BITS_Int233 = 2;
const uint SAMPLE_BITS_Int16 = 3;
const uint SAMPLE_BITS_Int4x4 = 4;
const uint SAMPLE_BITS_Inti444 = 5;
const uint SAMPLE_BITS_Int444i = 6;
const uint SAMPLE_BITS_Int565 = 7;
const uint SAMPLE_BITS_Int8x2 = 8;
const uint SAMPLE_BITS_Int8x3 = 9;
const uint SAMPLE_BITS_Int8x4 = 10;
const uint SAMPLE_BITS_Int16x2 = 11;
const uint SAMPLE_BITS_Int16x3 = 12;
const uint SAMPLE_BITS_Int16x4 = 13;
const uint SAMPLE_BITS_Int1010102 = 14;
const uint SAMPLE_BITS_Int2101010 = 15;
const uint SAMPLE_BITS_Int101010i = 16;
const uint SAMPLE_BITS_Inti101010 = 17;
const uint SAMPLE_BITS_Float16x4 = 18;
const uint SAMPLE_BITS_Float32x4 = 19;

uint get_sample_bits() {
  return parameter.space.z;
}

/** Forward declarations.
 *
 * For all signals in transfer functions we assume normalized values.
 */

vec4 demux_uint(uint, uint kind);
uint mux_uint(vec4, uint kind);

vec4 parts_normalize(vec4, uint);
vec4 parts_denormalize(vec4, uint);

vec4 parts_transfer(vec4, uint);
vec4 parts_untransfer(vec4, uint);

float transfer_oe_bt709(float val);
float transfer_eo_bt709(float val);

float transfer_oe_bt470m(float val);
float transfer_eo_bt470m(float val);

float transfer_oe_bt601(float val);
float transfer_eo_bt601(float val);

float transfer_oe_smpte240(float val);
float transfer_eo_smpte240(float val);

float transfer_oe_srgb(float val);
float transfer_eo_srgb(float val);

float transfer_oe_bt2020_10b(float val);
float transfer_eo_bt2020_10b(float val);

// Used in Bt.2100, this differentiates between scene, electrical, display light.
float transfer_eo_smpte2084(float val);
float transfer_eo_inv_smpte2084(float val);
float transfer_scene_display_smpte2084(float val);
float transfer_display_scene_smpte2084(float val);
float transfer_oe_smpte2084(float val);
float transfer_oe_inv_smpte2084(float val);

// Used Reference: BT.709-6, Section 1.2
float transfer_oe_bt709(float val) {
  // TODO: is there a numerically better way?
  if (val >= 0.018)
    return 1.099 * pow(val, 0.45) - 0.099;
  else
    return 4.500 * val;
}

// Used Reference: BT.709-6, Section 1.2, inverted.
float transfer_eo_bt709(float val) {
  // TODO: is there a numerically better way?
  if (val >= transfer_oe_bt709(0.018))
    return pow((val + 0.099) / 1.099, 1.0 / 0.45);
  else
    return val / 4.500;
}

// Used Reference: BT.470, Table 1, Item 5
float transfer_oe_bt470m(float val) {
  return pow(val, 1.0 / 2.200);
}

// Used Reference: BT.470, Table 1, Item 5
float transfer_eo_bt470m(float val) {
  return pow(val, 2.200); 
}

// Used Reference: BT.601-7, Section 2.6.4
float transfer_oe_bt601(float val) {
  return transfer_eo_bt709(val);
}

// Used Reference: BT.601-7, Section 2.6.4
float transfer_eo_bt601(float val) {
  return transfer_oe_bt709(val);
}

// Used Reference: https://www.kernel.org/doc/html/v4.11/media/uapi/v4l/pixfmt-007.html#colorspace-smpte-240m-v4l2-colorspace-smpte240m
float transfer_oe_smpte240(float val) {
  if (val < 0.0228)
    return 4.0 * val;
  else
    return 1.1115 * pow(val, 0.45);
}

// Used Reference: https://www.kernel.org/doc/html/v4.11/media/uapi/v4l/pixfmt-007.html#colorspace-smpte-240m-v4l2-colorspace-smpte240m
float transfer_eo_smpte240(float val) {
  if (val < 0.0913)
    return val / 4.0;
  else
    return pow(val / 1.1115, 1.0 / 0.045);
}

// Used Reference: https://www.kernel.org/doc/html/v4.11/media/uapi/v4l/pixfmt-007.html#
// Transfer function. Note that negative values for L are only used by the Y’CbCr conversion.
float transfer_oe_srgb(float val) {
  if (val < -0.0031308)
    return -1.055 * pow(-val, 1.0 / 2.4) + 0.055;
  else if (val <= 0.04045)
    return val * 12.92;
  else
    return 1.055 * pow(val, 1.0 / 2.4) - 0.055;
}

// Used Reference: https://www.kernel.org/doc/html/v4.11/media/uapi/v4l/pixfmt-007.html
float transfer_eo_srgb(float val) {
  if (val < -0.04045)
    return -pow((-val + 0.055) / 1.055, 2.4);
  else if (val <= 0.04045)
    return val / 12.92;
  else
    return pow((val + 0.055) / 1.055, 2.4);
}

// Used Reference: https://www.kernel.org/doc/html/v4.11/media/uapi/v4l/pixfmt-007.html#colorspace-bt-2020-v4l2-colorspace-bt2020
float transfer_oe_bt2020_10b(float val) {
  return transfer_oe_bt709(val);
}

// Used Reference: https://www.kernel.org/doc/html/v4.11/media/uapi/v4l/pixfmt-007.html#colorspace-bt-2020-v4l2-colorspace-bt2020
float transfer_eo_bt2020_10b(float val) {
  return transfer_eo_bt709(val);
}


// Used Reference: BT.2100-2, Table 4, Reference PQ EOTF
#define SMPTE2084_M1 (2610.0/16384.0)
#define SMPTE2084_M2 (2523.0/4096.0)
#define SMPTE2084_C1 (3424.0/4096.0)
#define SMPTE2084_C2 (2413.0/128.0)
#define SMPTE2084_C3 (2392.0/128.0)

// Used Reference: BT.2100-2, Table 4, Reference PQ EOTF
// Note: the output is _display_ color value Y and _not_ scene luminance.
float transfer_eo_smpte2084(float val) {
  float N = pow(val, 1.0 / SMPTE2084_M2);
  float nom = max(N - SMPTE2084_C1, 0.0);
  float denom = SMPTE2084_C2 - SMPTE2084_C3 * N;
  return pow(nom / denom, 1.0 / SMPTE2084_M1);
}
// Used Reference: BT.2100-2, Table 4, Reference PQ OETF
// Note: the input is _display_ color value Y and _not_ scene luminance.
float transfer_eo_inv_smpte2084(float val) {
  float Y = pow(val, SMPTE2084_M1);
  float nom = SMPTE2084_C1 + SMPTE2084_C2 * Y;
  float denom = SMPTE2084_C3 * Y + 1.0;
  return pow(nom / denom, SMPTE2084_M2);
}

// Used Reference: BT.2100-2, Table 4, Reference PQ OOTF
// Used Reference: Python `colour science`: https://github.com/colour-science/colour/blob/a196f9536c44e2101cde53446550d64303c0ab46/colour/models/rgb/transfer_functions/itur_bt_2100.py#L276
// IMPORTANT: we map to a normalized linear color range Y, and _not_ to display luminance F_D.
float transfer_scene_display_smpte2084(float val) {
  float e_prime = transfer_oe_bt709(59.5208 * val);
  return pow(e_prime, 2.4) / 100.0;
}

// Used Reference: BT.2100-2, Table 4, Reference PQ OOTF
float transfer_display_scene_smpte2084(float val) {
  float e_prime = pow(val * 100.0, 1.0 / 2.4);
  return transfer_eo_bt709(e_prime) / 59.5208;
}

float transfer_oe_smpte2084(float val) {
  return transfer_eo_inv_smpte2084(transfer_scene_display_smpte2084(val));
}
float transfer_oe_inv_smpte2084(float val) {
  return transfer_display_scene_smpte2084(transfer_eo_smpte2084(val));
}

// TODO: https://github.com/colour-science/colour/blob/a196f9536c44e2101cde53446550d64303c0ab46/colour/models/rgb/transfer_functions/arib_std_b67.py#L108
vec3 transfer_scene_display_bt2100hlg(vec3 rgb) {
  return vec3(0.0);
}

// GLSL compiler wants this...
void main() {}

/** All decode methods work in several stages:
 *
 * 1. Demux the bit-encoded components into a vector.
 * 2. Reorder the components into a normalized form for the color type.
 * 3. Apply transfer function (and primary transform such as YUV).
 * 4. We now hold a vector of floating point linear color encoding, write it.
 *
 * The encoding works the other way around. Note that there are some invalid
 * combinations (Bits::Int332 and Parts::A for example) and it is expected that
 * the calling layer handles those.
 */

void decode_r8ui() {
  uint num = imageLoad(image_r8ui, ivec2(gl_FragCoord)).x;
  vec4 components = demux_uint(num, get_sample_bits());

  // FIXME: YUV transform and accurate YUV transform.
  vec4 electrical = parts_normalize(components, get_sample_parts());
  vec4 primaries = parts_untransfer(electrical, get_transfer());

  f_color = primaries;
}

void encode_r8ui() {
}

vec4 demux_uint(uint num, uint kind) {
  switch (kind) {
  case SAMPLE_BITS_Int8:
    return vec4(num);
  case SAMPLE_BITS_Int332:
    return vec4(num & 0x3, (num >> 2) & 0xf, num >> 5, 0.0);
  case SAMPLE_BITS_Int233:
    return vec4(num & 0xf, (num >> 3) & 0xf, num >> 6, 0.0);
  }
}

vec4 parts_normalize(vec4 components, uint parts) {
  switch (parts) {
  case SAMPLE_PARTS_A:
    return vec4(0.0, 0.0, 0.0, components.x);
  case SAMPLE_PARTS_R:
    return vec4(components.x, 0.0, 0.0, 1.0);
  case SAMPLE_PARTS_G:
    return vec4(0.0, components.x, 0.0, 1.0);
  case SAMPLE_PARTS_B:
    return vec4(0.0, 0.0, components.x, 1.0);
  case SAMPLE_PARTS_Luma:
    return vec4(vec3(components.x), 1.0);
  case SAMPLE_PARTS_LumaA:
    return vec4(vec3(components.x), 1.0);
  case SAMPLE_PARTS_Rgb:
  case SAMPLE_PARTS_Rgbx:
    return vec4(components.xyz, 1.0);
  case SAMPLE_PARTS_Bgr:
  case SAMPLE_PARTS_Bgrx:
    return vec4(components.zyx, 1.0);
  case SAMPLE_PARTS_Rgba:
    return components.xyzw;
  case SAMPLE_PARTS_Bgra:
    return components.xyzw;
  case SAMPLE_PARTS_Argb:
    return components.yzwx;
  case SAMPLE_PARTS_Abgr:
    return components.wzyx;
  case SAMPLE_PARTS_Xrgb:
    return vec4(components.yzw, 1.0);
  case SAMPLE_PARTS_Xbgr:
    return vec4(components.wzy, 1.0);
  }
}

vec4 parts_transfer(vec4 electrical, uint function) {
#define TRANSFER_WITH_XYZ(E, FN) vec4(FN(E.x), FN(E.y), FN(E.z), E.a)
  switch (function) {
  case TRANSFER_Bt709:
  return TRANSFER_WITH_XYZ(electrical, transfer_eo_bt709);
  case TRANSFER_Bt470M:
  return TRANSFER_WITH_XYZ(electrical, transfer_eo_bt470m);
  case TRANSFER_Bt601:
  return TRANSFER_WITH_XYZ(electrical, transfer_eo_bt601);
  case TRANSFER_Smpte240:
  return TRANSFER_WITH_XYZ(electrical, transfer_eo_smpte240);
  case TRANSFER_Linear:
  return electrical;
  case TRANSFER_Srgb:
  return TRANSFER_WITH_XYZ(electrical, transfer_eo_srgb);
  case TRANSFER_Bt2020_10bit:
  case TRANSFER_Bt2020_12bit:
  return TRANSFER_WITH_XYZ(electrical, transfer_eo_bt2020_10b);
  case TRANSFER_Smpte2084:
  return TRANSFER_WITH_XYZ(electrical, transfer_eo_smpte2084);
  return TRANSFER_WITH_XYZ(electrical, transfer_eo_smpte2084);
  case TRANSFER_Bt2100Hlg:
  // FIXME: unimplemented.
  return electrical;
  }
}
