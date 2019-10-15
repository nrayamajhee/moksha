#version 300 es
precision mediump float;
in vec3 object_pos, surface_normal, view_dir;
uniform vec4 color;

struct AmbientLight {
	vec3 color;
};

vec3 calc_amb_light(AmbientLight light) {
	return light.color * color.rgb;
}

struct PointLight {
	vec3 position;
	vec3 color;
	float linear;
	float quadratic;
};

vec3 calc_point_light(PointLight light, vec3 normal) {
	// light
	vec3 light_dir = normalize(light.position - object_pos); 
	
	// diffuse
	float diff = max(dot(normal, light_dir), 0.0);
	vec3 diffuse = diff * light.color;

	// specular
	float spec_fac = 1.0;
	vec3 reflection = reflect(-light_dir, normal);
	float spec = pow(max(dot(view_dir, reflection), 0.0), 64.0);
	vec3 specular = spec_fac * spec * light.color;

	// attenuation
	float distance = length(light.position - object_pos);
	float attenuation = 1.0 / (1.0 + light.linear * distance + light.quadratic * (distance * distance));

	return (diffuse + specular) * attenuation * color.rgb;
}

uniform int num_l_amb, num_l_point;
uniform AmbientLight amb_lights[10];
uniform PointLight point_lights[10];
uniform bool flat_shade;

out vec4 outputColor;

void main() {
	// normals
	vec3 normal = normalize(surface_normal);
	if (flat_shade) {
		normal = normalize(cross(dFdx(object_pos),dFdy(object_pos)));
	}

	vec3 result = vec3(0.0,0.0,0.0);
	for (int i = 0; i < num_l_amb; i++) {
		result += calc_amb_light(amb_lights[i]);
	}
	for (int i = 0; i < num_l_point; i++) {
		result += calc_point_light(point_lights[i], normal);
	}
	
	outputColor = vec4(result, 1.0);
}
