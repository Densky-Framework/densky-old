import { HookEmitter, Promisable } from "densky/cloud.ts";
import { HTTPRequest } from "./request.ts";

export type HttpHooks = {
  beforeRequest(req: HTTPRequest): Promisable<HTTPRequest | null | undefined>;
  request(req: HTTPRequest): Promisable<Response | null | undefined>;
};

export const HTTP_HOOKS = new HookEmitter<HttpHooks>();
