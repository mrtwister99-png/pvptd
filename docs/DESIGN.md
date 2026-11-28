# PvP TD – design dokument

> **Hlavní zdroj pravdy je finální GDD „Meme TD 1v1 – Finální GDD“ (verze 1.0, zamknuto).** Tento dokument popisuje,
> co z GDD je v kódu, jak to ve hře teď funguje a kde se kód od GDD ještě liší (sekce **8. Rozdíly kódu proti GDD**).
> Když se tento dokument a GDD rozcházejí, platí GDD. Poslední úprava: **2026-11-28**.

## 1. Vize (jednou větou)

Rychlá a pestrá mobilní **1v1 PvP tower defense**. Kořeny má v Line Tower Wars (posílání jednotek soupeři + income), progres a zápasy jsou ve stylu moderních mobilních karetních TD. **Žádné pay-to-win**: za peníze jen kosmetika, rozhoduje skill.

## 2. Odkud vycházíme

- **Původní inspirace**: WC3 custom mapa (Line Tower Wars), 7+ hráčů na životy, nekonečné vlny, income tick každých 10 s, posílání jednotek dalšímu hráči v kruhu.
- **Co už je hotové v kódu (Bevy, Android + PC)**:
  - menu: **HRÁT → VS BOT / TEST**, **BALÍČEK** (výběr 4 Basic + 2 Special)
  - dvě pole 7×14, stavění a cesta creepů kolem věží; soupeřem je bot
  - katalog 15 Basic + 10 Special věží; ve VS BOT se staví jen z balíčku
  - výběr konkrétní věže pro stavbu, placené upgrady za gold a prodej (věže se nespojují)
  - Basic ranky 1–7, Special ranky 1–3; stavba Special věže stojí diamant
  - kola, bossové, **34 posílaných jednotek**, gold/income a efektové systémy
  - **testovací režim** (hřiště) pro zkoušení věží a jednotek
- **Rozdělané / chybí**: online PvP, účty/progres karet, žebříček, ověření vyvážení hraním na telefonu.

## 3. Nová koncepce – hlavní pilíře

1. **1v1** – dva hráči, každý má svou lajnu, posílají si jednotky navzájem.
2. **Kola** – zápas je rozdělený na kola; během kola se posílají jednotky.
3. **Žebříček** – hodnocení (rating / ligy), matchmaking podle síly.
4. **Karty** – hráč sbírá karty (věže i jednotky) a vylepšuje je. Musí to jít nasbírat **zdarma**.
5. **Monetizace** – jen skiny/kosmetika. Žádné kupování síly.
6. **Hratelné až ke 30. vlně** – dobrý hráč se dostane ke 30. vlně, pak pojistka zápas rychle ukončí.

## 4. Rozhodnuto

### A. Zápas a kola (2026-11-10, upraveno 2026-11-28)

- **Délka zápasu**: na začátku hry (málo karet) cca **4–5 min**, se silnějšími kartami se prodlužuje. Vlny jsou laděné tak, aby šlo dojít **až ke 30. vlně**.
- **Kolo** = jedna automatická vlna: hromada normálních creepů + **jeden větší** creep. Každé kolo je vlna silnější.
- **Každé 5. kolo** přijde navíc **boss**.
- **Posílání**: hráč posílá jednotky soupeři **během vlny**. Po vlně je krátká pauza, pak začne další kolo a posílá se znovu.
- **Automatické vlny chodí vždy** a k nim navíc to, co pošle soupeř.
- **Životy jako přetahovaná**: dohromady je **30 životů**, oba hráči začínají s **15**. Když mi projde creep, já ztratím 1 a soupeř získá 1. Prohrává ten, kdo má 0, takže vítěz má všech 30.
- **Cena průniku**: normální creep **1** život, větší creep **2**, boss **4** (konstanta, jde změnit).
- **Matchmaking hlavně podle síly balíčku** (úrovně karet), aby proti sobě hráli podobně silní.
- **Pojistka**: **po 30. kole** roste HP vln o 15 % za kolo navíc, takže zápas krátce po 30. vlně skončí.
- **Časování**: posílat se dá **celou dobu kola**, mezi koly je pauza **7 s**.
- **Stavět a vylepšovat věže se dá kdykoliv**, i během vlny.

### B. Ekonomika (2026-11-20)

