attribute vec4 aPosition; // input: vertext position
attribute vec2 aTexCoord; // input: texture coordinate (position)
varying vec2 vTexCoord; // output: passed to fragment shader

void main() {
    gl_Position = aPosition;
    vTexCoord = aTexCoord;
}
