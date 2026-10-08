# Top 30 AI-agent memory systems (stars desc, updated recently)
# fetched 2026-10-07

1. https://github.com/mem0ai/mem0
2. https://github.com/volcengine/OpenViking
3. https://github.com/topoteretes/cognee
4. https://github.com/getzep/graphiti
5. https://github.com/supermemoryai/supermemory
6. https://github.com/letta-ai/letta
7. https://github.com/MemoriLabs/Memori
8. https://github.com/memvid/memvid
9. https://github.com/NevaMind-AI/memU
10. https://github.com/semantica-agi/semantica
11. https://github.com/EverMind-AI/EverOS
12. https://github.com/MemTensor/MemOS
13. https://github.com/plastic-labs/honcho
14. https://github.com/getzep/zep
15. https://github.com/FlowElement-xinliuyuansu/m_flow
16. https://github.com/agentscope-ai/ReMe
17. https://github.com/zilliztech/memsearch
18. https://github.com/kayba-ai/agentic-context-engine
19. https://github.com/tigerless-labs/agent-memory
20. https://github.com/moorcheh-ai/memanto
21. https://github.com/cortexkit/magic-context
22. https://github.com/doobidoo/mcp-memory-service
23. https://github.com/supermemoryai/supermemory-mcp
24. https://github.com/langchain-ai/langmem
25. https://github.com/BAI-LAB/MemoryOS
26. https://github.com/Prismer-AI/PrismerCloud
27. https://github.com/ghostwright/phantom
28. https://github.com/vshulcz/deja-vu
29. https://github.com/LycheeMem/LycheeMem
30. https://github.com/mindscale-noah/MindMemOS

---

## Principy memory (5 vět na repo)

### 1. mem0ai/mem0
Memory funguje jako vrstva nad LLM: z každé konverzace jediným LLM callem (ADD-only, bez UPDATE/DELETE) extrahuje fakty, které se kumulují a nikdy se nepřepisují. Fakty potvrzené agentem se ukládají se stejnou váhou jako fakty od uživatele. Entity se extrahují, embeddují a propojují napříč vzpomínkami pro boost retrievalu. Vyhledávání je multi-signálové — semantické embeddingy, BM25 keyword a entity matching běží paralelně a skóre se fůzují. Časové uvažování (temporal reasoning) řadí správnou datovanou instanci faktu podle toho, jestli se dotaz týká současnosti, minulosti nebo budoucnosti.

### 2. volcengine/OpenViking
Kontext (knowledge, memory i skills) je organizovaný jako virtuální souborový systém pod `viking://` URI, který agent prochází příkazy ls/tree/read/write/grep. Každý adresář má generované souhrny ve třech vrstvách: L0 abstrakt (jedna věta), L1 přehled a L2 plný obsah, takže agent posoudí relevanci dřív, než otevře soubor. Semantické vyhledávání se scope-uje na adresář/podstrom místo plochého vektorového indexu. Commitnutí session archivuje konverzaci a extrahuje vzpomínky jako čitelný Markdown, který lze inspektovat, editovat a mergovat. Volitelný `ov compile` pak zdrojový materiál organizuje do wiki, knowledge grafu nebo reportu.

### 3. topoteretes/cognee
Cognee staví z dokumentů, kódu a konverzací self-hosted knowledge graph jako dlouhodobou paměť agentů. Pipeline `remember` (dřív cognify) extrahuje entity a vztahy do grafu a zároveň embedduje text — bez LLM klíče použije lokální GLiNER model a lokální embeddingy na CPU. `recall` pak vyhledává v grafu i embeddingech a vrací zdrojové pasáže. Paměť je možné strukturovat přes vlastní datové modely a ontologie (Pydantic), takže graf odpovídá doméně aplikace. Agenty se připojují přes pluginy/MCP a lekce ze session se destilují do trvanlivé znalosti použitelné v dalších bězích.

### 4. getzep/graphiti
Graphiti staví temporální knowledge graf: entity (uzly), fakty/vztahy (hrany jako triplety) a epizody (surová zdrojová data jako provenance). Každý fakt má časové okno validity — když se informace změní, starý fakt se invaliduje, ale nemaže, takže lze dotazovat „co platilo kdy". Data se ingestují inkrementálně jako epizody, graf se aktualizuje v reálném čase bez dávkového přepočtu. Ontologie je buď prescribed (Pydantic modely typů entit a hran), nebo learned (struktura emergentně vzniká z dat). Retrieval je hybridní — semantické embeddingy + BM25 + průchod grafem, bez závislosti na LLM sumarizaci při dotazu.

