use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub angle: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProceduralObstacle {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub obstacle_type: String, // "rock" | "coral"
    pub color: String,
    pub vertices: Vec<Vertex>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CurrentVent {
    pub id: i32,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    #[serde(rename = "forceType")]
    #[ts(rename = "forceType")]
    pub force_type: String, // "push" | "pull" | "vortex"
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BiomeArea {
    pub id: String,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(rename = "sporeSpawnRate")]
    #[ts(rename = "sporeSpawnRate")]
    pub spore_spawn_rate: f32,
    #[serde(rename = "sporeEnergyValue")]
    #[ts(rename = "sporeEnergyValue")]
    pub spore_energy_value: f32,
    #[serde(rename = "hazardDamage")]
    #[ts(rename = "hazardDamage")]
    pub hazard_damage: f32,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ProceduralWorld {
    pub seed: String,
    pub width: f32,
    pub height: f32,
    pub obstacles: Vec<ProceduralObstacle>,
    pub vents: Vec<CurrentVent>,
    pub biomes: Vec<BiomeArea>,
}

pub struct ClimatePoint {
    pub biome_id: &'static str,
    pub name: &'static str,
    pub target_toxicity: f32,
    pub target_viscosity: f32,
    pub spore_spawn_rate: f32,
    pub spore_energy_value: f32,
    pub hazard_damage: f32,
    pub color: &'static str,
}

pub const CLIMATE_MATRIX: [ClimatePoint; 4] = [
    ClimatePoint {
        biome_id: "abyssal_barrens",
        name: "💀 Abyssal Barrens",
        target_toxicity: 0.15,
        target_viscosity: 0.15,
        spore_spawn_rate: 0.18,
        spore_energy_value: 30.0,
        hazard_damage: 0.0,
        color: "rgba(15, 23, 42, 0.45)",
    },
    ClimatePoint {
        biome_id: "algae_shallows",
        name: "🌿 Algae Shallows",
        target_toxicity: 0.20,
        target_viscosity: 0.80,
        spore_spawn_rate: 1.40,
        spore_energy_value: 15.0,
        hazard_damage: 0.0,
        color: "rgba(21, 128, 61, 0.16)",
    },
    ClimatePoint {
        biome_id: "acid_pool",
        name: "🧪 Sulphuric Shallows",
        target_toxicity: 0.80,
        target_viscosity: 0.20,
        spore_spawn_rate: 0.65,
        spore_energy_value: 45.0,
        hazard_damage: 0.08,
        color: "rgba(234, 179, 8, 0.14)",
    },
    ClimatePoint {
        biome_id: "cybernetic_vents",
        name: "🔥 Cybernetic Vents",
        target_toxicity: 0.80,
        target_viscosity: 0.80,
        spore_spawn_rate: 0.90,
        spore_energy_value: 50.0,
        hazard_damage: 0.04,
        color: "rgba(168, 85, 247, 0.16)",
    },
];

// --------------------------------------------------------------------------
// Deterministic Hash & PRNG
// --------------------------------------------------------------------------
pub fn hash_string_to_int(s: &str) -> u32 {
    let mut hash: u32 = 0;
    for c in s.chars() {
        hash = hash.wrapping_shl(5).wrapping_sub(hash).wrapping_add(c as u32);
    }
    hash
}

pub struct Mulberry32 {
    a: u32,
}

impl Mulberry32 {
    pub fn new(seed: &str) -> Self {
        let a = hash_string_to_int(seed);
        Self { a }
    }

    pub fn next(&mut self) -> f32 {
        let mut t = self.a.wrapping_add(0x6D2B79F5);
        self.a = t;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61));
        let u = (t ^ (t >> 14)) as f32;
        u / 4294967296.0
    }
}

// --------------------------------------------------------------------------
// Toroidal Fractal Noise
// --------------------------------------------------------------------------
pub fn sample_toroidal_noise(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    seed: &str,
    octave_offset: &str,
) -> f32 {
    let mut rand = Mulberry32::new(&format!("{}{}", seed, octave_offset));

    let mut val = 0.0f32;
    let mut total_amp = 0.0f32;
    let mut amp = 1.0f32;
    let mut freq = 1.0f32;

    for _oct in 0..4 {
        let phase_x = rand.next() * std::f32::consts::PI * 2.0;
        let phase_y = rand.next() * std::f32::consts::PI * 2.0;

        const SCALE: f32 = std::f32::consts::PI * 2.0;
        let wx = (x / width) * SCALE * freq + phase_x;
        let wy = (y / height) * SCALE * freq + phase_y;

        val += (wx.sin() * wy.cos()) * amp;

        total_amp += amp;
        amp *= 0.55;
        freq *= 2.0;
    }

    (val / total_amp) * 0.5 + 0.5
}