- **Start**: **250 g** (stačí na čtyři nejlevnější Basic věže za 40+50+70+80 g), income **20**.
- **Income tick každých 7 s**: dostaneš tolik goldu, kolik máš income.
- **Poslaná jednotka trvale zvedne income** (např. DAVID za 5 g → +1). Pro každou jednotku zvlášť platí pásma: 1.–50. odeslání 100 %, do 100. 80 %, do 200. 60 %, do 300. 35 %, do 500. 15 %, pak nic.
- **Gold za zabití creepa**: normální 15, větší 40, boss 80, mega boss 150 (poslané jednotky nic).
- Síla věže = úroveň karty × rank v zápase.

### C. Karty, balíček a vylepšování věží (2026-11-20, upraveno 2026-11-28 podle GDD)

- **Karty jsou věže i jednotky.** Získávají se po zápase z **truhel** (a dalších odměn).
- **Balíček**: do zápasu si hráč vybírá **vždy přesně 4 Basic a 2 Special věže** z katalogu 15 + 10. Obrazovka BALÍČEK víc nepustí, tlačítko HOTOVO funguje jen u úplného balíčku a žádná věž nesmí být dvakrát. Bot si losuje balíček podle stejného pravidla.
- **Úroveň karty zvedá všechny statistiky** dané věže nebo jednotky. U jednotek to znamená silnější posílané creepy.
- **Stavění v zápase**: hráč zvolí typ ze svého balíčku a postaví právě tuto věž na vybrané prázdné políčko. Special věž při stavbě spotřebuje také jeden diamant.
- **Vylepšování**: Basic věže se vylepšují za gold až na rank 7, Special až na rank 3. Ceny a statistiky jsou definované podle GDD.
- **Věže se nespojují.** Jediná cesta k vyššímu ranku je **UPGRADE za gold** (tlačítko UPGRADE, na klávesnici Enter na věži). Proto má balíček pevně 4 Basic + 2 Special.
- **Ceny upgradu (GDD 6.1 a 6.2)**: Basic R2–R7 stojí 70 / 90 / 120 / 155 / 200 / 260 % ceny stavby, Special R2–R3 80 / 130 % základní ceny. Násobky dmg, intervalu a dosahu jsou podle tabulek GDD.
- **Max rank** je 7 pro Basic a 3 pro Special.
- **Upgrade nemění bludiště** – věž zůstane na svém políčku.
- **Prodej věže**: teď vrací 50 % všeho, co do věže šlo (stavba + upgrady). GDD chce 70 % před vlnou a 50 % během vlny (viz sekce 8).
- **Jednotky**: posílat se dá **neomezeně, dokud máš gold**. V zápase jsou **odemčené všechny jednotky**. Sílu jim zvedají **jen karty** (z truhel), během zápasu se jednotky nevylepšují.

### D. Soupeř a obrazovka (2026-11-20)

- **Začínáme 1v1 proti botovi** v telefonu (bez serveru). Online proti lidem až potom.
- **Přepínání mezi lajnami**: hráč vidí svoje bludiště a tlačítkem (nebo Tab) se přepne na soupeřovo.
- **Bot** staví bludiště, vylepšuje věže za gold (nejdřív ty s nejnižším rankem) a během kola posílá jednotky. Část goldu dává do posílání i na začátku, aby mu rostl income.

### E. Režimy hry (2026-11-28)

Po stisku **HRÁT** se vybírá režim:

| Režim | Co to je |
|---|---|
| **VS BOT** | normální zápas 1v1 proti botovi podle pravidel výše |
| **TEST** | hřiště na zkoušení, nic se nepočítá |

**TEST**:
- je jen **vlastní hrací pole**, bot nehraje a přepínání na soupeře chybí,
- v nabídce stavění je **všech 25 věží** (Basic i Special), balíček se neuplatní,
- **gold i diamanty jsou neomezené**, takže jde přes **UPGRADE** zkusit **jakýkoli rank**,
- **životy se nepočítají**, prohrát nejde,
- jednotky z panelu **S** jdou **kdykoli na vlastní pole**, aby šly zkoušet proti vlastním věžím,
- tlačítko **VLNA +1** pošle na vlastní pole další automatickou vlnu (kolo +1),
- **PRODAT** se jmenuje **ROZBÍT** a věž zmizí hned, bez odpočtu.

### F. Vlny a posílané jednotky – vyvážení ke 30. vlně (2026-11-28)

**Problém**: vlny rostly mnohem rychleji než ekonomika. Vlna 30 měla creepy po ~334 000 HP (celkem 5 mil.), přitom hráč má v obraně řádově desítky tisíc goldu. Bot proti botovi skončil v 8. kole.

