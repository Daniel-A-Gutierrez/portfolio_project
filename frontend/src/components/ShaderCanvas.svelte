<script lang="ts">
    export interface Props {shader:string};
    import {Canvas} from 'glsl-canvas-js';
    import {ContextMode} from 'glsl-canvas-js/dist/cjs/context/context';
    import {onMount} from 'svelte'; 
    let {shader, ...rest} : Props = $props();
    let canvas : HTMLCanvasElement;
    let container : HTMLDivElement;
    onMount( () =>
    {
        const glsl = new Canvas(canvas, {
            alpha: false,
            antialias: true,
            mode: ContextMode.Flat,
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
<div bind:this={container} {...rest}>
    <canvas bind:this={canvas} ></canvas>
</div>