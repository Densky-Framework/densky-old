import { HTTPError, HTTPRequest, IController, StatusCode } from "densky";

export default class _ implements IController {
  async ANY(req: HTTPRequest) {
    const data = req.data.get("data");

    if (!data) return new HTTPError(StatusCode.BAD_REQUEST, "Expecting data");

    await req.cookies.set(req.params.get("name")!, data, {
      path: "/",
      expires: new Date(Date.now() + 50000),
      raw: true,
    });

    return new Response("OK", { status: StatusCode.OK });
  }
}
