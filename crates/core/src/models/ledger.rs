use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    #[serde(skip)]
    pub id: Uuid,
    #[serde(serialize_with = "serialize_uuid_as_string", deserialize_with = "deserialize_uuid_from_string")]
    pub kid_id: Uuid,
    pub amount: Decimal,
    pub entry_type: EntryType,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

// Custom serialization for UUID to ensure it's stored as a string
fn serialize_uuid_as_string<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&uuid.to_string())
}

fn deserialize_uuid_from_string<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Uuid::parse_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntryType {
    Earned,
    Adjusted,
}

impl LedgerEntry {
    pub fn new(kid_id: Uuid, amount: Decimal, entry_type: EntryType, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            kid_id,
            amount,
            entry_type,
            description: description.trim().to_string(),
            created_at: Utc::now(),
        }
    }

    pub fn earned(kid_id: Uuid, amount: Decimal, description: String) -> Self {
        Self::new(kid_id, amount, EntryType::Earned, description)
    }

    pub fn adjusted(kid_id: Uuid, amount: Decimal, description: String) -> Self {
        Self::new(kid_id, amount, EntryType::Adjusted, description)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub kid_id: Uuid,
    pub balance: Decimal,
    pub entries: Vec<LedgerEntry>,
}

impl Ledger {
    pub fn new(kid_id: Uuid, entries: Vec<LedgerEntry>) -> Self {
        let balance = Self::calculate_balance(&entries);
        Self {
            kid_id,
            balance,
            entries,
        }
    }

    pub fn calculate_balance(entries: &[LedgerEntry]) -> Decimal {
        entries.iter().map(|e| e.amount).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_ledger_entry_creation() {
        let kid_id = Uuid::new_v4();
        let entry = LedgerEntry::earned(
            kid_id,
            dec!(5.00),
            "Completed homework".to_string(),
        );
        assert_eq!(entry.amount, dec!(5.00));
        assert_eq!(entry.entry_type, EntryType::Earned);
    }

    #[test]
    fn test_ledger_balance_calculation() {
        let kid_id = Uuid::new_v4();
        let entries = vec![
            LedgerEntry::earned(kid_id, dec!(5.00), "Task 1".to_string()),
            LedgerEntry::earned(kid_id, dec!(3.50), "Task 2".to_string()),
            LedgerEntry::adjusted(kid_id, dec!(-1.00), "Correction".to_string()),
        ];
        let ledger = Ledger::new(kid_id, entries);
        assert_eq!(ledger.balance, dec!(7.50));
    }
}

