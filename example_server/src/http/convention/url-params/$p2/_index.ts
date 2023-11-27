import { HTTPRequest } from "densky/http-router.ts";

export function GET(req: HTTPRequest) {
  return new Response(
    "PARAM: Matched (" + req.params.get("p2") + ") " + req.pathname,
  );
}
