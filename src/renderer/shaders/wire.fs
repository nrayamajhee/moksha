#version 300 es
precision mediump float;
in vec4 f_color;
in vec3 frag_bc;
out vec4 outputColor;

float edgeFactor(){
	vec3 d = fwidth(frag_bc);
	vec3 a3 = smoothstep(vec3(0.0), d*1.5, frag_bc);
	return min(min(a3.x, a3.y), a3.z);
}

void main() {
	if(gl_FrontFacing){
		outputColor = vec4(f_color.xyz, (1.0-edgeFactor())*0.95);
	}
	else{
		outputColor = vec4(f_color.xyz, (1.0-edgeFactor())*0.2);
	}

}
