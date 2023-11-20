import { HTTPResponse } from "densky/http-router.ts";

export async function GET() {
  return await HTTPResponse.view("index");
}
