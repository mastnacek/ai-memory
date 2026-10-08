# Proposal 03: Synaptická plasticita a spustitelné kontrakty (Memory Fitness & Executable AST Invariants)

**Status:** Draft / Architectural Proposal  
**Cílová vrstva:** `uma-core/src/domain.rs`, `uma-core/src/contracts/`, `uma-cli/src/slices/doctor/`  
**Datum:** 2026-10-08  

---

## 1. Abstrakt a fundamentální problém

Dlouhodobé paměti v softwarových projektech čelí **dvěma smrtelným patologiím**, které způsobují, že po 6 měsících vývoje přestávají fungovat:

1. **Hnití paměti (Memory Rot & Zombie Rules):** Softwarová architektura se vyvíjí. Knihovna se vymění, jazyk dostane novou verzi, starý vzor přestane platit. V pasivní paměti však stará pravidla zůstávají navždy. Agent se pak řídí „zombie pravidly“, která byla správná v roce 2024, ale v roce 2026 jsou škodlivým anachronismem.
2. **Bezzubost volného textu (Unenforceable Prose):** Popis typu *„Nikdy nevolejte metodu X uvnitř smyčky Y“* je pouze prose text. LLM model má tendenci textové zákazy při vysoké komplexitě úkolu ignorovat, protože text nemá žádnou přímou vymahatelnost v nástrojích kompilátoru.

### Řešení: Neuro-symbolická evoluce paměti
Navrhujeme spojit dva principy:
* **Synaptická plasticita (Hebbovské učení):** Každý paměťový fakt získává dynamické skóre zdatnosti (**Fitness Weight** $\in \langle 0.0, 1.0 \rangle$). Fakty, které vedou k úspěšně prošlým testům, posilují; fakty vedoucí k chybám nebo dlouhodobě nepoužívané degradují a jsou automaticky navrženy k archivaci.
* **Spustitelné kontrakty (Executable Memory):** Důležitá architektonická rozhodnutí obsahují strojově ověřitelný kontrakt (např. AST vzor pro `ast-grep` nebo regex pro pre-commit linter). Paměť se tak přímo transformuje v **živý integrační test architektury**.

---

## 2. Část A: Synaptická plasticita (Dynamická váha a degradace)

V biologickém mozku paměťové stopy (engramy) posilují, pokud vedou k úspěchu (Long-Term Potentiation), a slábnou, pokud jsou neaktivní (Long-Term Depression).

