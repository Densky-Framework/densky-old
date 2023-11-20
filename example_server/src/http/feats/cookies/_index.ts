import { HTTPRequest } from "densky/http-router.ts";

export function GET(req: HTTPRequest) {
  return Response.json(req.cookies.raw);
}
