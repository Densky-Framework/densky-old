import { HTTPRequest } from "densky";

export function GET(req: HTTPRequest) {
  return new Response("ROUTE: Matched " + req.pathname);
}
