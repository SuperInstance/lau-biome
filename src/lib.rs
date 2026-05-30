use rand::Rng;
use rand_xoshiro::Xoshiro256StarStar;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Biome
// ---------------------------------------------------------------------------

/// The 10 distinct ecological zones in the LAU game world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Biome {
    Meadow,
    Forest,
    Desert,
    Tundra,
    Ocean,
    Volcano,
    CrystalCaves,
    FloatingIslands,
    MushroomGrove,
    AncientRuins,
}

impl Biome {
    /// Return all biome variants.
    pub fn all() -> [Biome; 10] {
        [
            Biome::Meadow,
            Biome::Forest,
            Biome::Desert,
            Biome::Tundra,
            Biome::Ocean,
            Biome::Volcano,
            Biome::CrystalCaves,
            Biome::FloatingIslands,
            Biome::MushroomGrove,
            Biome::AncientRuins,
        ]
    }
}

// ---------------------------------------------------------------------------
// Material
// ---------------------------------------------------------------------------

/// Raw materials that can be found in biomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Material {
    Wood,
    Stone,
    Crystal,
    Sand,
    Ice,
    Lava,
    Mushroom,
    AncientMetal,
    CloudStuff,
    Seaweed,
}

// ---------------------------------------------------------------------------
// BiomeProperties
// ---------------------------------------------------------------------------

/// The set of properties that define a biome's character.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiomeProperties {
    pub temperature: f64,
    pub humidity: f64,
    pub base_vibe: f64,
    pub growth_rate: f64,
    pub max_biodiversity: f64,
    pub available_materials: Vec<Material>,
}

impl BiomeProperties {
    /// A vibe value in [0.0, 1.0] that represents how "pleasant" or
    /// "dangerous" the biome feels, derived from its raw base_vibe.
    pub fn normalized_vibe(&self) -> f64 {
        (self.base_vibe.clamp(-1.0, 1.0) + 1.0) * 0.5
    }
}

// ---------------------------------------------------------------------------
// BiomeRule
// ---------------------------------------------------------------------------

/// Associates a biome with its properties and world-generation settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiomeRule {
    pub biome: Biome,
    pub properties: BiomeProperties,
    /// How often agents naturally spawn in this biome (0.0 – 1.0).
    pub spawn_rate: f64,
    /// How tightly conservation rules are enforced: 0.5 = relaxed, 1.0 = strict.
    pub conservation_strictness: f64,
}

// ---------------------------------------------------------------------------
// BiomeTransition
// ---------------------------------------------------------------------------

/// Describes an ecotone zone between two adjacent biomes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiomeTransition {
    pub from: Biome,
    pub to: Biome,
    /// Number of tiles the transition spans.
    pub width: usize,
}

impl BiomeTransition {
    /// Return the interpolated vibe at a position within the transition
    /// zone.  `position` is a normalised distance from `from` (0.0) to
    /// `to` (1.0).  Uses a smoothstep-like cubic interpolation.
    pub fn transition_vibe(&self, position: f64, from_props: &BiomeProperties, to_props: &BiomeProperties) -> f64 {
        let pos = position.clamp(0.0, 1.0);
        // Smooth Hermite interpolation.
        let t = pos * pos * (3.0 - 2.0 * pos);
        from_props.base_vibe + t * (to_props.base_vibe - from_props.base_vibe)
    }
}

// ---------------------------------------------------------------------------
// BiomeMap
// ---------------------------------------------------------------------------

/// A 2D grid of biomes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiomeMap {
    pub width: usize,
    pub height: usize,
    pub biomes: Vec<Vec<Biome>>,
}

impl BiomeMap {
    /// Build a biome map from a raw grid (width / height are inferred).
    pub fn from_raw(grid: Vec<Vec<Biome>>) -> Self {
        let height = grid.len();
        let width = if height > 0 { grid[0].len() } else { 0 };
        BiomeMap { width, height, biomes: grid }
    }

