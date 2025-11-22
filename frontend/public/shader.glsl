    
#ifdef GL_ES
precision mediump float;
#endif

#define iTime u_time
#define iResolution vec3(u_resolution, 1.0)

uniform float u_time;
uniform vec2 u_resolution;

#define PI 3.14159
#define e  2.71828

void mainImage( out vec4 fragColor, in vec2 fragCoord )
{
    vec2 uv  = (fragCoord-0.5*iResolution.xy)/iResolution.y;
    fragColor = vec4((sin(u_time)-length(uv)) * (sin(u_time)));
}

void main() {
    vec4 color = vec4(0.0);
    mainImage(color, gl_FragCoord.xy);
    gl_FragColor = color;
}

