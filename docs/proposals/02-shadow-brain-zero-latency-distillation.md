# Proposal 02: Dvourychlostní stínový mozek (Shadow Worker & Zero-Latency Memory Mining)

**Status:** Draft / Architectural Proposal  
**Cílová vrstva:** `uma-pi-extension/src/workers/`, `uma-core/src/sources/`, TypeSafe Jev API  
**Datum:** 2026-10-08  

---

## 1. Abstrakt a fundamentální problém

Současné pokusy o automatickou extrakci paměti (např. v Mem0, MemGPT/Letta, Zep nebo v dosavadním draftu UMA S9 Import) narážejí na **dvě protichůdné bariéry**:

1. **Memory Tax (Latence a zdržování):** Pokud má agent na konci každého úkolu provést sebereflexi a vytěžit paměť, musí se zastavit konverzace, spustit těžký model (Claude Sonnet / GPT-4o), vygenerovat shrnutí a uživatel 15–30 sekund čeká na dokončení. Uživatelé takové pluginy brzy vypínají.
2. **Amnézie z přetížení (Agent Bias):** Pokud necháme rozhodnutí „kdy uložit paměť“ na primárním agentovi, agent to v 90 % případů neudělá. Když programuje složitý algoritmus, celá jeho kapacita kontextu a pozornosti směřuje do kódu, nikoliv do správy databáze.

### Řešení: Dvourychlostní symbióza (Shadow Worker)
Využijeme dobu, kdy hlavní kognitivní model (System 2, např. Claude 3.7) přemýšlí nebo streamuje odpověď (což trvá 5–25 sekund). Během této doby běží na pozadí **asynchronní Stínový pracovník (Shadow Worker)** poháněný bleskovým modelem **Jev (System 1)**. 

Stínový pracovník nepozorovaně analyzuje telemetrii, kompilátor a tón uživatele a připravuje strukturované návrhy paměti do mezipaměti (`staging area`) v reálném čase s **nulovým zpožděním pro člověka i agenta**.

---

## 2. Architektura dvourychlostního zpracování

```text
ČASOVÁ OSA TAHU (TURN LIFECYCLE)

0s          1s                                    10s                     15s
├───────────┼──────────────────────────────────────┼───────────────────────┤
Uživatel    Claude 3.7 začíná generovat            Claude dokončuje        Turn končí
odešle      kód a vysvětlení...                    odpověď...              
prompt                                                                     
 │                                                                          │
 ├─────────────────────────────────────────────────┐                        │
 │ PARALELNÍ BĚH STÍNOVÉHO PRACOVNÍKA              │                        │
 ▼                                                 ▼                        ▼
JEV (System 1):                               JEV (Draft):              TUI Status:
- Triage promptu & exit kódů                  - Vygeneruje návrh        [UMA: 1 draft]
- Detekce: "Uživatel opravil agenta"            faktu do .staging/      
- Určení typu: correction                       (Nulové zdržení!)       (Člověk schválí
                                                                        až má čas)
```

### Schéma komponent:

