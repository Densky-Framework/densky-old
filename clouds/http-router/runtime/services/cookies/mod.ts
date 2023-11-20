import {
  CookieOptions,
  getDecodedCookies,
  logger,
  setEncodedCookie,
} from "./signer.ts";

export class Cookies {
  raw: Record<string, string> | null = null;

  constructor(readonly reqHeaders: Headers, readonly resHeaders: Headers) {}

  async parse() {
    this.raw = await getDecodedCookies(this.reqHeaders);
  }

  getRaw(key: string): string | null {
    if (!this.raw) {
      throw new Error(
        "Trying to get cookie before parse it. First execute `.parse()`",
      );
    }

    return this.raw[key] ?? null;
  }

  get<T = string>(key: string): T | null {
    const raw = this.getRaw(key);
    if (!raw) return null;

    const type = raw.slice(0, 2);
    const data = raw.slice(2);

    switch (type) {
      case "j:": {
        if (data === "null") return null;

        try {
          return JSON.parse(data) as T;
        } catch (e) {
          logger.warn("Cookies.get error parsing:", e);
          return null;
        }
      }

      case "t:":
        return data as unknown as T;

      case "n:": {
        const parsed = parseFloat(data);
        if (Number.isNaN(parsed)) return null;
        return parsed as unknown as T;
      }

      case "b:":
        return (data === "1") as unknown as T;

      default:
        return raw as unknown as T;
    }
  }

  async setRaw(key: string, value: string, options: CookieOptions = {}) {
    await setEncodedCookie(this.resHeaders, key, value, options);
  }

  async set(key: string, value: unknown, options: CookieOptions = {}) {
    let cookieValue: string;

    switch (typeof value) {
      case "number":
        cookieValue = "n:" + value.toString();
        break;

      case "object": {
        if (value === null) {
          cookieValue = "j:null";
          break;
        }

        cookieValue = "j:" + JSON.stringify(value);
        break;
      }

      case "boolean":
        cookieValue = "b:" + (value ? "1" : "0");
        break;

      default:
        cookieValue = `t:${value}`;
        break;
    }

    await this.setRaw(key, cookieValue, options);
  }
}
