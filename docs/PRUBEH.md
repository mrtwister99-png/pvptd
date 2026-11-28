# Průběh projektu

## Finální datum

- Doplní se po dokončení projektu.

## Zápisy průběhu

### 2026-11-23 – Začátek

- Projekt spuštěn. Další kroky a změny se budou doplňovat sem podle domluvy.

### 2026-11-28 – Testovací režim

- Po HRAT se vybírá režim: VS BOT nebo TEST.
- TEST: jen vlastní hrací pole, všech 25 věží, neomezený gold i diamanty, životy se nepočítají,
  creepi z panelu S jdou kdykoli na vlastní lajnu, tlačítko VLNA +1 pošle automatickou vlnu,
  PRODAT = okamžitě ROZBÍT. Rank se zkouší přes UPGRADE (gold je neomezený).
- Opraven pád u věže DJ (zkratka názvu se 2 písmeny).

### 2026-11-28 – Vyvážení ke 30. vlně, 34 jednotek, balíček

- Zjištění: bot proti botovi skončil v 8. kole. Vlny rostly mnohem rychleji než gold
  (vlna 30 měla creepy po ~334 000 HP), vlny 16–30 se počítaly jako HP jednotky × 2,5 × škála × pojistka.
- Nová křivka: HP vlny ≈ 1× (vlna 20) až 2,2× (vlna 30) goldu ve věžích podle modelu ekonomiky.
  Vlny 7–15 nižší a plynulé, vlny 16–34 = HP jednotky × 0,25 × škála, pojistka až po 30. kole.
- Jednotky: HP za gold roste s cenou (2,0 → 3,8); dřív byly prostřední jednotky horší než DAVID.
  Nové jednotky 31–34: TITAN, BANSHEE, HIEROPHANT, OVERLORD. Panel S je seřazený podle ceny.
- Balíček: funkce `deck_is_complete` + testy (vždy 4 Basic + 2 Special, žádná věž dvakrát).
- Bot od 3. kola investuje do income i bez plného bludiště; opraven pád bota bez goldu na věž.
- Simulace bot vs bot (`SIM_ENDLESS=1 GAME_SPEED=5`): zápas běžel do 51. kola bez pádu, income ve 20. kole
  100–220 / 7 s, věže 3–5 tisíc g. Boti ale propouštěli creepy už od 1. kola (tehdy si ještě rozbíjeli bludiště spojováním,
  při 5× rychlosti se asi míjí střely), takže skutečnou obtížnost 15.–30. kola je potřeba ověřit hraním.
- Dokumentace přepsaná (DESIGN, PLAN, nové README).

### 2026-11-28 – Finální GDD, konec spojování věží

- Platí finální GDD „Meme TD 1v1“ (verze 1.0). Podle něj se věže **nespojují**, rank se kupuje jen upgradem za gold
  (proto balíček 4 Basic + 2 Special).
- Odstraněno spojování: tah věže na věž, nesení věže Enterem i duch přetahované věže. Enter na věži teď věž vylepší.
- Bot místo spojování vylepšuje věže za gold (nejnižší rank první).
- Dokumentace: GDD je hlavní zdroj pravdy, do DESIGN.md přibyl seznam rozdílů kódu proti GDD a do PLAN.md krok
  „převod na finální GDD“. Dnešní přeladění vln a 34 jednotek GDD nedodržuje a při převodu se nahradí.

### 2026-11-28 – Převod na finální GDD

- Data GDD v `src/wave_def.rs`: 10 typů creepů, 6 bossů, tabulka vln 1–30, 10 posílaných jednotek, ekonomika,
  životy, damage typy a diamanty. Staré tabulky (34 jednotek z WC3 mapy, vlastní křivka vln) jsou pryč.
- Ekonomika: +3 / tick (max 40), kill gold 2–4, boss 150, gold za přeživší vlnu, prodej 70 / 50 %.
- Diamanty (domluveno): zabitý boss = diamant, propuštěný = diamant soupeři; stavba i upgrade Special = 1 diamant;
  kdo diamanty nepotřebuje, dostane 400 g.
- Životy: boss 3, od vlny 26 leak −2 / +1, po vlně 30 rozhodují životy → leaky → čas zabití bosse → sudden death.
- Creepové se schopnostmi (štít, regenerace, kopie, aura rychlosti, náhodná rychlost, enrage), bossové se schopnostmi
  (fyzický štít, aura rychlosti, debuff věží, vypínání věží, omráčení, tři fáze Mámy).
- 5 damage typů (125 / 75 %), slow cap 50 %, bossové mají poloviční CC.
- Hlídač Pepa: 10 s na opravu zazděné cesty, pak prodá nejdražší blokující věž. Max 75 věží, nestaví se pod creepy.
- Domluvené odchylky od GDD: mapa zůstává 7 × 14, poslané jednotky vyběhnou hned, technika zůstává Bevy.
