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
- [ ] Start 30 g, income 10, tick 7 s, gold za zabití.
- [x] Poslaná jednotka se objeví na **soupeřově** lajně a trvale zvedne income. Posílat jde jen během kola.
- [ ] Posílat jde neomezeně, dokud je gold, všechny jednotky odemčené.

### Krok 4 – náhodná stavba a spojování (merge)
- [ ] Balíček 5 věží (zatím napevno v kódu, výběr přijde s kartami).
- [ ] Tlačítko „postav“: náhodná věž z balíčku, rank 1, na vybrané políčko; cena 10, 20, 30… g.
- [ ] Přetažení věže na stejnou věž stejného ranku → **náhodná věž z balíčku o rank výš** na cílovém místě; uvolněné políčko se vrátí do bludiště (přepočet cesty).
- [ ] Max rank 7. Statistiky pro ranky 1–7 (dnes jsou úrovně 1–5 → tabulku rozšířit).
- [ ] Odstranit staré vylepšování lvl 1–5 za gold (a rozhodnout, co s prodejem).

### Krok 5 – bot
- [ ] Bot staví, když má gold, a skládá jednoduché bludiště.
- [ ] Spojuje dvojice, když může.
- [ ] Posílá jednotky, když má gold navíc (aby to nebyla jen obrana).
- [ ] Úrovně obtížnosti bota (lehký / normální / těžký) – později.

### Krok 6 – testování a ladění na telefonu
- [ ] Délka zápasu 4–5 min, rank 7 je reálný? Strop ceny stavby?
- [ ] Ceny jednotek, income, gold za zabití, růst vln.

## Test na PC

`AUTOPLAY=1 cargo run` přeskočí menu a rovnou spustí zápas.

## Další etapa (po botovi)
Karty a truhly, výběr balíčku, úrovně karet, uložení progresu v telefonu → online 1v1, účty, žebříček → kosmetika, diamanty, obchod.

## Otázky k vyřešení během práce
- **Prodej věží**: zůstává? Při merge TD obvykle prodej není, ale v bludišti se může hodit (přestavění cesty).
- **Statistiky ranků 6–7**: dopočítat ze stávající tabulky (stejný růst), nebo navrhnout znovu.
- **Hlídač** (bourá věž při úplném zablokování cesty) zůstává pro obě strany.
