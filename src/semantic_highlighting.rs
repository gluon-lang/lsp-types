use base64;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use crate::VersionedTextDocumentIdentifier;
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SemanticHighlightingClientCapability {
    /// `true` if the client supports semantic highlighting support text documents. Otherwise, `false`. It is `false` by default.
    pub semantic_highlighting: bool,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize, Clone)]
pub struct SemanticHighlightingServerCapability {
    /// A "lookup table" of semantic highlighting [TextMate scopes](https://manual.macromates.com/en/language_grammars)
    /// supported by the language server. If not defined or empty, then the server does not support the semantic highlighting
    /// feature. Otherwise, clients should reuse this "lookup table" when receiving semantic highlighting notifications from
    /// the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SemanticHighlightingToken {
    pub character: u32,
    pub length: u16,
    pub scope: u16,
}

impl SemanticHighlightingToken {
    /// Deserializes the tokens from a base64 encoded string
    fn deserialize_tokens<'de, D>(
        deserializer: D,
    ) -> Result<Option<Vec<SemanticHighlightingToken>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let opt_s = Option::<String>::deserialize(deserializer)?;

        if let Some(s) = opt_s {
            let bytes = base64::decode_config(s.as_str(), base64::STANDARD)
                .map_err(|_| serde::de::Error::custom("Error parsing base64 string"))?;
            let mut res = Vec::new();
            for chunk in bytes.chunks_exact(8) {
                res.push(SemanticHighlightingToken {
                    character: u32::from_be_bytes(<[u8; 4]>::try_from(&chunk[0..4]).unwrap()),
                    length: u16::from_be_bytes(<[u8; 2]>::try_from(&chunk[4..6]).unwrap()),
                    scope: u16::from_be_bytes(<[u8; 2]>::try_from(&chunk[6..8]).unwrap()),
                });
            }
            Result::Ok(Some(res))
        } else {
            Result::Ok(None)
        }
    }

    /// Serialize the tokens to a base64 encoded string
    fn serialize_tokens<S>(
        tokens: &Option<Vec<SemanticHighlightingToken>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(tokens) = tokens {
            let mut bytes = vec![];
            for token in tokens {
                bytes.extend_from_slice(&token.character.to_be_bytes());
                bytes.extend_from_slice(&token.length.to_be_bytes());
                bytes.extend_from_slice(&token.scope.to_be_bytes());
            }
            serializer.collect_str(&base64::display::Base64Display::with_config(
                &bytes,
                base64::STANDARD,
            ))
        } else {
            serializer.serialize_none()
        }
    }
}

/// Represents a semantic highlighting information that has to be applied on a specific line of the text document.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct SemanticHighlightingInformation {
    /// The zero-based line position in the text document.
    pub line: i32,

    /// A base64 encoded string representing every single highlighted characters with its start position, length and the "lookup table" index of
    /// of the semantic highlighting [TextMate scopes](https://manual.macromates.com/en/language_grammars).
    /// If the `tokens` is empty or not defined, then no highlighted positions are available for the line.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "SemanticHighlightingToken::deserialize_tokens",
        serialize_with = "SemanticHighlightingToken::serialize_tokens"
    )]
    pub tokens: Option<Vec<SemanticHighlightingToken>>,
}

/// Parameters for the semantic highlighting (server-side) push notification.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticHighlightingParams {
    /// The text document that has to be decorated with the semantic highlighting information.
    pub text_document: VersionedTextDocumentIdentifier,

    /// An array of semantic highlighting information.
    pub lines: Vec<SemanticHighlightingInformation>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_serialization;

    #[test]
    fn test_semantic_highlighting_information_serialization() {
        test_serialization(
            &SemanticHighlightingInformation {
                line: 10,
                tokens: Some(vec![
                    SemanticHighlightingToken {
                        character: 0x00000001,
                        length: 0x0002,
                        scope: 0x0003,
                    },
                    SemanticHighlightingToken {
                        character: 0x00112222,
                        length: 0x0FF0,
                        scope: 0x0202,
                    },
                ]),
            },
            r#"{"line":10,"tokens":"AAAAAQACAAMAESIiD/ACAg=="}"#,
        );

        test_serialization(
            &SemanticHighlightingInformation {
                line: 22,
                tokens: None,
            },
            r#"{"line":22}"#,
        );
    }
}
