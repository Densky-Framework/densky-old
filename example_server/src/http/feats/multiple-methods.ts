export function GET() {
  return new Response("You are getting data");
}

export function POST() {
  return new Response("You are posting data");
}

export default () => {
  return new Response("Catched by any fallback");
};
