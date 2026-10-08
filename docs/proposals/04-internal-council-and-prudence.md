# Proposal 04: Vnitřní sněm a epistemická opatrnost (Internal Council, Adversarial Skepticism & Risk Budgeting)

**Status:** Draft / Architectural Proposal  
**Cílová vrstva:** `uma-pi-extension/src/council/`, `uma-core/src/risk/`, TypeSafe Jev API  
**Vazby na předchozí návrhy:**  
- Navazuje na [Proposal 01 (Kognitivní imunitní systém)](file:///D:/01_programovani/ai-memory/docs/proposals/01-cognitive-immune-interceptor.md) – rozšiřuje reaktivní reflex o hloubkovou preventivní skepsi.  
- Využívá telemetrii z [Proposal 02 (Dvourychlostní stínový mozek)](file:///D:/01_programovani/ai-memory/docs/proposals/02-shadow-brain-zero-latency-distillation.md) k výpočtu historického rizika souborů.  
- Čerpá z vah a kontraktů v [Proposal 03 (Synaptická plasticita a spustitelné kontrakty)](file:///D:/01_programovani/ai-memory/docs/proposals/03-synaptic-plasticity-executable-contracts.md) k vyhodnocení integrity změn.  
**Datum:** 2026-10-08  

---

## 1. Abstrakt a fundamentální problém

I když má AI agent dokonalé úložiště paměti (UMA Core), reaktivní imunitní reflex ([Proposal 01](file:///D:/01_programovani/ai-memory/docs/proposals/01-cognitive-immune-interceptor.md)), asynchronní těžbu ([Proposal 02](file:///D:/01_programovani/ai-memory/docs/proposals/02-shadow-brain-zero-latency-distillation.md)) a synaptické posilování ([Proposal 03](file:///D:/01_programovani/ai-memory/docs/proposals/03-synaptic-plasticity-executable-contracts.md)), stále se nechová jako skutečný seniorní inženýr či myslitel.

### Proč agent stále selhává:
1. **Syndrom nekritické ochoty (Eager Pleaser):** LLM modely jsou trénovány tak, aby okamžitě vygenerovaly odpověď na jakékoliv zadání. Neumí se zastavit, položit ruku na bradu a říct si: *„To je sice hezký nápad, ale je to pitomost, protože...“*
2. **Absence kognitivní bolesti (Somatic Markers):** Člověk cítí při pohledu na křehký kód v produkci *fyzickou nevolnost a strach* na základě minulých průšvihů. Agent nemá strach ani pud sebezáchovy – s klidem přemaže databázový ovladač v pátek v 17:00, protože o to uživatel požádal.
3. **Chybějící prospektivní paměť:** Agent myslí v jednorozměrném toku tokenů. Nedokáže si v hlavě vytvořit mentální závazek: *„Změním funkci A, ale to znamená, že zítra nebo za 10 minut musím aktualizovat i konfiguraci B a migrační skript C.“*

### Řešení: Vnitřní sněm a rozpočet rizika
Navrhujeme zavést do UMA **Vnitřní sněm (The Internal Council)** – kognitivní dialektiku, která simuluje vnitřní dialog člověka (*Anděl vs. Ďábel*), dynamicky počítá rozpočet rizika (*Risk Budget & Blast Radius*) a spravuje knihu budoucích závazků (*Prospective Debt Ledger*).

---

## 2. Čtyři pilíře expertní kognice

```text
┌────────────────────────────────────────────────────────────────────────┐
│  UŽIVATELSKÉ ZADÁNÍ                                                    │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  PILÍŘ I: VNITŘNÍ SNĚM (Anděl vs. Ďábel / Adversarial Debate)          │
│  - Anděl (Tvůrce / Claude): Navrhne přímočaré řešení a postup          │
│  - Ďábel (Skeptik / Jev):    Aktivně hledá skryté chyby a scénáře pádů │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  PILÍŘ II: METRIKA BOLESTI A BLAST RADIUS (Risk Budgeting)             │
│  - Výpočet "Pain Score" zasažených souborů na základě historie chyb    │
│  - Stanovení stupně opatrnosti: Zelená (Prototyp) -> Rudá (Jádro)      │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  PILÍŘ III: PROSPEKTIVNÍ ZÁVAZKY (Prospective Debt Ledger)             │
│  - Zápis vedlejších povinností vyvolaných krokem                       │
│  - "Pokud sáhneš na API, vzniká dluh: aktualizovat mock v testu X"     │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  PILÍŘ IV: EPISTEMICKÁ POKORA (Confidence Calibration)                 │
│  - Posouzení: Máme v .uma/ dostatek kontextu pro bezpečný zásah?      │
│  - Pokud NE -> Stop, zákaz mutace, vynucení vyšetřovacího módu         │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Technická realizace v UMA

### Pilíř I: Dialektika „Anděl vs. Ďábel“ (Adversarial Critic)
Předtím, než agent zahájí provádění plánu, vytvoří krátký syntetický záměr (*Intent Statement*). Tento záměr je předložen **Skeptikovi** (model Jev z TypeSafe AI), jehož jedinou rolí je **najít nejhorší možný scénář selhání**:

```typescript
// uma-pi-extension/src/council/skeptic.ts
export async function consultSkeptic(intent: string, files: string[]): Promise<SkepticCritique> {
  return await evaluateWithJev({
    state: {
      proposedIntent: intent,
      targetFiles: files,
      knownPastIncidents: await loadPastIncidentsForFiles(files)
    },
    questions: [
      {
        id: "failure_mode",
        type: "choice",
        options: [
          "concurrency_deadlock",
          "unhandled_null_edge_case",
          "silent_data_corruption",
          "breaking_public_contract",
          "none_acceptable_risk"
        ],
        prompt: "Identify the primary architectural failure mode of this intent."
      },
      {
        id: "devil_objection",
        type: "noul",
        prompt: "Should an expert senior architect reject this change without preliminary tests?"
      },
      {
        id: "challenge_question",
        type: "score",
        levels: ["benign", "questionable", "dangerous"],
        prompt: "Risk level of proposed architectural action"
      }
    ]
  });
}
```

Pokud Skeptik označí záměr jako `dangerous` nebo `devil_objection > 0.70`, **agent nedostane povolení začít psát kód**. Místo toho je do jeho myšlenkového proudu injektována skepse:
> *„[Vnitřní kritik / Ďábel varuje]: Navrhuješ změnit sdílenou SQLite instanci v `indexer/mod.rs`. Co se stane, když MCP server obdrží dotaz během zápisu? Způsobíš `database is locked`. Nejprve navrhni ošetření zámků nebo napiš stress-test.“*

---

### Pilíř II: Somatické markery a „Metrika bolesti souborů“ (Pain Score)
Člověk cítí k některým částem kódu instinktivní odpor, protože si pamatuje, kolik nocí nad nimi strávil při ladění. UMA zavádí **Pain Score $\in \langle 0, 100 \rangle$** pro každý soubor v repozitáři.

#### Výpočet Pain Score v `uma-core/src/risk/pain.rs`:
1. **Frekvence oprav (`type: correction`):** Kolikrát byl tento soubor zmíněn v opravách chyb v `.uma/correction/`? ($+15$ bodů za každý záznam).
2. **Git churn po incidentech:** Kolikrát byl soubor revertován v git historii? ($+20$ bodů za revert).
3. **Kauzální propojení:** Kolik ostatních souborů na něm závisí? (Fan-in metrika).

```text
Pain Score = 0–20 (Nízké riziko):  Běžný režim. Rychlá editace povolena.
Pain Score = 21–60 (Střední riziko): Vyžaduje spuštění lokálních testů po editaci.
Pain Score = 61–100 (Kritická zóna): ZÁKAZ přímé editace. Povolen pouze TDD režim
                                     (nejprve selhávající test, pak teprve oprava).
```

Když se agent pokusí editovat soubor s Pain Score $> 60$ (např. [`uma-core/src/indexer/ops.rs`](file:///D:/01_programovani/ai-memory/uma/uma-core/src/indexer/ops.rs)), UMA Interceptor ([Proposal 01](file:///D:/01_programovani/ai-memory/docs/proposals/01-cognitive-immune-interceptor.md)) mu oznámí:
> *„[UMA Risk Warning]: Soubor `indexer/ops.rs` má Pain Score 85 (historie zamykacích chyb a pádů reindexace). Nesmíš do něj zasáhnout bez předchozího ověření v integračním testu.“*

---

### Pilíř III: Prospektivní paměť závazků (Prospective Debt Ledger)
Člověk při práci průběžně generuje mentální „TODO“ položky, které ho chrání před nedodělky.

UMA vytváří během session v paměti RAM dynamický seznam **Kognitivních dluhů (Debt Ledger)**:

```typescript
interface ProspectiveDebt {
  id: string;
  sourceAction: string;     // Např. "Změna signatury Store::search_all"
  requiredAction: string;   // "Aktualizovat volání v slices/search/mod.rs a slices/mcp/tools.rs"
  blocking: boolean;        // Blokuje dokončení session?
}
```

* Když agent změní veřejný kontrakt nebo trait, UMA mu do ledgeru automaticky zapíše dluh.
* **Konec tahů bez nedodělků:** Agent nemůže prohlásit úkol za hotový a ukončit práci, dokud ledger obsahuje otevřené blokující závazky.

---

### Pilíř IV: Epistemická pokora (Kalibrace jistoty)
Znakem skutečného mistra je vědět, kdy nevím. Pokud agent dostane úkol zasahující do subsystému, o kterém v `.uma/` neexistuje žádný záznam a v němž kód obsahuje složité makra či externí FFI:

* Jev vyhodnotí **Index obeznámenosti (Familiarity Index)**.
* Pokud je znalost nízká, agent je donucen přejít do **Read-Only Explorative Mode**:
  1. Musí nejprve přečíst minimálně 3 související soubory.
  2. Musí zformulovat své pochopení formou hypotézy.
  3. Teprve po potvrzení smí provést první mutaci kódu.

---

## 4. Reálný scénář z praxe (Jak to vypadá v akci)

### Kontext:
Uživatel zadá zdánlivě jednoduchý úkol:  
*„Zrychli vyhledávání v UMA tím, že indexer nebude otevírat databázi pokaždé znovu, ale udržíme statické globální spojení přes `lazy_static` nebo `Mutex`.“*

### Běžný agent:
Nafasuje úkol, napíše `static ref CONN: Mutex<Connection>`, zkompiluje to, zavře issue a při prvním paralelním testu celá aplikace spadne na deadlocku vláken.

### Agent s Vnitřním sněmem (Proposal 04):
1. **Anděl navrhne:** *„Přidáme globální `Mutex<Connection>` do `Store::central_indexer`.“*
2. **Ďábel (Jev) okamžitě zaútočí:**  
   *`failure_mode: concurrency_deadlock`*, *`devil_objection: 0.94`*.  
   *„Pozor: SQLite spojení držené v statickém mutexu v multithreadovém testovacím runneru Rustu způsobí zablokování threadpoolu a poruší isolační kontrakt z Proposal 01.“*
3. **Pain Score kalkulátor:** Soubor `store/mod.rs` má skóre 72 $\rightarrow$ vyžadován TDD přístup.
4. **Výsledek vnitřního dialogu:**  
   Agent se zastaví a napíše uživateli:  
   *„Analyzoval jsem návrh na statické spojení. Vnitřní kontrola rizik ukázala vysoké nebezpečí deadlocku v testech. Místo statického mutexu navrhuji zavést `r2d2` connection pool nebo zapnout WAL mód a ponechat krátkodobá scoped spojení s `busy_timeout`. Zde je srovnání rizik...“*

**Výsledek:** Uživatel dostane reakci zralého softwarového architekta, nikoliv slepého generátoru kódu.

---

## 5. Kompletní kognitivní pyramida UMA (Architektura 4.0)

Čtyři návrhy dohromady tvoří ucelený kognitivní systém, který odpovídá struktuře lidského myšlení:

| Úroveň | Vrstva mysli | Návrh v UMA | Co zajišťuje v praxi |
| :--- | :--- | :--- | :--- |
| **L4** | **Soudnost & Sebekritika** | **Proposal 04: Vnitřní sněm** | Anděl vs. Ďábel, kalkulace rizika (Blast Radius), epistemická pokora a prospektivní závazky. |
| **L3** | **Zkušenost & Adaptace** | **[Proposal 03: Plasticita & Kontrakty](file:///D:/01_programovani/ai-memory/docs/proposals/03-synaptic-plasticity-executable-contracts.md)** | Hebbovské učení, dynamická váha pravidel, poločas rozpadu a strojově vynucované AST invarianty. |
| **L2** | **Podvědomí & Spánek** | **[Proposal 02: Stínový mozek](file:///D:/01_programovani/ai-memory/docs/proposals/02-shadow-brain-zero-latency-distillation.md)** | Zero-latency těžba znalostí na pozadí během generování kódu, bez zdržování člověka. |
| **L1** | **Reflex & Pud sebezáchovy**| **[Proposal 01: Imunitní interceptor](file:///D:/01_programovani/ai-memory/docs/proposals/01-cognitive-immune-interceptor.md)** | Blesková blokace zakázaných akcí v hooku `tool_call` (Jev 80 ms) dřív, než kód dopadne na disk. |
| **L0** | **Paměťová schránka** | **UMA Core (OKF v0.2 + Rust)** | Deterministické Markdowny v `.uma/`, ULID, Git provenience, FTS5 + vektorová cache. |

Tento čtyřvrstvý model překračuje hranice pouhé „paměti pro AI“ a stává se **plnohodnotným kognitivním rámcem pro autonomní inženýrství**.
