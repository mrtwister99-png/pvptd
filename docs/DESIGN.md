# Meme TD 1v1 – design v kódu

> **Hlavní zdroj pravdy je finální GDD „Meme TD 1v1 – Finální GDD“ (verze 1.0, zamknuto).** Tento dokument popisuje,
> jak je GDD převedené do hry, co jsme k němu domluvili navíc a kde se kód od GDD zatím liší (sekce **7**).
> Když se tento dokument a GDD rozcházejí a nejde o domluvenou odchylku, platí GDD. Poslední úprava: **2026-11-28**.

## 1. Vize

Online **1v1 Line Tower Defence** v meme stylu: oba hráči dostanou stejnou vlnu ve stejný čas, staví průchozí bludiště,
posílají soupeři creepy za income a přetahují se o životy. Pod memy poctivá Legion TD matematika. **Žádné pay-to-win.**
Zápas trvá **20–30 min**: vyhrává, kdo dostane soupeře na 0 životů, nebo má víc životů po vlně 30.

## 2. Co je hotové v kódu (Bevy, Android + PC)

- menu: **HRÁT → VS BOT / TEST**, **BALÍČEK** (vždy 4 Basic + 2 Special)
- dvě pole 7 × 14, stavění bludiště, Hlídač Pepa hlídá průchodnost
- 15 Basic + 10 Special věží (GDD 7 a 8), ranky 1–7 / 1–3 jen **upgradem za gold** (věže se nespojují)
- **10 typů creepů** se schopnostmi (GDD 9), **vlny 1–30** z tabulky GDD 10, **6 bossů** (GDD 11)
- **10 posílaných jednotek** (GDD 4.4) s limity, tick income s capem (GDD 4.1), diamanty za bosse
- **5 damage typů** (GDD 5), životy a sudden death po vlně 30 (GDD 2)
- bot (staví, vylepšuje, posílá), testovací režim

## 3. Rozhodnuto

### A. Zápas, kola a životy (GDD 2)

- Vlna přijde na obě lajny ve stejný čas. Před 1. vlnou je **15 s na stavbu**, mezi vlnami pauza **7 s**.
- Vlna končí, když z obou lajn zmizí její creepové, nejpozději po 60 s (vlna 30 čeká, až dojdou všichni).
- **Životy**: start 15 : 15, celkem 30, max 30. Průchod: malý creep −1 / +1 soupeři, velký (elite) −2 / +2, **boss −3 / +3**.
- **Od vlny 26** projitý creep bere aspoň 2 životy, ale soupeři přidá o 1 méně (bere 2 → dá 1, boss bere 3 → dá 2).
- Vyhrává, kdo dostane soupeře na 0.
- **Po vlně 30**: víc životů vyhrává. Při shodě rozhoduje víc způsobených leaků, pak rychlejší zabití posledního bosse.
  Když je i to stejné, jede **sudden death**: vlny pokračují s +30 % HP za vlnu a **první průchod = konec**.

### B. Ekonomika (GDD 4.1–4.3)

- **Start 250 g** (GDD počáteční gold neuvádí; domluveno 2026-11-28).
- **Tick income každých 7 s**: base **+3**, k tomu income z poslaných jednotek; **max 40 za tick**. Běží pořád, i mezi vlnami.
- Income z poslané jednotky se poprvé vyplatí při **dalším ticku**.
- **Gold za zabití**: 2–4 podle typu creepa (Wojak, Zoomer, Brambora, Kopírovací Bot 2; Karen, Wi-Fi Upír, Influencer, AdBlock, Optimalizátor 3; Zombi, Chad 4), **boss 150**. Poslané jednotky nedávají nic.
- **Gold za přeživší vlnu** podle tabulky vln (45 g ve vlně 1 … 330 g ve vlně 25). Vzorec 25 + 3 × vlna z GDD 4.1 tabulce neodpovídá, platí tabulka.
- **Prodej věže**: **70 %** zaplaceného goldu před vlnou, **50 %** během vlny. Diamanty se nevrací. Prodej trvá 3 s.

### C. Diamanty (GDD 4.2, upřesněno 2026-11-28)

- Diamant dává **zabitý boss**. Když bosse propustíš, diamant dostane **soupeř**.
- **Postavit Special věž stojí gold + 1 diamant.** **Upgrade Special na R2 stojí gold + 1 diamant, na R3 další gold + 1 diamant.**
- Na obě Special věže až na R3 je potřeba 6 diamantů. Kdo už diamanty nepotřebuje, dostane za bosse místo diamantu **400 g**.

