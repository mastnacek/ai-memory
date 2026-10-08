 ### 1. Verdikt v kostce (Executive Summary)

  • Architektura & Kód (9/10): Vynikající zvládnutí Vertical Slice Architecture (VSA). Čisté hranice, žádné
  křížové importy mezi řezy, striktní dodržování limitů délky souborů (< 300–400 řádků), 121 procházejících testů
  bez varování.
  • Bezpečnostní & Consent Model (9.5/10): Fail-closed approval gate, interaktivní modal s diffem a možností
  editace, MCP read-only by default. Tady je přístup špičkový.
  • Storage & Indexační Engine (5.5/10): Zde je největší slabina. Obsahuje reálný výkonnostní bug (reindexace při
  každém prázdném vyhledávání), ignorování parametru projektu při vyhledávání v jiné scope, chybějící
  WAL/busy_timeout v SQLite a paměťově neškálovatelný vektorový in-memory brute-force scan místo slibovaného
  LanceDB.
  • Filozofie "Local-First" (6/10): Rozpor mezi deklarovanou nezávislostí na cloudu a realitou. Zápis do storu
  synchronně blokuje vlákno na HTTP požadavku do OpenRouteru. Pokud jste offline, sémantika stojí.
  ──────
  ### 2. Co se povedlo na výbornou (Kde projekt exceluje)
  1. Striktní dodržování VSA (Vertical Slice Architecture):
      • Composition Root v `main.rs je absolutně čistý – pouze parsování argumentů a delegace.
      • Každý řez v  řeší svůj problém od argumentů po prezentaci a má vlastní README.md.
      • Žádný řez neimportuje jiný řez; veškeré sdílení jde přes uma-core nebo .
  2. Fail-Closed Approval Gate & Consent Model:
      • V approval_gate.ts je neprůstřelná brána: pokud agent běží v neinteraktivním režimu (pi -p, subagent,
      skript) a nemá povolen autoApprove, zápis se striktně zamítne.
      • Nástroje jako consolidate a import pouze navrhují, nikdy samy nemutují paměť.
      • V mod.rs:47-59 je MCP server read-only a zápisové nástroje ani nepublikuje, pokud není explicitně předán
      příznak --allow-writes.
  3. Datový model OKF v0.2 + Frontmatter:
      • Ukládání faktů jako Markdown + YAML frontmatter s ULID v .uma/ zaručuje, že data nejsou uvězněna v
      proprietární binární databázi a jsou čitelná pro člověka i git diff.
      • Řetězení změn přes supersedes, hlídání stale_after a odfiltrování zastaralých faktů z výchozího
      vyhledávání je pro zamezení halucinacím agenta zásadní.
  4. Resilient Hybrid Search (RRF):
      • V mod.rs:104-140 hybridní vyhledávání při výpadku vektorů nezhavaruje, ale transparentně degraduje na BM25
      a tuto skutečnost předá volajícímu v SearchOutcome.

  ──────
  ### 3. Kritická místa, chyby a technický dluh (Bugs & Flaws)
  #### 3.1. Závažný výkonnostní bug: Každý "miss" ve vyhledávání maže a reindexuje celý FTS index!
  V souboru uma-core/src/store/search.rs` (řádky 44–54):

    SearchMode::Keyword => {
        let hits = search_keyword(indexer.connection(), &opts)?;
        if hits.is_empty() {
            // The index is a rebuildable cache, so an empty result may
            // just mean a stale one. Rebuild and retry once.
            let indexed = Self::reindex_all()?;
            if indexed > 0 {
                return Ok(SearchOutcome::complete(search_keyword(
                    indexer.connection(),
                    &opts,
                )?));
            }
        }
        Ok(SearchOutcome::complete(hits))
    }
  Problém: Pokud agent hledá termín, který v paměti vůbec neexistuje (běžný stav při rešerši):

  1. hits.is_empty() je true.
  2. Zavolá se Self::reindex_all().
  3. V Indexer::reindex_from_dirs se provede DELETE FROM facts_fts; a rekurzivně se projdou a zparsují všechny
  markdown soubory na disku.
  4. Protože indexed > 0 (např. 32 faktů), dotaz se spustí znovu.
  5. Znovu nic nenajde a vrátí prázdný výsledek.
  Důsledek: Každé neúspěšné hledání v keyword módu zbytečně smaže a od nuly znovu postaví celou FTS tabulku na
  disku! Při 500 faktech to znamená obrovský I/O zásek při pouhém překlepu nebo nenalezeném slovu. Reindexace musí
  být explicitní (--reindex) nebo vázaná na kontrolu mtime složky, nikoliv na neshodu dotazu.
  ──────
  #### 3.2. Rozbitý cross-project scope u uma list a Store::project

  V uma-core/src/store/mod.rs` (řádky 58–62):
    pub fn project(_project_name: String) -> Result<Self> {
        let root = Self::find_git_root()?.join(".uma");
        std::fs::create_dir_all(&root).context("Failed to create project store directory")?;
        Ok(Self::new(root))
    }
  Všimněte si podtržítka: _project_name je zcela ignorováno!
  • Pokud jste v repozitáři ProjektA a spustíte:
  uma list --scope ProjektB
      1. resolve_scope vytvoří Scope::Project("ProjektB").
      2. get_store zavolá Store::project("ProjektB").
      3. Store::project ignoruje "ProjektB", najde git_root aktuálního adresáře a načte soubory z ProjektA/.uma!
  • Pokud jste mimo jakýkoliv git repozitář (např. domovský adresář) a spustíte uma list --scope ProjektB, příkaz
  spadne na Not inside a git repository.
  • Přitom v mod.rs:49 nápověda tvrdí: Cross-project recall: uma search ... --scope <scope>. U search to funguje
  jen proto, že SQLite index má sloupec project_name. U list je cross-project zobrazení rozbité.
  ──────
  #### 3.3. Nekonzistence názvu binárky: uma vs uma-cli

  V Cargo.toml:1-6 je definován pouze balíček name = "uma-cli".
  Chybí explicitní:

    [[bin]]
    name = "uma"
    path = "src/main.rs"

  • Důsledek: cargo build i cargo install vygenerují spustitelný soubor uma-cli.exe (resp. uma-cli), nikoliv uma.
  • Všechna dokumentace, prompt v AGENTS.md i instrukce v SKILL.md instruují agenta: "Spusť uma write / uma
  search". Pokud agent tyto příkazy spustí v shellu, dostane command not found: uma.
  • V TypeScript extension to autor musel obcházet v findUmaBinary hledáním jak uma-cli, tak uma.
  ──────
  #### 3.4. Vektorové vyhledávání: Slib versus Realita (LanceDB vs paměťový scan)
  V PRD §3 a §4 se píše:
  │ Indexes: SQLite FTS5 (keyword) + LanceDB (vectors)
  │ Primary — best for CLI performance, embedding (LanceDB via lancedb-rs)

  V kódu však LanceDB vůbec nefiguruje. Místo toho:

  1. Vektory jsou uloženy v SQLite jako f64 (8 bajtů na prvek místo standardních 4 bajtů f32 – zbytečný 2× nárůst
  velikosti).
  2. V uma-core/src/search/semantic.rs` (řádky 20–22) se při každém sémantickém dotazu volá
  load_all_embeddings(conn), který natáhne veškeré existující vektory v celé databázi do HashMap v RAM a počítá
  kosínovou podobnost v jednovláknové smyčce.
  3. Pro 30 faktů to trvá 0.1 ms. Pro tisíce faktů (např. po importu z 578 session zmíněných v PRD) to bude
  alokovat stovky megabajtů a zbytečně vytěžovat CPU. Chybí buď LanceDB, nebo SQLite vektorové rozšíření (sqlite-
  vec).
  ──────
  #### 3.5. Konkurence a zámky v SQLite (database is locked hazard)
  UMA je navržena pro souběžné použití více klienty: CLI, Pi agent s TUI, MCP server pro Claude Code / Cursor a
  potenciální subagenti.
  Všechny tyto procesy přistupují ke stejné centrální databázi index.db.
  V Indexer::open:

  • Není nastaven PRAGMA busy_timeout (výchozí SQLite hodnota je 0 ms).
  • Není zapnut PRAGMA journal_mode = WAL (zůstává defaultní rollback journal).
  • Zápisy v index_fact (DELETE + INSERT) nejsou uzavřeny v transakci.
  Důsledek: Pokud Pi extension nebo CLI zapisuje факт nebo reindexuje a MCP server ve stejný okamžik obdrží
  vyhledávací dotaz z Cursoru, SQLite okamžitě selže s chybou database is locked (code 5). Nastavení WAL režimu a
  alespoň 5000ms busy_timeout je pro multi-client aplikaci nutnost.
  ──────
  #### 3.6. Iluze "Local-First": Blokující cloudové volání při zápisu

  Projekt se hrdě označuje jako local-first, ale vektorizace v Store::write volá synchronní HTTP požadavek na
  OpenRouter:
  • Timeout je 30 sekund s až 3 pokusy o retry s exponenciálním zpožděním (embeddings.rs:52-138).
  • Pokud jste offline, na slabém připojení nebo OpenRouter hlásí chybu 429/500, operace uma write může zablokovat
  TUI i agenta na desítky sekund.
  • Chyba vektorizace je navíc v Store::write ignorována (let _ = ...), takže uživatel ani neví, zda se vektor
  uložil, nebo tiše selhal.
  • Chybí lokální offline fallback (např. fastembed-rs, ONNX Runtime nebo lokální Ollama endpoint).
  ──────
  #### 3.7. Jazyková slepota v konsolidačním enginu (similarity.rs)
  V similarity.rs:10-53:

  • Stop-slova (STOPWORDS) a markery negace (NEGATION_MARKERS) jsou pouze v angličtině (not, never, avoid, don't).
  • Stemmer provádí triviální ořezávání anglických koncovek (-ies, -ation, -ing, -ed).
  • Přitom v PRD se explicitně počítá s importem stovek starých relací v češtině a v extract.rs:88-102 jsou české
  markery přítomny (vzdy, nikdy, preferuju, spatne).
  • Důsledek v praxi: Pokud agent uloží dvě česká pravidla:
      1. "Vždy používat pnpm"
      2. "Nikdy nepoužívat pnpm"
      Konsolidační algoritmus českou negaci nerozezná, vyhodnotí je přes Jaccardův index jako téměř identické
      texty a navrhne je ke sloučení jako duplicity, místo aby vyhlásil kontradikci! Česká flexe navíc způsobí, že
      skloňovaná slova ("databáze", "databázi") Jaccard nespojí.
  ──────
  #### 3.8. Riziko visícího procesu v uma sync
  V git.rs:14-28 se volá externí git přes std::process::Command.
  • Pokud má vzdálený repozitář nastavené HTTPS nebo SSH vyžadující interakci (např. chybějící SSH klíč nebo
  heslo), git na Windows může otevřít dialogové okno nebo zablokovat standardní vstup.
  • Chybí nastavení environmentální proměnné:
  .env("GIT_TERMINAL_PROMPT", "0")
  aby git okamžitě selhal namísto nekonečného čekání na terminálový prompt.
  ──────
  ### 4. Agentní perspektiva: Paměť versus Pasivní dokumentace
  V PRD §2 (S3) je uvedeno:
  │ Auto-Recall & Context Injection — ON HOLD (Operator preference): Auto-injecting facts into turns is paused to
  │ prevent context noise and hallucinations.

  Rozhodnutí operátora chápu – nekontrolovaná injekce 10 náhodných faktů do každého promptu plýtvá tokeny a mate
  model.
  Ale je třeba si nalít čistého vína:
  V momentě, kdy systém nemá auto-recall ani automatický hook před generováním odpovědi, UMA v současném stavu
  není "paměť agenta", ale "strukturovaná externí znalostní báze".
  Agent o minulých rozhodnutích neví, dokud:

  1. Buď operátor explicitně nenapíše: "Najdi v paměti..."
  2. Nebo agent sám proaktivně nezavolá uma_search (což většina LLM modelů bez velmi silného system promptu
  spontánně neudělá).

  Pokud má být UMA skutečnou autonomní pamětí, je potřeba lehký kompromis pro S3: např. Recall-on-Start (při
  startu session nebo změně úlohy načíst top-3 decision a preference z daného projektu a vložit je do systémového
  kontextu).
  ──────
  ### 5. Doporučený plán nápravy (Prioritizovaný backlog)

   Priorita      | Úkol                                  | Kde upravit   | Proč
  ---------------|---------------------------------------|---------------|----------------------------------------
   P0 (Kritické) | Odstranit automatický reindex_all()   | search.rs:44- | Zabrání zbytečnému mazání a
                 | při prázdném výsledku vyhledávání.    | 54            | rebuildování celého indexu při každém
                 |                                       |               | nenalezeném slovu.
   P0 (Kritické) | Zapnout SQLite WAL a                  | mod.rs:32-37  | Zabrání pádům na database is locked
                 | busy_timeout(5000).                   |               | při souběhu CLI, MCP a Pi extension.
   P1 (Vysoká)   | Přidat [[bin]] name = "uma" do        | Cargo.toml    | Aby generovaná binárka odpovídala
                 | manifestu CLI.                        |               | dokumentaci a skillům (uma, nikoliv
                 |                                       |               | uma-cli).
   P1 (Vysoká)   | Opravit Store::project a uma list --  | mod.rs:58     | Aby výpis projektové paměti
                 | scope <proj>.                         |               | respektoval zadaný název projektu i z
                 |                                       |               | jiného pracovního adresáře.
   P2 (Střední)  | Doplnit české stop-slova a markery    | similarity.rs | Aby systém v českých textech
                 | negace do konsolidátoru.              | :10-38        | nerozpoznával protiklady jako
                 |                                       |               | duplicity.
   P2 (Střední)  | Ošetřit GIT_TERMINAL_PROMPT=0 u       | git.rs:15     | Prevence zamrznutí procesu na pozadí
                 | synchronizace.                        |               | při chybějících pověřeních pro git
                 |                                       |               | remote.
   P3 (Výhled)   | Lokální embedding fallback (např.     | embeddings.rs | Skutečná nezávislost na internetu a
                 | ONNX / fastembed-rs).                 |               | OpenRouter API.