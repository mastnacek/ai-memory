# Proposal 05: Procedurální svalová paměť, kognitivní priming a emoční valence (Procedural Motor Compilation, Spreading Activation & Affective Saliency)

**Status:** Draft / Architectural Proposal  
**Cílová vrstva:** `uma-core/src/muscle/`, `uma-core/src/priming/`, `uma-core/src/domain.rs`, `uma-pi-extension/`  
**Vazby na předchozí návrhy:**  
- Využívá Stínového pracovníka z [Proposal 02](file:///D:/01_programovani/ai-memory/docs/proposals/02-shadow-brain-zero-latency-distillation.md) k automatické detekci opakujících se motorických vzorců.  
- Spolupracuje s Kognitivním imunitním systémem z [Proposal 01](file:///D:/01_programovani/ai-memory/docs/proposals/01-cognitive-immune-interceptor.md) – svalové rutiny podléhají imunitní kontrole.  
- Moduluje synaptickou váhu a poločas rozpadu z [Proposal 03](file:///D:/01_programovani/ai-memory/docs/proposals/03-synaptic-plasticity-executable-contracts.md) pomocí emoční valence (*Saliency Multiplier*).  
- Poskytuje rychlou alternativu k Vnitřnímu sněmu z [Proposal 04](file:///D:/01_programovani/ai-memory/docs/proposals/04-internal-council-and-prudence.md) pro rutiny s nulovým rizikem (*Low Pain Score*).  
**Datum:** 2026-10-08  

---

## 1. Abstrakt a neurobiologické východisko

Všechny existující paměti pro AI agenty redukují lidskou kognici na **deklarativní textovou paměť** (*„vím, že...“*). V lidském mozku je však deklarativní paměť v neokortexu a hipokampu pouze jednou z mnoha složek.

Klíčové kognitivní vrstvy, které dělají lidského experta rychlým a intuitivním, sídlí jinde:
1. **Procedurální svalová paměť (Bazální ganglia & Mozeček):** Znalost *„jak se to dělá“* zautomatizovaná do motorických celků bez účasti vědomí (teorie **ACT-R** Johna Andersona – *Production Compilation*).
2. **Asociační sítě a Priming (Temporální lalok):** Podvědomé předaktivování souvisejících konceptů dříve, než na ně přijde vědomá řeč (Collins & Quillian).
3. **Emoční valence a paměť na otřesy (Amygdala):** Schopnost okamžitě a doživotně zafixovat vzpomínku na základě krizového šoku nebo triumfu (*Flashbulb Memory*).

Tento návrh doplňuje UMA o tyto tři fundamentální mechanismy a odstraňuje největší zdroj plýtvání tokeny a latence v agentních bězích.

---

## 2. Pilíř I: Procedurální svalová paměť (Motor Chunks)

### Fundamentální problém současných agentů:
Dnešní agent řeší každý triviální motorický krok (kontrola gitu, formátování, spuštění specifického testu, zjištění linter chyb) přes drahé uvažování velkého modelu (System 2):
* Vygeneruje 150 tokenů vnitřního monologu: *„Nyní zavolám příkaz cargo test pro modul search...“*
* Zavolá nástroj $\rightarrow$ počká na odpověď $\rightarrow$ vygeneruje dalších 150 tokenů: *„Vidím, že test prošel, nyní spustím git status...“*
* Na banální rutinu propálí **1000 tokenů a 15 sekund**, které člověk zvládne za půl sekundy svalovou pamětí v terminálu.

### Řešení v UMA: Zkompilované motorické rutiny (Compiled Action Chunks)
Agent nemusí řídit každý krok. Vysloví pouze **motorický záměr**:
`uma_muscle("verify_slice", { slice: "search" })`

Celá sekvence proběhne v **nativním Rustovém jádře UMA** bez mezer a bez LLM volání:
```text
┌────────────────────────────────────────────────────────┐
│  Agent (Claude 3.7): "uma_muscle('verify_slice')"      │
└───────────────────────────┬────────────────────────────┘
                            │ Jedno volání nástroje
                            ▼
┌────────────────────────────────────────────────────────┐
│  UMA RUST RUNTIME (Mozeček & Bazální ganglia)          │
│  1. cargo fmt --check                                  │
│  2. cargo clippy -- -D warnings                        │
│  3. cargo test -p uma-cli -- slices::search            │
│  4. git diff --stat                                    │
│  Vše běží paralelně / pipelinovaně v lokálním OS!      │
└───────────────────────────┬────────────────────────────┘
                            │ Vrátí zkomprimovaný souhrn
                            ▼
┌────────────────────────────────────────────────────────┐
│  Výsledek pro Agenta:                                  │
│  "✓ Formát čistý, Clippy 0 chyb, 8 testů OK (0.04s),   │
│   2 modifikované soubory. Připraveno k commitu."       │
└────────────────────────────────────────────────────────┘
```
**Úspora:** 85 % času a 90 % tokenů na rutinních vývojářských kolečkách.

### Automatická syntéza svalové paměti:
Stínový pracovník ([Proposal 02](file:///D:/01_programovani/ai-memory/docs/proposals/02-shadow-brain-zero-latency-distillation.md)) sleduje historii příkazů. Pokud agent během dvou relací zopakuje stejnou sekvenci tří nástrojů s identickými návratovými kódy, UMA mu automaticky navrhne:
*„Detekován opakovaný motorický vzorec. Kompiluji novou svalovou rutinu `run_migration_and_smoke_test`.“*

---

## 3. Pilíř II: Kognitivní Priming & Šíření aktivace (Spreading Activation)

### Neurobiologický princip:
Když lidský mozek zachytí podnět *„auto“*, neurony okamžitě pošlou slabý excitační signál do okolních uzlů: *kola, motor, volant, silnice*. Když pak člověk uvidí slovo *volant*, rozpozná ho bleskově, protože synapsie už byly **předaktivované (primed)**.

### Implementace v UMA: Asociační předhřívání cache
Místo čekání na pomalý full-text nebo vektorový dotaz udržuje UMA v paměti **asociační graf vazeb**:

```text
Dotyk se souborem: "uma-core/src/indexer/ops.rs"
       │
       ▼
┌─────────────────────────────────────────────────────────────┐
│  UMA Asociační Priming Engine                               │
│  Předaktivuje do L1 rychlé paměti:                          │
│  - Fakt [01M4D7...]: "SQLite WAL mode requirement" (přímá)  │
│  - Fakt [01M4D9...]: "FTS5 schema rebuild rules"   (přímá)  │
│  - Fakt [01M4E1...]: "Windows file lock gotchas"   (asociace│
│                      přes SQLite na Windows)                │
└─────────────────────────────────────────────────────────────┘
```

Když agent v dalším kroku položí dotaz na databázi nebo se pokusí otevřít transakci, **relevantní pravidla už má načtená v L1 kontextu bez jakékoliv latence**.

---

## 4. Pilíř III: Amygdala a blesková paměť na traumata (Affective Saliency)

### Neurobiologický princip (Flashbulb Memory):
Člověk neukládá vzpomínky podle počtu slov, ale podle **emoční intenzity**. Banální úpravu CSS zapomenete za 2 dny. Pád platební brány na produkci nebo bezpečnostní exploit si pamatujete do detailu do konce kariéry.

### Rozšíření OKF v0.2 frontmatteru o Emoční valenci:
V [`uma-core/src/domain.rs`](file:///D:/01_programovani/ai-memory/uma/uma-core/src/domain.rs#L190) rozšiřujeme fakt o koeficient závažnosti `saliency`:

```yaml
---
id: 01M4F9Z20KLT891AB01
title: Never run SQLite in default journal mode under concurrent multi-client
type: correction
status: stable
saliency:
  shock_level: "critical_outage"  # [routine, bug, incident, critical_outage]
  multiplier: 10.0                # Multiplikátor synaptické váhy
  immune_to_decay: true           # Zombie Reaper nesmí tento fakt NIKDY smazat!
---
```

### Důsledky pro životní cyklus:
* Fakty s `immune_to_decay: true` **nikdy nepodléhají poločasu rozpadu** z [Proposal 03](file:///D:/01_programovani/ai-memory/docs/proposals/03-synaptic-plasticity-executable-contracts.md).
* V Kognitivním imunitním interceptoru ([Proposal 01](file:///D:/01_programovani/ai-memory/docs/proposals/01-cognitive-immune-interceptor.md)) mají absolutní prioritu – i nepatrný náznak jejich porušení způsobí okamžité zastavení nástroje.

---

## 5. Pilíř IV: Ebbinghausova křivka a efekt testování (Retrieval Practice)

### Neurobiologický princip (The Testing Effect):
Z výzkumů paměti (Roediger & Karpicke) vyplývá, že pouhé pasivní čtení textu paměť neposiluje. Co paměť skutečně zpevňuje, je **namáhavé aktivní vybavování (Retrieval Practice)**.

### Implementace v UMA: Noční snění (`uma doctor --dream`)
Při periodické údržbě paměti UMA neprovádí jen pasivní kompresi, ale **aktivní testování agenta**:
1. UMA vybere fakt, kterému klesá synaptická váha (blíží se k poločasu rozpadu).
2. Vygeneruje pro malý model (Jev / SLM) syntetický dotaz:  
   *„Jaké je pravidlo pro přístup k SQLite indexu z více vláken?“*
3. Pokud model správně odpoví $\rightarrow$ synaptická váha faktu se zresetuje na 1.0 a prodlouží se jeho životnost.
4. Pokud selže $\rightarrow$ fakt je označen jako „nezřetelný“ a v dalším běhu je předložen operátorovi k revizi nebo upřesnění.

---

## 6. Kompletní kognitivní pyramida UMA (Úrovně L0 až L5)

S tímto pátým návrhem je kognitivní architektura UMA kompletní:

| Úroveň | Kognitivní vrstva | Odpovídající návrh | Biologický ekvivalent |
| :--- | :--- | :--- | :--- |
| **L5** | **Motorická rutina & Priming** | **Proposal 05 (Svalová paměť)** | **Mozeček, bazální ganglia, asociační priming** (autonomní exekuce rutin bez tokenů). |
| **L4** | **Soudnost & Sebekritika** | **[Proposal 04 (Vnitřní sněm)](file:///D:/01_programovani/ai-memory/docs/proposals/04-internal-council-and-prudence.md)** | **Prefrontální kortex** (Anděl vs. Ďábel, kalkulace rizika, prospektivní závazky). |
| **L3** | **Zkušenost & Adaptace** | **[Proposal 03 (Synaptická plasticita)](file:///D:/01_programovani/ai-memory/docs/proposals/03-synaptic-plasticity-executable-contracts.md)** | **Synaptická plasticita & Hebbovské učení** (posilování vah, AST invarianty v CI). |
| **L2** | **Podvědomí & Spánek** | **[Proposal 02 (Stínový mozek)](file:///D:/01_programovani/ai-memory/docs/proposals/02-shadow-brain-zero-latency-distillation.md)** | **Spánková konsolidace v hipokampu** (asynchronní těžba telemetrie bez latence). |
| **L1** | **Pud sebezáchovy & Reflex**| **[Proposal 01 (Imunitní interceptor)](file:///D:/01_programovani/ai-memory/docs/proposals/01-cognitive-immune-interceptor.md)** | **Mozkový kmen & reflexní oblouk** (okamžitá blokace zakázaných zásahů za 80 ms). |
| **L0** | **Paměťová schránka** | **UMA Core (OKF v0.2 + Rust)** | **Dlouhodobá engramová banka** (Markdowny v `.uma/`, ULID, Git provenience, FTS5). |

Tato architektura překonává veškeré dosavadní pokusy v oboru: **spojuje rychlost nativního kódu, bezpečnost formálních kontraktů a pružnost biologického myšlení**.