### D. Balíček a věže (GDD 6–8, 12)

- Do zápasu **vždy přesně 4 Basic a 2 Special** (obrazovka BALÍČEK jinak nepustí, žádná věž dvakrát). Bot losuje podle stejného pravidla.
- **Věže se nespojují.** Rank se kupuje **upgradem za gold** (tlačítko UPGRADE, na klávesnici Enter na věži); upgrade nemění bludiště.
- Ceny upgradu: Basic R2–R7 = 70 / 90 / 120 / 155 / 200 / 260 % ceny stavby, Special R2–R3 = 80 / 130 % (+ diamant, viz C).
- Násobky dmg / intervalu / dosahu podle tabulek GDD 6.1 a 6.2. Dosah se kvůli malé mapě 7 × 14 násobí 0,5 (`TOWER_RANGE_SCALE`).
- **Max 75 věží** na hráče. Během vlny nejde stavět na políčko, kde stojí creep.

### E. Damage typy (GDD 5)

| Typ | Silný (125 %) proti | Slabý (75 %) proti | Věže |
|---|---|---|---|
| Fyzické | rychlým (Zoomer) | obrněným (Karen, Chad, Optimalizátor, Daňový Úřad) | Nerfka, Babička, DJ, Pepa, Kasička, Tchyně, Gigachad, Doge, Trollface |
| Průrazné | obrněným a bossům | štítům (AdBlock, Máma) | Kaktus, Sniper |
| Magické | obrněným a regeneraci (Wi-Fi Upír) | magické odolnosti (Optimalizátor) | Router, Muchomůrka, Mrazák, Tesla, Daňový úřad, Shrek, Haxor, Karen, Černá díra |
| Výbušné | skupinám (splash) | bossům | Bramboromet, Toustovač, Katapult |
| Čisté | – | – | Chuck Norris, Admin Banhammer |

- Navíc: Zoomer dostává od Mrazáku +25 %, Tlustý Zombi od Toustovače +25 %.
- Zpomalení má společný limit **50 %**. Bossové mají **o 50 % kratší** slow a stun.
- Daňový úřad (věž) ruší štít; Karen (věž) umlčí schopnosti bosse.

### F. Creepové (GDD 9)

| Creep | Rychlost | Schopnost |
|---|---|---|
| Wojak | 1,0× | žádná |
| Zoomer Doomer | 1,55× | rychlý, slabý na led |
| Tlustý Zombi z Lidlu | 0,6× | tank, slabý na oheň |
| Karen Manažerka | 0,85× | obrněná (fyzické 75 %) |
| Wi-Fi Upír | 0,95× | když 2 s nedostane zásah, léčí 1,5 % HP/s |
| Cringe Influencer | 1,0× | aura +10 % rychlost okolním creepům |
| AdBlock Rytíř | 0,9× | štít 30 % HP |
| Kopírovací Bot | 1,0× | po smrti 2 kopie s 20 % HP |
| Lagující Brambora | 0,8–1,3× | každé 3 s náhodná rychlost |
| Chad Kurýr Elite | 1,15× | obrněný, pod 50 % HP +25 % rychlost |
| Daňový Optimalizátor | 0,9× | je v tabulce vln, v seznamu creepů chybí → obrněný + magická odolnost |

HP mají creepové přímo z tabulky vln (sloupec „HP mod“ z GDD 9 se u vln nepoužívá).

### G. Vlny a bossové (GDD 10 a 11)

- Vlny 1–30 přesně podle tabulky GDD 10 (počty, typy, HP, odměny). Vlny 29–30 jsou „smíšené“: střídá se všech 10 typů. Malí creepové ve vlně 30 jsou elite (leak 2).
- Bossové každou 5. vlnu:

| Vlna | Boss | HP | Co umí |
|---|---|---|---|
| 5 | Pan Učitel | 2 800 | každých 8 s na 2 s −35 % fyzického dmg; tabule: věže v okolí střílí o 15 % pomaleji |
| 10 | Šéf z Práce | 6 500 | pod 50 % HP +30 % rychlost; aura +20 % rychlost creepům |
| 15 | Tchyně | 13 500 | věže v okolí −15 % dmg |
| 20 | Daňový Úřad | 27 000 | obrněný; každých 7 s vypne 1 náhodnou věž v okolí na 2 s; když projde, sebere 100 g |
| 25 | Babička s Vařečkou | 52 000 | rychlá 1,3×; každých 5 s omráčí věže v okolí na 1 s |
| 30 | Tvoje Máma Volá | 100 000 | fáze 1: přivolává Kopírovací Boty; fáze 2: každých 8 s štít 15 % HP; fáze 3: +40 % rychlost, +20 % čistého dmg; každých 10 s vypne náhodnou věž na 3 s |

