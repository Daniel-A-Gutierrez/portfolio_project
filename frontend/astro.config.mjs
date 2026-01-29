// @ts-check
import { defineConfig, passthroughImageService } from 'astro/config';
import purgecss from 'astro-purgecss';
import mdx from '@astrojs/mdx';

import playformCompress from '@playform/compress';

import svelte from '@astrojs/svelte';

// https://astro.build/config
export default defineConfig({
  //image: {service: passthroughImageService()},
  integrations: [mdx(), purgecss(), svelte(), playformCompress()],
  
});