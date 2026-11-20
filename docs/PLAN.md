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
- [ ] Odstranit dvojí registraci kamery a menu v `src/lib.rs`.
- [ ] Smazat starý `main.rs` v kořeni projektu.
- [ ] Ověřit, že hra jde přeložit a spustit na PC.

### Krok 1 – dvě lajny a přepínání
- [ ] `Side` + `SideState`, přesunout gold/income/grid/cestu/životy do `SideState`.
- [ ] Druhá lajna (bot) vedle první, zatím prázdná, vlny chodí na obě.
- [ ] Tlačítko / swipe pro přepnutí kamery na lajnu soupeře a zpět.
- [ ] Na soupeřově lajně nejde stavět (jen se dívat).

### Krok 2 – kola, životy, výhra
- [ ] Kolo = vlna normálních creepů + 1 větší, každé 5. kolo boss, pauza 7 s mezi koly.
- [ ] Vlny rostou, v pozdních kolech rychleji (pojistka ~15 min).
- [ ] Průnik: normální 1 / větší 2 / boss 3–5 životů; **já ztratím, soupeř získá** (start 15/15, celkem 30).
- [ ] Obrazovka výhry / prohry, návrat do menu.

### Krok 3 – ekonomika a posílání soupeři
- [ ] Start 30 g, income 10, tick 7 s, gold za zabití.
- [ ] Poslaná jednotka se objeví na **soupeřově** lajně a trvale zvedne income.
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

## Další etapa (po botovi)
Karty a truhly, výběr balíčku, úrovně karet, uložení progresu v telefonu → online 1v1, účty, žebříček → kosmetika, diamanty, obchod.

## Otázky k vyřešení během práce
- **Prodej věží**: zůstává? Při merge TD obvykle prodej není, ale v bludišti se může hodit (přestavění cesty).
- **Statistiky ranků 6–7**: dopočítat ze stávající tabulky (stejný růst), nebo navrhnout znovu.
- **Hlídač** (bourá věž při úplném zablokování cesty) zůstává pro obě strany.
