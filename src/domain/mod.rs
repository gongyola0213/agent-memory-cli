#[derive(Debug, Clone)]
pub enum DomainEvent {
    UserCreated { uid: String },
    IdentityLinked { uid: String, channel: String },
    ScopeMemberAdded { scope_id: String, uid: String },
}

pub trait EventObserver {
    fn on_event(&self, _event: &DomainEvent) -> Result<(), String> {
        Ok(())
    }
}

pub struct NoopObserver;

impl EventObserver for NoopObserver {
    fn on_event(&self, event: &DomainEvent) -> Result<(), String> {
        match event {
            DomainEvent::UserCreated { uid } => {
                let _ = uid;
            }
            DomainEvent::IdentityLinked { uid, channel } => {
                let _ = (uid, channel);
            }
            DomainEvent::ScopeMemberAdded { scope_id, uid } => {
                let _ = (scope_id, uid);
            }
        }
        Ok(())
    }
}
