// Test model name resolution

const BRAND_COLORS = {
  "qwen3-max": "#8b5cf6",
  "deepseek-chat-v3.1": "#4d6bfe",
  "claude-sonnet-4-5": "#ff6b35",
  "grok-4": "#000000",
  "gemini-2-5-pro": "#4285f4",
  "gpt-5": "#10a37f",
};

const MODEL_ALIASES = {
  qwen: "qwen3-max",
  deepseek: "deepseek-chat-v3.1",
  "claude sonnet": "claude-sonnet-4-5",
  "claude-sonnet": "claude-sonnet-4-5",
  grok: "grok-4",
  grok4: "grok-4",
  gemini: "gemini-2-5-pro",
  gpt5: "gpt-5",
};

const METAS = {
  "gpt-5": {
    id: "gpt-5",
    name: "GPT‑5",
    color: BRAND_COLORS["gpt-5"],
  },
  "claude-sonnet-4-5": {
    id: "claude-sonnet-4-5",
    name: "Claude Sonnet 4.5",
    color: BRAND_COLORS["claude-sonnet-4-5"],
  },
  "deepseek-chat-v3.1": {
    id: "deepseek-chat-v3.1",
    name: "DeepSeek v3.1",
    color: BRAND_COLORS["deepseek-chat-v3.1"],
  },
  "gemini-2-5-pro": {
    id: "gemini-2-5-pro",
    name: "Gemini 2.5 Pro",
    color: BRAND_COLORS["gemini-2-5-pro"],
  },
  "grok-4": {
    id: "grok-4",
    name: "Grok 4",
    color: BRAND_COLORS["grok-4"],
  },
  "qwen3-max": {
    id: "qwen3-max",
    name: "Qwen3 Max",
    color: BRAND_COLORS["qwen3-max"],
  },
};

function normalizeId(id) {
  return id
    .toLowerCase()
    .trim()
    .replace(/[\s._]+/g, "-")
    .replace(/-+/g, "-");
}

function resolveCanonicalId(id) {
  if (!id) return undefined;
  const raw = id;
  const lower = id.toLowerCase();
  const norm = normalizeId(id);

  // 1) exact canonical
  if (METAS[raw]) return raw;
  if (METAS[norm]) return norm;

  // 2) alias table
  if (MODEL_ALIASES[lower]) return MODEL_ALIASES[lower];
  if (MODEL_ALIASES[norm]) return MODEL_ALIASES[norm];

  // 3) common punctuation variants
  const dotToDash = lower.replace(/[._]+/g, "-");
  if (MODEL_ALIASES[dotToDash]) return MODEL_ALIASES[dotToDash];

  // 4) heuristic contains matching
  if (lower.includes("gemini")) return "gemini-2-5-pro";
  if (lower.includes("grok")) return "grok-4";
  if (lower.includes("deepseek")) return "deepseek-chat-v3.1";
  if (/(claude).*?(sonnet)/.test(lower) || lower.includes("sonnet"))
    return "claude-sonnet-4-5";
  if (lower.includes("qwen")) return "qwen3-max";
  if (/gpt[- ]?5|gpt5/.test(lower)) return "gpt-5";

  return undefined;
}

function getModelMeta(id) {
  if (METAS[id]) return METAS[id];
  const canon = resolveCanonicalId(id);
  if (canon && METAS[canon]) {
    const base = METAS[canon];
    return { ...base, id };
  }
  return { id, name: id, color: "#a1a1aa" };
}

function getModelName(id) {
  return getModelMeta(id).name;
}

// Test with IDs from backend data
const testIds = [
  "deepseek-chat-v3.1",
  "claude-sonnet-4-5",
  "gemini-2.5-pro",  // Note: with dot
  "grok-4",
  "qwen3-max",
  "gpt-5",
];

console.log("\nTesting model name resolution:");
console.log("================================");
testIds.forEach(id => {
  const name = getModelName(id);
  const canon = resolveCanonicalId(id);
  console.log(`${id.padEnd(25)} -> ${name.padEnd(20)} (canon: ${canon})`);
});
