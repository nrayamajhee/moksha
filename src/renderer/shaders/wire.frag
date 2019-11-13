#version 300 es
// Thanks to Florian Boesh for his tutorial on how to achieve fancy wireframe with
// barycentric coordinates. Please refer to the following url for further details:
// <http://codeflow.org/entries/2012/aug/02/easy-wireframe-display-with-barycentric-coordinates/>
precision mediump float;
uniform vec4 color;
in vec3 frag_bc;
out vec4 outputColor;

float edgeFactor(){
	vec3 d = fwidth(frag_bc);
	vec3 a3 = smoothstep(vec3(0.0), d*1.5, frag_bc);
	return min(min(a3.x, a3.y), a3.z);
}

void main() {
	if(gl_FrontFacing){
		outputColor = vec4(color.xyz, (1.0-edgeFactor())*0.95);
	}
	else{
		outputColor = vec4(color.xyz, (1.0-edgeFactor())*0.2);
	}

}
