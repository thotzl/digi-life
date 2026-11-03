//! `SpeciesLedger` (TCK-156): the Ocean's pure mirror of the stored species (`species_records`) and the diff the
//! store applies. Replaces the seven species inserts and the cache re-read of `engine.rs`: the ledger follows the
//! store's insert rule itself (a species is new unless its genome id or its latin name is known), so it never has to
//! read the store back after an insert. The engine loads it from the store at startup and after a retention prune.

use std::collections::{HashMap, HashSet};

use crate::shared::dto::CreatureAgent;

/// One tracked species (id = genome).
#[derive(Debug, Clone, PartialEq)]
pub struct LedgerEntry {
    pub id: String,
    pub latin_name: String,
    pub alive: bool,
    pub peak: i32,
}

/// A change for the species store, in the order it happened.
#[derive(Debug, Clone, PartialEq)]
pub enum SpeciesChange {
    /// A species seen for the first time (stored alive with peak 1)
    Discovered {
        genome: String,
        latin_name: String,
        parent_name: Option<String>,
        generation: u32,
        carnivory: f32,
    },
    /// New peak population
    Peak { id: String, peak: i32 },
    /// An extinct species has living members again
    Revived { id: String },
    /// The last member died
    Extinct { id: String, latin_name: String },
}

/// Stored species in insertion order.
#[derive(Debug, Clone, Default)]
pub struct SpeciesLedger {
    entries: Vec<LedgerEntry>,
    names: HashSet<String>,
    ids: HashSet<String>,
    /// Census scratch (no allocation per tick beyond the keys)
    counts: HashMap<String, i32>,
}

impl SpeciesLedger {
    /// Replaces the ledger with the stored species (insertion order).
    pub fn load(&mut self, entries: impl IntoIterator<Item = LedgerEntry>) {
        self.clear();
        for entry in entries {
            self.ids.insert(entry.id.clone());
            self.names.insert(entry.latin_name.clone());
            self.entries.push(entry);
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.names.clear();
        self.ids.clear();
    }

    pub fn entries(&self) -> &[LedgerEntry] {
        &self.entries
    }

    /// Records `agent`'s species if it is new (`Discovered`); a known genome or latin name is ignored like the store's
    /// `INSERT OR IGNORE`.
    pub fn discover(&mut self, agent: &CreatureAgent, parent_name: Option<&str>, out: &mut Vec<SpeciesChange>) {
        let name = &agent.phenotype.latin_name;
        if self.ids.contains(&agent.species_id) || self.names.contains(name) {
            return;
        }
        self.ids.insert(agent.species_id.clone());
        self.names.insert(name.clone());
        self.entries.push(LedgerEntry {
            id: agent.species_id.clone(),
            latin_name: name.clone(),
            alive: true,
            peak: 1,
        });
        out.push(SpeciesChange::Discovered {
            genome: agent.genome.clone(),
            latin_name: name.clone(),
            parent_name: parent_name.map(str::to_string),
            generation: agent.generation,
            carnivory: agent.phenotype.carnivory,
        });
    }

    /// Census of the living population: new peaks, revived and extinct species, in ledger order.
    pub fn census(&mut self, creatures: &[CreatureAgent], out: &mut Vec<SpeciesChange>) {
        self.counts.clear();
        for c in creatures {
            *self.counts.entry(c.species_id.clone()).or_insert(0) += 1;
        }
        for entry in &mut self.entries {
            let count = self.counts.get(&entry.id).copied().unwrap_or(0);
            if count > 0 {
                if count > entry.peak {
                    entry.peak = count;
                    out.push(SpeciesChange::Peak {
                        id: entry.id.clone(),
                        peak: count,
                    });
                }
                if !entry.alive {
                    entry.alive = true;
                    out.push(SpeciesChange::Revived { id: entry.id.clone() });
                }
            } else if entry.alive {
                entry.alive = false;
                out.push(SpeciesChange::Extinct {
                    id: entry.id.clone(),
                    latin_name: entry.latin_name.clone(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biology::creature_factory::{SpawnOptions, spawn_creature};
    use crate::biology::genome::WILDTYPE_GENOME;

    fn agent(id: u32) -> CreatureAgent {
        spawn_creature(
            WILDTYPE_GENOME,
            SpawnOptions {
                id,
                px: 0.0,
                py: 0.0,
                heading_angle: 0.0,
                generation: 2,
                methylations: None,
                synapse_weights: None,
            },
            crate::config::test_config(),
        )
    }

    #[test]
    fn test_ledger_discovers_once_and_diffs_peak_extinction_and_revival() {
        let mut ledger = SpeciesLedger::default();
        let mut changes = Vec::new();
        let a = agent(1);
        ledger.discover(&a, Some("Parentus"), &mut changes);
        ledger.discover(&a, None, &mut changes);
        assert!(
            matches!(&changes[..], [SpeciesChange::Discovered { generation: 2, parent_name: Some(p), .. }] if p == "Parentus"),
            "{:?}",
            changes
        );
        // Same latin name under another genome id: ignored like the store's unique name
        let mut renamed = agent(2);
        renamed.species_id = "other-genome".into();
        ledger.discover(&renamed, None, &mut changes);
        assert_eq!(changes.len(), 1);

        changes.clear();
        ledger.census(&[a.clone(), agent(3)], &mut changes);
        assert_eq!(
            changes,
            vec![SpeciesChange::Peak {
                id: a.species_id.clone(),
                peak: 2
            }]
        );
        changes.clear();
        ledger.census(&[a.clone()], &mut changes);
        assert!(changes.is_empty(), "no new peak");
        ledger.census(&[], &mut changes);
        assert!(
            matches!(&changes[..], [SpeciesChange::Extinct { latin_name, .. }] if *latin_name == a.phenotype.latin_name)
        );
        changes.clear();
        ledger.census(&[a.clone()], &mut changes);
        assert_eq!(
            changes,
            vec![SpeciesChange::Revived {
                id: a.species_id.clone()
            }]
        );

        ledger.load([LedgerEntry {
            id: "x".into(),
            latin_name: "Xus".into(),
            alive: false,
            peak: 4,
        }]);
        assert_eq!(ledger.entries().len(), 1);
        changes.clear();
        ledger.discover(&a, None, &mut changes);
        assert_eq!(changes.len(), 1, "a reload forgets species the store pruned");
    }
}
