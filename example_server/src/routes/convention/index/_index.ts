import { HTTPRequest } from "densky";

export function GET(req: HTTPRequest) {
  return new Response("INDEX: Matched " + req.pathname);
}
