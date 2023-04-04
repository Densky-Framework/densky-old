import { HTTPRequest, IController } from "densky";

export default class Controller implements IController {
  GET(req: HTTPRequest) {
    return new Response(
      "DEEP: Matched (" + req.params.get("p2") + ") " + req.pathname,
    );
  }
}
