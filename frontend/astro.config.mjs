// @ts-check
import { defineConfig } from 'astro/config';
import purgecss from 'astro-purgecss';
import mdx from '@astrojs/mdx';
import min from 'astro-min';

import playformCompress from '@playform/compress';

// https://astro.build/config
export default defineConfig({
  integrations: [mdx(), purgecss(), playformCompress()],
});