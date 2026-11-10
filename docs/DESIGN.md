# PvP TD – design dokument

> Živý dokument. Co je rozhodnuté, je v sekci **Rozhodnuto**. Co se ještě řeší, je v **Otevřené otázky**.
> Každé téma projdeme zvlášť a výsledek sem zapíšeme.

## 1. Vize (jednou větou)

Rychlá a pestrá mobilní **1v1 PvP tower defense**. Kořeny má v Line Tower Wars (posílání jednotek soupeři + income), progres a zápasy jsou ve stylu moderních mobilních karetních TD. **Žádné pay-to-win**: za peníze jen kosmetika, rozhoduje skill.

## 2. Odkud vycházíme

- **Původní inspirace**: WC3 custom mapa (Line Tower Wars), 7+ hráčů na životy, nekonečné vlny, income tick každých 10 s, posílání jednotek dalšímu hráči v kruhu.
- **Co už je hotové v kódu (Bevy 0.15, Android + PC)**:
  - pole 7×14, stavění věží, pathfinding creepů kolem věží
  - 7 typů věží × 5 úrovní (arrow, cannon, frost, fire, poison, sniper, shock), prodej se 75% vratkou
  - 30 vln + nekonečný režim, mega/boss vlny
  - posílání jednotek (panel S) → permanentní +income, income tick 7 s
  - efekty (DoT, slow, chain, push), čísla poškození, pooly entit
  - hlavní menu (funguje jen HRÁT)
- **Rozdělané / chybí**: speciální schopnosti jednotek, ikony 3 věží, Leaderboard/Nastavení, žádný soupeř (zatím singleplayer).

## 3. Nová koncepce – hlavní pilíře

1. **1v1** – dva hráči, každý má svou lajnu, posílají si jednotky navzájem.
2. **Kola** – zápas je rozdělený na kola; během kola se posílají jednotky.
3. **Žebříček** – hodnocení (rating / ligy), matchmaking podle síly.
4. **Karty** – hráč sbírá karty (věže? jednotky?) a vylepšuje je. Musí to jít nasbírat **zdarma**.
5. **Monetizace** – jen skiny/kosmetika. Žádné kupování síly.
6. **Krátké zápasy** – cíl cca ? min (viz otázky).

## 4. Rozhodnuto

_(zatím nic – doplníme během probírání)_

## 5. Otevřené otázky

### A. Zápas a kola
- Jak dlouho má trvat jeden zápas? (3 min? 5 min? 10 min?)
- Co je „kolo“? Pevný čas (např. 30 s), nebo dokud nepadne vlna?
- Je mezi koly pauza na stavění, nebo se staví pořád?
- Jdou i **automatické vlny** (jako v originále), nebo jen to, co pošle soupeř?
- Jak se vyhrává: životy na nulu? Limit kol a pak rozhoduje počet životů?
- Co když se zápas táhne – zrychlení / sudden death (už máš „SUDDEN DEATH“ vlnu)?

### B. Ekonomika
- Zůstává income tick + permanentní income za posílání?
- Limit posílání za kolo (proti spamu)?
- Startovní gold a income?

### C. Karty a progres (nejdůležitější kvůli „žádné P2W“)
- Co jsou karty: věže, posílané jednotky, nebo obojí?
- Bere si hráč do zápasu **balíček** (např. 5 věží z 7+), nebo má vždy všechny?
- Vylepšení karet mimo zápas – mění to síly v zápase? (Pokud ano, je to riziko nerovnosti; alternativa: v ranked se úrovně **srovnají**, kartu jen musíš mít odemčenou.)
- Jak se karty získávají zdarma (odměny za výhry, denní úkoly, truhly)?
- Upgrady věží v zápase (lvl 1–5) zůstávají?

### D. Multiplayer / technika
- Online proti lidem hned, nebo nejdřív proti botovi (AI soupeř) a online až později?
- Server: vlastní (Rust), nebo služba? Účty, ukládání progresu, ochrana proti podvádění.
- Jak se ukáže soupeřova lajna na malém displeji (minimapa, přepínání)?

### E. Obsah a vzhled
- Jména a grafika jednotek jsou zatím z WC3 mapy (Sheep, Frost Wolf…) – před vydáním je dát **vlastní**.
- Styl grafiky / skinů.

## 6. Navržené pořadí práce

1. Doladit tento dokument (sekce A → C → B → D).
2. Udělat **1v1 proti botovi** lokálně – ověří, jestli jsou kola a posílání zábavné (bez serveru).
3. Karty a balíček, zatím uložené lokálně.
4. Online 1v1 + účty + žebříček.
5. Kosmetika, obchod, vydání.

## 7. Technické poznámky

- `src/lib.rs` registruje kameru a menu **dvakrát** (řádky ~27–36) → vznikají 2 kamery a 2 menu. Opravit.
- `main.rs` v kořeni projektu je starý duplikát, nepoužívá se (balík bere `src/`).
