import { HTTPError, HTTPRequest, StatusCode } from "densky/http-router.ts";

export default async (req: HTTPRequest) => {
  const data = req.data.get("data");

  if (!data) return new HTTPError(StatusCode.BAD_REQUEST, "Expecting data");

  await req.cookies.set(req.params.get("name")!, data, {
    path: "/",
    expires: new Date(Date.now() + 50000),
  });

  return new Response("OK", { status: StatusCode.OK });
};
