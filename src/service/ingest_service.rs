use crate::repository::{event_repo, metric_repo, topk_repo};
use rusqlite::Connection;
use serde_json::Value;

pub struct IngestInput<'a> {
    pub uid: &'a str,
    pub scope_id: &'a str,
    pub event_type: &'a str,
    pub payload: &'a Value,
    pub idempotency_key: Option<&'a str>,
    pub event_id: &'a str,
    pub now: &'a str,
}

pub enum MaterializedCounter {
    FoodPref(String),
    SpendCategory(String),
    RequestPattern(String),
}

pub fn derive(event_type: &str, payload: &Value) -> Result<Option<MaterializedCounter>, String> {
    match event_type {
        "meal.rated" => {
            let cuisine = payload
                .get("cuisine")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "meal.rated requires string field: cuisine".to_string())?;
            Ok(Some(MaterializedCounter::FoodPref(cuisine.to_string())))
        }
        "expense.logged" => {
            let category = payload
                .get("category")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "expense.logged requires string field: category".to_string())?;
            Ok(Some(MaterializedCounter::SpendCategory(
                category.to_string(),
            )))
        }
        "request.logged" => {
            let pattern = payload
                .get("pattern")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "request.logged requires string field: pattern".to_string())?;
            Ok(Some(MaterializedCounter::RequestPattern(
                pattern.to_string(),
            )))
        }
        _ => Ok(None),
    }
}

pub fn ingest(conn: &mut Connection, input: IngestInput<'_>) -> Result<IngestOutcome, String> {
    let derived = derive(input.event_type, input.payload)?;

    let tx = conn
        .transaction()
        .map_err(|e| format!("failed to begin tx: {e}"))?;

    if let Some(key) = input.idempotency_key {
        if event_repo::idempotency_exists(&tx, input.scope_id, input.uid, key)? {
            tx.commit()
                .map_err(|e| format!("failed to commit tx: {e}"))?;
            return Ok(IngestOutcome::Duplicate {
                idempotency_key: key.to_string(),
            });
        }
    }

    event_repo::insert(
        &tx,
        event_repo::NewEvent {
            event_id: input.event_id,
            uid: input.uid,
            scope_id: input.scope_id,
            event_type: input.event_type,
            event_ts: input.now,
            payload_json: &input.payload.to_string(),
            idempotency_key: input.idempotency_key,
        },
    )?;

    if let Some(counter) = derived {
        let (topic, item) = match counter {
            MaterializedCounter::FoodPref(v) => ("food_pref", v),
            MaterializedCounter::SpendCategory(v) => ("spend_category", v),
            MaterializedCounter::RequestPattern(v) => ("request_pattern", v),
        };

        metric_repo::upsert_counter(&tx, input.scope_id, input.uid, topic, &item, 1.0, input.now)?;
        rebuild_topk(&tx, input.scope_id, input.uid, topic, input.now)?;
    }

    tx.commit()
        .map_err(|e| format!("failed to commit tx: {e}"))?;

    Ok(IngestOutcome::Inserted {
        event_id: input.event_id.to_string(),
        event_type: input.event_type.to_string(),
    })
}

fn rebuild_topk(
    tx: &rusqlite::Transaction<'_>,
    scope_id: &str,
    uid: &str,
    topic: &str,
    now: &str,
) -> Result<(), String> {
    topk_repo::clear(tx, scope_id, uid, topic)?;
    let rows = metric_repo::topk_source(tx, scope_id, uid, topic)?;
    for (idx, (key, score)) in rows.into_iter().enumerate() {
        let item_key = key.splitn(3, ':').nth(2).unwrap_or_default().to_string();
        topk_repo::insert(
            tx,
            topk_repo::TopkRow {
                scope_id,
                uid,
                topic,
                rank: (idx + 1) as i64,
                item_key: &item_key,
                weight: score,
                now,
            },
        )?;
    }
    Ok(())
}

pub enum IngestOutcome {
    Duplicate {
        idempotency_key: String,
    },
    Inserted {
        event_id: String,
        event_type: String,
    },
}
