#version 300 es
precision mediump float;
in vec3 object_pos, surface_normal, view_dir, frag_bc;
uniform vec4 color;

float edgeFactor(){
	vec3 d = fwidth(frag_bc);
	vec3 a3 = smoothstep(vec3(0.0), d*1.5, frag_bc);
	return min(min(a3.x, a3.y), a3.z);
}

#define DIR 0
#define POINT 1
#define SPOT 2
#define MAX_NUM_LIGHTS 20

struct Light {
	vec3 position;
	vec3 color;

	vec3 direction;
	float cutoff;
	float outer_cutoff;

	float linear;
	float intensity;
	float quadratic;
};

vec3 calc_amb_light(Light light, vec3 f_color) {
	return light.color * f_color * light.intensity;
}

vec3 calc_light(Light light, vec3 normal, int type, vec3 f_color) {
	// light
	vec3 light_dir = (type == DIR) ? normalize(light.direction): normalize(light.position - object_pos); 
	
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

	// spot
	if (type == SPOT) {
		float theta = dot(light_dir , normalize(-light.direction));
		float epsilon = light.cutoff - light.outer_cutoff;
		float intensity = clamp((theta - light.outer_cutoff) / epsilon, 0.0, 1.0);
		return (diffuse + specular) * attenuation * f_color * light.intensity * intensity;
	}

	return (diffuse + specular) * attenuation * f_color * light.intensity;
}

//struct Material {
	//bool wire_overlay;
	//bool flat_shade;
	//bool has_albedo;
//}

uniform int num_l_amb, num_l_point, num_l_dir, num_l_spot;
uniform Light amb_lights[MAX_NUM_LIGHTS];
uniform Light point_lights[MAX_NUM_LIGHTS];
uniform Light dir_lights[MAX_NUM_LIGHTS];
uniform Light spot_lights[MAX_NUM_LIGHTS];
uniform bool flat_shade;
uniform bool wire_overlay;
uniform bool has_albedo;
in vec2 frag_tex;
uniform sampler2D sampler;

out vec4 outputColor;

void main() {
	// normals
	vec3 normal = flat_shade? normalize(cross(dFdx(object_pos),dFdy(object_pos))) : normalize(surface_normal);

	vec3 result = vec3(0.0,0.0,0.0);
	vec3 frag_color = has_albedo?
		texture(sampler, frag_tex).rgb:
		color.rgb;

	for (int i = 0; i < num_l_amb; i++) {
		result += calc_amb_light(amb_lights[i], frag_color);
	}
	for (int i = 0; i < num_l_dir; i++) {
		result += calc_light(dir_lights[i], normal, DIR, frag_color);
	}
	for (int i = 0; i < num_l_point; i++) {
		result += calc_light(point_lights[i], normal, POINT, frag_color);
	}
	for (int i = 0; i < num_l_spot; i++) {
		result += calc_light(spot_lights[i], normal, SPOT, frag_color);
	}
	
	outputColor = wire_overlay?
		vec4(mix(color.xyz,result, edgeFactor()), 1.0):
		vec4(result, 1.0);
}
