import { HTTPRequest } from "densky";

export function GET(req: HTTPRequest) {
  if (req.url.searchParams.has("mid")) {
    return new Response("MIDDLEWARE: Matched " + req.pathname);
  }
}