```text
┌────────────────────────────────────────────────────────────────────────┐
│  PI RUNTIME & TERMINAL TELEMETRY                                       │
│  - Uživatelský prompt                                                 │
│  - Výstup nástroje: run_command (exit code, compiler error, test fail)│
│  - Git diff posledního tahu                                            │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Stream událostí na pozadí (non-blocking)
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  UMA SHADOW WORKER (Node.js Worker Thread / Rust Task)                 │
│                                                                        │
│  Krok 1: Signal Triage via JEV (70–120 ms)                             │
│  State: [Poslední chybová hláška + Výtka uživatele + Cesta k souboru] │
│  Q1 (Choice): "Event classification" ->                                │
│               [routine_work, correction, architecture_decision,        │
│                preference_expressed, transient_chatter]                │
│  Q2 (Noul):   "Is this durable knowledge worth preserving?"           │
│  Q3 (Score):  "Confidence score (1 to 5)"                              │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Detekována trvanlivá znalost (Confidence >= 4)
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  STAGING AREA (.uma/.staging/<ULID>.json)                              │
│  - Zformulovaný atomický návrh (title, tags, type, context, rule)      │
│  - Provázáno se session ID a commitem                                  │
│  - Nezasahuje do aktivního indexu .uma/ (žádné znečištění!)            │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  SEAMLESS HUMAN CONSENT (Nerušené schválení)                           │
│  - Žádný agresivní vyskakovací modal uprostřed kódu                    │
│  - Diskrétní indikátor v Pi status baru: "✦ 1 pending memory draft"   │
│  - Klávesová zkratka (např. Alt+M nebo slash command `/uma review`)    │
│  - Hromadné schválení při commitu nebo na konci session                │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Telemetrie a vzorce zachycení (Passive Sensing Triggers)

Stínový pracovník nepotřebuje číst tisíce řádků chatu. Zaměřuje se výhradně na **zlomy v chování (Behavioral Inflection Points)**:

| Signální zlom | Co se stalo v běhu | Co Jev vyhodnotí | Výsledný typ faktu |
| :--- | :--- | :--- | :--- |
| **Compiler / Test Recovery** | `cargo test` selhal $\rightarrow$ uživatel dal instrukci $\rightarrow$ test prošel. | Klíčový trik nebo oprava předpokladu. | `type: correction` |
| **Explicit Rejection** | Uživatel revertnul git diff nebo napsal *„takhle ne, radši X“*. | Negativní preference / zakázaný vzor. | `type: preference` |
| **New Dependency Added** | `Cargo.toml` / `package.json` změněn o novou knihovnu. | Architektonická volba frameworku/knihovny. | `type: decision` |
| **Procedural Sequence** | 3 příkazy v shellu za sebou bez chyby ukončené commitem. | Znovupoužitelný postup. | `type: skill` (s `template`) |

---

## 4. Technická specifikace Staging Area

Aby automaticky těžená paměť neotrávila produkční repozitář, zavádíme mezistupeň: **Staging Store**.

### Adresářová struktura:
```text
.uma/
├── decision/         <-- Schválená produkční paměť
├── preference/       <-- Schválená produkční paměť
└── .staging/         <-- Skrytá pracovní plocha stínového pracovníka
    ├── 01JNB8Q1...json
    └── 01JNB8R5...json
```

### Formát staged záznamu (`.uma/.staging/<ULID>.json`):
```json
{
  "id": "01JNB8Q1X4KZ9...",
  "status": "draft",
  "confidence": 0.92,
  "detected_type": "correction",
  "suggested_title": "Direct Store testing instead of mocks",
  "provenance": {
    "session_id": "pi-sess-8821",
    "trigger_command": "cargo test --test roundtrip",
    "exit_code": 101,
    "user_instruction": "V testech nepouzivej mocky, testuj primo Store"
  },
  "proposed_fact": {
    "scope": "project:ai-memory",
    "type": "correction",
    "title": "Use direct Store integration tests instead of mocks",
    "tags": ["testing", "rust", "store"],
    "body": "### Context\nMocking the Store hides filesystem and SQLite locking bugs.\n\n### Rule\nAlways write integration tests against real temporary directories (`tempfile::tempdir()`). Do not mock `Store`.\n\n### Consequences\nTests accurately mirror production file I/O behavior."
  }
}
```

---

## 5. UI/UX: Nenápadné schvalování bez approval fatigue

Většina schvalovacích rozhraní selhává na tom, že uživatele vyrušuje. Pokud při každém nápadu vyskočí okno, člověk je otrávený.

### Implementace v Pi TUI:
1. **Během práce:** Ve spodní liště svítí pouze malá tečka nebo ikona:  
   `[pi:ready] [git:main] [UMA: 1 draft]`
2. **Kdy dochází k revizi:**
   * **Možnost A (On-demand):** Uživatel stiskne `Ctrl+M` nebo napíše `/uma review`.
   * **Možnost B (Natural checkpoint):** Když agent zahlásí *„Úkol je hotový, mohu commitnout?“*, Pi TUI nabídne:
     ```text
     ┌────────────────────────────────────────────────────────┐
     │ UMA Memory Staging Review (1 draft available)          │
     │                                                        │
     │ [CORRECTION] Use direct Store integration tests        │
     │ Rule: Always test against real tempdir, never mock.   │
     │                                                        │
     │ [A]pprove & Save    [E]dit Draft    [D]iscard   [S]kip │
     └────────────────────────────────────────────────────────┘
     ```
3. **Schválení jedním tlačítkem:** Uživatel stiskne `A`, staged JSON se převede na platný Markdown soubor OKF v0.2 v `.uma/correction/` a přidá se do gitu společně s commitem kódu.

---

## 6. Přínos a revoluční hodnota
* **Zero Cognitive Tax:** Člověk ani agent nemusí přemýšlet o tom, kdy a jak paměť formulovat.
* **100% záchyt důležitých rozhodnutí:** Zachytí se i věci, na které by vývojář zapomněl, protože vznikly organicky během ladění chyb.
* **Čistota produkčních dat:** Díky `.staging/` vrstvě a konečnému lidskému souhlasu se do hlavní paměti nedostane žádný halucinační šum.
