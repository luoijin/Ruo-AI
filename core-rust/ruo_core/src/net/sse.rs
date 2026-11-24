use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ParsedEvent {
    Meta {
        message_id: String,
        detected_language: Option<String>,
        study_step: Option<i32>,
        model: Option<String>,
        fallback: bool,
    },
    Delta {
        text: String,
    },
    Citation {
        attachment_id: String,
        page: Option<i32>,
        snippet: Option<String>,
    },
    Done {
        finish_reason: Option<String>,
    },
    Error {
        code: String,
        message: String,
        retry_after_sec: Option<u64>,
    },
    Unknown {
        event: String,
        data: String,
    },
}

#[derive(Default)]
pub struct SseParser {
    buffer: String,
}

impl SseParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn feed(&mut self, chunk: &str) -> Vec<ParsedEvent> {
        self.buffer.push_str(chunk);
        let mut events = Vec::new();

        while let Some(pos) = self.buffer.find("\n\n") {
            let block = self.buffer[..pos].to_string();
            self.buffer = self.buffer[pos + 2..].to_string();

            if let Some(ev) = self.parse_block(&block) {
                events.push(ev);
            }
        }
        events
    }

    fn parse_block(&self, block: &str) -> Option<ParsedEvent> {
        let mut event_type = "message";
        let mut data_lines = Vec::new();

        for line in block.lines() {
            let line = line.trim_end_matches('\r');
            if line.starts_with(':') {
                continue; // SSE comment
            }
            if let Some(stripped) = line.strip_prefix("event:") {
                event_type = stripped.trim();
            } else if let Some(stripped) = line.strip_prefix("data:") {
                data_lines.push(stripped.trim_start());
            }
        }

        if data_lines.is_empty() {
            return None;
        }

        let data = data_lines.join("\n");

        match event_type {
            "meta" => {
                #[derive(Deserialize)]
                struct MetaPayload {
                    message_id: String,
                    detected_language: Option<String>,
                    study_step: Option<i32>,
                    model: Option<String>,
                    #[serde(default)]
                    fallback: bool,
                }
                if let Ok(m) = serde_json::from_str::<MetaPayload>(&data) {
                    Some(ParsedEvent::Meta {
                        message_id: m.message_id,
                        detected_language: m.detected_language,
                        study_step: m.study_step,
                        model: m.model,
                        fallback: m.fallback,
                    })
                } else {
                    Some(ParsedEvent::Unknown {
                        event: event_type.to_string(),
                        data,
                    })
                }
            }
            "delta" => {
                #[derive(Deserialize)]
                struct DeltaPayload {
                    text: String,
                }
                if let Ok(d) = serde_json::from_str::<DeltaPayload>(&data) {
                    Some(ParsedEvent::Delta { text: d.text })
                } else {
                    Some(ParsedEvent::Unknown {
                        event: event_type.to_string(),
                        data,
                    })
                }
            }
            "citation" => {
                #[derive(Deserialize)]
                struct CitationPayload {
                    attachment_id: String,
                    page: Option<i32>,
                    snippet: Option<String>,
                }
                if let Ok(c) = serde_json::from_str::<CitationPayload>(&data) {
                    Some(ParsedEvent::Citation {
                        attachment_id: c.attachment_id,
                        page: c.page,
                        snippet: c.snippet,
                    })
                } else {
                    Some(ParsedEvent::Unknown {
                        event: event_type.to_string(),
                        data,
                    })
                }
            }
            "done" => {
                #[derive(Deserialize)]
                struct DonePayload {
                    finish_reason: Option<String>,
                }
                let finish_reason = serde_json::from_str::<DonePayload>(&data)
                    .ok()
                    .and_then(|p| p.finish_reason);
                Some(ParsedEvent::Done { finish_reason })
            }
            "error" => {
                #[derive(Deserialize)]
                struct ErrorPayload {
                    code: String,
                    message: String,
                    retry_after_sec: Option<u64>,
                }
                if let Ok(e) = serde_json::from_str::<ErrorPayload>(&data) {
                    Some(ParsedEvent::Error {
                        code: e.code,
                        message: e.message,
                        retry_after_sec: e.retry_after_sec,
                    })
                } else {
                    Some(ParsedEvent::Unknown {
                        event: event_type.to_string(),
                        data,
                    })
                }
            }
            _ => Some(ParsedEvent::Unknown {
                event: event_type.to_string(),
                data,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_chunked_parsing() {
        let mut parser = SseParser::new();
        let ev1 = parser.feed("event: meta\ndata: {\"message_id\":\"m1\",\"detected_language\":\"tl-PH\",\"fallback\":false}\n\n");
        assert_eq!(ev1.len(), 1);
        match &ev1[0] {
            ParsedEvent::Meta {
                message_id,
                detected_language,
                fallback,
                ..
            } => {
                assert_eq!(message_id, "m1");
                assert_eq!(detected_language.as_deref(), Some("tl-PH"));
                assert!(!fallback);
            }
            _ => panic!("Expected Meta event"),
        }

        // Test chunk split across feeds
        let ev2 = parser.feed("event: delta\nda");
        assert_eq!(ev2.len(), 0);

        let ev3 = parser.feed("ta: {\"text\":\"Kumusta!\"}\n\n");
        assert_eq!(ev3.len(), 1);
        match &ev3[0] {
            ParsedEvent::Delta { text } => assert_eq!(text, "Kumusta!"),
            _ => panic!("Expected Delta event"),
        }

        let ev4 = parser.feed("event: done\ndata: {\"finish_reason\":\"stop\"}\n\n");
        assert_eq!(ev4.len(), 1);
        match &ev4[0] {
            ParsedEvent::Done { finish_reason } => {
                assert_eq!(finish_reason.as_deref(), Some("stop"))
            }
            _ => panic!("Expected Done event"),
        }
    }
}