**Pravidlo ladění**: HP celé vlny má odpovídat tomu, kolik má hráč goldu ve věžích (podle modelu ekonomiky, kde hráč posílá ~25 % goldu za income):

| Vlna | HP vlny / gold ve věžích | HP normálního creepa (předtím) |
|---|---|---|
| 1–6 | 0,3–0,7 (beze změny) | 21–227 |
| 10 | ~0,7 | 425 (1 020) |
| 15 | ~0,9 | 861 (5 125) |
| 20 | ~1,1 | ~1 700 (12 100) |
| 25 | ~1,5 | ~3 700 (66 000) |
| 30 | ~2,2 | ~7 900 (334 000) |
| 31+ | pojistka +15 %/kolo | rychle roste |

- **Vlny 1–15** mají vlastní tabulku (`get_wave_def`); vlny 7–15 jsou nově nižší a HP roste plynule (vlna 8 je dál *sudden death* s dvojnásobným HP).
- **Vlny 16–34** jsou podle posílaných jednotek: vlna N = 15 creepů jednotky N, HP = HP jednotky × **0,25** × škála vlny (max 2,2). Každá 5. vlna má mega bosse (3× HP creepa).
- **Vlna 35+** navazuje na vlnu 34: +12 % HP a +1 creep za 2 vlny (max 40), k tomu pojistka.

**Posílané jednotky**: HP za gold teď **roste s cenou**, od **2,0** (DAVID až ROGUE, tj. jednotky do 60 g) po **3,8** (OVERLORD). Dřív měly prostřední jednotky (LUKAS…SORCERESS) jen 1,3–1,5 HP/g, tedy méně HP **i** méně income za gold než DAVID, a vyplatilo se jen spamovat nejlevnější. Drahé jednotky dávají méně income za gold, a proto mají víc HP.

| # | Jednotka | Cena | HP | Income | Zvláštnost |
|---|---|---|---|---|---|
| 1 | DAVID | 5 | 10 | +1 | |
| 2 | DAMIAN | 10 | 20 | +2 | |
| 3 | OLGA | 15 | 30 | +2 | |
| 4 | KAMIL | 25 | 50 | +3 | |
| 5 | LUKAS | 40 | 80 | +4 | |
| 6 | ROGUE | 60 | 120 | +5 | rychlý 120 % |
| 7 | BERSERK | 85 | 180 | +6 | enrage pod 30 % HP |
| 8 | ARCHER | 110 | 250 | +7 | střílí zpět, zpomalí věž |
| 9 | WYVERN | 140 | 330 | +8 | létá |
| 10 | OGRE | 180 | 440 | +9 | resist push/slow |
| 11 | GRYPHON | 220 | 550 | +10 | létá, rychlý 130 % |
| 12 | PRIEST | 270 | 690 | +11 | léčí 8 HP/s okolí |
| 13 | CAPTAIN | 330 | 870 | +12 | aura +15 % max HP |
| 14 | SORCERESS | 400 | 1 100 | +13 | Frost/Fire jen 50 % |
| 15 | BOSS | 500 | 1 400 | +15 | immune stun, enrage |
| 16 | ENCHANT | 600 | 1 750 | +16 | |
| 17 | NECRO | 700 | 2 050 | +17 | |
| 18 | WYRM | 800 | 2 400 | +18 | létá |
| 19 | INFERNAL | 900 | 2 750 | +19 | aura: náhodný cíl věží |
| 20 | BOSS 2 | 1 000 | 3 100 | +20 | immune stun, resisty |
| 21 | CHIMERA | 1 200 | 3 750 | +22 | létá, rychlá |
| 22 | DOOM | 1 400 | 4 500 | +24 | |
| 23 | LICH | 1 600 | 5 200 | +26 | |
| 24 | PITLORD | 1 800 | 5 900 | +28 | stomp, 100 % push resist |
| 25 | BOSS 3 | 2 000 | 6 700 | +30 | resisty, enrage |
| 26 | PHOENIX | 2 300 | 7 800 | +32 | létá, aura −25 % dmg |
| 27 | KNIGHT | 2 600 | 8 900 | +34 | aura −25 % dosah |
| 28 | ARCHLICH | 3 000 | 10 500 | +36 | |
| 29 | VOID | 3 400 | 12 000 | +38 | létá |
| 30 | FINAL | 4 000 | 14 400 | +40 | mega aura, resisty, enrage |
| 31 | **TITAN** (nová) | 4 600 | 16 800 | +42 | pomalý 85 %, immune stun, 80 % push / 50 % slow resist |
| 32 | **BANSHEE** (nová) | 5 300 | 19 600 | +44 | létá, rychlá 125 %, Frost/Fire jen 50 % |
| 33 | **HIEROPHANT** (nová) | 6 100 | 22 900 | +46 | léčí 60 HP/s okolí |
| 34 | **OVERLORD** (nová) | 7 000 | 26 700 | +48 | aura +15 % max HP, enrage pod 40 % |

