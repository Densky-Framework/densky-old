import { HTTPRequest } from "densky";

export function GET(req: HTTPRequest) {
  return Response.json(req.cookies.get(req.params.get("name")!));
}
