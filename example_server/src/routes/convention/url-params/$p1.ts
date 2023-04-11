import { HTTPRequest } from "densky";

export function GET(req: HTTPRequest) {
  return new Response(
    "PARAM: Matched (" + req.params.get("p1") + ") " + req.pathname,
  );
}