### 5. supermemoryai/supermemory
Supermemory automaticky extrahuje fakty z konverzací a staví z nich uživatelské profily (stabilní fakta + nedávná aktivita), dostupné jedním ~50ms callem. Systém řeší temporální změny, kontradikce a automatické zapomínání expirovaných informací. Hybridní search kombinuje RAG nad dokumenty a personalizované vzpomínky v jednom dotazu. Connectory (Drive, Gmail, Notion, GitHub) synchronizují externí zdroje přes webhooky a multimodální extractory zpracují PDF, obrázky (OCR), videa i kód (AST-aware chunking). Vše žije v jediné memory struktuře a ontologii, dostupné přes API, MCP server nebo pluginy pro coding agenty.

### 6. letta-ai/letta
Letta (dřív MemGPT) staví stateful agenty, kteří se učí tím, že programaticky přepisují vlastní kontext — memory bloky, skills, system prompty i harness. Paměť je organizovaná do memory bloků, které agent sám edituje jako součást svého context window (princip self-editing memory z MemGPT). MemFS trackuje veškerý kontext včetně memory bloků v gitu, takže paměť je verzovaná a synchronizovatelná do vlastního repozitáře. Periodické „dreaming" přes /sleeptime konsoliduje zkušenosti mimo hlavní konverzaci. Letta Cloud drží paměť, identitu a konverzace agenta napříč stroji, takže běh (laptop, CI, sandbox) je oddělený od stavu.

### 7. MemoriLabs/Memori
Memori se napojuje na LLM klienta (register/attribution) a automaticky v pozadí persistuje konverzace a recalluje relevantní vzpomínky do dalších dotazů — bez změny kódu aplikace. Princip je „memory z toho, co agent dělá, ne jen co říká" — zachytává chování a infrastrukturní události, ne jen text. Je LLM-, datastore- i framework-agnostický, paměť se váže na entity_id (uživatel) a process_id (agent/proces). Recall probíhá automaticky při dalších callech, takže odpovědi zohlední dříve řečené preference. Nad tím stojí dashboard s memories, analytics a playgroundem pro inspekci uloženého.

### 8. memvid/memvid
Memvid balí data, embeddingy, search strukturu a metadata do jediného přenositelného souboru (.mv2) — žádná databáze ani server. Paměť je organizovaná jako append-only sekvence „Smart Frames" inspirovaná video enkódováním: imutabilní jednotky s timestampy, checksumy a metadaty, efektivně komprimované a paralelně čitelné. Díky tomu podporuje dotazy nad minulými stavy paměti, timeline inspekci vývoje znalostí a crash safety. Capsule kontexty jsou self-contained sdílené kapsle s pravidly a expirací. Smart Recall dává sub-5ms lokální přístup s prediktivním cache, time-travel debugging umožní rewind/replay/branch libovolného stavu.

### 9. NevaMind-AI/memU
memU ukládá osobní paměť jako sdílenou LLM wiki napříč sessiony, agenty i zařízeními; jádro logiky má jen ~500 řádků. Host adaptéry (sidecar binárky) čtou session logy daného agenta (jsonl/sqlite) a naplánovaný „bridging" task z nich krájí self-contained joby. Agent sám (ne externí LLM služba) z jobů destiluje vzpomínky a znovupoužitelné Markdown skills — rozhodne, zda nic nedělat, patchnout existující skill, nebo vytvořit nový. `commit` odešle změněné skill soubory zpět, memU embedduje jméno+popis a ukládá pod `skill` track; MemoryService sám nedělá žádné LLM cally. Retrieval funguje přes stálou instrukci v konfiguračním souboru hosta (AGENTS.md/CLAUDE.md), která říká agentovi spustit `retrieve` (progressive retrieve) před odpovědí.

### 10. semantica-agi/semantica
Semantica je deterministická graph-native vrstva pod LLM: z podnikových dat staví Context Graph + knowledge graph bez nutnosti LLM (LLM jen volitelné, vendor-neutral). Entity, vztahy a pravidla jsou explicitní přes ontologie a kontrolované slovníky (OWL, SHACL, SKOS), takže význam entit je strojově čitelný, ne jen embedding. Každý fakt nese W3C PROV-O provenance a rozhodnutí jsou first-class dotazovatelné objekty s kauzálními vazbami a audit trailem. Reasoning běží deterministicky — forward chaining, Rete síť, Datalog a SPARQL s vysvětlitelnými cestami. Konflikty faktů se detekují a flagují (ne tiše přepisují), entity resolution dělá blocking + semantickou deduplikaci a storage je polyglotní (RDF i LPG: Oxigraph, Neo4j, FalkorDB...), vše swapovatelné.

