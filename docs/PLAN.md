# Plán implementace – 1v1 proti botovi

Cíl první etapy: **hratelný zápas 1v1 proti botovi v telefonu** podle finálního GDD (Meme TD 1v1, verze 1.0) a [DESIGN.md](DESIGN.md), hratelný až ke 30. vlně.
Co z GDD ještě chybí, je v DESIGN.md, sekce **8. Rozdíly kódu proti GDD**.
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
- [x] Kolo = vlna normálních creepů + 1 větší, každé 5. kolo boss, pauza 7 s mezi koly. (Na začátku 15 s stavba. Kolo končí, když z obou lajn zmizí creepi vlny, nejpozději po 60 s.)
- [x] Vlny rostou plynule až ke 30. kolu, **po 30. kole** pojistka +15 % HP za kolo.
- [x] Průnik: normální 1 / větší 2 / boss 4 životy, poslané jednotky 1.
- [x] Obrazovka výhry / prohry, návrat do menu (nový zápas začne od nuly).

### Krok 3 – ekonomika a posílání soupeři
- [x] Start 250 g, income 20, tick 7 s, gold za zabití.
- [x] Poslaná jednotka se objeví na **soupeřově** lajně a trvale zvedne income. Posílat jde jen během kola.
- [x] Posílat jde neomezeně, dokud je gold, všechny jednotky odemčené.
- [x] **34 jednotek** (přibyly TITAN, BANSHEE, HIEROPHANT, OVERLORD), v panelu S seřazené podle ceny.

### Krok 4 – výběr stavby a vylepšování
- [x] Katalog 15 Basic + 10 Special věží; balíček obsahuje **vždy přesně 4 Basic a 2 Special** (`deck_is_complete`, testy).
- [x] Výběr typu věže v herním panelu; prázdné políčko postaví právě zvolený typ z balíčku.
- [x] Vylepšení Basic věže za gold až na rank 7, Special až na rank 3; ceny a statistiky vycházejí z GDD.
- [x] Stavba Special věže vyžaduje a spotřebuje jeden diamant.
- [x] **Spojování věží odstraněno** (podle GDD): rank se kupuje jen upgradem za gold, upgrade nemění bludiště.
- [x] Prodej zůstává: vrací 50 % investovaného goldu.

### Krok 5 – bot
- [x] Bot staví, když má gold, a skládá bludiště „had“ (zdi přes lajnu s mezerou střídavě vlevo/vpravo), nikdy si cestu nezazdí.
- [x] Vylepšuje věže za gold (nejnižší rank první), když má aspoň 6 věží.
- [x] Posílá jednotky z goldu navíc; od 3. kola občas investuje do income, i když ještě nemá dost věží.
- [x] Opraven pád, když bot nemá na žádnou věž (přetečení rozpočtu).
- [ ] Úrovně obtížnosti bota (lehký / normální / těžký) – později.

### Krok 6 – režimy hry
- [x] Po HRÁT výběr **VS BOT** / **TEST** (ZPĚT vrátí hlavní menu).
- [x] TEST: jen vlastní pole, všech 25 věží, neomezený gold a diamanty, životy se nepočítají, jednotky kdykoli na vlastní pole, tlačítko VLNA +1, okamžité ROZBÍT místo prodeje.

### Krok 7 – vyvážení ke 30. vlně
- [x] Model ekonomiky (income, zabití, posílání) a porovnání s HP vln: vlny od 8. kola rostly mnohem rychleji než gold hráče.
- [x] Nová křivka vln 7–34 (HP vlny ≈ 1× až 2,2× goldu ve věžích), pojistka až po 30. kole.
- [x] HP posílaných jednotek: HP za gold roste s cenou (2,0 → 3,8), aby se vyplatily i dražší jednotky.
- [x] Testy: vlny rostou každé kolo, vlna 30 zůstává v hratelném rozsahu, dražší jednotka nemá méně HP za gold.
- [ ] Ověřit hraním na telefonu (hlavně kola 15–30) a doladit `WAVE_HP_SCALE` a tabulku vln 7–15.