- Při příchodu bosse se dole ukáže jeho hláška.

### H. Posílané jednotky (GDD 4.4, upřesněno 2026-11-28)

- Posílá se **během vlny**, jednotka vyběhne **hned** na soupeřovu lajnu (domluveno, „příští vlna“ z GDD neplatí).
- **Max 12 jednotek za vlnu, max 2 Mini-Bossové za vlnu.** Poslané jednotky nedávají soupeři gold za zabití.
- HP = procento HP malého creepa vlny, která právě běží.

| Jednotka | Cena | Income / tick | HP vůči vlně | Leak | Vypadá jako |
|---|---|---|---|---|---|
| Minion z Chatu | 25 | +1 | 70 % | 1 | Wojak |
| Zoomer Express | 45 | +1 | 65 %, rychlý | 1 | Zoomer |
| Karen Balíček | 80 | +2 | 130 % | 1 | Karen |
| Wi-Fi Upír | 120 | +3 | 115 % | 1 | Wi-Fi Upír |
| AdBlock Rytíř | 165 | +4 | 120 % + štít | 2 | AdBlock |
| Kopírovací Bot | 220 | +5 | 110 % | 2 | Kopírovací Bot |
| Influencer Squad | 300 | +6 | 120 % | 2 | Influencer |
| Chad Kurýr | 400 | +8 | 150 % | 2 | Chad |
| Troll Tank | 550 | +10 | 200 % | 2 | Tlustý Zombi |
| Mini-Boss Ratio | 800 | +13 | 300 % | 3 | velký Chad |

### I. Hlídač Pepa (GDD 3)

- Když hráč zazdí cestu (z horní řady nevede cesta do cíle), Pepa se ozve („HELE, TADY TO NEJDE!“) a hráč má **10 s** na opravu.
- Když to neopraví, hra mu sama **prodá nejdražší blokující věž** za normální výkupní cenu.

### J. Režimy hry

| Režim | Co to je |
|---|---|
| **VS BOT** | zápas 1v1 proti botovi podle pravidel výše |
| **TEST** | hřiště: jen vlastní pole, všech 25 věží, neomezený gold i diamanty, životy se nepočítají, jednotky z panelu S kdykoli na vlastní pole bez limitu, VLNA +1 pošle další vlnu z tabulky, PRODAT = okamžitě ROZBÍT |

### Věže – katalog GDD (15 Basic + 10 Special)

| # | Věž | Typ | Co dělá |
|---|---|---|---|
| 1 | Bramboromet | výbušné | splash, na R7 15 % šance na stun 0,5 s |
| 2 | Nerfka | fyzické | rychlá střelba, +5 % rychlost za sousední Nerfku |
| 3 | Babička | fyzické | každý 6. zásah 2× dmg + stun 1 s |
| 4 | Router | magické | slow 12 %, na R7 disconnect 1,5 s |
| 5 | Kaktus | průrazné | snižuje armor, bleed |
| 6 | Toustovač | výbušné | burn 8 dmg/s, na R7 exploze po smrti |
| 7 | Muchomůrka | magické | jed se stacky, po 5 stackách halucinace |
| 8 | Mrazák | magické | slow a freeze |
| 9 | Tesla | magické | chain 3 cíle, na R7 5 |
| 10 | Sniper | průrazné | dlouhý dosah, kritika, poprava slabých |
| 11 | Katapult | výbušné | těžký splash |
| 12 | DJ | fyzické | knockback, každých 10 s DROP |
| 13 | Pepa z Hospody | fyzické | opilost, creep jde chvíli opačně |
| 14 | Kasička | fyzické | ekonomika, každých 5 vln výplata |
| 15 | Daňový úřad | magické | cíl dostává +8 % dmg, ruší štít a heal |
| 16 | Chuck Norris | čisté | poprava pod 30 % HP, boss −20 % max HP |
| 17 | Tchyně | fyzické | kritika, −20 % max HP, stun bosse |
| 18 | Shrek Bažina | magické | slow 60 %, vrací creepy na start |
| 19 | Gigachad Gym | fyzické | aura +dmg a rychlost věžím |
| 20 | Doge Miner | fyzické | gold za vlny, To The Moon |
| 21 | Trollface | fyzické | teleport dozadu, prohození |
| 22 | Haxor | magické | hack, DDOS |
| 23 | Karen Manažerka | magické | zpomalí a umlčí bosse |
| 24 | Černá díra do Brna | magické | vtahuje, vrací creepa soupeři |
| 25 | Admin Banhammer | čisté | každý 5. zásah ubere část HP |