// --------------------------------------------------------------------------
// Core Generator
// --------------------------------------------------------------------------
pub fn generate_world(seed: &str, width: f32, height: f32) -> ProceduralWorld {
    let mut rand = Mulberry32::new(seed);

    // A. Generate Organic Biome Grid (Cell-Size 80)
    let mut biomes = Vec::new();
    let cell_size = 80.0f32;
    let cols = (width / cell_size) as i32;
    let rows = (height / cell_size) as i32;

    for c in 0..cols {
        for r in 0..rows {
            let bx = (c as f32) * cell_size;
            let by = (r as f32) * cell_size;

            let tx = bx + cell_size / 2.0;
            let ty = by + cell_size / 2.0;

            let toxicity = sample_toroidal_noise(tx, ty, width, height, seed, "toxicity");
            let viscosity = sample_toroidal_noise(tx, ty, width, height, seed, "viscosity");

            let mut best_climate = &CLIMATE_MATRIX[0];
            let mut min_distance = f32::INFINITY;

            for climate in &CLIMATE_MATRIX {
                let d_t = toxicity - climate.target_toxicity;
                let d_v = viscosity - climate.target_viscosity;
                let distance = (d_t * d_t + d_v * d_v).sqrt();

                if distance < min_distance {
                    min_distance = distance;
                    best_climate = climate;
                }
            }

            biomes.push(BiomeArea {
                id: best_climate.biome_id.to_string(),
                name: best_climate.name.to_string(),
                x: bx,
                y: by,
                width: cell_size,
                height: cell_size,
                spore_spawn_rate: best_climate.spore_spawn_rate,
                spore_energy_value: best_climate.spore_energy_value,
                hazard_damage: best_climate.hazard_damage,
                color: best_climate.color.to_string(),
            });
        }
    }

    // B. Generate Circular solid obstacles (Reefs / Rock barriers)
    let mut obstacles = Vec::new();
    let num_obstacles = 18 + (rand.next() * 10.0).floor() as i32;

    for i in 0..num_obstacles {
        let radius = 180.0 + rand.next() * 260.0;
        let x = radius + rand.next() * (width - radius * 2.0);
        let y = radius + rand.next() * (height - radius * 2.0);

        let type_roll = rand.next();
        let (obstacle_type, color) = if type_roll > 0.6 {
            ("coral".to_string(), "rgba(244, 63, 94, 0.75)".to_string())
        } else {
            ("rock".to_string(), "rgba(100, 116, 139, 0.8)".to_string())
        };

        // Generate jagged, non-circular polygon vertices
        let mut vertices = Vec::new();
        let num_vertices = 5 + (rand.next() * 4.0).floor() as i32; // 5 to 8 vertices
        for j in 0..num_vertices {
            let angle = ((j as f32) / (num_vertices as f32)) * std::f32::consts::PI * 2.0;
            let deform = 0.65 + rand.next() * 0.5; // [65% to 115% of base radius]
            let r = radius * deform;
            vertices.push(Vertex {
                x: x + r * angle.cos(),
                y: y + r * angle.sin(),
                r,
                angle,
            });
        }

        obstacles.push(ProceduralObstacle {
            id: i + 1,
            x,
            y,
            radius,
            obstacle_type,
            color,
            vertices,
        });
    }

    // C. Generate Thermal Current Vents
    let mut vents = Vec::new();
    let num_vents = 6 + (rand.next() * 4.0).floor() as i32;

    for i in 0..num_vents {
        let radius = 600.0 + rand.next() * 800.0;
        let x = radius + rand.next() * (width - radius * 2.0);
        let y = radius + rand.next() * (height - radius * 2.0);

        let force_roll = rand.next();
        let force_type = if force_roll <= 0.35 {
            "push".to_string()
        } else if force_roll <= 0.70 {
            "pull".to_string()
        } else {
            "vortex".to_string()
        };

        let mut strength = 0.08 + rand.next() * 0.16;
        if force_type == "vortex" {
            strength *= 1.4;
        }

        vents.push(CurrentVent {
            id: i + 1,
            x,
            y,
            radius,
            force_type,
            strength,
        });
    }

    ProceduralWorld {
        seed: seed.to_string(),
        width,
        height,
        obstacles,
        vents,
        biomes,
    }
}

// --------------------------------------------------------------------------
// Trainer Specific Lightweight Generator (99.9% smaller payload!)
// --------------------------------------------------------------------------
pub fn generate_trainer_world(seed: &str, width: f32, height: f32) -> ProceduralWorld {
    let mut rand = Mulberry32::new(seed);

    // Generate Circular solid obstacles (Reefs / Rock barriers) deterministically
    let mut obstacles = Vec::new();
    let num_obstacles = 18 + (rand.next() * 10.0).floor() as i32;

    for i in 0..num_obstacles {
        let radius = 180.0 + rand.next() * 260.0;
        let x = radius + rand.next() * (width - radius * 2.0);
        let y = radius + rand.next() * (height - radius * 2.0);

        let type_roll = rand.next();
        let (obstacle_type, color) = if type_roll > 0.6 {
            ("coral".to_string(), "rgba(244, 63, 94, 0.75)".to_string())
        } else {
            ("rock".to_string(), "rgba(100, 116, 139, 0.8)".to_string())
        };

        // Generate jagged, non-circular polygon vertices
        let mut vertices = Vec::new();
        let num_vertices = 5 + (rand.next() * 4.0).floor() as i32; // 5 to 8 vertices
        for j in 0..num_vertices {
            let angle = ((j as f32) / (num_vertices as f32)) * std::f32::consts::PI * 2.0;
            let deform = 0.65 + rand.next() * 0.5; // [65% to 115% of base radius]
            let r = radius * deform;
            vertices.push(Vertex {
                x: x + r * angle.cos(),
                y: y + r * angle.sin(),
                r,
                angle,
            });
        }

        obstacles.push(ProceduralObstacle {
            id: i + 1,
            x,
            y,
            radius,
            obstacle_type,
            color,
            vertices,
        });
    }

    ProceduralWorld {
        seed: seed.to_string(),
        width,
        height,
        obstacles,
        vents: Vec::new(),
        biomes: Vec::new(),
    }
}
