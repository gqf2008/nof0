// The Next.js proxy implementation has been replaced by a dedicated Rust
// backend. This stub exists only so historical imports continue to resolve.
export const runtime = "edge";

export async function GET() {
  throw new Error("Legacy Next.js API route is disabled. Use the Rust backend.");
}

export async function OPTIONS() {
  throw new Error("Legacy Next.js API route is disabled. Use the Rust backend.");
}
