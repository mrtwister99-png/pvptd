# Plán implementace – 1v1 proti botovi

Cíl první etapy: **hratelný zápas 1v1 proti botovi v telefonu** podle finálního GDD (Meme TD 1v1, verze 1.0) a [DESIGN.md](DESIGN.md), hratelný až ke 30. vlně.
Co z GDD ještě chybí, je v DESIGN.md, sekce **7. Rozdíly kódu proti GDD**.
Karty, truhly a online přijdou až potom. Každý krok končí tím, že hra jde spustit a vyzkoušet.

## Hlavní technická změna (hotovo)

Hra má **dvě lajny** a stav každého hráče je zvlášť:

- komponenta `Side` (`Player` / `Bot`) na věžích, creepech, střelách a hlídači
- stav hráče na jednom místě: `Sides([SideState; 2])`, kde `SideState` obsahuje gold, diamanty, income, životy, grid, cestu, počet staveb a balíček
- lajna bota stojí ve světě vedle hráčovy (posunutá v ose X) a kamera se mezi nimi přepíná
- systémy (střelba, pohyb, zásahy, income) pracují pro každou stranu zvlášť
- režim zápasu `GameMode` (`VsBot` / `Test`) se vybírá v menu a systémy podle něj zapínají nebo vypínají bota, kola a přepínání lajn

Online soupeř pak bude jen jiná „strana“, kterou neřídí bot, ale síť.

## Kroky

### Krok 0 – úklid
- [x] Odstranit dvojí registraci kamery a menu v `src/lib.rs`.
- [x] Smazat starý `main.rs` v kořeni projektu.
- [x] Ověřit, že hra jde přeložit a spustit na PC.

### Krok 1 – dvě lajny a přepínání
- [x] `Side` + `SideState`, přesunout gold/income/grid/cestu/životy do `SideState`.
- [x] Druhá lajna (bot) vedle první, vlny chodí na obě.
- [x] Tlačítko pro přepnutí kamery na lajnu soupeře a zpět (vpravo nahoře, na PC i klávesa Tab; swipe zatím ne).
- [x] Na soupeřově lajně nejde stavět (jen se dívat).

### Krok 2 – kola, životy, výhra
- [x] Kolo = vlna z tabulky GDD (malí + velcí creepové, každá 5. vlna boss), pauza 7 s mezi koly. (Na začátku 15 s stavba. Kolo končí, když z obou lajn zmizí creepi vlny, nejpozději po 60 s; vlna 30 čeká na všechny.)
- [x] Po vlně 30 rozhodují životy, leaky a čas zabití bosse, jinak sudden death (+30 % HP za vlnu, první průchod = konec).
- [x] Průnik: malý 1 / velký 2 / boss 3 životy, od vlny 26 aspoň 2 (soupeř dostane o 1 méně); poslané jednotky podle tabulky.
- [x] Obrazovka výhry / prohry, návrat do menu (nový zápas začne od nuly).

### Krok 3 – ekonomika a posílání soupeři
- [x] Start 250 g, income +3 / tick 7 s (max 40), gold za zabití 2–4 / boss 150, gold za přeživší vlnu.
- [x] Poslaná jednotka se objeví hned na **soupeřově** lajně a trvale zvedne income (od dalšího ticku). Posílat jde jen během kola.
- [x] 10 jednotek z GDD, max 12 za vlnu a 2 Mini-Bossové.

### Krok 4 – výběr stavby a vylepšování
- [x] Katalog 15 Basic + 10 Special věží; balíček obsahuje **vždy přesně 4 Basic a 2 Special** (`deck_is_complete`, testy).
- [x] Výběr typu věže v herním panelu; prázdné políčko postaví právě zvolený typ z balíčku.
- [x] Vylepšení Basic věže za gold až na rank 7, Special až na rank 3; ceny a statistiky vycházejí z GDD.
- [x] Stavba Special věže i každý její upgrade stojí jeden diamant (diamanty za bosse).
- [x] **Spojování věží odstraněno** (podle GDD): rank se kupuje jen upgradem za gold, upgrade nemění bludiště.
- [x] Prodej: 70 % před vlnou, 50 % během vlny.

### Krok 5 – bot
- [x] Bot staví, když má gold, a skládá bludiště „had“ (zdi přes lajnu s mezerou střídavě vlevo/vpravo), nikdy si cestu nezazdí.
- [x] Vylepšuje věže za gold (nejnižší rank první), když má aspoň 6 věží.
- [x] Posílá jednotky z goldu navíc (dodržuje limity GDD); od 3. kola občas investuje do income, i když ještě nemá dost věží.
- [x] Opraven pád, když bot nemá na žádnou věž (přetečení rozpočtu).
- [ ] Úrovně obtížnosti bota (lehký / normální / těžký) – později.

### Krok 6 – režimy hry
- [x] Po HRÁT výběr **VS BOT** / **TEST** (ZPĚT vrátí hlavní menu).
- [x] TEST: jen vlastní pole, všech 25 věží, neomezený gold a diamanty, životy se nepočítají, jednotky kdykoli na vlastní pole, tlačítko VLNA +1, okamžité ROZBÍT místo prodeje.

