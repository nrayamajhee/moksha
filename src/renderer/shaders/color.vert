#version 300 es
uniform mat4 model, view, proj;
uniform vec3 eye;
uniform bool wire_overlay;
uniform bool has_albedo;

in vec3 position, normal, barycentric;
in vec2 tex_coords;
out vec3 surface_normal, object_pos, view_dir, frag_bc;
out vec2 frag_tex;

void main() {
	object_pos = vec3(model * vec4(position, 1.0));
	gl_Position = proj * view * vec4(object_pos, 1.0);
	surface_normal = mat3(transpose(inverse(model))) * normal;
	view_dir = normalize(eye - object_pos);
	if (has_albedo) {
		frag_tex = tex_coords;
	}
	if (wire_overlay) {
		frag_bc = barycentric;
	}
}
