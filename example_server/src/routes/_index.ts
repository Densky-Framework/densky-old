import { HTTPResponse } from "densky";

export async function GET() {
  return await HTTPResponse.view("index.html");
}
