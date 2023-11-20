import { setCWD, type CompileOptions } from "densky/mod.ts";

// I will not need this anymore.
const pathname = new URL(import.meta.resolve("./")).pathname;
setCWD(pathname);

console.log("Running on " + pathname);

export default {
  routesPath: "src/routes",
  wsPath: "src/ws",
  staticPath: "src/static",
  staticPrefix: "/static",
  viewsPath: "src/views",
  verbose: true,
  // dynamicHtml: true,
} as CompileOptions;