Nové jednotky 31–34 mají čísla 31–34, aby se nerozbily stávající schopnosti navázané na čísla jednotek. V panelu **S** jsou jednotky seřazené **podle ceny**. Zvláštní schopnosti jednotek 16, 17, 22, 23, 28 jsou zatím jen v plánu (mají jen staty).

### Věže – katalog GDD (15 Basic + 10 Special)

| # | Věž | Co dělá |
|---|---|---|
| 1 | Bramboromet | splash a šance na stun na R7 |
| 2 | Nerfka | rychlá střelba, zesílení od sousedních Nerfek |
| 3 | Babička | každý šestý zásah je kritický a omráčí |
| 4 | Router | zpomalení, na R7 krátký stun |
| 5 | Kaktus | kletba a bleed |
| 6 | Toustovač | burn a exploze při smrti creepa na R7 |
| 7 | Muchomůrka | jed, který se skládá do stacků |
| 8 | Mrazák | zpomalení a možnost zmrazení |
| 9 | Tesla | řetězový zásah, na R7 zasáhne více cílů |
| 10 | Sniper | dlouhý dosah, kritický zásah a poprava slabých creepů |
| 11 | Katapult | vysoké plošné poškození |
| 12 | DJ | knockback a pravidelný plošný DROP |
| 13 | Pepa | opilost může změnit směr creepa |
| 14 | Kasička | gold za kola a bonus každých pět kol |
| 15 | Daňový úřad | kletba zvyšující utržené poškození |
| 16 | Chuck Norris | true damage a poprava, bonus proti bossům |
| 17 | Tchýně | kritické zásahy, snížení max HP a stun bosse |
| 18 | Shrek Bažina | zpomalení a návrat creepů na start jednou za kolo |
| 19 | Gigachad Gym | aura zvyšující poškození a rychlost okolních věží |
| 20 | Doge Miner | gold za kola a jednorázové zdvojnásobení goldů |
| 21 | Trollface | knockback, na R3 prohodí creepa s nejvyšším a nejnižším HP |
| 22 | Haxor | šance na hack, na R3 plošné omráčení jednou za kolo |
| 23 | Karen Manažerka | zpomalení, umlčení a snížení max HP bosse |
| 24 | Černá díra do Brna | přitahování; na R3 šance vrátit creepa soupeři s dvojnásobným HP |
| 25 | Admin Banhammer | každý pátý zásah ubere část aktuálního HP |

Parametry věží jsou v `src/tower_def.rs`; popisy schopností jsou v `src/special.rs`. Některé schopnosti jsou v současné implementaci zjednodušené a je třeba je doladit při herním testování.

## 5. Otevřené otázky

### A. Zápas a kola – doladit čísla při testování
- Ověřit hraním na telefonu, že dobrý hráč opravdu dojde ke 30. vlně a že zápas trvá rozumně dlouho (30 kol × ~40 s ≈ 20 min, to je víc než původní cíl 15 min).
- Délka jednoho kola (kolik creepů, rozestupy).
- Přesná cena průniku bosse (3, 4, nebo 5).

### B. Ekonomika – doladit čísla při testování
- Model ekonomiky počítá s tím, že hráč posílá ~25 % goldu. Když lidé posílají víc nebo méně, posune se obtížnost vln.
- Gold za zabití podle typu creepa, ceny jednotek 31–34.

### C. Karty a vylepšování – doladit
- Úrovně karet dávají sílu a férovost řeší matchmaking podle síly balíčku. Proto musí být vylepšování **jen za věci získané hraním, nikdy za peníze**.
- Jsou v zápase **vylepšení celého typu věže** (zaplatíš a všechny věže daného typu jsou silnější), nebo jen stavění a upgrady jednotlivých věží?
- Jak velký rozdíl mezi úrovní karty 1 a max? (Malý = víc skillu, velký = víc progresu.)

### D. Multiplayer / technika
- Server: vlastní (Rust), nebo služba? Účty, ukládání progresu, ochrana proti podvádění.
- Jak se ukáže soupeřova lajna na malém displeji (minimapa, přepínání swipem)?

