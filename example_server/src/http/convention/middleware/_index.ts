import { HTTPRequest } from "densky";

export function GET(req: HTTPRequest) {
  return new Response(
    "INDEX: Matched " + req.pathname +
      "\nYou can use ?mid in the url for test middleware",
  );
}
