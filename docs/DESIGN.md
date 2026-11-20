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

### A. Zápas a kola (2026-11-10)

- **Délka zápasu**: na začátku hry (málo karet) cca **4–5 min**, se silnějšími kartami se prodlužuje, max **~15 min**.
- **Kolo** = jedna automatická vlna: hromada normálních creepů + **jeden větší** creep. Každé kolo je vlna silnější.
- **Každé 5. kolo** přijde navíc **boss**.
- **Posílání**: hráč posílá jednotky soupeři **během vlny**. Po vlně je krátká pauza, pak začne další kolo a posílá se znovu.
- **Automatické vlny chodí vždy** a k nim navíc to, co pošle soupeř.
- **Životy jako přetahovaná**: dohromady je **30 životů**, oba hráči začínají s **15**. Když mi projde creep, já ztratím 1 a soupeř získá 1. Prohrává ten, kdo má 0, takže vítěz má všech 30.
- **Cena průniku**: normální creep **1** život, větší creep **2**, boss **3–5** (doladit).
- **Matchmaking hlavně podle síly balíčku** (úrovně karet), aby proti sobě hráli podobně silní.
- **Pojistka**: v pozdějších kolech vlny sílí čím dál rychleji, takže zápas skončí nejpozději kolem ~15 min.
- **Časování**: posílat se dá **celou dobu kola**, mezi koly je pauza **7 s**.
- **Stavět a vylepšovat věže se dá kdykoliv**, i během vlny.

### C. Karty a spojování věží (2026-11-20)

- **Karty jsou věže i jednotky.** Získávají se po zápase z **truhel** (a dalších odměn).
- **Balíček**: do zápasu si hráč vybere **5 věží**.
- **Úroveň karty zvedá všechny statistiky** dané věže nebo jednotky. U jednotek to znamená silnější posílané creepy.
- **Stavění v zápase**: tlačítko „postav“ vytvoří **náhodnou** věž z 5 vybraných, rank 1.
  Cena: první **10 g**, každá další o **+10 g** víc (10, 20, 30…).
- **Spojování (merge)**: dvě **stejné** věže stejného ranku se spojí → vznikne **náhodná věž z balíčku o rank vyšší**.
- **Max rank 7** (rank 1 + až 6 spojení; na rank 7 je potřeba 64 staveb).
- **Bludiště zůstává**: hráč si cestu staví sám, jak to v rychlosti zvládne. Spojení věže z bludiště odebere, takže se cesta změní (je to i taktika).
- **Spojení přetažením**: věž přetáhneš na stejnou věž a nová věž vznikne **na místě, kam jsi táhl**.
- **Jednotky**: posílat se dá **neomezeně, dokud máš gold**. V zápase jsou **odemčené všechny jednotky**. Sílu jim zvedají **jen karty** (z truhel), během zápasu se jednotky nevylepšují.

- **V zápase se jen staví a spojuje**, žádná vylepšení typu věže za gold.

### D. Soupeř a obrazovka (2026-11-20)

- **Začínáme 1v1 proti botovi** v telefonu (bez serveru). Online proti lidem až potom.
- **Přepínání mezi lajnami**: hráč vidí svoje bludiště a tlačítkem/swipem se přepne na soupeřovo.

### B. Ekonomika (2026-11-20)

- **Start**: **30 g** (hned jdou postavit 2 věže: 10 + 20 g), income **10**.
- **Income tick každých 7 s**: dostaneš tolik goldu, kolik máš income.
- **Poslaná jednotka trvale zvedne income** (např. Sheep za 5 g → +1).
- **Gold za zabití creepa**: ano.
- Tím se **ruší** dosavadní vylepšování lvl 1–5 za gold (Basic Tower → Phantom Archer…). Síla věže = úroveň karty × rank v zápase.

## 5. Otevřené otázky

### A. Zápas a kola – doladit čísla při testování
- Délka jednoho kola (kolik creepů, rozestupy), od kterého kola se zrychluje růst vln.
- Přesná cena průniku bosse (3, 4, nebo 5).

### B. Ekonomika – doladit čísla při testování
- Ceny jednotek a kolik income dávají, gold za zabití podle typu creepa.

### C. Karty a spojování – doladit
- Úrovně karet dávají sílu a férovost řeší matchmaking podle síly balíčku. Proto musí být vylepšování **jen za věci získané hraním, nikdy za peníze**.
- Rank 7 = 64 staveb ≈ 21 000 g při ceně +10 g. Ověřit při testování, jestli je to za 10–15 min reálné, případně dát strop ceny.
- Jsou v zápase **vylepšení celého typu věže** (zaplatíš a všechny arrow jsou silnější), nebo jen stavění a spojování?
- Jak velký rozdíl mezi úrovní karty 1 a max? (Malý = víc skillu, velký = víc progresu.)

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
