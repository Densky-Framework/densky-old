import { HTTPRequest, IController } from "densky";

export default class Controller implements IController {
  GET(req: HTTPRequest) {
    return new Response(
      "INDEX: Matched " + req.pathname +
        "\nYou can use ?mid in the url for test middleware",
    );
  }
}
