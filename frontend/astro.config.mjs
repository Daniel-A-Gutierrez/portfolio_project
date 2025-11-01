// @ts-check
import { defineConfig } from 'astro/config';
import purgecss from 'astro-purgecss';
import mdx from '@astrojs/mdx';

import playformCompress from '@playform/compress';

import svelte from '@astrojs/svelte';

// https://astro.build/config
export default defineConfig({
  integrations: [mdx(), purgecss(), svelte()],
});