### 11. EverMind-AI/EverOS
EverOS je local-first memory runtime, jehož source of truth jsou čitelné Markdown soubory — editovatelné, diffable a git-versionable, cascade watcher synchronizuje změny do indexů. Nad Markdownem staví lokální trojici SQLite + LanceDB pro rychlé keyword/vektor vyhledávání, bez serverových závislostí. Paměť je rozdělená na user track (episodes, profile) a agent track (cases, skills) jako oddělené first-class plochy. Retrieval je ortogonální: filtruje se podle user_id, agent_id, app_id, project_id i session_id. Offline „reflection" mezi sessionami merguje clustery epizod a rafinuje profily a skills, takže se paměť sama vyvíjí; nad tím stojí editovatelná Knowledge Wiki s taxonomií vázaná na zdroje.

### 12. MemTensor/MemOS
MemOS je „Memory Operating System": jednotné API pro add/retrieve/edit/delete paměti, strukturované jako inspektovatelný a editovatelný graf, ne black-box embedding store. Nativně podporuje multi-modální paměť — text, obrázky, tool traces i personas se retrievalují a reasonují společně. Knowledge base se spravují jako komponovatelné memory cubes s izolací, kontrolovaným sdílením a dynamickou kompozicí napříč uživateli, projekty a agenty. MemScheduler provádí memory operace asynchronně s milisekundovou latencí pro produkční stabilitu. Paměť se rafinuje natural-language feedbackem (oprava/doplnění/nahrazení) a lokální pluginy pro agenty (OpenClaw, Hermes, DeepSeek Harness) jedou hybrid retrieval (FTS5 + vektor) se skill evolucí přes vrstvy L1 traces → L2 policies → L3 world models.

### 13. plastic-labs/honcho
Honcho běží ve smyčce Store → Reason → Query → Inject: zprávy/události/tool traces se ukládají na session a Honcho v pozadí asynchronně reasonuje a aktualizuje reprezentace. Model je peer-centric — uživatelé, agenti, skupiny, projekty i ideje jsou entity (peers), které se vyvíjejí v čase, a workspaces → peers → sessions → messages tvoří hierarchii. Místo chunk matchingu extrahuje z konverzací závěry (reasoning-first memory) a umí modelovat, co jeden peer ví o druhém (multi-peer perspective). Query vrací buď prompt-ready kontext (session.context s token budgetem), search výsledky, peer reprezentace, nebo natural-language odpověď přes chat endpoint. Výsledek se injectne do libovolného LLM callu/frameworku; běží managed, lokálně přes CLI, nebo self-hosted FastAPI.

