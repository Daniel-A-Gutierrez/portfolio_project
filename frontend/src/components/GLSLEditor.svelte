
<script>
    import {onMount} from "svelte";

    let {uid,...rest} = $props();

    let container;
    const mobileBreakpoint = 576;


    function calcResponsiveWidth()
    {
        if(container == null){return}
        let w = Math.min(window.innerWidth, container.offsetWidth);
        return w < mobileBreakpoint ? container.offsetWidth : container.offsetWidth / 2;
    }

    onMount( () => {

        let glslEditor = new GlslEditor(`#${uid}`, { 
            canvas_size: calcResponsiveWidth(),
            canvas_draggable: false,
            theme: 'monokai',
            multipleBuffers: false,
            watchHash: true,
            fileDrops: true,
            menu: false,
            lineWrapping: false
        });
        console.log(glslEditor);
        let resizer =  () => {
            let w = calcResponsiveWidth();
            glslEditor.shader.canvas.canvas.style="background-color:rgb(1,1,1);";
            glslEditor.shader.canvas.canvas.width=w;
            //glslEditor.shader.canvas.canvas.height=w;
        };
        window.addEventListener('resize',resizer);
        return () => {window.removeEventListener('resize',resizer)};
    });
</script>

<div id={uid} bind:this={container} class="glsl-editor-container" {...rest}></div>

<style define:vars={{uid}}>
    :global{
        .glsl-editor-container{
            display:flex;
            flex-direction: column;
        }

        .glsl-editor-container > .ge_editor {
            width: 100%;
            order:2;
        }
        
        .glsl-editor-container > .ge_canvas_container 
        {
            display:block;
            width: 100%;
            position:unset;
            order:1;
        }
        .glsl-editor-container > .ge_canvas_container > canvas 
        {
            display:block;
            margin-left:auto;
            margin-right:auto;
        }
        @media (min-width:576px) {
            .glsl-editor-container > .ge_canvas_container 
            {
                order:2;
                width: 50%;
            }
            .glsl-editor-container{
                flex-direction: row;
            }
            .glsl-editor-container > .ge_editor {
                order: 1;
                width: 50%;
            }
            .glsl-editor-container > .ge_canvas_container > canvas 
            {
                margin-left:unset;
                margin-right:unset;
            }
        }
    }
</style>

