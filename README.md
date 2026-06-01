# lau-biome

> 10 distinct ecological zones where each biome IS a mathematical structure. The Meadow teaches symmetry. The Volcano teaches chaos. The Crystal Caves teach lattice theory.

## What This Does

The biome system for the **Lau (Layered Agent-UI)** gamified learning platform. Each of the 10 biomes is a distinct ecological zone with unique resources, properties, and — crucially — an underlying mathematical structure that players discover through exploration.

This isn't just cosmetic. Biome properties (temperature, moisture, elevation, resources) are generated from deterministic mathematical functions. Same seed = same world. Every time.

## The Key Idea

Game worlds usually fake diversity with noise functions. Lau uses actual mathematics. The Forest biome has tree placement following a Poisson point process. The Crystal Caves have lattice structures from Bravais lattices. The Floating Islands use eigenvalue-based height maps. Players discover these patterns organically — "why do the crystals always form hexagons?" — and that's the teaching moment.

## Install

```bash
cargo add lau-biome
```

## Quick Start

```rust
use lau_biome::{Biome, WorldMap, BiomeGenerator};

// Generate a world with a specific seed (deterministic)
let mut gen = BiomeGenerator::with_seed(42);
let map = gen.generate(64, 64); // 64x64 world

// Query what biome is at each position
for y in 0..64 {
    for x in 0..64 {
        let biome = map.biome_at(x, y);
        let temp = map.temperature_at(x, y);
        let moisture = map.moisture_at(x, y);
    }
}
```

### The 10 Biomes

```rust
let biomes = Biome::all();
for b in &biomes {
    println!("{}: {}", b.name(), b.description());
    println!("  Resources: {:?}", b.resources());
    println!("  Math concept: {}", b.mathematical_theme());
}
```

| Biome | Visual | Resources | Math Theme |
|-------|--------|-----------|------------|
| Meadow | Rolling grass, flowers | Wheat, herbs, flowers | Symmetry groups |
| Forest | Dense trees, streams | Wood, berries, mushrooms | Poisson processes |
| Desert | Sand dunes, cacti | Sandstone, gems, heat | Fourier analysis |
| Tundra | Ice, snow, aurora | Ice crystals, frost flowers | Crystallography |
| Ocean | Waves, coral, depth | Fish, pearls, kelp | Wave equations |
| Volcano | Lava, smoke, obsidian | Obsidian, magma crystals | Chaos theory |
| Crystal Caves | Glowing formations | Crystals, minerals | Lattice theory |
| Floating Islands | Sky islands, waterfalls | Cloud ore, sky wood | Eigenvalue decomposition |
| Mushroom Grove | Giant fungi, spores | Mushrooms, bioluminescence | Fibonacci spirals |
| Ancient Ruins | Stone temples, glyphs | Artifacts, knowledge | Number theory |

### Biome Properties

```rust
let forest = Biome::Forest;

// Environmental properties
println!("Temperature range: {:?}", forest.temp_range());     // (5.0, 25.0)°C
println!("Moisture range: {:?}", forest.moisture_range());    // (0.4, 0.9)
println!("Elevation range: {:?}", forest.elevation_range());  // (0.1, 0.6)

// Resources
let resources = forest.resources();  // vec!["wood", "berries", "mushrooms"]
let rarity = forest.resource_rarity("berries"); // Some(0.3)

// Mathematical properties
println!("Lattice type: {:?}", forest.lattice_type()); // Some("Poisson")
println!("Symmetry group: {:?}", forest.symmetry_group()); // Some("D6")
```

### Deterministic Generation

```rust
use lau_biome::BiomeGenerator;

// Same seed always produces the same world
let gen1 = BiomeGenerator::with_seed(12345);
let gen2 = BiomeGenerator::with_seed(12345);

let map1 = gen1.generate(32, 32);
let map2 = gen2.generate(32, 32);

assert_eq!(map1.biome_at(10, 10), map2.biome_at(10, 10));
// Always passes — deterministic generation
```

## API Reference

### Biome (enum)

| Method | Description |
|--------|-------------|
| `Biome::all()` | All 10 biome variants |
| `biome.name()` | Human-readable name |
| `biome.description()` | Flavor text |
| `biome.resources()` | Available resources |
| `biome.resource_rarity(name)` | Drop probability |
| `biome.temp_range()` | (min, max) temperature |
| `biome.moisture_range()` | (min, max) moisture |
| `biome.elevation_range()` | (min, max) elevation |
| `biome.lattice_type()` | Mathematical generation method |
| `biome.symmetry_group()` | Point symmetry group |

### BiomeGenerator

| Method | Description |
|--------|-------------|
| `BiomeGenerator::with_seed(seed)` | Create deterministic generator |
| `BiomeGenerator::new()` | Random seed |
| `gen.generate(width, height)` | Generate complete world map |

### WorldMap

| Method | Description |
|--------|-------------|
| `map.biome_at(x, y)` | Biome at position |
| `map.temperature_at(x, y)` | Temperature value |
| `map.moisture_at(x, y)` | Moisture value |
| `map.elevation_at(x, y)` | Height value |
| `map.resources_at(x, y)` | Available resources |
| `map.width()` / `map.height()` | Dimensions |

## How It Works

Biome assignment uses layered noise functions:
1. **Temperature layer** — latitude-based + noise perturbation
2. **Moisture layer** — independent noise with different frequency
3. **Elevation layer** — fractal Brownian motion
4. **Biome selection** — Whittaker diagram mapping (temp × moisture → biome)

Resource placement uses biome-specific probability distributions. Crystal Caves use hexagonal grid placement (Bravais lattice). Forest trees use Poisson disk sampling. Mushroom spirals follow Fibonacci angles.

All generation uses Xoshiro256** PRNG — seeded, deterministic, and fast.

## Testing

37 tests covering: all 10 biome properties, deterministic generation, resource probabilities, boundary conditions, serialization, neighbor queries, Whittaker mapping.

## Part of the Lau Platform

- **lau-git-world** — Git-native game worlds
- **lau-quest** — Quest/mission system
- **lau-biome** — You are here
- **lau-spatial** — Spatial indexing
- **lau-audio** — Procedural audio
- **lau-scheduler** — Game loop
- **lau-memory-arena** — Entity allocator
- **lau-genealogy** — Lineage tracking
- **lau-recipe** — Crafting recipes

## License

MIT
