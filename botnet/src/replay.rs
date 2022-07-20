use crate::bot_actions::BotAction;
use crate::config::RECORDING_QUEUE_MESSAGE_LIMIT;
use botnet_api::{Bay, EntityID};
use crossbeam_channel::{bounded as bounded_channel, RecvError, Sender};
use rkyv::{Archive, Deserialize, Serialize};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::mem::ManuallyDrop;
use std::thread::{self, JoinHandle};

/// Records a replay of a game so that it can later be visualized with `botnet-replay-viewer`.
pub struct ReplayRecorder {
    record_sender: ManuallyDrop<Sender<ReplayRecord>>,
    recording_thread: ManuallyDrop<JoinHandle<()>>,
}

#[derive(Archive, Serialize, Deserialize)]
pub enum ReplayRecord {
    GameVersion(Box<str>),
    InitialNextEntityID(u64),
    InitialBayState {
        bay_id: EntityID,
        bay: Box<Bay>,
    },
    TickStart,
    BotAction {
        bay_id: EntityID,
        bot_id: EntityID,
        bot_action: BotAction,
    },
    RechargeBots {
        bay_id: EntityID,
        bot_ids: Box<[EntityID]>,
    },
}

impl ReplayRecorder {
    pub fn new(initial_bays: &[(EntityID, &Bay)], initial_next_entity_id: u64) -> Self {
        let (recording_sender, record_receiver) = bounded_channel(RECORDING_QUEUE_MESSAGE_LIMIT);

        // Start a background thread to take care of writing ReplayRecords to a file
        // Records are prepended with their size in bytes, written as a little-endian integer
        let recording_thread = thread::spawn(move || {
            let mut replay_file = BufWriter::new(File::create("example.rplay").unwrap());

            loop {
                match record_receiver.recv() {
                    Ok(record) => {
                        // TODO: Measure and tweak scratch space
                        let record = rkyv::to_bytes::<_, 1024>(&record).unwrap();
                        let record_len = (record.len() as u64).to_le_bytes();
                        replay_file.write_all(&record_len).unwrap();
                        replay_file.write_all(&record).unwrap();
                    }
                    Err(RecvError) => break,
                }
            }

            replay_file.flush().unwrap();
        });

        let this = Self {
            record_sender: ManuallyDrop::new(recording_sender),
            recording_thread: ManuallyDrop::new(recording_thread),
        };

        this.record_game_version();
        this.record_initial_next_entity_id(initial_next_entity_id);
        this.record_initial_bay_states(initial_bays);

        this
    }

    fn record_game_version(&self) {
        let game_version = env!("CARGO_PKG_VERSION").into();
        self.record_sender
            .send(ReplayRecord::GameVersion(game_version))
            .unwrap();
    }

    fn record_initial_next_entity_id(&self, initial_next_entity_id: u64) {
        self.record_sender
            .send(ReplayRecord::InitialNextEntityID(initial_next_entity_id))
            .unwrap();
    }

    fn record_initial_bay_states(&self, bays: &[(EntityID, &Bay)]) {
        for (bay_id, bay) in bays {
            self.record_sender
                .send(ReplayRecord::InitialBayState {
                    bay_id: *bay_id,
                    bay: Box::new((**bay).clone()),
                })
                .unwrap();
        }
    }

    pub fn record_tick_start(&self) {
        self.record_sender.send(ReplayRecord::TickStart).unwrap();
    }

    pub fn record_bot_action(&self, bay_id: EntityID, bot_id: EntityID, bot_action: BotAction) {
        self.record_sender
            .send(ReplayRecord::BotAction {
                bay_id,
                bot_id,
                bot_action,
            })
            .unwrap();
    }

    pub fn record_recharge_bots(&self, bay_id: EntityID, bot_ids: &[EntityID]) {
        self.record_sender
            .send(ReplayRecord::RechargeBots {
                bay_id,
                bot_ids: bot_ids.clone().into(),
            })
            .unwrap();
    }
}

impl Drop for ReplayRecorder {
    fn drop(&mut self) {
        unsafe {
            // Signal to the recording thread to stop
            drop(ManuallyDrop::take(&mut self.record_sender));

            // Wait for the recording thread to finish
            ManuallyDrop::take(&mut self.recording_thread)
                .join()
                .expect("Recording thread failed");
        }
    }
}
