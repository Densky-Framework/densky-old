import { Promisable } from "densky/cloud.ts";
import type { HTTPRequest } from "./request.ts";

export enum StatusCode {
  CONTINUE = 100,
  SWITCHING = 101,
  PROCESSING = 102,

  OK = 200,
  CREATED = 201,
  ACCEPTED = 202,
  NON_AUTHORITATIVE = 203,
  NO_CONTENT = 204,
  RESET_CONTENT = 205,
  PARTIAL_CONTENT = 206,
  MULTI_STATUS = 207,
  ALREADY = 208,
  IM_USED = 226,

  MULTIPLE_CHOICES = 300,
  MOVED = 301,
  FOUND = 302,
  SEE_OTHER = 303,
  NOT_MODIFIED = 304,
  USE_PROXY = 305,
  TEMP_REDIRECT = 307,
  PERM_REDIRECT = 308,

  BAD_REQUEST = 400,
  UNAUTHORIZED = 401,
  PAYMENT_REQ = 402,
  FORBIDDEN = 403,
  NOT_FOUND = 404,
  NOT_METHOD = 405,
  NOT_ACCEPTABLE = 406,
  TIMEOUT = 408,
  CONFLICT = 409,
  GONE = 410,
  LARGE_PAYLOAD = 413,
  LONG_URI = 414,
  UNSUPPORTED_MEDIA = 415,
  TEAPOT = 418,
  LOCKED = 423,
  FAILED_DEPENDENCY = 424,
  UPGRADE = 426,
  TOO_MANY = 429,
  CLIENT_CLOSED = 499,

  INTERNAL_ERR = 500,
  NOT_IMPLEMENTED = 501,
  BAD_GATEWAY = 502,
  UNAVAILABLE = 503,
  GATEWAY_TIMEOUT = 504,
  INSUFFICIENT_STORAGE = 507,
}

export const statusMessages: Record<StatusCode, string> = {
  100: "Continue",
  101: "Switching Protocol",
  102: "Processing",

  200: "Ok",
  201: "Created",
  202: "Accepted",
  203: "Non-authoritative Information",
  204: "No Content",
  205: "Reset Content",
  206: "Partial Content",
  207: "Multi Status",
  208: "Already Reported",
  226: "IM Used",

  300: "Multiple Choices",
  301: "Moved Permanently",
  302: "Found",
  303: "See Other",
  304: "Not Modified",
  305: "Use Proxy",
  307: "Temporaly Redirect",
  308: "Permanently Redirect",

  400: "Bad Request",
  401: "Unauthorized",
  402: "Payment Required",
  403: "Forbidden",
  404: "Not Found",
  405: "Method Not Allowed",
  406: "Not Acceptable",
  408: "Request Timeout",
  409: "Conflict",
  410: "Gone",
  413: "Payload Too Large",
  414: "Request-URI Too Long",
  415: "Unsopported Media Type",
  418: "I'm Teapot",
  423: "Locked",
  424: "Failed Dependency",
  426: "Upgrade Required",
  429: "Too Many Requests",
  499: "Client Closed Request",

  500: "Internal Server Error",
  501: "Not Implemented",
  502: "Bad Gateway",
  503: "Service Unavailable",
  504: "Gateway Timeout",
  507: "Insufficient Storage",
};

export enum HTTPMethod {
  GET = "GET",
  POST = "POST",
  DELETE = "DELETE",
  PATCH = "PATH",
  ANY = "ANY",
}

export type HTTPMethodStr =
  | "GET"
  | "POST"
  | "DELETE"
  | "PATH"
  | "OPTIONS"
  | "ANY";

export type Entry = (req: HTTPRequest) => Promisable<Response | undefined>;
export interface EntryController {
  default?: Entry;
  GET?: Entry;
  POST?: Entry;
  DELETE?: Entry;
  PATCH?: Entry;
}
