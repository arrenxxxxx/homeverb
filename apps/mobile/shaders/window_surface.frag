#include <flutter/runtime_effect.glsl>

uniform vec2 origin;
uniform vec2 extent;
uniform float pixelRatio;
uniform vec4 firstColor;
uniform vec4 lastColor;
out vec4 fragColor;

void main() {
  // Sample at physical pixel origins and truncate RGB, as in the approved artwork.
  vec2 position = FlutterFragCoord().xy - origin - vec2(0.5 / pixelRatio);
  float blend = clamp(dot(position / extent, vec2(0.35, 0.65)), 0.0, 1.0);
  vec4 value = mix(firstColor, lastColor, blend);
  vec3 rgb = floor(value.rgb * 255.0 + 0.00001) / 255.0;
  fragColor = vec4(rgb * value.a, value.a);
}
