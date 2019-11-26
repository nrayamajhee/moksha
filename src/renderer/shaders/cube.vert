#version 300 es
in vec3 position;
uniform mat4 view, proj;
out vec3 frag_tex;

void main() {
	gl_Position = (proj * view * vec4(position, 1.0)).xyww;
	frag_tex = position;
}
