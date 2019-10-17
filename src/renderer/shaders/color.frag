#version 300 es
// Thanks to Florian Boesh for his tutorial on how to achieve fancy wireframe with
// barycentric coordinates. Please refer to the following url for further details:
// <http://codeflow.org/entries/2012/aug/02/easy-wireframe-display-with-barycentric-coordinates/>
precision mediump float;
in vec3 object_pos, surface_normal, view_dir;
uniform vec4 color;


struct Light {
	vec3 position;
	vec3 color;
	float linear;
	float quadratic;
};

vec3 calc_amb_light(Light light) {
	return light.color * color.rgb;
}

vec3 calc_light(Light light, vec3 normal, bool dir) {
	// light
	//vec3 light_dir = dir? normalize(-light.position): normalize(light.position - object_pos); 
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

uniform int num_l_amb, num_l_point, num_l_dir;
uniform Light amb_lights[10];
uniform Light point_lights[10];
uniform Light dir_lights[10];
uniform bool flat_shade;

out vec4 outputColor;

void main() {
	// normals
	vec3 normal = flat_shade? normalize(cross(dFdx(object_pos),dFdy(object_pos))) : normalize(surface_normal);

	vec3 result = vec3(0.0,0.0,0.0);
	for (int i = 0; i < num_l_amb; i++) {
		result += calc_amb_light(amb_lights[i]);
	}
	for (int i = 0; i < num_l_dir; i++) {
		result += calc_light(dir_lights[i], normal, true);
	}
	for (int i = 0; i < num_l_point; i++) {
		result += calc_light(point_lights[i], normal, false);
	}
	
	outputColor = vec4(result, 1.0);
}