    /// Generate a biome map of the given size using seeded value noise.
    /// Each cell's biome is determined by the dominant noise value across
    /// several octaves, with a per-biome offset derived from the seed.
    pub fn generate(seed: u64, width: usize, height: usize) -> Self {
        // Build a seeded permutation table for deterministic noise that
        // varies with the seed.
        let perm = Permutation::new(seed);

        let mut biomes = Vec::with_capacity(height);
        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let bx = x as f64 / width as f64;
                let by = y as f64 / height as f64;

                // Compute a noise-like value per biome and pick the highest.
                let weights = Biome::all().map(|b| {
                    let idx = b as usize;
                    let mut n = seeded_noise(&perm, bx * 4.0 + by * 2.0, bx * 3.0 - by * 5.0, idx);
                    n += 0.3 * seeded_noise(&perm, bx * 8.0 + 1.7, by * 8.0 + 3.1, idx);
                    n
                });

                let (best_idx, _) = weights
                    .iter()
                    .enumerate()
                    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                    .unwrap();

                row.push(Biome::all()[best_idx]);
            }
            biomes.push(row);
        }

        BiomeMap { width, height, biomes }
    }

    /// Look up the biome at the given coordinates.  Panics on out-of-bounds.
    pub fn get_biome(&self, x: usize, y: usize) -> Biome {
        self.biomes[y][x]
    }

    /// Return the biome at the approximate centre of the map.
    pub fn biome_at_center(&self) -> Biome {
        let cx = self.width / 2;
        let cy = self.height / 2;
        self.get_biome(cx, cy)
    }

    /// Count how many tiles each biome occupies.
    pub fn biome_counts(&self) -> HashMap<Biome, usize> {
        let mut counts = HashMap::new();
        for row in &self.biomes {
            for &b in row {
                *counts.entry(b).or_insert(0) += 1;
            }
        }
        counts
    }

    /// Return the biome that covers the most tiles.
    pub fn dominant_biome(&self) -> Biome {
        let counts = self.biome_counts();
        counts
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(b, _)| b)
            .expect("biome map is empty")
    }

    /// Return an iterator over `(x, y, biome)` tuples.
    pub fn iter(&self) -> BiomeMapIter<'_> {
        BiomeMapIter {
            map: self,
            x: 0,
            y: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// BiomeMapIter
// ---------------------------------------------------------------------------

pub struct BiomeMapIter<'a> {
    map: &'a BiomeMap,
    x: usize,
    y: usize,
}

impl<'a> Iterator for BiomeMapIter<'a> {
    type Item = (usize, usize, Biome);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y >= self.map.height {
            return None;
        }
        let item = (self.x, self.y, self.map.biomes[self.y][self.x]);
        self.x += 1;
        if self.x >= self.map.width {
            self.x = 0;
            self.y += 1;
        }
        Some(item)
    }
}

// ---------------------------------------------------------------------------
// Permutation-table noise (seed-aware)
// ---------------------------------------------------------------------------

/// A seeded permutation table (size 256) for generating reproducible
/// value noise that varies with the seed.
struct Permutation {
    table: [usize; 512],
}

impl Permutation {
    fn new(seed: u64) -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(seed);
        let mut p: [usize; 256] = std::array::from_fn(|i| i);
        // Fisher-Yates shuffle seeded by the seed.
        for i in (1..256).rev() {
            let j = rng.gen_range(0..=i);
            p.swap(i, j);
        }
        // Double up to avoid wrapping.
        let mut table = [0usize; 512];
        for i in 0..512 {
            table[i] = p[i & 255];
        }
        Permutation { table }
    }

    fn hash(&self, x: i64, y: i64, offset: usize) -> f64 {
        let ux = x.wrapping_rem(256) as usize;
        let uy = y.wrapping_rem(256) as usize;
        let a = self.table[ux & 255];
        let b = a.wrapping_add(uy) & 255;
        let c = self.table[b];
        let idx = c.wrapping_add(offset * 7 + 13);
        (idx & 0xFF) as f64 / 256.0
    }
}

