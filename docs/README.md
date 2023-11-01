# Densky Framework Documentation

[![Built with Astro](https://astro.badg.es/v2/built-with-astro/tiny.svg)](https://astro.build)

## 🚀 Project Structure

Inside of your Astro , you'll see the following folders and files:

```
.
├── public/
├── src/
│   ├── assets/
│   ├── components/
│   ├── content/
│   │   ├── docs/
│   │   └── config.ts
│   ├── icons/
│   ├── pages/
│   ├── styles/
│   ├── utils/
│   └── env.d.ts
├── astro.config.mjs
├── package.json
└── tsconfig.json
```

Astro looks for `.md` or `.mdx` files in the `src/content/docs/` directory. Each file is exposed as a route based on its file name.

Images can be added to `src/assets/` and embedded in Markdown with a relative link.

Static assets, like favicons, can be placed in the `public/` directory.

## 🧞 Commands

All commands are run from the root of the project, from a terminal:

| Command                         | Action                                           |
| :------------------------       | :----------------------------------------------- |
| `npm install` or `bun install` | Installs dependencies                            |
| `npm run dev`                  | Starts local dev server at `localhost:4321`      |
| `npm run build`                | Build your production site to `./dist/`          |
| `npm run preview`              | Preview your build locally, before deploying     |
| `npm run astro ...`            | Run CLI commands like `astro add`, `astro check` |
| `npm run astro -- --help`      | Get help using the Astro CLI                     |

