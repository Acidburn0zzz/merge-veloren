extern crate diesel;

use super::{
    error::Error,
    establish_connection,
    models::{
        Achievement as AchievementModel, CharacterAchievement, DataMigration, NewAchievement,
        NewDataMigration,
    },
    schema,
};
use common::comp::{self, AchievementItem};
use crossbeam::{channel, channel::TryIter};
use diesel::{
    prelude::*,
    result::{DatabaseErrorKind, Error as DieselError},
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use tracing::{error, info, warn};

/// Available database operations when modifying a player's characetr list
enum AchievementLoaderRequestKind {
    LoadCharacterAchievementList {
        entity: specs::Entity,
        character_id: i32,
    },
}

type LoadCharacterAchievementsResult = (specs::Entity, Result<Vec<comp::Achievement>, Error>);

/// Wrapper for results
#[derive(Debug)]
pub enum AchievementLoaderResponse {
    LoadCharacterAchievementListResponse(LoadCharacterAchievementsResult),
}

pub struct AchievementLoader {
    update_rx: Option<channel::Receiver<AchievementLoaderResponse>>,
    update_tx: Option<channel::Sender<AchievementLoaderRequestKind>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl AchievementLoader {
    pub fn new(db_dir: String) -> Self {
        let (update_tx, internal_rx) = channel::unbounded::<AchievementLoaderRequestKind>();
        let (internal_tx, update_rx) = channel::unbounded::<AchievementLoaderResponse>();

        let handle = std::thread::spawn(move || {
            while let Ok(request) = internal_rx.recv() {
                if let Err(e) = internal_tx.send(match request {
                    AchievementLoaderRequestKind::LoadCharacterAchievementList {
                        entity,
                        character_id,
                    } => AchievementLoaderResponse::LoadCharacterAchievementListResponse((
                        entity,
                        load_character_achievement_list(character_id, &db_dir),
                    )),
                }) {
                    error!(?e, "Could not send send persistence request");
                }
            }
        });

        Self {
            update_tx: Some(update_tx),
            update_rx: Some(update_rx),
            handle: Some(handle),
        }
    }

    pub fn load_character_achievement_list(&self, entity: specs::Entity, character_id: i32) {
        if let Err(e) = self.update_tx.as_ref().unwrap().send(
            AchievementLoaderRequestKind::LoadCharacterAchievementList {
                entity,
                character_id,
            },
        ) {
            error!(?e, "Could not send character achievement load request");
        }
    }

    /// Returns a non-blocking iterator over AchievementLoaderResponse messages
    pub fn messages(&self) -> TryIter<AchievementLoaderResponse> {
        self.update_rx.as_ref().unwrap().try_iter()
    }
}

impl Drop for AchievementLoader {
    fn drop(&mut self) {
        drop(self.update_tx.take());
        if let Err(e) = self.handle.take().unwrap().join() {
            error!(?e, "Error from joining character loader thread");
        }
    }
}

fn load_character_achievement_list(
    character_id: i32,
    db_dir: &str,
) -> Result<Vec<comp::Achievement>, Error> {
    let character_achievements = schema::character_achievement::dsl::character_achievement
        .filter(schema::character_achievement::character_id.eq(character_id))
        .load::<CharacterAchievement>(&establish_connection(db_dir))?;

    Ok(character_achievements
        .iter()
        .map(comp::Achievement::from)
        .collect::<Vec<comp::Achievement>>())
}

pub fn sync(db_dir: &str) -> Result<Vec<AchievementModel>, Error> {
    let achievements = load_data();
    let connection = establish_connection(db_dir);

    // Use the full dataset for checks
    let persisted_achievements =
        schema::achievement::dsl::achievement.load::<AchievementModel>(&connection)?;

    // Get a hash of the Vec<Achievement> to compare to the migration table
    let result = schema::data_migration::dsl::data_migration
        .filter(schema::data_migration::title.eq(String::from("achievements")))
        .first::<DataMigration>(&connection);

    info!(?result, "result: ");

    let should_run = match result {
        Ok(migration_entry) => {
            let checksum = &migration_entry.checksum;
            info!(?checksum, "checksum: ");

            migration_entry.checksum != hash(&achievements).to_string()
        },
        Err(diesel::result::Error::NotFound) => {
            let migration = NewDataMigration {
                title: "achievements",
                checksum: &hash(&achievements).to_string(),
                last_run: chrono::Utc::now().naive_utc(),
            };

            diesel::insert_into(schema::data_migration::table)
                .values(&migration)
                .execute(&connection)?;

            true
        },
        Err(_) => {
            error!("Failed to run migrations"); // TODO better error messaging

            false
        },
    };

    if (should_run || persisted_achievements.is_empty()) && !achievements.is_empty() {
        let items = &achievements;

        info!(?items, "Achievements need updating...");

        // Make use of the unique constraint in the DB, attempt to insert, on unique
        // failure check if it needs updating and do so if necessary
        for item in &achievements {
            let new_item = NewAchievement::from(item);

            if let Err(error) = diesel::insert_into(schema::achievement::table)
                .values(&new_item)
                .execute(&connection)
            {
                match error {
                    DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                        let entry = persisted_achievements
                            .iter()
                            .find(|&a| &a.checksum == &new_item.checksum);

                        if let Some(existing_item) = entry {
                            if existing_item.details != new_item.details {
                                match diesel::update(
                                    schema::achievement::dsl::achievement.filter(
                                        schema::achievement::checksum
                                            .eq(String::from(&existing_item.checksum)),
                                    ),
                                )
                                .set(schema::achievement::details.eq(new_item.details))
                                .execute(&connection)
                                {
                                    Ok(_) => warn!(?existing_item.checksum, "Updated achievement"),
                                    Err(err) => return Err(Error::DatabaseError(err)),
                                }
                            }
                        }
                    },
                    _ => return Err(Error::DatabaseError(error)),
                }
            }
        }

        // Update the checksum for the migration
        diesel::update(schema::data_migration::dsl::data_migration)
            .filter(schema::data_migration::title.eq(String::from("achievements")))
            .set((
                schema::data_migration::checksum.eq(hash(&achievements).to_string()),
                schema::data_migration::last_run.eq(chrono::Utc::now().naive_utc()),
            ))
            .execute(&connection)?;
    } else {
        info!("No achievement updates required");
    }

    let data = schema::achievement::dsl::achievement.load::<AchievementModel>(&connection)?;

    Ok(data)
    // Ok(data.iter().map(comp::Achievement::from).collect::<_>())
}

fn load_data() -> Vec<AchievementItem> {
    let manifest_dir = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "data/achievements.ron");

    info!(?manifest_dir, "manifest_dir:");

    match std::fs::canonicalize(manifest_dir) {
        Ok(path) => match std::fs::File::open(path) {
            Ok(file) => ron::de::from_reader(file).expect("Error parsing achievement data"),
            Err(error) => panic!(error.to_string()),
        },
        Err(error) => {
            warn!(?error, "Unable to find achievement data file");
            Vec::new()
        },
    }
}

pub fn hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/// Holds a list of achievements available to players.
///
/// This acts as the reference for checks on achievements, and holds id's as
/// well as details of achievement
#[derive(Debug)]
pub struct AvailableAchievements(pub Vec<AchievementModel>);
