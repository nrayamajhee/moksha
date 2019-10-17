#version 300 es
uniform mat4 model, view, proj;
uniform vec3 eye;

in vec3 position, normal;
out vec3 surface_normal, object_pos, view_dir;

void main() {
	object_pos = vec3(model * vec4(position, 1.0));
	gl_Position = proj * view * vec4(object_pos, 1.0);
	surface_normal = mat3(transpose(inverse(model))) * normal;
	view_dir = normalize(eye - object_pos);
}
