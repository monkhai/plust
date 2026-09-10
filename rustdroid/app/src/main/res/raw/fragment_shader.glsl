precision mediump float;

varying vec2 vTexCoord;
uniform sampler2D yTexture;
uniform sampler2D uvTexture;

void main() {
    float y = texture2D(yTexture, vTexCoord).r;
    float u = texture2D(uvTexture, vTexCoord).r - 0.5;
    float v = texture2D(uvTexture, vTexCoord).a - 0.5;

    float r = y + 1.402 * v;
    float g = y - 0.344 * u - 0.714 * v;
    float b = y + 1.772 * u;

    gl_FragColor = vec4(r, g, b, 1.0);
}