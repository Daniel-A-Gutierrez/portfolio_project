// 1. Import utilities from `astro:content`
import { defineCollection, z } from 'astro:content';

// 2. Import loader(s)
import { glob, file,  } from 'astro/loaders';

// 3. Define your collection(s)
const blog = defineCollection({ loader : glob({base : "src/posts", pattern : "*.mdx" } ),
    schema : z.object({
        title : z.string(),
        description : z.string(),
        preview : z.string(),
        postnum : z.number().optional(),
        published : z.string(),
        layout : z.undefined().or(z.string()),
        tags : z.undefined().or(z.array(z.string())),
        draft : z.undefined().or(z.boolean())
    })});

// 4. Export a single `collections` object to register your collection(s)
export const collections = { blog };

// for more info on collections : https://docs.astro.build/en/guides/content-collections/