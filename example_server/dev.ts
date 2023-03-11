import { DevServer } from "densky";
import { compileOptions } from "./config.ts";

const server = new DevServer({ port: 8000, verbose: true }, compileOptions);

server.start();
