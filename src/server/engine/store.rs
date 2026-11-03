//! Store ports of the engine (TCK-156): the repository traits of TCK-159 (`wiki/contracts/persistence-repo.md`)
//! bundled as `EngineStore`, the store opening with an in-memory fallback, and the Ocean snapshot writer.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};

use crate::persistence::{open_db, CatalogueRepo, LogDbError, MemoryDb, OceanSnapshotRepo, SpeciesRepo, TrainerRepo};
use crate::sim::ocean::OceanSnapshot;

/// Every repository the engine thread uses. Implemented by the SQLite `Db` and the in-memory `MemoryDb`.
pub trait EngineStore: SpeciesRepo + CatalogueRepo + TrainerRepo + OceanSnapshotRepo {}

impl<T: SpeciesRepo + CatalogueRepo + TrainerRepo + OceanSnapshotRepo> EngineStore for T {}

/// The engine's store at `path` (prepared by `init_db` in `main.rs`). If it cannot be opened, the engine runs on an
/// empty `MemoryDb` and the error text is returned for the UI (instead of the former `expect` panic that silently
/// killed the engine thread).
pub fn open_store(path: &Path) -> (Box<dyn EngineStore>, Option<String>) {
    match open_db(path) {
        Ok(db) => (Box::new(db), None),
        Err(e) => {
            log::error!("opening the database '{}' for the engine failed, running without persistence: {}", path.display(), e);
            (Box::new(MemoryDb::new()), Some(e.to_string()))
        }
    }
}

/// A write of the Ocean snapshot (`simulation_state`).
pub enum SnapshotCmd {
    Save(Box<OceanSnapshot>),
    Clear,
}

/// Who writes the Ocean snapshot: the engine's store on the frame thread (`Inline`, tests), or a writer thread with
/// its own connection (`Background`, the app), so the 10 s snapshot is serialised off the frame thread. Saves and
/// clears go through the same channel, so a reset's clear is never overtaken by an older save.
pub enum SnapshotWriter {
    Inline,
    Background(Sender<SnapshotCmd>),
}

impl SnapshotWriter {
    /// Starts the writer thread on the database at `path`; `Inline` if the thread cannot start. If the thread cannot
    /// open the database it ends, and later writes fall back to the engine's store.
    pub fn background(path: PathBuf) -> Self {
        let (tx, rx) = channel::<SnapshotCmd>();
        let spawned = std::thread::Builder::new().name("ocean-snapshot".into()).spawn(move || {
            let db = match open_db(&path) {
                Ok(db) => db,
                Err(e) => {
                    log::error!("Ocean snapshot writer: opening '{}' failed: {}", path.display(), e);
                    return;
                }
            };
            while let Ok(cmd) = rx.recv() {
                write(&db, cmd);
            }
        });
        match spawned {
            Ok(_) => SnapshotWriter::Background(tx),
            Err(e) => {
                log::error!("starting the Ocean snapshot writer failed, writing on the engine thread: {}", e);
                SnapshotWriter::Inline
            }
        }
    }

    pub fn save(&self, snapshot: OceanSnapshot, store: &dyn EngineStore) {
        self.send(SnapshotCmd::Save(Box::new(snapshot)), store);
    }

    pub fn clear(&self, store: &dyn EngineStore) {
        self.send(SnapshotCmd::Clear, store);
    }

    fn send(&self, cmd: SnapshotCmd, store: &dyn EngineStore) {
        match self {
            SnapshotWriter::Inline => write(store, cmd),
            SnapshotWriter::Background(tx) => {
                if let Err(e) = tx.send(cmd) {
                    write(store, e.0);
                }
            }
        }
    }
}

fn write(store: &(impl OceanSnapshotRepo + ?Sized), cmd: SnapshotCmd) {
    match cmd {
        SnapshotCmd::Save(snapshot) => match serde_json::to_string(&*snapshot) {
            Ok(json) => store.save_ocean_snapshot(&json).log_db_err("saving the Ocean snapshot"),
            Err(e) => log::error!("serialising the Ocean snapshot failed: {}", e),
        },
        SnapshotCmd::Clear => store.clear_ocean_state().log_db_err("clearing the Ocean snapshot"),
    }
}

/// The stored Ocean snapshot, if one exists and parses (errors are logged; the Ocean then starts fresh).
pub fn read_snapshot(store: &dyn EngineStore) -> Option<OceanSnapshot> {
    let json = store.ocean_snapshot().unwrap_or_else(|e| {
        log::error!("reading the Ocean snapshot failed: {}", e);
        None
    })?;
    serde_json::from_str(&json).map_err(|e| log::error!("Ocean snapshot unreadable, starting fresh: {}", e)).ok()
}
