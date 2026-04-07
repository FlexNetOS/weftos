//! Relationship (edge) types, confidence levels, and the `Relationship` struct.

use crate::entity::EntityId;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Confidence
// ---------------------------------------------------------------------------

/// Confidence level for an extracted relationship.
///
/// Maps directly from Graphify's three-tier confidence model. The associated
/// weight values are used as edge weights in graph algorithms and the
/// CausalGraph bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Confidence {
    /// Deterministically extracted from AST / document structure.
    Extracted,
    /// Inferred by LLM or heuristic reasoning.
    Inferred,
    /// Ambiguous -- multiple interpretations possible.
    Ambiguous,
}

impl Confidence {
    /// Map to edge weight for graph algorithms.
    ///
    /// - `EXTRACTED` -> 1.0
    /// - `INFERRED`  -> 0.7
    /// - `AMBIGUOUS`  -> 0.4
    pub fn to_weight(&self) -> f32 {
        match self {
            Self::Extracted => 1.0,
            Self::Inferred => 0.7,
            Self::Ambiguous => 0.4,
        }
    }

    /// Map to a score used in JSON export confidence_score field.
    ///
    /// - `EXTRACTED` -> 1.0
    /// - `INFERRED`  -> 0.5
    /// - `AMBIGUOUS`  -> 0.2
    pub fn to_score(&self) -> f64 {
        match self {
            Self::Extracted => 1.0,
            Self::Inferred => 0.5,
            Self::Ambiguous => 0.2,
        }
    }

    /// Return the uppercase string form.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Extracted => "EXTRACTED",
            Self::Inferred => "INFERRED",
            Self::Ambiguous => "AMBIGUOUS",
        }
    }

    /// All valid uppercase string representations.
    pub const VALID_STRINGS: &[&str] = &["EXTRACTED", "INFERRED", "AMBIGUOUS"];

    /// Parse from an uppercase string.
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s {
            "EXTRACTED" => Some(Self::Extracted),
            "INFERRED" => Some(Self::Inferred),
            "AMBIGUOUS" => Some(Self::Ambiguous),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// RelationType
// ---------------------------------------------------------------------------

/// Relationship type taxonomy covering code and forensic domains.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    // --- Code domain (10 variants) ---
    Calls,
    Imports,
    ImportsFrom,
    DependsOn,
    Contains,
    Implements,
    Configures,
    Extends,
    MethodOf,
    Instantiates,

    // --- Forensic domain (11 variants) ---
    WitnessedBy,
    FoundAt,
    Contradicts,
    Corroborates,
    AlibiedBy,
    Precedes,
    DocumentedIn,
    OwnedBy,
    ContactedBy,
    LocatedAt,
    SemanticallySimilarTo,

    // --- Shared ---
    RelatedTo,
    CaseOf,
    Custom(String),
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Calls => write!(f, "calls"),
            Self::Imports => write!(f, "imports"),
            Self::ImportsFrom => write!(f, "imports_from"),
            Self::DependsOn => write!(f, "depends_on"),
            Self::Contains => write!(f, "contains"),
            Self::Implements => write!(f, "implements"),
            Self::Configures => write!(f, "configures"),
            Self::Extends => write!(f, "extends"),
            Self::MethodOf => write!(f, "method_of"),
            Self::Instantiates => write!(f, "instantiates"),
            Self::WitnessedBy => write!(f, "witnessed_by"),
            Self::FoundAt => write!(f, "found_at"),
            Self::Contradicts => write!(f, "contradicts"),
            Self::Corroborates => write!(f, "corroborates"),
            Self::AlibiedBy => write!(f, "alibied_by"),
            Self::Precedes => write!(f, "precedes"),
            Self::DocumentedIn => write!(f, "documented_in"),
            Self::OwnedBy => write!(f, "owned_by"),
            Self::ContactedBy => write!(f, "contacted_by"),
            Self::LocatedAt => write!(f, "located_at"),
            Self::SemanticallySimilarTo => write!(f, "semantically_similar_to"),
            Self::RelatedTo => write!(f, "related_to"),
            Self::CaseOf => write!(f, "case_of"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl RelationType {
    /// Return the snake_case string form.
    pub fn as_str(&self) -> String {
        self.to_string()
    }
}

// ---------------------------------------------------------------------------
// Relationship
// ---------------------------------------------------------------------------

/// A directed, typed relationship between two entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub source: EntityId,
    pub target: EntityId,
    pub relation_type: RelationType,
    pub confidence: Confidence,
    pub weight: f32,
    pub source_file: Option<String>,
    pub source_location: Option<String>,
    pub metadata: serde_json::Value,
}

impl Relationship {
    /// Return the relation type as a snake_case string.
    pub fn relation_type_str(&self) -> String {
        self.relation_type.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{DomainTag, EntityType};

    #[test]
    fn confidence_weights() {
        assert!((Confidence::Extracted.to_weight() - 1.0).abs() < f32::EPSILON);
        assert!((Confidence::Inferred.to_weight() - 0.7).abs() < f32::EPSILON);
        assert!((Confidence::Ambiguous.to_weight() - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn confidence_scores() {
        assert!((Confidence::Extracted.to_score() - 1.0).abs() < f64::EPSILON);
        assert!((Confidence::Inferred.to_score() - 0.5).abs() < f64::EPSILON);
        assert!((Confidence::Ambiguous.to_score() - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn confidence_serde_uppercase() {
        let c = Confidence::Extracted;
        let json = serde_json::to_string(&c).unwrap();
        assert_eq!(json, "\"EXTRACTED\"");
        let back: Confidence = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Confidence::Extracted);
    }

    #[test]
    fn relation_type_serde() {
        let r = RelationType::ImportsFrom;
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(json, "\"imports_from\"");
        let back: RelationType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, RelationType::ImportsFrom);
    }

    #[test]
    fn relationship_roundtrip() {
        let src = EntityId::new(&DomainTag::Code, &EntityType::Module, "a", "a.py");
        let tgt = EntityId::new(&DomainTag::Code, &EntityType::Module, "b", "b.py");
        let rel = Relationship {
            source: src,
            target: tgt,
            relation_type: RelationType::Imports,
            confidence: Confidence::Extracted,
            weight: 1.0,
            source_file: Some("a.py".into()),
            source_location: Some("L1".into()),
            metadata: serde_json::json!({}),
        };
        let json = serde_json::to_string(&rel).unwrap();
        let back: Relationship = serde_json::from_str(&json).unwrap();
        assert_eq!(back.relation_type, RelationType::Imports);
        assert_eq!(back.confidence, Confidence::Extracted);
    }
}
