use crate::model::{
    Category, CategoryInsert, Item, ItemInsert, ItemState, ItemStateInsert, Rating,
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OpenFlags};

const MIGRATIONS: &[&str] = &[include_str!("migrations/initialize_tables.sql")];

fn run_migrations(conn: &Connection) -> rusqlite::Result<()> {
    for migration in MIGRATIONS {
        conn.execute_batch(migration)?;
    }
    Ok(())
}

pub fn connection() -> Result<Connection> {
    let db_path = config::application_storage(false)?.join("spaced_recall.db");
    if db_path.exists() {
        let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        Ok(conn)
    } else {
        let conn = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )?;
        run_migrations(&conn)?;
        Ok(conn)
    }
}

// ── Category ──
pub fn create_category(category: CategoryInsert, connection: Connection) -> Result<()> {
    let created_at = category
        .created_at
        .unwrap_or_else(|| chrono::Utc::now())
        .timestamp();
    connection.execute(
        "INSERT INTO category (name, description, created_at) VALUES (?1, ?2, ?3)",
        (&category.name, &category.description, created_at),
    )?;
    Ok(())
}
pub fn get_categories(connection: Connection) -> Result<Vec<Category>> {
    let mut stmt = connection.prepare("SELECT id, name, description, created_at FROM category")?;
    let categories = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: DateTime::from_timestamp(row.get(3)?, 0).unwrap(),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(categories)
}

// ── Item ──
pub fn create_item(item: ItemInsert, connection: Connection) -> Result<()> {
    let created_at = item
        .created_at
        .unwrap_or_else(|| chrono::Utc::now())
        .timestamp();
    let changes = connection.execute(
        "INSERT INTO item (category_id, front, back, created_at) VALUES (?1, ?2, ?3, ?4)",
        (&item.category_id, &item.front, &item.back, created_at),
    )?;
    if changes > 1 {
        return Err(anyhow::anyhow!("Query changed {changes} rows"));
    }
    let change_id = connection.last_insert_rowid();
    let default_item_state = ItemStateInsert {
        item_id: change_id,
        stability: None,
        difficulty: None,
        due_at: Utc::now(),
        last_reviewed_at: None,
        reps: 0,
        lapses: 0,
    };
    create_item_state(default_item_state, connection)
}
fn create_item_state(item_state: ItemStateInsert, connection: Connection) -> Result<()> {
    connection.execute(
        "INSERT INTO item_state (item_id, stability, difficulty, due_at, last_reviewed_at, reps, lapses) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (
            item_state.item_id,
            item_state.stability,
            item_state.difficulty,
            item_state.due_at.timestamp(),
            item_state.last_reviewed_at.map(|dt| dt.timestamp()),
            item_state.reps,
            item_state.lapses,
        ),
    )?;
    Ok(())
}
pub fn get_items_by_category(category_id: i64, connection: Connection) -> Result<Vec<Item>> {
    let mut stmt = connection.prepare(
        "SELECT id, category_id, front, back, created_at FROM item WHERE category_id = ?1",
    )?;
    let items = stmt
        .query_map([category_id], |row| {
            Ok(Item {
                id: row.get(0)?,
                category_id: row.get(1)?,
                front: row.get(2)?,
                back: row.get(3)?,
                created_at: DateTime::from_timestamp(row.get(4)?, 0).unwrap(),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(items)
}
pub fn get_due_items(limit: u8, connection: Connection) -> Result<Vec<(Item, ItemState)>> {
    let now = Utc::now().timestamp();
    let mut stmt = connection.prepare(
        "SELECT i.id, i.category_id, i.front, i.back, i.created_at,
                s.item_id, s.stability, s.difficulty, s.due_at, s.last_reviewed_at, s.reps, s.lapses
         FROM item_state s
         JOIN item i ON i.id = s.item_id
         WHERE s.due_at <= ?1
         ORDER BY s.due_at ASC
         LIMIT ?2",
    )?;
    let rows = stmt
        .query_map((now, limit), |row| {
            let item = Item {
                id: row.get(0)?,
                category_id: row.get(1)?,
                front: row.get(2)?,
                back: row.get(3)?,
                created_at: DateTime::from_timestamp(row.get(4)?, 0).unwrap(),
            };
            let state = ItemState {
                item_id: row.get(5)?,
                stability: row.get(6)?,
                difficulty: row.get(7)?,
                due_at: DateTime::from_timestamp(row.get(8)?, 0).unwrap(),
                last_reviewed_at: row
                    .get::<_, Option<i64>>(9)?
                    .and_then(|s| DateTime::from_timestamp(s, 0)),
                reps: row.get(10)?,
                lapses: row.get(11)?,
            };
            Ok((item, state))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}
pub fn update_item_state(item_id: i64, rating: Rating, connection: Connection) -> Result<()> {
    let current = get_item_state(item_id, &connection)?;
    let now = Utc::now();

    let scheduler = crate::scheduling::Scheduler::new()?;
    let (memory, due_at) = scheduler.process_review(&current, rating, now)?;

    let learned = is_learned(&rating, current.reps, current.lapses);
    let reps = update_reps(&rating, current.reps);
    let lapses = update_lapses(&rating, current.lapses, learned);

    connection.execute(
        "UPDATE item_state SET stability = ?1, difficulty = ?2, due_at = ?3, last_reviewed_at = ?4, reps = ?5, lapses = ?6 WHERE item_id = ?7",
        (
            memory.stability as f64,
            memory.difficulty as f64,
            due_at.timestamp(),
            now.timestamp(),
            reps,
            lapses,
            item_id,
        ),
    )?;
    Ok(())
}
fn get_item_state(item_id: i64, connection: &Connection) -> Result<ItemState> {
    let state = connection.query_row(
        "SELECT item_id, stability, difficulty, due_at, last_reviewed_at, reps, lapses FROM item_state WHERE item_id = ?1",
        [item_id],
        |row| {
            Ok(ItemState {
                item_id: row.get(0)?,
                stability: row.get(1)?,
                difficulty: row.get(2)?,
                due_at: DateTime::from_timestamp(row.get(3)?, 0).unwrap(),
                last_reviewed_at: row.get::<_, Option<i64>>(4)?.and_then(|s| DateTime::from_timestamp(s, 0)),
                reps: row.get(5)?,
                lapses: row.get(6)?,
            })
        },
    )?;
    Ok(state)
}

fn update_reps(rating: &Rating, current: i64) -> i64 {
    match rating {
        Rating::Good | Rating::Easy => current + 1,
        _ => current,
    }
}

fn update_lapses(rating: &Rating, current: i64, is_learned: bool) -> i64 {
    if is_learned {
        match rating {
            Rating::Hard | Rating::Again => current + 1,
            _ => current,
        }
    } else {
        current
    }
}

fn is_learned(rating: &Rating, reps: i64, lapses: i64) -> bool {
    reps > lapses
}
