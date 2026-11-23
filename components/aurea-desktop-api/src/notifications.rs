// SPDX-License-Identifier: GPL-3.0-or-later
//
// AureaOS — Desktop API
// Copyright (C) 2026 Alessandro Ceccarelli

//! Shared desktop notification contracts.
//!
//! The system notification service owns presentation, history and lifecycle.
//! Producers submit requests; they do not draw notification surfaces or manage
//! the notification center themselves. The trusted IPC boundary must authorize
//! each producer and validate action responses.

/// Maximum number of actions exposed by one notification.
pub const MAX_NOTIFICATION_ACTIONS: usize = 4;

/// Opaque identifier assigned by the notification service.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NotificationId(String);

impl NotificationId {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return None;
        }
        Some(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Importance used by the system notification policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotificationUrgency {
    Low,
    #[default]
    Normal,
    Critical,
}

/// A user-visible action offered by a notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationAction {
    id: String,
    label: String,
}

impl NotificationAction {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Option<Self> {
        let id = id.into();
        let label = label.into();
        if id.trim().is_empty() || label.trim().is_empty() {
            return None;
        }
        Some(Self { id, label })
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Content submitted to the system notification service.
///
/// Creation time, source identity, grouping and display policy are assigned
/// by the trusted service rather than accepted from an untrusted producer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotificationRequest {
    title: String,
    body: Option<String>,
    urgency: NotificationUrgency,
    actions: Vec<NotificationAction>,
}

impl NotificationRequest {
    pub fn new(
        title: impl Into<String>,
        body: Option<String>,
        urgency: NotificationUrgency,
        actions: Vec<NotificationAction>,
    ) -> Option<Self> {
        let title = title.into();
        if title.trim().is_empty() || actions.len() > MAX_NOTIFICATION_ACTIONS {
            return None;
        }

        for (index, action) in actions.iter().enumerate() {
            if actions[..index]
                .iter()
                .any(|previous| previous.id == action.id)
            {
                return None;
            }
        }

        Some(Self {
            title,
            body: body.filter(|value| !value.trim().is_empty()),
            urgency,
            actions,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    pub const fn urgency(&self) -> NotificationUrgency {
        self.urgency
    }

    pub fn actions(&self) -> &[NotificationAction] {
        &self.actions
    }
}

/// System-owned notification operations.
///
/// Authorization, producer identity, rate limiting and action dispatch belong
/// to the trusted service/IPC implementation, not to this provider trait.
pub trait NotificationProvider: Send + Sync {
    fn publish(&self, request: NotificationRequest) -> Result<NotificationId, NotificationError>;
    fn dismiss(&self, id: &NotificationId) -> Result<(), NotificationError>;
}

/// Notification service failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationError {
    Unavailable,
    NotFound,
    AccessDenied,
    RateLimited,
    InvalidRequest,
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_NOTIFICATION_ACTIONS, NotificationAction, NotificationRequest, NotificationUrgency,
    };

    #[test]
    fn request_requires_a_title() {
        assert!(
            NotificationRequest::new("  ", None, NotificationUrgency::Normal, Vec::new(),)
                .is_none()
        );
    }

    #[test]
    fn request_limits_actions_and_rejects_duplicate_ids() {
        let action = || NotificationAction::new("open", "Open").unwrap();
        let too_many = vec![action(); MAX_NOTIFICATION_ACTIONS + 1];
        assert!(
            NotificationRequest::new("Update", None, NotificationUrgency::Normal, too_many,)
                .is_none()
        );

        assert!(
            NotificationRequest::new(
                "Update",
                None,
                NotificationUrgency::Normal,
                vec![action(), action()],
            )
            .is_none()
        );
    }

    #[test]
    fn request_normalizes_empty_body_and_keeps_urgency() {
        let request = NotificationRequest::new(
            "System update",
            Some("  ".to_owned()),
            NotificationUrgency::Low,
            Vec::new(),
        )
        .unwrap();

        assert_eq!(request.body(), None);
        assert_eq!(request.urgency(), NotificationUrgency::Low);
    }
}