### Krok 7 – vyvážení ke 30. vlně (nahrazeno GDD)
- [x] Vlastní křivka vln a 34 jednotek (2026-11-28 dopoledne) – **nahrazena tabulkami z GDD** (krok 9).

### Krok 8 – testování a ladění na telefonu
- [x] Automatické testy: katalog 25 věží, ceny upgradů a diamanty Special, balíček, tabulky GDD (vlny, jednotky), cap income, životy od vlny 26, damage typy, rozhodnutí po vlně 30, Pepa.
- [x] Simulacemi (bot vs bot) najít násobek HP pro mapu 7 × 14: `MAP_HP_SCALE` = 0,2.
- [ ] Hraním ověřit obtížnost a délku zápasu 20–30 min, případně doladit `MAP_HP_SCALE`.
- [ ] GDD 15: kdy hráči dostanou první diamant, které věže se nepoužívají, jestli není tick 7 s moc hektický.

## Test na PC

| Proměnná | Co dělá |
|---|---|
| `AUTOPLAY=1` | přeskočí menu a rovnou spustí zápas VS BOT |
| `AUTOPLAY=test` | přeskočí menu a rovnou spustí TEST |
| `AUTOPLAY_BOTH=1` | bot hraje i za tebe (bot proti botovi) |
| `SIM_ENDLESS=1` | zápas neskončí při 0 životech (jde vidět, do kterého kola obrana stíhá) |
| `GAME_SPEED=5` | hra běží 5× rychleji (simulace) |
| `MAP_HP_SCALE=0.3` | jiný násobek HP creepů (výchozí 0,2) – na ladění |
| `MAP_RANGE_SCALE=1.0` | jiný násobek dosahu věží (výchozí 0,5) – na ladění |
| `CARGO_MANIFEST_DIR=<projekt>` | když se spouští přímo `target/debug/future_line_td.exe`, aby hra našla obrázky |
| `HIDDEN_WINDOW=1` | okno se otevře mimo obrazovku (automatické testy) |
| `SCREENSHOT_AT="15,20" SCREENSHOT_DIR=slozka` | screenshoty v daných sekundách zápasu (`SCREENSHOT_VIEW=bot` = kamera na lajně bota) |

Na začátku každého kola se do konzole píše řádek `EKONOMIKA kolo N …` (gold, income, hodnota věží a životy obou stran) a každý průnik jako `PROSLO na lajne …`.

Příklad simulace vyvážení (bot proti botovi, 5× rychleji, bez konce zápasu):

```powershell
$env:SIM_ENDLESS=1; $env:AUTOPLAY=1; $env:AUTOPLAY_BOTH=1; $env:GAME_SPEED=5; cargo run
```

Ovládání: šipky posouvají vybrané políčko (Shift + šipka o dvě), Enter na prázdném políčku staví, Enter na věži ji vylepší za gold, S na věži prodá (okno ANO/NE), jinak S otevře panel jednotek. Pauzu otevře tlačítko PAUSE nebo mezerník; restart, návrat do menu i ukončení hry vyžadují potvrzení ANO/NE.

### Krok 9 – převod na finální GDD (2026-11-28)
- [x] Data GDD odděleně v `src/wave_def.rs` (bez kreslení, připravené pro Rust core).
- [x] Ekonomika GDD 4.1–4.3: +3 / tick, cap 40, kill gold 2–4, boss 150, gold za přeživší vlnu z tabulky, prodej 70 / 50 %.
- [x] 10 posílaných jednotek z GDD 4.4, limity 12 za vlnu a 2 Mini-Bossové; vyběhnou hned (domluveno).
- [x] 10 typů creepů se schopnostmi (štít, regenerace, kopie, aura rychlosti, náhodná rychlost, enrage) a tabulka vln 1–30.
- [x] 6 bossů s jejich schopnostmi, diamanty (stavba i upgrade Special = 1 diamant, nepotřebný = 400 g, domluveno).
- [x] Životy GDD 2: boss 3, od vlny 26 leak −2 / +1, rozhodnutí po vlně 30, sudden death +30 % HP za vlnu.
- [x] 5 damage typů GDD 5, slow cap 50 %, kratší CC na bosse.
- [x] Hlídač Pepa: 10 s na opravu, pak prodá nejdražší blokující věž; max 75 věží; nestavět pod creepy.
- [x] Mapa zůstává 7 × 14, technika zůstává Bevy (domluveno).
- [ ] Pepa jako postavička na cestě, zčervenání cesty, meme zvuky a kill feed (GDD 13).

## Další etapa (po botovi)
Karty a truhly, úrovně karet, uložení progresu v telefonu → online 1v1, účty, žebříček → kosmetika, diamanty, obchod.

## Otázky k vyřešení během práce
- **Statistiky ranků 6–7**: dopočítat ze stávající tabulky (stejný růst), nebo navrhnout znovu.
- **Hlídač Pepa** hlídá obě strany i v TEST režimu (zazděnou cestu po 10 s prodá).
- **Zvláštní schopnosti jednotek 16, 17, 22, 23, 28** zatím chybí (mají jen staty).
