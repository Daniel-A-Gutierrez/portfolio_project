<script lang="ts">
    export interface Props {shader:string, height:string, width:string};
    import {Canvas} from '/src/scripts/glsl-canvas/glsl.js';
    import {onMount} from 'svelte'; 
    let {shader, height, width, ...rest} : Props = $props();
    let canvas : HTMLCanvasElement;
    let container : HTMLDivElement;
    onMount( () =>
    {
        if(!height && !width){console.warn("Height or width not set on Shader Canvas!");}

        const glsl = new Canvas(canvas, {
            alpha: false,
            antialias: true,
            mode: "flat",
            extensions: ["EXT_shader_texture_lod"]
        });
        glsl.load(shader);
        let resizer = () => {
            glsl.canvas.style="background-color:rgb(1,1,1);";
            glsl.canvas.width=container.offsetWidth;
            glsl.canvas.height=container.offsetHeight;
        };
        resizer();
        window.addEventListener('resize', resizer);
        return () => {window.removeEventListener('resize',resizer)};
    });
</script>
<div bind:this={container} style="width:{width}; height:{height};" {...rest}>
    <canvas bind:this={canvas}></canvas>
</div>

<style>
    canvas 
    {
        width: 100% !important;
        height: 100% !important;
    }
</style>