import { HTTPRequest, IController } from "densky";

export default class Controller implements IController {
  GET(req: HTTPRequest) {
    return new Response("ROUTE: Matched " + req.pathname);
  }
}