### Krok 8 – testování a ladění na telefonu
- [x] Automatické testy katalogu 25 věží, cen/upgradů, rank capů, diamantové ceny Special stavby, balíčku a křivky vln.
- [ ] Délka zápasu: 30 kol trvá déle než původních 15 min – rozhodnout, jestli zkrátit kola.
- [ ] Ceny jednotek, income, gold za zabití.

## Test na PC

| Proměnná | Co dělá |
|---|---|
| `AUTOPLAY=1` | přeskočí menu a rovnou spustí zápas VS BOT |
| `AUTOPLAY=test` | přeskočí menu a rovnou spustí TEST |
| `AUTOPLAY_BOTH=1` | bot hraje i za tebe (bot proti botovi) |
| `SIM_ENDLESS=1` | zápas neskončí při 0 životech (jde vidět, do kterého kola obrana stíhá) |
| `GAME_SPEED=5` | hra běží 5× rychleji (simulace) |
| `HIDDEN_WINDOW=1` | okno se otevře mimo obrazovku (automatické testy) |
| `SCREENSHOT_AT="15,20" SCREENSHOT_DIR=slozka` | screenshoty v daných sekundách zápasu (`SCREENSHOT_VIEW=bot` = kamera na lajně bota) |

Na začátku každého kola se do konzole píše řádek `EKONOMIKA kolo N …` (gold, income, hodnota věží a životy obou stran) a každý průnik jako `PROSLO na lajne …`.

Příklad simulace vyvážení (bot proti botovi, 5× rychleji, bez konce zápasu):

```powershell
$env:SIM_ENDLESS=1; $env:AUTOPLAY=1; $env:AUTOPLAY_BOTH=1; $env:GAME_SPEED=5; cargo run
```

Ovládání: šipky posouvají vybrané políčko (Shift + šipka o dvě), Enter na prázdném políčku staví, Enter na věži ji vylepší za gold, S na věži prodá (okno ANO/NE), jinak S otevře panel jednotek. Pauzu otevře tlačítko PAUSE nebo mezerník; restart, návrat do menu i ukončení hry vyžadují potvrzení ANO/NE.

### Krok 9 – převod na finální GDD
- [ ] Ekonomika podle GDD 4.1–4.3: +3 / tick, cap 40, kill gold 2–4, bonus za vlnu, prodej 70 / 50 %.
- [ ] 10 posílaných jednotek z GDD 4.4 (místo 34 z WC3 mapy), limity 12 za vlnu a 2 Mini-Bossové.
- [ ] 10 typů creepů z GDD 9 a tabulka vln 1–30 z GDD 10, endless sudden death po vlně 30 (+30 % HP za vlnu).
- [ ] 6 bossů z GDD 11, diamant podle GDD 4.2 (1. a 2. diamant odemyká Special věže).
- [ ] Životy podle GDD 2 (boss 3, od vlny 26 leak −2 / +1, sudden death a tie-breaky po vlně 30).
- [ ] 5 damage typů z GDD 5.
- [ ] Mapa 16 × 12 s hadovkou, Hlídač Pepa (10 s na opravu), max 75 věží.
- [ ] Rozhodnout přechod na Rust core + server + Godot klient (GDD 14).

## Další etapa (po botovi)
Karty a truhly, úrovně karet, uložení progresu v telefonu → online 1v1, účty, žebříček → kosmetika, diamanty, obchod.

## Otázky k vyřešení během práce
- **Statistiky ranků 6–7**: dopočítat ze stávající tabulky (stejný růst), nebo navrhnout znovu.
- **Hlídač** (bourá věž při úplném zablokování cesty) zůstává pro obě strany; v TEST režimu nevybíhá.
- **Zvláštní schopnosti jednotek 16, 17, 22, 23, 28** zatím chybí (mají jen staty).
