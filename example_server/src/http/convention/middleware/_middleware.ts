import { HTTPRequest } from "densky/http-router.ts";

export function GET(req: HTTPRequest) {
  if (req.url.searchParams.has("mid")) {
    return new Response("MIDDLEWARE: Matched " + req.pathname);
  }
}
