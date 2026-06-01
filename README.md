# lau-biome

> Biome system for the LAU game — distinct ecological zones with unique properties, resources, and mathematical characteristics.

## What This Does

Biome system for the LAU game — distinct ecological zones with unique properties, resources, and mathematical characteristics.. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-biome
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_biome::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub enum Biome 
    pub fn all() -> [Biome; 10] 
pub enum Material 
pub struct BiomeProperties 
    pub fn normalized_vibe(&self) -> f64 
pub struct BiomeRule 
pub struct BiomeTransition 
    pub fn transition_vibe(&self, position: f64, from_props: &BiomeProperties, to_props: &BiomeProperties) -> f64 
pub struct BiomeMap 
    pub fn from_raw(grid: Vec<Vec<Biome>>) -> Self 
    pub fn generate(seed: u64, width: usize, height: usize) -> Self 
    pub fn get_biome(&self, x: usize, y: usize) -> Biome 
    pub fn biome_at_center(&self) -> Biome 
    pub fn biome_counts(&self) -> HashMap<Biome, usize> 
    pub fn dominant_biome(&self) -> Biome 
    pub fn iter(&self) -> BiomeMapIter<'_> 
pub struct BiomeMapIter<'a> 
pub fn biome_properties(biome: Biome) -> BiomeProperties 
pub fn biome_rule(biome: Biome) -> BiomeRule 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**37 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT
