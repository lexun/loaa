use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use crate::error::Result;

/// A penalty that can be applied to kids for misbehavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Penalty {
    #[serde(skip)]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    /// The amount to deduct (stored as positive, applied as negative)
    pub value: Decimal,
    /// Account this penalty belongs to
    #[serde(default)]
    pub account_id: Uuid,
    /// Owner of this penalty (user_id as string)
    #[serde(default)]
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Penalty {
    pub fn new(name: String, description: String, value: Decimal, account_id: Uuid, owner_id: String) -> Result<Self> {
        let now = Utc::now();
        let penalty = Self {
            id: Uuid::new_v4(),
            name: name.trim().to_string(),
            description: description.trim().to_string(),
            value,
            account_id,
            owner_id,
            created_at: now,
            updated_at: now,
        };
        penalty.validate()?;
        Ok(penalty)
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(crate::error::Error::Validation("Penalty name cannot be empty".to_string()));
        }
        if self.value <= Decimal::ZERO {
            return Err(crate::error::Error::Validation("Penalty value must be positive".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn test_account_id() -> Uuid {
        Uuid::new_v4()
    }

    #[test]
    fn test_penalty_creation() {
        let account_id = test_account_id();
        let penalty = Penalty::new(
            "Uncharitable arguing".to_string(),
            "Arguing without kindness".to_string(),
            dec!(0.10),
            account_id,
            "test-owner".to_string(),
        ).unwrap();
        assert_eq!(penalty.name, "Uncharitable arguing");
        assert_eq!(penalty.value, dec!(0.10));
        assert_eq!(penalty.account_id, account_id);
    }

    #[test]
    fn test_penalty_validation_empty_name() {
        let result = Penalty::new(
            "   ".to_string(),
            "".to_string(),
            dec!(0.10),
            test_account_id(),
            "test-owner".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_penalty_validation_zero_value() {
        let result = Penalty::new(
            "Test".to_string(),
            "".to_string(),
            dec!(0),
            test_account_id(),
            "test-owner".to_string(),
        );
        assert!(result.is_err());
    }
}
