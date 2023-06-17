use std::ops::Deref;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use futures::executor::block_on;
use sea_orm::{ActiveValue, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, QuerySelect, QueryFilter, QueryTrait, QueryOrder};
use serde_json::json;
use uuid::Uuid;
use domain_model::{Action, AuditDetails, AuditEvent, AuditTags, Deployment, Order, Position, Simulation};
use crate::entities::audit;
use crate::entities::prelude::{Audit};

pub struct AuditService<T: ConnectionTrait> {
    repository: AuditRepository<T>,
}

impl<T: ConnectionTrait> AuditService<T> {
    pub fn new(db: Arc<T>) -> Self {
        AuditService { repository: AuditRepository { db } }
    }

    pub fn log_simulation(&self, simulation: Simulation) {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            tags: simulation.audit_tags(),
            event: AuditDetails::Simulation(simulation),
        };
        self.save(event);
    }

    pub fn log_deployment(&self, deployment: Deployment) {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            tags: deployment.audit_tags(),
            event: AuditDetails::Deployment(deployment),
        };
        self.save(event);
    }

    pub fn log_order(&self, order: Order) {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            tags: order.audit_tags(),
            event: AuditDetails::Order(order),
        };
        self.save(event);
    }

    pub fn log_position(&self, position: Position) {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            tags: position.audit_tags(),
            event: AuditDetails::Position(position),
        };
        self.save(event);
    }
    pub fn log_action(&self, action: Action) {
        let event = AuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            tags: action.audit_tags(),
            event: AuditDetails::Action(action),
        };
        self.save(event);
    }

    fn save(&self, audit_event: AuditEvent) {
        self.repository.save(audit_event)
            .expect("Error during audit saving");
    }

    pub fn get(&self, from_timestamp: Option<DateTime<Utc>>, tags: Vec<String>, limit: Option<u64>) -> Vec<AuditEvent> {
        self.repository.find_by(
            from_timestamp,
            tags,
            limit,
        ).unwrap()
    }
}

struct AuditRepository<T: ConnectionTrait> {
    db: Arc<T>,
}

impl<T: ConnectionTrait> AuditRepository<T> {
    fn save(&self, audit_event: AuditEvent) -> Result<(), DbErr> {
        let audit_event = audit::ActiveModel {
            id: ActiveValue::Set(audit_event.id.as_bytes().to_vec()),
            timestamp: ActiveValue::Set(audit_event.timestamp),
            tags: ActiveValue::Set(json!(audit_event.tags)),
            event: ActiveValue::Set(json!(audit_event.event)),
        };
        block_on(Audit::insert(audit_event).exec(self.db.deref()))?;
        Ok(())
    }

    pub fn find_by(&self,
                   from_timestamp: Option<DateTime<Utc>>,
                   tags: Vec<String>,
                   limit: Option<u64>) -> Result<Vec<AuditEvent>, DbErr> {
        let mut condition = Condition::all();
        for tag in tags {
            condition = condition.add(audit::Column::Tags.contains(&tag));
        }
        if let Some(from_timestamp) = from_timestamp {
            condition = condition.add(audit::Column::Timestamp.gte(from_timestamp));
        }

        let result = block_on(audit::Entity::find()
            .filter(condition)
            .apply_if(limit, QuerySelect::limit)
            .order_by_desc(audit::Column::Timestamp)
            .all(self.db.deref()))?
            .into_iter()
            .map(|model| {
                AuditEvent {
                    id: Uuid::from_slice(&model.id).unwrap(),
                    timestamp: model.timestamp,
                    tags: serde_json::from_value(model.tags).unwrap(),
                    event: serde_json::from_value(model.event).unwrap(),
                }
            }).collect();

        Ok(result)
    }
}
