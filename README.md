# Line TD (PvP tower defense)

Mobilní 1v1 tower defense v Rustu a Bevy 0.15 (Android + PC). Každý hráč staví bludiště z věží na své lajně, vylepšuje je za gold, přežívá automatické vlny a posílá soupeři jednotky, které mu zvednou income.

Hlavní zdroj pravdy je finální GDD **Meme TD 1v1** (verze 1.0). Co z něj v kódu ještě chybí, je v [docs/DESIGN.md](docs/DESIGN.md), sekce 7.

## Spuštění

```powershell
cargo run
```

V menu **HRÁT** vybereš režim:

- **VS BOT** – zápas 1v1 proti botovi podle GDD: 30 vln, 6 bossů, 30 životů jako přetahovaná, po vlně 30 rozhodují životy nebo sudden death.
- **TEST** – hřiště: jen tvoje pole, všech 25 věží, neomezený gold, životy se nepočítají, jednotky z panelu S jdou na tvoje pole, VLNA +1 pošle další vlnu.

**BALÍČEK**: do zápasu si vybíráš vždy 4 Basic a 2 Special věže.

## Dokumentace

- [docs/DESIGN.md](docs/DESIGN.md) – pravidla hry, ekonomika, režimy, vyvážení vln a tabulka 34 jednotek
- [docs/PLAN.md](docs/PLAN.md) – plán implementace, co je hotové, proměnné prostředí pro testy a simulace
- [docs/PRUBEH.md](docs/PRUBEH.md) – deník změn

## Testy

```powershell
cargo test
```
