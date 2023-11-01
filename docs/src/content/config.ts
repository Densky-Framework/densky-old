import { defineCollection, z } from "astro:content";

export const collections = {
  docs: defineCollection({
    schema: ({ image: SchemaContext }) =>
      z.object({
        title: z.string(),
        sidebar: z.object({
          label: z.string().optional(),
          hidden: z.boolean().optional()
        }).optional()
      }),
  }),
};
