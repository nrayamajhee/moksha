#version 300 es
precision mediump float;
in vec3 frag_tex;
out vec4 outputColor;
uniform samplerCube sampler;

void main() {
	outputColor = texture(sampler, frag_tex);
}
