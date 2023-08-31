import { HTTPRequest, HTTPResponse } from "densky";

export async function GET(req: HTTPRequest) {
  const condition =
    (req.url.searchParams.get("condition") || "true") === "true";
  const param = req.params.get("param") || "PARAM";
  const num = parseInt(req.url.searchParams.get("num") || "0") || 0;

  return await HTTPResponse.view("dynamic", {
    condition,
    param,
    num,
    obj: {hi: 1},
    title: "Dynamic",
  });
}
