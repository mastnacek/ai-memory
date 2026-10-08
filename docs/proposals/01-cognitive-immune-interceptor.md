# Proposal 01: Kognitivní imunitní systém (Active Tool-Call Interceptor via Jev)

**Status:** Draft / Architectural Proposal  
**Cílová vrstva:** `uma-pi-extension/src/hooks/`, `uma-core/src/interceptor/`, TypeSafe Jev API  
**Datum:** 2026-10-08  

---

## 1. Abstrakt a fundamentální problém

Všechny existující paměťové systémy pro AI agenty (včetně Mem0, Letta, Zep i dosavadního návrhu UMA) trpí stejnou systémovou vadou: **fungují jako pasivní knihovna**. Předpokládají, že agent si má jít do paměti *vyhledat* instrukci, přečíst si ji a dobrovolně se jí řídit.

### Proč pasivní vyhledávání selhává v praxi:
1. **Nevědomost o neznalosti (Unknown Unknowns):** Agent v novém kontextu netuší, že o dané věci existuje dřívější rozhodnutí, a proto nezavolá `uma_search`.
2. **Kognitivní přetížení a tokenová eroze:** I když agent fakt najde, během dlouhého generování kódu instrukci zapomene nebo ji přebije silnější váha z pretrénovaných dat modelu.
3. **Pozdní detekce chyb:** Na porušení pravidla přijde až lidský operátor při code review, což vede k frustraci.

### Řešení: Paměť jako podvědomý reflex
Místo pasivního RAGu zavádíme **Kognitivní imunitní systém**. UMA se stává aktivním interceptorem na úrovni nástrojů (`tool_call`). Dříve než se jakákoliv změna kódu zapíše na disk, bleskový System 1 model (Jev od TypeSafe AI) vyhodnotí diff proti platným pravidlům v paměti. Pokud detekuje porušení, nástroj se zablokuje a agent dostane okamžitou syntetickou zpětnou vazbu k samoopravě.

---

## 2. Architektura toku událostí (Sequence & Latency Budget)

Celá operace musí proběhnout v přísném časovém limitu **pod 150 ms**, aby nezpomalila práci agenta.

```text
┌────────────────────────┐
│  Kódující Agent        │
│  (Claude 3.7 / Sonnet) │
└───────────┬────────────┘
            │ 1. Volání nástroje: write_to_file / replace_file_content
            ▼
┌────────────────────────────────────────────────────────────────────────┐
│  Pi Hook: tool_call (UMA Immune Interceptor)                           │
│  - Zadrží spuštění nástroje (soubor na disku se ještě nezmění)          │
│  - Extrahuje navrhovaný unified diff                                   │
│  - Načte kompaktní L1 index aktivních pravidel (názvy + id + tags)      │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ 2. Asynchronní POST (OpenRouter / Jev)
                                    │    Latency budget: 70–120 ms
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  JEV (TypeSafe AI - System 1 Decision Engine)                          │
│                                                                        │
│  State: [Kódový diff + Zasažená cesta + L1 Index pravidel]             │
│  Q1 (Noul):   "Porušuje tento diff aktivní architektonická pravidla?"   │
│  Q2 (Choice): "Které pravidlo bylo porušeno?" -> [rule_id | none]      │
│  Q3 (Score):  "Závažnost porušení" -> [1: drobná .. 5: fatální]        │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ 3. Typovaná odpověď
                                    ▼
                    ┌───────────────┴───────────────┐
            Noul < 0.35                      Noul >= 0.75 & Severity >= 3
                    │                               │
                    ▼                               ▼
       ┌────────────────────────┐      ┌────────────────────────────────────────┐
       │ AKCE POVOLENA          │      │ REFLEXNÍ BLOKACE (Kognitivní disonance)│
       │ Nástroj zapíše soubor  │      │ - Nástroj selže dřív než sáhne na disk │
       │ Práce pokračuje        │      │ - Vrátí chybovou hlášku agentovi:      │
       └────────────────────────┘      │   "[UMA Immune Block]: Diff porušuje   │
                                       │    pravidlo <ID> (<Title>).            │
                                       │    Důvod: <Detail>. Oprav kód!"        │
                                       └───────────────────┬────────────────────┘
                                                           │
                                                           ▼
                                       ┌────────────────────────────────────────┐
                                       │ Agent provede vnitřní sebereflexi      │
                                       │ a vygeneruje čistý opravený kód        │
                                       │ (Operátor chybu ani neviděl!)          │
                                       └────────────────────────────────────────┘
```

---

## 3. Technická specifikace a implementace v UMA

### A. Záchytný bod v Pi Extension (`approval_gate.ts` rozšíření)
V TypeScriptovém rozšíření Pi registrujeme interceptor na událost `tool_call`:

```typescript
// uma-pi-extension/src/hooks/immune_interceptor.ts
import type { ToolCallEvent, ExtensionContext } from "@earendil-works/pi-coding-agent";
import { evaluateWithJev } from "../shared/jev_client.js";
import { getL1ActiveRulesIndex } from "../shared/cache.js";

const MUTATING_CODE_TOOLS = new Set([
  "write_to_file",
  "replace_file_content",
  "run_command"
]);

export async function evaluateImmuneInterceptor(
  event: ToolCallEvent,
  ctx: ExtensionContext
): Promise<{ allow: boolean; rejectionReason?: string }> {
  if (!MUTATING_CODE_TOOLS.has(event.toolName)) {
    return { allow: true };
  }

  // 1. Získání navrhované změny
  const payload = extractPayload(event.toolName, event.input);
  if (!payload) return { allow: true };

  // 2. Načtení kompaktního L1 indexu pravidel (drženého v paměti RAM)
  const activeRules = await getL1ActiveRulesIndex(ctx.cwd);
  if (activeRules.length === 0) return { allow: true };

  // 3. Dotaz na Jev
  try {
    const assessment = await evaluateWithJev({
      state: {
        tool: event.toolName,
        targetPath: payload.path,
        diff: payload.diff,
        rules: activeRules
      },
      questions: [
        { id: "violates", type: "noul", prompt: "Does this code change violate any listed active rule?" },
        { id: "rule_id", type: "choice", options: activeRules.map(r => r.id), prompt: "Which rule is violated?" },
        { id: "severity", type: "score", levels: ["trivial", "warning", "critical"], prompt: "Violation severity" }
      ]
    });

    // 4. Vyhodnocení prahů
    if (assessment.violates.probability > 0.75 && assessment.severity.value !== "trivial") {
      const violatedRule = activeRules.find(r => r.id === assessment.rule_id.value);
      return {
        allow: false,
        rejectionReason: `[UMA Immune System Block]: Your proposed change in '${payload.path}' violates active memory rule [${violatedRule?.id}] "${violatedRule?.title}". Context: ${violatedRule?.ruleText}. You must revise your implementation to comply with this constraint.`
      };
    }
  } catch (error) {
    // Fail-open pojistka pro latenci/offline režim: chybu zalogujeme, ale práci neblokujeme
    console.warn("UMA Interceptor fallback: JEV unavailable, proceeding.", error);
  }

  return { allow: true };
}
```

### B. Optimalizace Jev payloadu (Zero-Token Bloat)
Aby se do Jeva neposílaly tisíce tokenů:
* Neposílá se celé tělo každého faktu.
* Posílá se pouze **L1 Constraint Vector**:
  ```json
  [
    { "id": "01M4D7S5", "title": "Strict Vertical Slice Architecture", "rule": "Feature slices never import each other directly; use shared kernel" },
    { "id": "01M4D9S7", "title": "No raw unwraps", "rule": "Never call .unwrap() in uma-core or uma-cli; use anyhow or thiserror" }
  ]
  ```
* Diff se omezuje na hlavičku souboru a přidané řádky (`+`).

---

## 4. Reálný scénář z praxe (Příklad fungování)

### Situace:
Agent dostane za úkol přidat novou funkcionalitu do `uma-cli/src/slices/search/`.
Během psaní kódu ho napadne zavolat pomocnou funkci z jiného řezu:
```rust
// V slices/search/outcome.rs:
use crate::slices::doctor::checks::inspect_index; // <-- ZÁKAZ!
```

### Co se stane:
1. Agent odešle `replace_file_content`.
2. **Normální systém:** Soubor se přepíše, kód se zkompiluje, pravidlo VSA je tiše porušeno.
3. **UMA Kognitivní imunitní systém:**
   * Pi hook zachytí diff: `+use crate::slices::doctor...`.
   * Jev (za 90 ms) odpoví: `violates: 0.98`, `rule_id: 01M4D7S5`, `severity: critical`.
   * Nástroj vrátí agentovi:
     ```text
     Error: [UMA Immune System Block]: Your proposed change in 'src/slices/search/outcome.rs' 
     violates active memory rule [01M4D7S5] "Strict Vertical Slice Architecture". 
     Rule: "Feature slices never import each other directly; use shared kernel".
     ```
   * Agent v dalším myšlenkovém kroku napíše:
     *„Omlouvám se, porušil jsem pravidlo VSA o izolaci řezů. Přesouvám společnou funkci do `src/shared/` a importuji ji odtud.“*
   * Soubor se zapíše správně.

---

## 5. Přínos a revoluční hodnota
* **Stoprocentní dodržování architektonických rozhodnutí:** Rozhodnutí v `.uma/` přestávají být pasivní dokumentací a stávají se živým exekučním mantinelem.
* **Žádná kognitivní zátěž pro člověka:** Operátor nemusí v code review neustále hlídat tytéž triviální chyby.
* **Okamžitá samooprava:** Agent se opraví v rámci jednoho promptovacího cyklu ještě před dokončením své odpovědi.
