import { HTTPRequest } from "densky/http-router.ts";

export function GET(req: HTTPRequest) {
  return new Response("ROUTE: Matched " + req.pathname);
}