/// Seeded value noise using a permutation table.
fn seeded_noise(perm: &Permutation, x: f64, y: f64, offset: usize) -> f64 {
    let ix = x.floor() as i64;
    let iy = y.floor() as i64;
    let fx = x - x.floor();
    let fy = y - y.floor();
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);

    let v00 = perm.hash(ix, iy, offset);
    let v10 = perm.hash(ix + 1, iy, offset);
    let v01 = perm.hash(ix, iy + 1, offset);
    let v11 = perm.hash(ix + 1, iy + 1, offset);

    let vx0 = v00 + sx * (v10 - v00);
    let vx1 = v01 + sx * (v11 - v01);
    vx0 + sy * (vx1 - vx0)
}

// ---------------------------------------------------------------------------
// Pre-built properties for all 10 biomes
// ---------------------------------------------------------------------------

/// Return the canonical `BiomeProperties` for a given biome.
pub fn biome_properties(biome: Biome) -> BiomeProperties {
    match biome {
        Biome::Meadow => BiomeProperties {
            temperature: 0.6,
            humidity: 0.5,
            base_vibe: 0.3,
            growth_rate: 0.8,
            max_biodiversity: 0.7,
            available_materials: vec![Material::Wood, Material::Stone],
        },
        Biome::Forest => BiomeProperties {
            temperature: 0.5,
            humidity: 0.7,
            base_vibe: 0.1,
            growth_rate: 0.9,
            max_biodiversity: 0.9,
            available_materials: vec![Material::Wood, Material::Stone, Material::Mushroom],
        },
        Biome::Desert => BiomeProperties {
            temperature: 0.9,
            humidity: 0.1,
            base_vibe: -0.2,
            growth_rate: 0.1,
            max_biodiversity: 0.2,
            available_materials: vec![Material::Sand, Material::Stone],
        },
        Biome::Tundra => BiomeProperties {
            temperature: 0.1,
            humidity: 0.3,
            base_vibe: -0.3,
            growth_rate: 0.15,
            max_biodiversity: 0.2,
            available_materials: vec![Material::Ice, Material::Stone],
        },
        Biome::Ocean => BiomeProperties {
            temperature: 0.5,
            humidity: 1.0,
            base_vibe: 0.0,
            growth_rate: 0.3,
            max_biodiversity: 0.6,
            available_materials: vec![Material::Seaweed, Material::Stone],
        },
        Biome::Volcano => BiomeProperties {
            temperature: 1.0,
            humidity: 0.2,
            base_vibe: -0.8,
            growth_rate: 0.05,
            max_biodiversity: 0.1,
            available_materials: vec![Material::Lava, Material::Stone, Material::Crystal],
        },
        Biome::CrystalCaves => BiomeProperties {
            temperature: 0.3,
            humidity: 0.4,
            base_vibe: 0.8,
            growth_rate: 0.4,
            max_biodiversity: 0.4,
            available_materials: vec![Material::Crystal, Material::Stone],
        },
        Biome::FloatingIslands => BiomeProperties {
            temperature: 0.4,
            humidity: 0.6,
            base_vibe: 0.6,
            growth_rate: 0.6,
            max_biodiversity: 0.5,
            available_materials: vec![Material::CloudStuff, Material::Stone, Material::Crystal],
        },
        Biome::MushroomGrove => BiomeProperties {
            temperature: 0.4,
            humidity: 0.8,
            base_vibe: 0.4,
            growth_rate: 0.7,
            max_biodiversity: 0.8,
            available_materials: vec![Material::Mushroom, Material::Wood],
        },
        Biome::AncientRuins => BiomeProperties {
            temperature: 0.5,
            humidity: 0.3,
            base_vibe: 0.2,
            growth_rate: 0.2,
            max_biodiversity: 0.3,
            available_materials: vec![Material::AncientMetal, Material::Stone, Material::Crystal],
        },
    }
}

