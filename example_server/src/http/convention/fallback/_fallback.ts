import { HTTPRequest } from "densky";

export function GET(req: HTTPRequest) {
  return new Response("FALLBACK: Matched " + req.pathname);
}
