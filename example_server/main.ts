import "./config.ts";
import { Server } from "densky";
import requestHandler from "./.densky/main.ts";

const server = new Server({ port: 8000, verbose: true }, requestHandler);

server.start();