### Rozšíření datového modelu OKF v0.2:
Do struktury faktu v [`uma-core/src/domain.rs`](file:///D:/01_programovani/ai-memory/uma/uma-core/src/domain.rs#L190) přidáváme pole `plasticity`:

```yaml
---
id: 01M4D7S5YART7AGWN7RDSRNRM1
title: Strict Vertical Slice Architecture
type: decision
status: stable
plasticity:
  weight: 0.95              # Aktuální synaptická váha (0.0 až 1.0)
  reinforcements: 18        # Kolikrát byl fakt úspěšně aplikován
  frustrations: 0           # Kolikrát vedl k chybě nebo zamítnutí
  last_activated: "2026-10-08T18:30:00Z"
  half_life_days: 90        # Poločas rozpadu bez aktivace
---
```

### Matematický model úpravy vah:

1. **Úspěšné posílení (LTP - Long-Term Potentiation):**
   * Podmínka: Agent při řešení úlohy načetl fakt $F$, provedl změny a `cargo test` / build **prošel na první pokus** s následným schváleným commitem.
   * Úprava:
     $$Weight_{new} = \min(1.0, Weight_{old} + 0.05)$$
2. **Kognitivní frustrace (LTD - Long-Term Depression):**
   * Podmínka: Po aplikaci faktu selhal kompilátor, testy hodily regresi nebo uživatel v modalu návrh odmítl se slovy *„takhle už to neděláme“*.
   * Úprava:
     $$Weight_{new} = \max(0.0, Weight_{old} - 0.25)$$
3. **Časový rozpad (Exponential Decay):**
   * Pokud fakt nebyl po dobu $t$ dní ani jednou aktivován v žádné relaci:
     $$Weight(t) = Weight_0 \cdot \left(\frac{1}{2}\right)^{\frac{t}{T_{half}}}$$

### Zombie Reaper (`uma doctor --prune-zombies`):
Nový diagnostický modul v `doctor` pravidelně kontroluje integritu paměti vůči codebase:
* **Hledání fantomových symbolů:** Pokud fakt mluví o modulu `crate::legacy_parser`, ale tento modul už v repozitáři 3 měsíce neexistuje, Jev označí fakt jako Zombie kandidáta.
* **Automatické vyřazení z L1 indexu:** Fakty s $Weight < 0.25$ automaticky vypadávají z rychlého kontextu agenta a jsou přesunuty do chladného archivu (`status: dormant`).

---

## 3. Část B: Spustitelné kontrakty (Executable Memory)

Místo toho, aby paměť byla pouze pasivním textem, stává se **generátorem testovacích invariantů**.

### Schéma kontraktu ve frontmatteru faktu:
```yaml
---
id: 01M4D7S5YART7AGWN7RDSRNRM1
title: Strict Vertical Slice Architecture
type: decision
status: stable
contract:
  engine: "ast-grep"
  severity: "deny"
  rule:
    pattern: "use crate::slices::$$$REST;"
    inside: "src/slices/**"
    message: "Inviolable VSA Rule: Slices must NEVER import each other directly! Use uma-core or src/shared."
---
```

### Jak kontrakt funguje v životním cyklu:

```text
┌────────────────────────────────────────────────────────────────────────┐
│  SCHVÁLENÍ FAKTU V UMA (Lidský souhlas v modalu)                      │
│  Operátor schválí architektonické rozhodnutí s kontraktem              │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  AUTOMATICKÝ EXPORT KONTRAKTU (uma contracts export)                   │
│  UMA vygeneruje nebo zaktualizuje:                                     │
│  - .uma/contracts/sgconfig.yml (pro ast-grep)                          │
│  - tests/architecture_invariants.rs (Rust integrační test)             │
│  - .git/hooks/pre-commit (volitelný rychlý git hook)                   │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│  DVOUÚROVŇOVÉ VYNUCOVÁNÍ (ENFORCEMENT)                                 │
│                                                                        │
│  1. Agentní reflex (JEV Interceptor):                                  │
│     Zastaví pokus o nevalidní kód ještě v paměti Pi agenta (Proposal 01)│
│                                                                        │
│  2. Deterministická závora (CI / cargo test):                          │
│     Pokud agent reflex obejde, 'cargo test' selže na testu:            │
│     test architecture_invariants::enforce_uma_contracts ... FAILED     │
└────────────────────────────────────────────────────────────────────────┘
```

### Ukázka vygenerovaného Rust integračního testu (`tests/architecture_invariants.rs`):
```rust
//! Generated by UMA — Universal Memory Architecture
//! Invariants compiled directly from active memory decisions.

#[test]
fn enforce_memory_decision_01m4d7s5_vsa_isolation() {
    let output = std::process::Command::new("ast-grep")
        .args(["scan", "--rule", ".uma/contracts/01m4d7s5.yml"])
        .output()
        .expect("Failed to execute ast-grep");

    assert!(
        output.status.success(),
        "Architecture invariant [01M4D7S5] violated!\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}
```

---

## 4. Čtyřfázový životní cyklus architektonického pravidla

Tento model vytváří přirozený evoluční trychtýř:

```text
┌──────────────────────┐
│ 1. STAGED CANDIDATE  │  Shadow Worker zachytí opakující se vzor / opravu.
└──────────┬───────────┘  Zatím nemá žádnou váhu ani vliv.
           │ Člověk schválí v TUI
           ▼
┌──────────────────────┐
│ 2. SOFT RULE         │  Fakt uložen v .uma/ s výchozí vahou 0.50.
└──────────┬───────────┘  Vynucován přes Jev Immune Interceptor (Proposal 01).
           │ Opakovaně úspěšně aplikován (Váha stoupne na >= 0.85)
           ▼
┌──────────────────────┐
│ 3. HARD CONTRACT     │  Fakt povýšen na exekutivní AST kontrakt.
└──────────┬───────────┘  Stává se součástí CI testů projektu.
           │ Změna technologií / Nahrazení novým rozhodnutím
           ▼
┌──────────────────────┐
│ 4. SUPERSEDED / DEAD │  Váha klesne pod 0.20 nebo je fakt explicitně nahrazen.
└──────────────────────┘  Kontrakt v CI se automaticky smaže, historie zůstává.
```

---

## 5. Přínos a revoluční hodnota
* **Konec rozporu mezi dokumentací a realitou:** Co je v paměti, to reálně platí v kódu. Není možné, aby paměť tvrdila jedno a codebase dělala druhé.
* **Organické samoočišťování:** Databáze nezamrzá v čase. Mrtvá a nepoužívaná pravidla sama vyblednou díky poločasu rozpadu.
* **Deterministická jistota:** Tým už nemusí spoléhat na to, zda má LLM model zrovna „dobrý den“ a dodrží instrukci. Nejdůležitější pravidla hlídá přímo kompilátor.
