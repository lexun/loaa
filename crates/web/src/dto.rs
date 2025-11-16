// Data Transfer Objects for client-server communication
// These are simple, serializable structs that work on both client (WASM) and server

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, Utc};

// Simple types
pub type UuidDto = String;

// Kid DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KidDto {
    pub id: UuidDto,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Task DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    pub id: UuidDto,
    pub name: String,
    pub description: String,
    pub value: Decimal,
    pub cadence: CadenceDto,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Cadence DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CadenceDto {
    Daily,
    Weekly,
    OneTime,
}

// LedgerEntry DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntryDto {
    pub id: UuidDto,
    pub kid_id: UuidDto,
    pub amount: Decimal,
    pub description: String,
    pub entry_type: EntryTypeDto,
    pub created_at: DateTime<Utc>,
}

// EntryType DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntryTypeDto {
    Earned,
    Adjusted,
}

// Ledger DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerDto {
    pub kid_id: UuidDto,
    pub balance: Decimal,
    pub entries: Vec<LedgerEntryDto>,
}

// Dashboard data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KidSummaryDto {
    pub kid: KidDto,
    pub balance: Decimal,
    pub recent_entry: Option<LedgerEntryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardDataDto {
    pub kid_summaries: Vec<KidSummaryDto>,
    pub total_kids: usize,
    pub active_tasks: usize,
}

// Conversion functions (only available on server side)
#[cfg(feature = "ssr")]
pub mod convert {
    use super::*;
    use loaa_core::models::*;

    impl From<Kid> for KidDto {
        fn from(kid: Kid) -> Self {
            KidDto {
                id: kid.id.to_string(),
                name: kid.name,
                created_at: kid.created_at,
                updated_at: kid.updated_at,
            }
        }
    }

    impl From<Task> for TaskDto {
        fn from(task: Task) -> Self {
            TaskDto {
                id: task.id.to_string(),
                name: task.name,
                description: task.description,
                value: task.value,
                cadence: task.cadence.into(),
                created_at: task.created_at,
                updated_at: task.updated_at,
            }
        }
    }

    impl From<Cadence> for CadenceDto {
        fn from(cadence: Cadence) -> Self {
            match cadence {
                Cadence::Daily => CadenceDto::Daily,
                Cadence::Weekly => CadenceDto::Weekly,
                Cadence::OneTime => CadenceDto::OneTime,
            }
        }
    }

    impl From<CadenceDto> for Cadence {
        fn from(dto: CadenceDto) -> Self {
            match dto {
                CadenceDto::Daily => Cadence::Daily,
                CadenceDto::Weekly => Cadence::Weekly,
                CadenceDto::OneTime => Cadence::OneTime,
            }
        }
    }

    impl From<LedgerEntry> for LedgerEntryDto {
        fn from(entry: LedgerEntry) -> Self {
            LedgerEntryDto {
                id: entry.id.to_string(),
                kid_id: entry.kid_id.to_string(),
                amount: entry.amount,
                description: entry.description,
                entry_type: entry.entry_type.into(),
                created_at: entry.created_at,
            }
        }
    }

    impl From<EntryType> for EntryTypeDto {
        fn from(et: EntryType) -> Self {
            match et {
                EntryType::Earned => EntryTypeDto::Earned,
                EntryType::Adjusted => EntryTypeDto::Adjusted,
            }
        }
    }

    impl From<Ledger> for LedgerDto {
        fn from(ledger: Ledger) -> Self {
            LedgerDto {
                kid_id: ledger.kid_id.to_string(),
                balance: ledger.balance,
                entries: ledger.entries.into_iter().map(Into::into).collect(),
            }
        }
    }
}