/// Return the canonical `BiomeRule` for a given biome.
pub fn biome_rule(biome: Biome) -> BiomeRule {
    let properties = biome_properties(biome);
    let (spawn_rate, conservation_strictness) = match biome {
        Biome::Meadow => (0.7, 0.4),
        Biome::Forest => (0.8, 0.6),
        Biome::Desert => (0.3, 0.3),
        Biome::Tundra => (0.2, 0.5),
        Biome::Ocean => (0.5, 0.5),
        Biome::Volcano => (0.1, 0.2),
        Biome::CrystalCaves => (0.4, 0.9),
        Biome::FloatingIslands => (0.5, 0.7),
        Biome::MushroomGrove => (0.6, 0.6),
        Biome::AncientRuins => (0.3, 0.8),
    };
    BiomeRule {
        biome,
        properties,
        spawn_rate,
        conservation_strictness,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Biome enum ---------------------------------------------------------

    #[test]
    fn test_biome_all_contains_10() {
        assert_eq!(Biome::all().len(), 10);
    }

    #[test]
    fn test_biome_all_contains_meadow() {
        assert!(Biome::all().contains(&Biome::Meadow));
    }

    #[test]
    fn test_biome_serde_roundtrip() {
        let json = serde_json::to_string(&Biome::CrystalCaves).unwrap();
        let back: Biome = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Biome::CrystalCaves);
    }

    // -- BiomeProperties ----------------------------------------------------

    #[test]
    fn test_normalized_vibe_positive() {
        let props = biome_properties(Biome::Meadow);
        let nv = props.normalized_vibe();
        assert!((0.0..=1.0).contains(&nv));
        assert!((nv - 0.65).abs() < 1e-10);
    }

    #[test]
    fn test_normalized_vibe_negative() {
        let props = biome_properties(Biome::Desert);
        let nv = props.normalized_vibe();
        assert!((0.0..=1.0).contains(&nv));
        assert!((nv - 0.4).abs() < 1e-10);
    }

    #[test]
    fn test_properties_serde_roundtrip() {
        let props = biome_properties(Biome::Forest);
        let json = serde_json::to_string(&props).unwrap();
        let back: BiomeProperties = serde_json::from_str(&json).unwrap();
        assert_eq!(props.temperature, back.temperature);
        assert_eq!(props.humidity, back.humidity);
        assert_eq!(props.available_materials.len(), back.available_materials.len());
    }

    // -- Material -----------------------------------------------------------

    #[test]
    fn test_material_serde_roundtrip() {
        let json = serde_json::to_string(&Material::Crystal).unwrap();
        let back: Material = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Material::Crystal);
    }

    // -- BiomeRule ----------------------------------------------------------

    #[test]
    fn test_biome_rule_meadow() {
        let rule = biome_rule(Biome::Meadow);
        assert_eq!(rule.biome, Biome::Meadow);
        assert!((rule.spawn_rate - 0.7).abs() < 1e-10);
        assert!((rule.conservation_strictness - 0.4).abs() < 1e-10);
    }

    #[test]
    fn test_biome_rule_crystal_caves_conservation_strict() {
        let rule = biome_rule(Biome::CrystalCaves);
        assert!((rule.conservation_strictness - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_biome_rule_serde_roundtrip() {
        let rule = biome_rule(Biome::Volcano);
        let json = serde_json::to_string(&rule).unwrap();
        let back: BiomeRule = serde_json::from_str(&json).unwrap();
        assert_eq!(rule.biome, back.biome);
        assert_eq!(rule.spawn_rate, back.spawn_rate);
        assert_eq!(rule.conservation_strictness, back.conservation_strictness);
    }

    // -- BiomeTransition ----------------------------------------------------

    #[test]
    fn test_transition_vibe_start() {
        let trans = BiomeTransition {
            from: Biome::Meadow,
            to: Biome::Desert,
            width: 5,
        };
        let from_p = biome_properties(Biome::Meadow);
        let to_p = biome_properties(Biome::Desert);
        let vibe = trans.transition_vibe(0.0, &from_p, &to_p);
        assert!((vibe - from_p.base_vibe).abs() < 1e-10);
    }

    #[test]
    fn test_transition_vibe_end() {
        let trans = BiomeTransition {
            from: Biome::Meadow,
            to: Biome::Desert,
            width: 5,
        };
        let from_p = biome_properties(Biome::Meadow);
        let to_p = biome_properties(Biome::Desert);
        let vibe = trans.transition_vibe(1.0, &from_p, &to_p);
        assert!((vibe - to_p.base_vibe).abs() < 1e-10);
    }

    #[test]
    fn test_transition_vibe_midpoint() {
        let trans = BiomeTransition {
            from: Biome::Meadow,
            to: Biome::Desert,
            width: 5,
        };
        let from_p = biome_properties(Biome::Meadow);
        let to_p = biome_properties(Biome::Desert);
        let vibe = trans.transition_vibe(0.5, &from_p, &to_p);
        // At 0.5, smoothstep gives exactly 0.5, so vibe is mid.
        let expected = from_p.base_vibe + 0.5 * (to_p.base_vibe - from_p.base_vibe);
        assert!((vibe - expected).abs() < 1e-10);
    }

    #[test]
    fn test_transition_clamps_position() {
        let trans = BiomeTransition {
            from: Biome::Forest,
            to: Biome::Tundra,
            width: 3,
        };
        let from_p = biome_properties(Biome::Forest);
        let to_p = biome_properties(Biome::Tundra);
        let vibe_over = trans.transition_vibe(2.0, &from_p, &to_p);
        let vibe_under = trans.transition_vibe(-0.5, &from_p, &to_p);
        assert!((vibe_over - to_p.base_vibe).abs() < 1e-10);
        assert!((vibe_under - from_p.base_vibe).abs() < 1e-10);
    }

    #[test]
    fn test_transition_serde_roundtrip() {
        let trans = BiomeTransition {
            from: Biome::FloatingIslands,
            to: Biome::MushroomGrove,
            width: 4,
        };
        let json = serde_json::to_string(&trans).unwrap();
        let back: BiomeTransition = serde_json::from_str(&json).unwrap();
        assert_eq!(trans.from, back.from);
        assert_eq!(trans.width, back.width);
    }

    // -- BiomeMap -----------------------------------------------------------

    #[test]
    fn test_biome_map_generate_dimensions() {
        let map = BiomeMap::generate(42, 16, 12);
        assert_eq!(map.width, 16);
        assert_eq!(map.height, 12);
        assert_eq!(map.biomes.len(), 12);
        assert_eq!(map.biomes[0].len(), 16);
    }

    #[test]
    fn test_biome_map_generate_deterministic() {
        let a = BiomeMap::generate(42, 32, 32);
        let b = BiomeMap::generate(42, 32, 32);
        assert_eq!(a.biomes, b.biomes);
    }

    #[test]
    fn test_biome_map_generate_different_seed() {
        let a = BiomeMap::generate(1, 16, 16);
        let b = BiomeMap::generate(2, 16, 16);
        // Almost certainly different.
        assert_ne!(a.biomes, b.biomes);
    }

    #[test]
    fn test_biome_map_get_biome() {
        let map = BiomeMap::generate(42, 10, 10);
        let b = map.get_biome(3, 5);
        assert!(Biome::all().contains(&b));
    }

    #[test]
    fn test_biome_map_center() {
        let map = BiomeMap::generate(99, 7, 7);
        let center = map.biome_at_center();
        assert_eq!(center, map.get_biome(3, 3));
    }

    #[test]
    fn test_biome_map_from_raw() {
        let grid = vec![
            vec![Biome::Meadow, Biome::Forest],
            vec![Biome::Desert, Biome::Ocean],
        ];
        let map = BiomeMap::from_raw(grid.clone());
        assert_eq!(map.width, 2);
        assert_eq!(map.height, 2);
        assert_eq!(map.biomes, grid);
    }

    #[test]
    fn test_biome_map_counts_sum() {
        let map = BiomeMap::generate(42, 8, 8);
        let counts = map.biome_counts();
        let total: usize = counts.values().sum();
        assert_eq!(total, 64);
    }

    #[test]
    fn test_biome_map_dominant_has_majority() {
        let grid = vec![
            vec![Biome::Desert, Biome::Desert, Biome::Desert],
            vec![Biome::Desert, Biome::Meadow, Biome::Meadow],
        ];
        let map = BiomeMap::from_raw(grid);
        assert_eq!(map.dominant_biome(), Biome::Desert);
    }

    #[test]
    fn test_biome_map_empty_dominant_panics() {
        let map = BiomeMap::from_raw(vec![]);
        let result = std::panic::catch_unwind(|| map.dominant_biome());
        assert!(result.is_err());
    }

    #[test]
    fn test_biome_map_iter_yields_all_cells() {
        let map = BiomeMap::generate(7, 5, 4);
        let count = map.iter().count();
        assert_eq!(count, 5 * 4);
    }

    #[test]
    fn test_biome_map_iter_items_valid() {
        let map = BiomeMap::generate(7, 5, 4);
        for (x, y, b) in map.iter() {
            assert!(x < 5);
            assert!(y < 4);
            assert_eq!(b, map.get_biome(x, y));
        }
    }

    #[test]
    fn test_biome_map_serde_roundtrip() {
        let map = BiomeMap::generate(42, 8, 8);
        let json = serde_json::to_string(&map).unwrap();
        let back: BiomeMap = serde_json::from_str(&json).unwrap();
        assert_eq!(map.width, back.width);
        assert_eq!(map.height, back.height);
        assert_eq!(map.biomes, back.biomes);
    }

    // -- Pre-built properties -----------------------------------------------

    #[test]
    fn test_meadow_properties() {
        let p = biome_properties(Biome::Meadow);
        assert!((p.temperature - 0.6).abs() < 1e-10);
        assert!((p.humidity - 0.5).abs() < 1e-10);
        assert!((p.growth_rate - 0.8).abs() < 1e-10);
        assert!(p.available_materials.contains(&Material::Wood));
    }

    #[test]
    fn test_desert_properties() {
        let p = biome_properties(Biome::Desert);
        assert!((p.temperature - 0.9).abs() < 1e-10);
        assert!((p.humidity - 0.1).abs() < 1e-10);
        assert!((p.growth_rate - 0.1).abs() < 1e-10);
        assert!(p.available_materials.contains(&Material::Sand));
    }

    #[test]
    fn test_crystal_caves_properties() {
        let p = biome_properties(Biome::CrystalCaves);
        assert!((p.temperature - 0.3).abs() < 1e-10);
        assert!((p.humidity - 0.4).abs() < 1e-10);
        assert!((p.base_vibe - 0.8).abs() < 1e-10);
        assert!(p.available_materials.contains(&Material::Crystal));
    }

    #[test]
    fn test_ocean_properties() {
        let p = biome_properties(Biome::Ocean);
        assert!((p.humidity - 1.0).abs() < 1e-10);
        assert!(p.available_materials.contains(&Material::Seaweed));
    }

    #[test]
    fn test_volcano_properties() {
        let p = biome_properties(Biome::Volcano);
        assert!((p.temperature - 1.0).abs() < 1e-10);
        assert!((p.base_vibe - (-0.8)).abs() < 1e-10);
        assert!(p.available_materials.contains(&Material::Lava));
    }

    #[test]
    fn test_tundra_properties() {
        let p = biome_properties(Biome::Tundra);
        assert!((p.temperature - 0.1).abs() < 1e-10);
        assert!(p.available_materials.contains(&Material::Ice));
    }

    #[test]
    fn test_floating_islands_properties() {
        let p = biome_properties(Biome::FloatingIslands);
        assert!(p.available_materials.contains(&Material::CloudStuff));
    }

    #[test]
    fn test_mushroom_grove_properties() {
        let p = biome_properties(Biome::MushroomGrove);
        assert!((p.humidity - 0.8).abs() < 1e-10);
        assert!((p.max_biodiversity - 0.8).abs() < 1e-10);
        assert!(p.available_materials.contains(&Material::Mushroom));
    }

    #[test]
    fn test_ancient_ruins_properties() {
        let p = biome_properties(Biome::AncientRuins);
        assert!(p.available_materials.contains(&Material::AncientMetal));
        assert!(p.available_materials.contains(&Material::Crystal));
    }

    // -- All biome rules available ------------------------------------------

    #[test]
    fn test_all_biomes_have_rules() {
        for b in Biome::all() {
            let rule = biome_rule(b);
            assert_eq!(rule.biome, b);
            assert!((0.0..=1.0).contains(&rule.spawn_rate));
            assert!((0.0..=1.0).contains(&rule.conservation_strictness));
        }
    }
}
