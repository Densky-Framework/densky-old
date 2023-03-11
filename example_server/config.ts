import { CompileOptions } from "densky/compiler.ts";
import { Globals } from "densky";

const pathname = new URL(import.meta.resolve("./")).pathname;
Globals.cwd = pathname;

console.log("Running on " + pathname);

export const compileOptions: CompileOptions = {
  routesPath: "src/routes",
  wsPath: "src/ws",
  staticPath: "src/static",
  staticPrefix: "/static",
  viewsPath: "src/views",
  verbose: true,
  dynamicHtml: true,
};
