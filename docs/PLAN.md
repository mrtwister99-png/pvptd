# Plán implementace – 1v1 proti botovi

Cíl první etapy: **hratelný zápas 1v1 proti botovi v telefonu** podle [DESIGN.md](DESIGN.md).
Karty, truhly a online přijdou až potom. Každý krok končí tím, že hra jde spustit a vyzkoušet.

## Hlavní technická změna

Dnes je ve hře **jedna lajna** a všechno je globální: `Gold`, `Income`, `Grid`, `PathMap`, `EscapedCount`.
Pro 1v1 potřebujeme **dvě lajny**, proto:

- komponenta `Side` (`Player` / `Bot`) na věžích, creepech, střelách a hlídači
- stav hráče na jednom místě: `Players { sides: [SideState; 2] }`, kde `SideState` obsahuje gold, income, životy, grid, cestu a počet staveb
- lajna bota stojí ve světě vedle hráčovy (posunutá v ose X) a kamera se mezi nimi přepíná
- systémy (střelba, pohyb, zásahy, income) pracují pro každou stranu zvlášť

Tohle je nejvíc práce, ale připraví to půdu i pro online: online soupeř je pak jen jiná „strana“, kterou neřídí bot, ale síť.

## Kroky

### Krok 0 – úklid (malý)
- [x] Odstranit dvojí registraci kamery a menu v `src/lib.rs`.
- [x] Smazat starý `main.rs` v kořeni projektu.
- [x] Ověřit, že hra jde přeložit a spustit na PC.

### Krok 1 – dvě lajny a přepínání
- [x] `Side` + `SideState`, přesunout gold/income/grid/cestu/životy do `SideState`.
- [x] Druhá lajna (bot) vedle první, zatím prázdná, vlny chodí na obě.
- [x] Tlačítko / swipe pro přepnutí kamery na lajnu soupeře a zpět. (tlačítko SOUPEŘ vpravo nahoře, na PC i klávesa Tab; swipe zatím ne)
- [x] Na soupeřově lajně nejde stavět (jen se dívat).

### Krok 2 – kola, životy, výhra
- [x] Kolo = vlna normálních creepů + 1 větší, každé 5. kolo boss, pauza 7 s mezi koly. (Na začátku 15 s stavba. Kolo končí, když z obou lajn zmizí creepi vlny, nejpozději po 60 s.)
- [x] Vlny rostou, v pozdních kolech rychleji (od kola 20 +15 % HP za kolo). Limit 30 vln zrušen.
- [x] Průnik: normální 1 / větší 2 / boss 4 životy, poslané jednotky 1.
- [x] Obrazovka výhry / prohry, návrat do menu (nový zápas začne od nuly).

### Krok 3 – ekonomika a posílání soupeři
- [x] Start 250 g, income 20, tick 7 s, gold za zabití.
- [x] Poslaná jednotka se objeví na **soupeřově** lajně a trvale zvedne income. Posílat jde jen během kola.
- [x] Posílat jde neomezeně, dokud je gold, všechny jednotky odemčené.

### Krok 4 – výběr stavby a vylepšování
- [x] Katalog 15 Basic + 10 Special věží; balíček obsahuje přesně 4 Basic a 2 Special.
- [x] Výběr typu věže v herním panelu; prázdné políčko postaví právě zvolený typ z balíčku.
- [x] Vylepšení Basic věže za gold až na rank 7, Special až na rank 3; ceny a statistiky vycházejí z GDD.
- [x] Stavba Special věže vyžaduje a spotřebuje jeden diamant.
- [x] Spojení stejných věží stejného ranku zůstává jako alternativní cesta k upgradu a zachová typ věže.
- [x] Prodej zůstává: vrací 50 % investovaného goldu.

### Krok 5 – bot
- [x] Bot staví, když má gold, a skládá bludiště „had“ (zdi přes lajnu s mezerou střídavě vlevo/vpravo), nikdy si cestu nezazdí.
- [x] Spojuje dvojice, když může.
- [x] Posílá jednotky z goldu navíc, během kola občas místo stavby investuje do income.
- [ ] Úrovně obtížnosti bota (lehký / normální / těžký) – později.

### Krok 6 – testování a ladění na telefonu
- [x] Automatické testy katalogu 25 věží, cen/upgradů, rank capů a diamantové ceny Special stavby.
- [ ] Délka zápasu 4–5 min, rank 7 je reálný? (Test: bez stropu měl bot v 7. kole jen 4 věže → přidán strop 100 g. Ověřit hraním.)
- [ ] Ceny jednotek, income, gold za zabití, růst vln.

## Test na PC

`AUTOPLAY=1 cargo run` přeskočí menu a rovnou spustí zápas.
`HIDDEN_WINDOW=1` otevře okno mimo obrazovku (pro automatické testy).
`AUTOPLAY_BOTH=1` = bot hraje i za tebe (bot proti botovi).
`SCREENSHOT_AT="15,20" SCREENSHOT_DIR=slozka` uloží screenshoty v daných sekundách zápasu (`SCREENSHOT_VIEW=bot` = kamera na lajně bota).
Na PC se šipky pohybují o jedno políčko. Drží-li se současně Shift a šipka, pohyb je o dvě políčka daným směrem.
Pauzu otevře tlačítko PAUSE; pokračovat lze tlačítkem POKRAČOVAT nebo mezerníkem. Restart, návrat do menu i ukončení hry vyžadují potvrzení ANO/NE.

## Další etapa (po botovi)
Karty a truhly, výběr balíčku, úrovně karet, uložení progresu v telefonu → online 1v1, účty, žebříček → kosmetika, diamanty, obchod.

## Otázky k vyřešení během práce
- **Prodej věží**: zůstává? Při merge TD obvykle prodej není, ale v bludišti se může hodit (přestavění cesty).
- **Statistiky ranků 6–7**: dopočítat ze stávající tabulky (stejný růst), nebo navrhnout znovu.
- **Hlídač** (bourá věž při úplném zablokování cesty) zůstává pro obě strany.