### 14. getzep/zep
Toto repo není samotný produkt, ale examples + integrace k Zep Cloudu — managed memory platformě postavené na Graphiti (viz #4). Zep drží per-user context grafy ve vlastním proprietárním Context Graph Engine (miliony grafů, retrieval sub-200ms) bez externí graph databáze. Spravuje uživatele, thready a zprávy built-in a nad nimi skládá governed, nízkolatenční kontext pro produkční agenty. Integrace existují pro LangGraph, CrewAI, AutoGen, Google ADK, Mastra, Vercel AI SDK aj., plus pluginy pro Claude Code/Codex/Cursor. Repo obsahuje i ingestion pipeline (Slack, emaily, dokumenty, fact triples), default ontologii a benchmark harness (LoCoMo, LongMemEval).

### 15. FlowElement-xinliuyuansu/m_flow
M-flow ukládá znalosti do čtyřvrstvého „cone graphu": Episode → Facet → FacetPoint → Entity, od abstraktních souhrnů k atomickým faktům. Retrieval není similarity matching — dotaz nejdřív vektorově najde anchor na vrstvě odpovídající granularitě, pak graf přebírá: evidence se propaguje po typovaných, semanticky vážených hranách a každá jednotka se skóruje nejsilnější řetězcem důkazů k dotazu. Každý hop přidává cost, takže konkurenceschopné zůstávají jen koherentní levné cesty — asociace jako kontrolovaná propagace, ne náhodný graph walk. Výsledkem je „bundle" jedné Episode (s facets a facetpoints), ze kterého LLM skládá odpověď. Princip napodobuje lidské vzpomínání: jeden silný asociační řetězec stačí k vyvolání celé vzpomínky.

### 16. agentscope-ai/ReMe
ReMe ukládá trvanlivou paměť jako obyčejný Markdown s frontmatter a wikilinks — „Memory as File", inspektovatelné a editovatelné běžnými nástroji, indexy jsou rebuildovatelné. Konverzace a zdroje se postupně mění v daily notes a dlouhodobou znalost (fakta, preference, procedury, vztahy) se zachováním odkazů na zdroje — self-evolving KB. Recall kombinuje BM25, volitelné embeddingy a wikilink expanzi a vrací řádkově přesné pasáže bez naložení celé KB do kontextu. Jeden lokální workspace sdílí víc agentů (QwenPaw, DeepSeek Harness, OpenClaw, Hermes) přes SKILL.md, CLI, HTTP, MCP nebo Python API. Memory tags jsou file-native entity tagy s rebuildovatelným indexem pro tag-filtrované vyhledávání.

### 17. zilliztech/memsearch
memsearch dělá cross-platform sémantickou paměť pro coding agenty: plugin pro Claude Code/Codex/DSH/OpenClaw/OpenCode automaticky zachytává každý konverzační turn a konverzace v jednom agentovi je searchable ve všech ostatních. Source of truth jsou Markdown soubory (denní .md v .memsearch/memory/), Milvus je jen „shadow index" — odvozený, rebuildable cache. Retrieval je progresivní 3-vrstvý (search → expand → transcript) s hybridním vyhledáváním: dense vektory + BM25 sparse + RRF reranking, volitelně remote Jev rerank. Smart dedup přes SHA-256 content hashing přeskakuje nezměněný obsah a file watcher indexuje v reálném čase. Třetí vrstvou je procedurální paměť: opakované workflow se destilují do instalovatelných agent skills a background tasky udržují trvanlivé PROJECT.md a USER.md.

### 18. kayba-ai/agentic-context-engine (ACE)
ACE přidává agentům perzistentní learning loop místo vektorové paměti: udržuje „Skillbook" — evolvující kolekci strategií, co fungovalo a co ne. Tři role: Agent vykonává tasky obohacené o strategie, Reflector analyzuje execution traces, SkillManager kurátuje Skillbook (přidává, rafinuje, maže). Klíčový je Recursive Reflector — místo jednorázové sumarizace píše a spouští Python kód v sandboxu, programaticky hledá vzorce a izoluje chyby, dokud nenajde actionable insighty. Učení se děje z feedbacku a traces (learn_from_feedback), bez fine-tuningu, trénovacích dat nebo vektorové DB. Postaveno na PydanticAI se strukturovaným outputem, runnable přes LiteLLM na 100+ providerů; vychází z ACE paperu (Stanford/SambaNova) a Dynamic Cheatsheet.

### 19. tigerless-labs/agent-memory
Runtime kombinuje retrieval engine a filesystem v jednom store: Markdown soubory jsou single source of truth, SQLite FTS5 index vedle nich je smazatelná rebuildable cache. Recall nepastuje text do kontextu — vrací L0 seznam (jednořádkový abstrakt, path, anchor, score) a agent otevírá soubory v potřebné hloubce (abstract → outline → full → raw trace), každá úroveň o řád dražší. Zápisy nečekají na uvážení agenta: destilace se spouští na hranicích konverzace a full trace se nejdřív zkopíruje, takže „missed by distiller" neznamená ztraceno. Sleep-time pass konsoliduje a zapomíná podle hodnoty na vlastním hodinách — s authority tiers: bez dozoru smí přidávat/updatovat, delete vždy jen jako návrh ke schválení; `recall --as-of` odpovídá k datu, supersede nechává řetězec netknutý. Žádný LLM klient uvnitř knihovny — úsudek si půjčuje od host agenta (zero API keys), links ve frontmatter nesou graf bez graph databáze.

### 20. moorcheh-ai/memanto
Memanto není knihovna, ale druhý agent — „Memory Agent", který vedle fleetu spravuje vzpomínky ostatních agentů: co uchovat, co je v konfliktu, co expiruje a kdo to potřebuje vědět. Observuje interakční streamy agentů a tahá z nich trvanlivé znalosti (rozhodnutí, preference, fakta, failure), konsoliduje duplicity (opakovaná pozorování zesilují confidence místo množení řádků) a reconciluje kontradikce supersede — „co je pravda teď" a „čemu jsme věřili tehdy" zůstávají různé otázky (`recall --as-of`). Zapomínání je policy (decay, expiry, deliberate delete) vykonávaná scheduled loopem, ne cleanup na uživateli. Před akcí agenta ho Memanto „briefne" minimálním relevantním řezem estate — agenti nequeryjí, dostávají briefing. Estate je exportovatelný do Open Knowledge Format (plain Markdown), migratable z/do Mem0, Letta, Supermemory, Zep; běží lokálně (Docker + Ollama) bez účtu.

### 21. cortexkit/magic-context
Magic Context je „hippocampus" pro coding agenty: nahrazuje built-in compaction vlastním cache-aware řízením kontextu, takže session běží donekonečna bez compaction pauz. „Historian" komprimuje historii a přitom z ní zvedá trvanlivé znalosti (rozhodnutí, constrainty, konvence) do project memory — paměť vzniká zdarma z práce, kterou agent už dělá. Přes noc „dreamer" agenti konsolidují: verifikují vzpomínky proti codebase, kurátují duplicity a zastaralé záznamy a promují, co se opakuje. Relevantní vzpomínky se surfacují automaticky každý turn a agent umí on-demand hledat napříč vzpomínkami, minulými konverzacemi i git historií. Paměť se sdílí napříč OpenCode, Pi a OMP, takže jedna session žije týdny až roky.

### 22. doobidoo/mcp-memory-service
Self-hosted memory backend s jedním servisem a všemi transporty: REST API, MCP (včetně remote + OAuth), CLI a dashboard. Agenti ukládají rozhodnutí a kontext jako vzpomínky propojené knowledge grafem s typovanými hranami (causes, fixes, contradicts) — sdílejí kauzální řetězce, ne jen fakta. Embeddingy běží lokálně přes ONNX, takže data neopouštějí infrastrukturu a retrieval je ~5ms. `X-Agent-ID` header auto-tagguje vzpomínky identitou agenta pro scoped retrieval a SSE eventy notifikují ostatní agenty o zápisech v reálném čase. Autonomní konsolidace komprimuje staré vzpomínky, takže se store nezahlcuje; funguje s LangGraph, CrewAI, AutoGen i libovolným HTTP klientem.

### 23. supermemoryai/supermemory-mcp
MCP server nad Supermemory API, který zpřístupňuje jedny vzpomínky všem LLM klientům (ChatGPT, Claude, Cursor...) — „universal memory" bez loginu a zdarma. Princip je jednoduchý: vzpomínky žijí v Supermemory cloudu a MCP transport je doručí do jakéhokoliv MCP klienta. Sdílí principy hlavního enginu (viz #5): extrakce faktů, profily, kontradikce, zapomínání. Setup je jediný příkaz / .dxt one-click install. Repo je legacy v1 — vývoj se přesunul do hlavního monorepa supermemory/apps/mcp.

### 24. langchain-ai/langmem
LangMem dodává funkční primitivy, přes které agent aktivně spravuje vlastní dlouhodobou paměť „in the hot path": memory tools (manage + search) jako běžné nástroje agenta nad libovolným storage. Agent sám rozhoduje, co a kdy uložit a kdy hledat — žádné speciální příkazy, konzistence paměti drží LLM. Druhý režim je background memory manager, který mimo konverzaci automaticky extrahuje, konsoliduje a updatuje znalosti. Storage je agnostický (InMemoryStore pro dev, Postgres pro produkci) s nativní integrací do LangGraph Long-term Memory Store a sémantickým indexem přes embeddingy. Kromě záznamů faktů podporuje i optimalizaci chování agenta přes prompt refinement z interakcí.

### 25. BAI-LAB/MemoryOS
MemoryOS přebírá principy správy paměti z operačních systémů: hierarchická storage architektura se čtyřmi moduly — Storage, Updating, Retrieval, Generation. Paměť je dělená na short-term, mid-term a long-term (persona) vrstvy s automatickým updatem uživatelského profilu a znalostí. Architektura je plug-and-play: storage enginy, update strategie a retrieval algoritmy jsou zaměnitelné moduly. Přes MemoryOS-MCP server se modularní paměťové nástroje injectují do libovolných agent klientů (Claude Desktop atd.). Podporuje univerzální LLM (OpenAI, Deepseek, Qwen, R1 modely) a vektorové DB (Chroma), EMNLP 2025 paper, +49% F1 na LoCoMo.

### 26. Prismer-AI/PrismerCloud
Prismer je „intelligence runtime" — integrovaná harness vrstva (context, error recovery, memory, cross-session learning) podle Anthropic principů pro long-running agenty. Memory pilíř má 4 typy paměti s LLM recallem a auto-konsolidací. Klíčový je Evolution princip: agenti se učí z výsledků ostatních — errory se analyzují a mění ve strategie, fixy v doporučení sdílená napříč všemi agenty. Kolem toho stojí kontextová vrstva (web → komprimovaný LLM-ready obsah), community fórum pro agenty i lidi s karma, task marketplace s credit escrow a real-time messaging. Připojení přes SDK (TS/Python/Go/Rust), MCP server nebo pluginy pro Claude Code/OpenCode/OpenClaw.

### 27. ghostwright/phantom
Phantom není jen paměť, ale AI co-worker s vlastním počítačem (dedikovaný VM/Docker stack), takže kontext přežije zavření chatu — každá session není „den první". Paměť běží na Qdrantu (vektorová DB) + Ollama embedding modelu, obojí startuje s docker compose; agent si navíc sám buduje infrastrukturu (databáze, dashboardy, MCP nástroje), která funguje jako trvanlivá extenze jeho schopností napříč sessionami. Self-evolution pipeline měří zlepšování a „evolution judge" hodnotí změny — agent registruje vlastní výtvory jako MCP tools pro budoucí použití. Kanály (Slack, Telegram, email, web chat) drží kontinuální identitu a vzpomínky z minulých týdnů. Model je zaměnitelný (Anthropic default, Z.AI, OpenRouter, Ollama, vLLM...) — „tools jsou stejné, paměť je stejná, mění se jen mozek".

### 28. vshulcz/deja-vu
deja-vu indexuje session soubory, které coding agenti (Claude Code, Codex, Cursor +35 dalších) už ukládají na disk — měsíce historie zpětně, bez modelu, embeddingů a serveru, jedno Go binárko. Recall přichází automaticky při startu session, před editací souboru a po selhání příkazu — nikdo nemusí hledat. Indexuje práci, ne jen řeč: soubory otevřené v turnu, příkazy s exit statusy, přesné spany editů; secrets se stripují při indexaci. `deja promote --state rejected` označí revertnuté rozhodnutí a každý další hit řekne, že se to zkusilo a proč se zahodilo; hit také reportuje, když se dotčené soubory mezitím změnily. Jeden index čtou všech 38 harnessů, sync přes SSH mezi stroji, `handoff` pro pokračování v jiném agentovi; přežívá compaction tím, že vrací to, co sumarizace zahodila.

### 29. LycheeMem/LycheeMem
LycheeMemory je lightweight framework: strukturovaná organizace konverzační paměti, lehká konsolidace a adaptivní retrieval, směřující k action-aware/usage-aware paměti. Sémantická paměť je „Compact" varianta na SQLite + LanceDB (žádný Neo4j), doplněná transformer rerankerem pro výběr evidence. Pluginy (OpenClaw native, Claude Code přes MCP+hooks, Hermes runtime) dělají automatický recall, turn mirroring a konsolidační workflow; k dispozici je i OpenAI-compatible Chat Completions endpoint s request-level kontrolou konsolidace (`consolidate`/`store`). Visual (multimodální) memory modul rozšiřuje paměť na obrázky. Na PinchBench proti nativní paměti OpenClaw: +6% skóre, −71% tokenů, −55% ceny.

### 30. mindscale-noah/MindMemOS
MindMemOS je přenositelná self-evolving memory vrstva: uživatelské profily, preference, projektová fakta, tool zkušenosti a skill kandidáti persistují jako reusable assety sdílené mezi agenty (OpenClaw, Hermes, Claude Code, OpenHands). Self-evoluce běží přes schema learning (automatické učení častých paměťových vzorů, versioned v1/v2 flow), „dreaming" (offline konsolidace) a feedback z korekcí, který optimalizuje add/search workflow. Memory a Skills jsou integrované: experience vzpomínky se destilují do skill kandidátů a výsledky/failure traces/feedback ze skill execution tečou zpátky do paměti. Pluginy injectují relevantní vzpomínky před interakcí a po ní konverzaci automaticky zapíší. Deployment je cloud nebo self-host (FastAPI + Qdrant + Neo4j + Kafka, volitelně ClickHouse/OTel), přístup přes HTTP API, Python SDK/CLI nebo pluginy.
