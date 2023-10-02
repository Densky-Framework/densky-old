import { HTTPRequest } from "densky";

export function GET(req: HTTPRequest) {
  return new Response(
    "DEEP: Matched (" + req.params.get("p2") + ") " + req.pathname,
  );
}