Parametry věží jsou v `src/tower_def.rs`, popisy schopností v `src/special.rs`. Damage typy Special věží GDD neurčuje (kromě čistého dmg u specialek), přiřadili jsme je podle schopnosti.

## 4. Otevřené otázky

- **Obtížnost na malé mapě**: GDD počítá s hadovkou 16 × 12, my máme 7 × 14 a poloviční dosah. Ověřit hraním, jestli jsou vlny z GDD zvládnutelné (viz PLAN, simulace).
- Délka zápasu (cíl 20–30 min), kdy hráči získají první diamant, které věže se nepoužívají (GDD 15).
- Tick 7 s není moc hektický? (Přepnutí na 10 s = konstanta `INCOME_TICK_SECS`.)
- Karty: +1,5 % dmg za level sbírky (max +13,5 %), matchmaking podle průměru 6 karet – zatím neimplementováno.

## 5. Pořadí práce (GDD 16)

1. ~~M0/M1 core v Bevy: věže, creepové, vlny 1–30, bossové, diamant, draft 4 + 2, bot~~ (hotovo, ladí se)
2. Ověřit balanc hraním, doladit.
3. M2: meme vizuál, zvuky, kill feed, Trollface při leaku.
4. M3: online (autoritativní server, Rust core + Godot klient podle GDD 14), ranked.
5. M4: karty, truhly, eventy.

## 6. Technické poznámky

- Všechna data a pravidla z GDD jsou v `src/wave_def.rs` (creepové, bossové, vlny, posílané jednotky, ekonomika, životy, damage typy, diamanty) – bez kreslení, aby šla později přesunout do Rust core.
- Spawn creepů, jejich schopnosti, bossové, Pepa, ekonomika kola: `src/systems/creep.rs`. Zásahy a damage typy: `src/systems/tower_combat.rs` a `src/systems/debuff.rs`.
- Stavba, upgrade, prodej, limity: `src/systems/tower_build.rs`. Bot: `src/systems/bot.rs`. Balíček: `src/deck.rs`.
- Režim zápasu: `GameMode` v `src/resources/mod.rs`, statistiky zápasu (limity posílání, leaky, sudden death): `MatchStats`.

## 7. Rozdíly kódu proti GDD

Domluvené odchylky (2026-11-28):

| Oblast | GDD | V kódu | Proč |
|---|---|---|---|
| Mapa | hadovka 16 × 12 | bludiště 7 × 14, dosah věží × 0,5 | domluveno: mapa zatím zůstává |
| Poslané jednotky | vyběhnou v příští vlně | vyběhnou hned do běžící vlny | domluveno |
| Diamanty | 1. a 2. diamant odemyká Special, upgrade diamant nestojí | stavba i každý upgrade Special = 1 diamant, nepotřebný diamant = 400 g | domluveno |
| Technika | Rust core + server + Godot | celé v Bevy, data GDD odděleně ve `wave_def.rs` | domluveno |
| Odměna za vlnu | vzorec 25 + 3 × vlna i tabulka | tabulka | rozpor v GDD |
| Start gold | neuvádí | 250 g | v GDD chybí |
| Daňový Optimalizátor | ve vlnách, v seznamu creepů chybí | obrněný + magická odolnost | v GDD chybí |

Co z GDD v kódu ještě není:

- Hlídač Pepa nechodí po cestě jako postavička a cesta při zazdění nezčervená (jen hláška a odpočet).
- Bossové nemají vizuální tabuli / vařečku, Šéfa z Práce jde teleportovat bez omezení 8 s.
- DoT stack max 3× (Muchomůrka má podle GDD 7 až 5 stacků – nechali jsme 5).
- Kill feed, meme zvuky, Trollface při leaku, emote (GDD 13).
- Karty a matchmaking (GDD 6.3 a 12), online (GDD 14).