### E. Obsah a vzhled
- Jména a grafika jednotek jsou zatím z WC3 mapy (Sheep, Frost Wolf…) – před vydáním je dát **vlastní**. Jednotky 16–34 zatím nemají obrázek (barevné kolečko).
- Styl grafiky / skinů.

## 6. Navržené pořadí práce

1. ~~Doladit tento dokument (sekce A → C → B → D).~~
2. ~~Udělat **1v1 proti botovi** lokálně~~ + testovací režim. Teď: ověřit vyvážení ke 30. vlně hraním.
3. Karty a balíček, uložené lokálně.
4. Online 1v1 + účty + žebříček.
5. Kosmetika, obchod, vydání.

## 7. Technické poznámky

- Vlny: `src/wave_def.rs` (`get_wave_def`, `unit_wave`, `WAVE_HP_SCALE`), škála vln a pojistka v `src/constants.rs` (`CREEP_GROWTH_*`, `LATE_ROUND*`). Testy v `wave_def.rs` hlídají, že vlny rostou a vlna 30 zůstává v hratelném rozsahu.
- Jednotky: `get_unit_def` v `src/wave_def.rs`; schopnosti v tabulkách `is_air`, `resist_for`, `enrage_for`, `heal_for`, `aura_for`, `tower_aura_for`, `shimmer_for`, `shoot_back_for`.
- Režim zápasu: zdroj `GameMode` v `src/resources/mod.rs`, výběr v `src/menu.rs`, chování testu v `test_mode_system` (`src/systems/creep.rs`).
- Balíček: `deck_is_complete` v `src/deck.rs`.

## 8. Rozdíly kódu proti GDD

Co GDD chce a v kódu zatím není nebo je jinak. Tohle je seznam práce, ne rozhodnutí – platí GDD.

| Oblast | GDD | Teď v kódu |
|---|---|---|
| Spojování věží | není | **odstraněno** (2026-11-28) ✅ |
| Balíček | 4 Basic + 2 Special | stejně ✅ |
| Ranky a ceny upgradů | R1–7 / R1–3, tabulky 6.1 a 6.2 | stejně ✅ |
| Mapa | hadovka 16 × 12, stavba jen vedle cesty, max 75 věží | volné bludiště 7 × 14 |
| Hlídač Pepa | BFS kontrola, 10 s na opravu, pak prodá nejdražší blokující věž | Hlídač boří věž při úplném zablokování |
| Životy | boss leak 3, od vlny 26 leak bere 2 a soupeři dá 1, sudden death po vlně 30 | boss 4, bez sudden death |
| Income | +3 / tick 7 s, cap 40 / tick, send income až od dalšího ticku | +20 / tick, bez capu |
| Gold za zabití | 2–4 za creepa, boss 150, bonus za přežitou vlnu 25 + 3 × vlna | 15 / 40 / 80 / 150, bez bonusu |
| Diamant | jen za zabitého bosse (leak = soupeři), 1. a 2. diamant odemyká Special věže, další = 400 g | za zabitého bosse, každá Special stavba stojí diamant |
| Prodej | 70 % před vlnou, 50 % během vlny, Special diamant nevrací | 50 % vždy |
| Posílané jednotky | **10 jednotek** (Minion z Chatu 25 g … Mini-Boss Ratio 800 g), HP v % vlny, max 12 za vlnu, max 2 Mini-Bossové | 34 jednotek z WC3 mapy |
| Creepové | 10 meme typů (Wojak, Zoomer Doomer, Tlustý Zombi…) se schopnostmi | jednotky z WC3 mapy |
| Vlny 1–30 | pevná tabulka (např. vlna 1: 18× Wojak 75 HP, vlna 30: 40× elite 3 200 HP + boss Tvoje Máma Volá 100 000 HP) | vlastní křivka z 2026-11-28 (viz F) |
| Bossové | 6 bossů s fázemi a debuffy věží (Pan Učitel, Šéf z Práce, Tchyně, Daňový Úřad, Babička, Máma) | obecný boss |
| Damage typy | 5 typů (fyzické, průrazné, magické, výbušné, čisté), 125 / 75 % | jen dílčí odolnosti |
| Délka zápasu | 20–30 min | podle vyvážení |
| Technika | Rust core + autoritativní server + Godot klient | celé v Bevy |

**Pozor:** přeladění vln a jednotek ze sekce F a nové jednotky 31–34 vznikly před přečtením GDD. GDD má vlastní tabulku vln 1–30 a 10 posílaných jednotek, takže se při převodu na GDD nahradí.
