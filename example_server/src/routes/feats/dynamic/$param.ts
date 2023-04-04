import { HTTPRequest, HTTPResponse, IController } from "densky";

export default class _ implements IController {
  async GET(req: HTTPRequest) {
    const condition =
      (req.url.searchParams.get("condition") || "true") === "true";
    const param = req.params.get("param") || "PARAM";
    const num = parseInt(req.url.searchParams.get("num") || "0") || 0;

    return await HTTPResponse.view("dynamic.html", {
      condition,
      param,
      num,
      title: "Dynamic",
    });
  }
}
